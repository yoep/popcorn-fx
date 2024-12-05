use bit_vec::BitVec;
use bitmask_enum::bitmask;
use derive_more::Display;
use log::{debug, error, info, trace, warn};

use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::{
    DefaultPeerListener, Peer, PeerEntry, PeerHandle, PeerId, PeerListener, PeerState,
    ProtocolExtensionFlags,
};
use async_trait::async_trait;
use popcorn_fx_core::available_port;
use popcorn_fx_core::core::{
    block_in_place, block_in_place_runtime, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
    Handle,
};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, Receiver, UnboundedReceiver, UnboundedSender};
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::torrents::file::{File, FilePriority};
use crate::torrents::fs::TorrentFileStorage;
use crate::torrents::peer_pool::PeerPool;
use crate::torrents::torrent_request_buffer::{PendingRequest, PendingRequestBuffer};
use crate::torrents::trackers::{
    AnnounceEvent, Announcement, TrackerEntry, TrackerHandle, TrackerManager, TrackerManagerEvent,
};
use crate::torrents::{
    FileIndex, InfoHash, Piece, PieceChunkPool, PieceIndex, PiecePart, PiecePriority, Result,
    TorrentError, TorrentInfo, TorrentMetadata, DEFAULT_TORRENT_EXTENSIONS,
    DEFAULT_TORRENT_OPERATIONS, DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
    DEFAULT_TORRENT_REQUEST_STRATEGIES,
};

const DEFAULT__TIMEOUT_SECONDS: u64 = 10;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// The chain of torrent operations that are executed for each torrent.
pub type TorrentOperations = Vec<Box<dyn TorrentOperation>>;

/// The strategies available to determine piece request order/priorities.
pub type RequestStrategies = Vec<Box<dyn RequestStrategy>>;

/// Possible flags which can be attached to a [Torrent].
///
/// The default value for the flag options is [TorrentFlags::AutoManaged],
/// which will retrieve the metadata if needed and automatically start the download.
#[bitmask(u16)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum TorrentFlags {
    None = 0b00000000,
    /// Indicates seed mode.
    SeedMode = 0b0000000000000001,
    /// Indicates if uploading data is allowed.
    UploadMode = 0b0000000000000010,
    /// Indicates if downloading data is allowed.
    DownloadMode = 0b0000000000000100,
    /// Indicates share mode.
    ShareMode = 0b0000000000001000,
    /// Applies an IP filter.
    ApplyIpFilter = 0b0000000000010000,
    /// Torrent is paused.
    Paused = 0b0000000000100000,
    /// Complete the torrent metadata from peers if needed.
    Metadata = 0b0000000001000000,
    /// Sequential download is enabled.
    SequentialDownload = 0b0000000010000100,
    /// Torrent should stop when ready.
    StopWhenReady = 0b0000000100000000,
    /// Torrent is auto-managed.
    /// This means that the torrent may be resumed at any point in time.
    AutoManaged = 0b0000001001000110,
}

impl Default for TorrentFlags {
    fn default() -> Self {
        TorrentFlags::AutoManaged
    }
}

/// The states of the torrent
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentState {
    /// The torrent is being initialized
    #[display(fmt = "initializing")]
    Initializing,
    /// The torrent has not started its download yet, and is currently checking existing files.
    #[display(fmt = "checking files")]
    CheckingFiles,
    /// The torrent is trying to retrieve the metadata from peers.
    #[display(fmt = "retrieving metadata")]
    RetrievingMetadata,
    /// The torrent is being downloaded. This is the state most torrents will be in most of the time.
    #[display(fmt = "downloading")]
    Downloading,
    /// In this state the torrent has finished downloading but still doesn't have the entire torrent.
    #[display(fmt = "finished")]
    Finished,
    /// In this state the torrent has finished downloading and is a pure seeder.
    #[display(fmt = "seeding")]
    Seeding,
    /// The torrent encountered an unrecoverable error.
    #[display(fmt = "error")]
    Error,
}

impl Default for TorrentState {
    fn default() -> Self {
        Self::Initializing
    }
}

/// The torrent data transfer statistics.
/// These statics both include rate based- and lifetime metrics.
#[derive(Debug, Display, Default, Clone, PartialEq)]
#[display(
    fmt = "upload: {}, upload_rate: {}, upload_useful: {}, download: {}, download_rate: {}, download_useful: {}, total_uploaded: {}, total_downloaded: {}, wanted_pieces: {}, completed_pieces: {}, size: {}, peers: {}",
    upload,
    upload_rate,
    upload_useful,
    download,
    download_rate,
    download_useful,
    total_uploaded,
    total_downloaded,
    wanted_pieces,
    completed_pieces,
    total_size,
    total_peers
)]
pub struct TorrentTransferStats {
    /// The bytes that have been transferred from the peer.
    pub upload: usize,
    /// The bytes per second that have been transferred from the peer.
    pub upload_rate: u64,
    /// The bytes that contain actual piece payload data.
    pub upload_useful: usize,
    /// The bytes per second that contain actual piece payload data.
    pub upload_useful_rate: u64,
    /// The bytes that have been transferred to the peer.
    pub download: usize,
    /// The bytes per second that the downloaded from the peer.
    pub download_rate: u64,
    /// The bytes that contain actual piece payload data.
    pub download_useful: usize,
    /// The bytes per second that contain actual piece payload data.
    pub download_useful_rate: u64,
    /// The total bytes that have been uploaded during the lifetime of the torrent.
    pub total_uploaded: usize,
    /// The total bytes that have been downloaded during the lifetime of the torrent.
    pub total_downloaded: usize,
    /// The total amount of pieces which are wanted by the torrent
    pub wanted_pieces: usize,
    /// The amount of pieces which have been completed by the torrent
    pub completed_pieces: usize,
    /// The total size, in bytes, of all interested files of the torrent.
    pub total_size: usize,
    /// The currently total active peer connections.
    pub total_peers: usize,
}

impl TorrentTransferStats {
    /// Get the progress, as a percentage, of the torrent download.
    pub fn progress(&self) -> f32 {
        if self.total_size == 0 {
            return 100.0;
        }

        let progress = (self.total_downloaded as f32 / self.total_size as f32) * 100.0;
        (progress * 100.0).round() / 100.0
    }

    /// Reset the rate- & second based metrics within the statistics.
    fn reset(&mut self) {
        self.upload = 0;
        self.upload_rate = 0;
        self.upload_useful = 0;
        self.upload_useful_rate = 0;
        self.download = 0;
        self.download_rate = 0;
        self.download_useful = 0;
        self.download_useful_rate = 0;
    }
}

/// The torrent configuration.
#[derive(Debug, Clone)]
pub struct TorrentConfig {
    pub peers_lower_limit: usize,
    pub peers_upper_limit: usize,
    pub peer_connection_timeout: Duration,
    pub tracker_connection_timeout: Duration,
}

impl TorrentConfig {
    /// Create a new torrent configuration builder.
    pub fn builder() -> TorrentConfigBuilder {
        TorrentConfigBuilder::builder()
    }
}

#[derive(Debug, Default)]
pub struct TorrentConfigBuilder {
    peers_lower_limit: Option<usize>,
    peers_upper_limit: Option<usize>,
    peer_connection_timeout: Option<Duration>,
    tracker_connection_timeout: Option<Duration>,
}

impl TorrentConfigBuilder {
    /// Create a new torrent configuration builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the lower limit for the number of peers.
    pub fn peers_lower_limit(mut self, limit: usize) -> Self {
        self.peers_lower_limit = Some(limit);
        self
    }

    /// Set the upper limit for the number of peers.
    pub fn peers_upper_limit(mut self, limit: usize) -> Self {
        self.peers_upper_limit = Some(limit);
        self
    }

    /// Set the timeout for peer connections.
    pub fn peer_connection_timeout(mut self, timeout: Duration) -> Self {
        self.peer_connection_timeout = Some(timeout);
        self
    }

    /// Set the timeout for tracker connections.
    pub fn tracker_connection_timeout(mut self, timeout: Duration) -> Self {
        self.tracker_connection_timeout = Some(timeout);
        self
    }

    /// Build the torrent configuration.
    pub fn build(self) -> TorrentConfig {
        let peers_lower_limit = self.peers_lower_limit.unwrap_or(10);
        let peers_upper_limit = self.peers_upper_limit.unwrap_or(100);
        let peer_connection_timeout = self
            .peer_connection_timeout
            .unwrap_or(Duration::from_secs(DEFAULT__TIMEOUT_SECONDS));
        let tracker_connection_timeout = self
            .tracker_connection_timeout
            .unwrap_or(Duration::from_secs(DEFAULT__TIMEOUT_SECONDS));

        TorrentConfig {
            peers_lower_limit,
            peers_upper_limit,
            peer_connection_timeout,
            tracker_connection_timeout,
        }
    }
}

/// Requests a new torrent creation based on the given data.
/// This is the **recommended** way to create new torrents.
///
/// # Examples
///
/// ```rust,no_run
/// use std::time::Duration;
/// use popcorn_fx_torrent::torrents::{Torrent, TorrentFlags, TorrentInfo, TorrentRequest, Result};
/// use popcorn_fx_torrent::torrents::fs::TorrentFileStorage;
/// use popcorn_fx_torrent::torrents::peers::extensions::Extensions;
///
/// fn create_new_torrent(metadata: TorrentInfo, extensions: Extensions, storage: Box<dyn TorrentFileStorage>) -> Result<Torrent> {
///     let request = Torrent::request()
///         .metadata(metadata)
///         .options(TorrentFlags::Metadata)
///         .extensions(extensions)
///         .storage(storage)
///         .peer_listener_port(6881);
///
///     Torrent::try_from(request)
/// }
/// ```
#[derive(Debug, Default)]
pub struct TorrentRequest {
    /// The torrent metadata information
    metadata: Option<TorrentInfo>,
    /// The torrent options
    options: Option<TorrentFlags>,
    /// The torrent configuration
    config: Option<TorrentConfig>,
    /// The port on which the torrent session is listening for new incoming peer connections
    peer_listener_port: Option<u16>,
    /// The protocol extensions that should be enabled
    protocol_extensions: Option<ProtocolExtensionFlags>,
    /// The extensions that should be enabled for this torrent
    extensions: Option<Extensions>,
    /// The storage strategy to use for the torrent data
    storage: Option<Box<dyn TorrentFileStorage>>,
    /// The operations used by the torrent for processing data
    operations: Option<Vec<Box<dyn TorrentOperation>>>,
    /// The request strategies used by the torrent
    request_strategies: Option<Vec<Box<dyn RequestStrategy>>>,
    /// The underlying Tokio runtime to use for asynchronous operations
    runtime: Option<Arc<Runtime>>,
}

impl TorrentRequest {
    /// Set the torrent metadata
    pub fn metadata(mut self, metadata: TorrentInfo) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the torrent options
    pub fn options(mut self, options: TorrentFlags) -> Self {
        self.options = Some(options);
        self
    }

    /// Set the torrent configuration
    pub fn config(mut self, config: TorrentConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the port on which the torrent session is listening for new incoming peer connections
    pub fn peer_listener_port(mut self, port: u16) -> Self {
        self.peer_listener_port = Some(port);
        self
    }

    /// Set the protocol extensions that should be enabled
    pub fn protocol_extensions(mut self, extensions: ProtocolExtensionFlags) -> Self {
        self.protocol_extensions = Some(extensions);
        self
    }

    /// Set the extensions that should be enabled for this torrent
    pub fn extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set the storage strategy to use for the torrent data
    pub fn storage(mut self, storage: Box<dyn TorrentFileStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set the operations used by the torrent for processing data
    pub fn operations(mut self, operations: Vec<Box<dyn TorrentOperation>>) -> Self {
        self.operations = Some(operations);
        self
    }

    /// Set the request strategies used by the torrent
    pub fn request_strategies(mut self, request_strategies: Vec<Box<dyn RequestStrategy>>) -> Self {
        self.request_strategies = Some(request_strategies);
        self
    }

    /// Set the underlying Tokio runtime to use for asynchronous operations
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Build the torrent from the given data.
    /// This is the same as calling `Torrent::try_from(self)`.
    pub fn build(self) -> Result<Torrent> {
        Torrent::try_from(self)
    }
}

impl TryFrom<TorrentRequest> for Torrent {
    type Error = TorrentError;

    fn try_from(request: TorrentRequest) -> Result<Self> {
        let metadata = request.metadata.ok_or(TorrentError::InvalidRequest(
            "metadata is missing".to_string(),
        ))?;
        let peer_listener_port = request
            .peer_listener_port
            .map(|e| Some(e))
            .unwrap_or_else(|| available_port!(6881, 31000))
            .ok_or(TorrentError::Io(
                "no available port found to start new peer listener".to_string(),
            ))?;
        let protocol_extensions = request
            .protocol_extensions
            .unwrap_or_else(DEFAULT_TORRENT_PROTOCOL_EXTENSIONS);
        let extensions = request
            .extensions
            .unwrap_or_else(DEFAULT_TORRENT_EXTENSIONS);
        let options = request.options.unwrap_or(TorrentFlags::default());
        let config = request
            .config
            .unwrap_or_else(|| TorrentConfig::builder().build());
        let storage = request.storage.ok_or(TorrentError::InvalidRequest(
            "file storage is missing".to_string(),
        ))?;
        let operations = request
            .operations
            .unwrap_or_else(DEFAULT_TORRENT_OPERATIONS);
        let request_strategies = request
            .request_strategies
            .unwrap_or_else(DEFAULT_TORRENT_REQUEST_STRATEGIES);
        let runtime = request
            .runtime
            .unwrap_or_else(|| Arc::new(Runtime::new().expect("expected a new runtime")));
        let peer_listener = Box::new(DefaultPeerListener::new(
            peer_listener_port,
            runtime.clone(),
        )?);

        Ok(Self::new(
            metadata,
            peer_listener,
            protocol_extensions,
            extensions,
            options,
            config,
            storage,
            operations,
            request_strategies,
            runtime,
        ))
    }
}

/// The torrent callbacks which are invoked when certain events occur.
pub type TorrentCallback = CoreCallback<TorrentEvent>;

/// A torrent operation which is executed in a chain during the lifetime of the torrent.
/// It provides a specific operation to be executed on the torrent in a sequential order.
///
/// The operation is always specific to one torrent, but should be allowed to create a new instance of the operation.
/// This allows the operation to store data which is specific to the torrent.
#[async_trait]
pub trait TorrentOperation: Debug + Display + Send + Sync {
    /// Execute the operation for the given torrent.
    /// The [TorrentData] structure exposes additional internal data of the torrent which is otherwise not exposed on the [Torrent].
    ///
    /// # Returns
    ///
    /// Returns [Some] if the chain needs to be continued, else [None].
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext>;

    /// Clone this operation into a new boxed instance.
    ///
    /// The new boxed instance should have a clean state if it stores data.
    fn clone_boxed(&self) -> Box<dyn TorrentOperation>;
}

/// A torrent request strategy which processes pending requests.
#[async_trait]
pub trait RequestStrategy: Debug + Display + Send + Sync {
    /// Determine if this strategy is supported for the given torrent.
    /// This determines if the strategy will be executed to create the pending request order.
    ///
    /// # Returns
    ///
    /// Returns true if the strategy is supports/applicable to the torrent, else false.
    async fn supports(&self, torrent: &TorrentContext) -> bool;

    /// Execute the request strategy to process pending requests.
    /// It should give a list of available peers to the context to accept and should not exceed the maximum number of requests.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_trait::async_trait;
    /// use derive_more::Display;    
    /// use popcorn_fx_torrent::torrents::{PendingRequestContext, RequestStrategy, TorrentContext};
    ///
    /// #[derive(Debug, Display)]
    /// #[display(fmt = "simple example")]
    /// pub struct Example{}
    ///
    /// #[async_trait]
    /// impl RequestStrategy for Example {
    ///     async fn supports(&self, torrent: &TorrentContext) -> bool {
    ///         true
    ///     }
    ///
    ///     async fn execute<'a>(&self, ctx: &'a PendingRequestContext<'a>, max_requests: usize) {
    ///         // select a request based on the strategy to process
    ///         let request = ctx.pending_requests_buffer.pending_requests().get(0).as_ref().unwrap();
    ///         // select the applicable peers based on the strategy
    ///         let available_peers = ctx.find_available_peers(&1).await;
    ///
    ///         ctx.accept(request, available_peers).await;
    ///     }
    ///
    ///     fn clone_boxed(&self) -> Box<dyn RequestStrategy> {
    ///         Box::new(Self{})
    ///     }
    /// }
    ///
    /// ```
    async fn execute<'a>(&self, ctx: &'a PendingRequestContext<'a>, max_requests: usize);

    /// Clone this strategy into a new boxed instance.
    /// If the strategy stores data, the new boxed instance should have a clean state.
    fn clone_boxed(&self) -> Box<dyn RequestStrategy>;
}

/// The context to execute a request strategy.
#[derive(Debug)]
pub struct PendingRequestContext<'a> {
    /// The pending requests buffer
    pub pending_requests_buffer: RwLockReadGuard<'a, PendingRequestBuffer>,
    /// The currently available peers
    pub peers: RwLockReadGuard<'a, Vec<Peer>>,
    /// The pieces of the torrent
    pub pieces: RwLockReadGuard<'a, Vec<Piece>>,
}

impl<'a> PendingRequestContext<'a> {
    /// Try to find all peers which have the given piece available.
    /// It returns all available peers that have the piece.
    pub async fn find_available_peers<'b>(&'b self, piece: &PieceIndex) -> Vec<&'b Peer> {
        let mut available_peers = Vec::new();

        for peer in self.peers.iter() {
            let peer_state = peer.state().await;
            if peer_state == PeerState::Closed {
                continue;
            }

            let is_piece_available = select! {
                _ = time::sleep(Duration::from_millis(200)) => {
                    warn!("Peer {} piece {} availability check timed out", peer, piece);
                    false
                },
                available = peer.remote_has_piece(*piece) => available,
            };

            if is_piece_available {
                available_peers.push(peer);
            }
        }

        available_peers
    }

    /// Accept the given request and request it from the given peer list.
    /// It returns true when the request has been accepted for the given peers, else false.
    pub async fn accept(&self, request: &PendingRequest, peers: Vec<&'a Peer>) -> bool {
        // check if we're allowed to accept the request
        if self.pending_requests_buffer.available_permits() == 0 || peers.len() == 0 {
            debug!(
                "Unable to accept {:?}, no permits available or no peers",
                request
            );
            return false;
        }

        let mut requested_from = Vec::new();
        for part in request.parts_to_request() {
            let selected_peer = peers.len().saturating_sub(1).min(part.part % peers.len());

            if let Some(peer) = peers.get(selected_peer) {
                let peer_handle = peer.handle();
                peer.request_piece_part(part).await;
                self.pending_requests_buffer.update_request_from(
                    request.piece(),
                    part.part,
                    peer_handle,
                );

                if !requested_from.contains(&peer_handle) {
                    requested_from.push(peer_handle);
                }
            } else {
                warn!(
                    "Selected peer index {} outside of the available peers list bounds {}",
                    selected_peer,
                    peers.len()
                );
            }
        }

        // check if any of the parts could be requested from one of the peers
        if requested_from.len() == 0 {
            trace!("Found no available peers for piece {}", request.piece());
            return false;
        }

        debug!(
            "Requested piece {} from {} peers",
            request.piece(),
            requested_from.len()
        );
        self.pending_requests_buffer.accept(request).await;
        true
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentEvent {
    /// Invoked when the status of the torrent has changed
    #[display(fmt = "torrent state has changed to {}", _0)]
    StateChanged(TorrentState),
    /// Invoked when the torrent metadata has been changed
    #[display(fmt = "torrent metadata has been changed")]
    MetadataChanged,
    /// Invoked when the active peer connections have changed
    PeersChanged,
    /// Invoked when the active trackers have been changed
    TrackersChanged,
    /// Invoked when the pieces have changed of the torrent
    #[display(fmt = "torrent pieces have changed")]
    PiecesChanged,
    /// Invoked when a piece has been completed.
    #[display(fmt = "piece {} has been completed", _0)]
    PieceCompleted(PieceIndex),
    /// Invoked when the files have changed of the torrent
    #[display(fmt = "torrent files have changed")]
    FilesChanged,
    /// Invoked when the torrent stats have been updated
    #[display(fmt = "torrent stats changed {}", _0)]
    Stats(TorrentTransferStats),
}

/// A torrent is an actual tracked torrent which is communicating with one or more trackers and peers.
///
/// Use [crate::torrents::TorrentInfo] if you only want to retrieve the metadata of a torrent.
#[derive(Debug)]
pub struct Torrent {
    handle: TorrentHandle,
    /// The unique peer id of this torrent
    /// This id is used as our client id when connecting to peers
    peer_id: PeerId,
    /// The port on which the torrent is listening for incoming peer connections
    peer_port: u16,
    /// The inner torrent instance reference holder
    instance: TorrentInstance,
    /// The shared runtime used by the torrent
    runtime: Arc<Runtime>,
}

impl Torrent {
    /// Create a new request builder for creating a new torrent.
    pub fn request() -> TorrentRequest {
        TorrentRequest::default()
    }

    fn new(
        metadata: TorrentInfo,
        peer_receiver: Box<dyn PeerListener>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        options: TorrentFlags,
        config: TorrentConfig,
        storage: Box<dyn TorrentFileStorage>,
        operations: Vec<Box<dyn TorrentOperation>>,
        request_strategies: Vec<Box<dyn RequestStrategy>>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let info_hash = metadata.info_hash.clone();
        let (event_sender, command_receiver) = unbounded_channel();
        let (tracker_sender, tracker_receiver) = tokio::sync::mpsc::channel(5);
        let cancellation_token = CancellationToken::new();
        let location = storage.path().to_path_buf();
        let inner = Arc::new(TorrentContext {
            handle,
            metadata: RwLock::new(metadata),
            peer_id,
            tracker_manager: TrackerManager::new(
                peer_id,
                peer_receiver.port(),
                info_hash,
                config.tracker_connection_timeout.clone(),
                tracker_sender,
                runtime.clone(),
            ),
            peer_pool: PeerPool::new(handle, config.peers_upper_limit),
            pieces: RwLock::new(Vec::with_capacity(0)),
            completed_pieces: RwLock::new(BitVec::with_capacity(0)),
            pending_requests: RwLock::new(PendingRequestBuffer::new(20)),
            piece_chunk_pool: PieceChunkPool::new(),
            files: RwLock::new(Vec::with_capacity(0)),
            protocol_extensions,
            extensions,
            storage,
            state: RwLock::new(Default::default()),
            options: RwLock::new(options),
            config: RwLock::new(config),
            stats: RwLock::new(TorrentTransferStats::default()),
            event_sender,
            callbacks: Default::default(),
            operations,
            request_strategies,
            cancellation_token,
        });

        let inner_main_loop = inner.clone();
        let torrent = Self {
            handle,
            peer_id,
            peer_port: peer_receiver.port(),
            instance: TorrentInstance::Owner(inner),
            runtime,
        };
        let torrent_main_loop = torrent.clone();

        // create a new separate thread which manages the internal torrent resources
        // this thread is automatically cancelled when the torrent is dropped
        torrent.runtime.spawn(async move {
            // start the main loop of the torrent
            inner_main_loop
                .start(
                    torrent_main_loop,
                    command_receiver,
                    tracker_receiver,
                    peer_receiver,
                )
                .await;
        });

        info!("Torrent {} created with location {:?}", torrent, location);
        torrent
    }

    /// Get the unique handle of this torrent.
    /// This handle identifies the torrent within a session.
    ///
    /// # Returns
    ///
    /// Returns the unique handle of this torrent.
    pub fn handle(&self) -> TorrentHandle {
        self.handle
    }

    /// Get the unique peer id of this torrent.
    /// This id is used within the peer clients to identify with remote peers.
    ///
    /// # Returns
    ///
    /// Returns the unique peer id of this torrent.
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get the port on which this torrent is listening for incoming peer connections.
    ///
    /// # Returns
    ///
    /// Returns the peer listener port.
    pub fn peer_port(&self) -> u16 {
        self.peer_port
    }

    /// Check if this torrent handle is still valid.
    ///
    /// # Returns
    ///
    /// Returns true if the handle is still valid, else false.
    pub fn is_valid(&self) -> bool {
        self.instance().is_some()
    }

    /// Get the current state of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the state of this torrent.
    pub async fn state(&self) -> TorrentState {
        match self.instance() {
            None => TorrentState::Error,
            Some(e) => e.state().await,
        }
    }

    /// Get the metadata of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the metadata of the torrent, or [TorrentError::InvalidHandle] when the torrent is invalid.
    pub async fn metadata(&self) -> Result<TorrentInfo> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;
        Ok(inner.metadata().await)
    }

    /// Check if the metadata of the torrent is known.
    /// It returns false when the torrent is still retrieving the metadata, else true.
    pub async fn is_metadata_known(&self) -> bool {
        if let Some(inner) = self.instance() {
            return inner.is_metadata_known().await;
        }
        false
    }

    /// Get the options of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the currently active options of the torrent
    pub async fn options(&self) -> TorrentFlags {
        if let Some(inner) = self.instance() {
            return inner.options_owned().await;
        }

        TorrentFlags::None
    }

    /// Add the given options to the torrent.
    ///
    /// It triggers the [TorrentEvent::OptionsChanged] event if the options changed.
    /// If the options are already present, this will be a no-op.
    pub async fn add_options(&self, options: TorrentFlags) {
        if let Some(inner) = self.instance() {
            inner.add_options(options).await;
        }
    }

    /// Remove the given options to the torrent.
    ///
    /// It triggers the [TorrentEvent::OptionsChanged] event if the options changed.
    /// If none of the given options are present, this will be a no-op.
    pub async fn remove_options(&self, options: TorrentFlags) {
        if let Some(inner) = self.instance() {
            inner.remove_options(options).await;
        }
    }

    /// Get the total amount of pieces for this torrent.
    /// If the metadata is still being retrieved, the total pieces cannot yet be known and this will result in 0.
    ///
    /// # Returns
    ///
    /// Returns the total pieces of this torrent when known.
    pub async fn total_pieces(&self) -> usize {
        if let Some(inner) = self.instance() {
            return inner.total_pieces().await;
        }

        0
    }

    /// Retrieve the torrent pieces, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the current torrent pieces when known, else [None].
    pub async fn pieces(&self) -> Option<Vec<Piece>> {
        if let Some(inner) = self.instance() {
            return inner.pieces().await;
        }

        None
    }

    /// Get the information about a specific piece within the torrent.
    /// If the pieces are not yet known, in case the metadata is still being retrieved, then it returns [None].
    ///
    /// If a piece index is requested out-of-bounds of the pieces, [None] will also be returned.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece index to retrieve the information of.
    ///
    /// # Returns
    ///
    /// Returns the piece info if available and found, otherwise [None].
    pub async fn piece_info(&self, piece: PieceIndex) -> Option<Piece> {
        if let Some(inner) = self.instance() {
            return inner
                .pieces
                .read()
                .await
                .iter()
                .find(|e| e.index == piece)
                .cloned();
        }

        None
    }

    /// Get the bitfield of the pieces indicating if a piece has been completed or not.
    /// It might return an empty bitfield if the torrent handle is no longer valid.
    ///
    /// # Returns
    ///
    /// Returns a bitfield which indicates if a piece has been completed or not.
    pub async fn piece_bitfield(&self) -> BitVec {
        if let Some(inner) = self.instance() {
            return inner.completed_pieces.read().await.clone();
        }

        BitVec::with_capacity(0)
    }

    /// Get the priorities of the pieces.
    /// It might return an empty array if the metadata is still being retrieved.
    pub async fn piece_priorities(&self) -> Vec<(PieceIndex, PiecePriority)> {
        if let Some(inner) = self.instance() {
            return inner.piece_priorities().await;
        }

        Vec::with_capacity(0)
    }

    /// Set the priorities of the pieces.
    /// Use [Torrent::piece_priorities] to get the current priorities with its [PieceIndex].
    ///
    /// Providing all piece indexes of the torrent is not required.
    pub async fn prioritize_pieces(&self, priorities: Vec<(PieceIndex, PiecePriority)>) {
        if let Some(inner) = self.instance() {
            inner.prioritize_pieces(priorities).await;
        }
    }

    /// Get of the given piece index has completed downloading, validating and written to the storage.
    ///
    /// # Returns
    ///
    /// Returns true if the piece has been downloaded, validated and written to storage, else false.
    pub async fn has_piece(&self, piece: PieceIndex) -> bool {
        if let Some(inner) = self.instance() {
            return inner.has_piece(piece).await;
        }

        false
    }

    /// Get if the given byte range has completed downloading, validating and written to the storage.
    ///
    /// # Returns
    ///
    /// Returns true if the bytes have been downloaded, validated and written to storage.
    pub async fn has_bytes(&self, range: &std::ops::Range<usize>) -> bool {
        if let Some(inner) = self.instance() {
            return inner.has_bytes(range).await;
        }

        false
    }

    /// Get the total files of the torrent.
    /// If the metadata is still being retrieved, the files cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the total files of the torrent when known, else [None].
    pub async fn total_files(&self) -> Option<usize> {
        if let Some(inner) = self.instance() {
            return Some(inner.total_files().await).filter(|e| e > &0);
        }

        None
    }

    /// Get the torrent files, if known.
    /// If the metadata is still being retrieved, the returned files array will be empty.
    ///
    /// # Returns
    ///
    /// Returns the torrent files when known.
    pub async fn files(&self) -> Vec<File> {
        if let Some(inner) = self.instance() {
            return inner.files().await;
        }

        Vec::with_capacity(0)
    }

    /// Set the priorities of the files.
    /// Use [Torrent::files] to get the current files with its [FileIndex].
    ///
    /// Providing all file indexes of the torrent is not required.
    pub async fn priorities_files(&self, priorities: Vec<(FileIndex, PiecePriority)>) {
        if let Some(inner) = self.instance() {
            inner.prioritize_files(priorities).await;
        }
    }

    /// Get the currently active peer connections of the torrent.
    pub async fn active_peer_connections(&self) -> usize {
        if let Some(inner) = self.instance() {
            return inner.active_peer_connections().await;
        }

        0
    }

    /// Announce this torrent to the known trackers.
    /// This will retrieve the announcement information from the trackers.
    ///
    /// # Returns
    ///
    /// Returns the announcement information, or [TorrentError::InvalidHandle] when the torrent is invalid.
    pub async fn announce(&self) -> Result<Announcement> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        // try to wait for at least 2 connections
        if inner.active_tracker_connections().await == 0 {
            let (tx, rx) = channel();
            let callback = inner.add_callback(Box::new(move |event| {
                if let TorrentEvent::TrackersChanged = event {
                    tx.send(()).unwrap()
                }
            }));

            let _ = rx
                .recv_timeout(Duration::from_secs(2))
                .map_err(|_| TorrentError::Timeout);
            let result = rx
                .recv_timeout(Duration::from_secs(2))
                .map_err(|_| TorrentError::Timeout);
            inner.remove_callback(callback);
            result?;
        }

        Ok(inner.announce_all().await)
    }

    /// Add the given metadata to the torrent.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// If the data was already known, this method does nothing.
    pub async fn add_metadata(&self, metadata: TorrentMetadata) {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle));

        match inner {
            Ok(inner) => {
                inner.add_metadata(metadata).await;
            }
            Err(e) => {
                error!("Failed to update metadata for torrent {}, {}", self, e);
            }
        }
    }

    /// Resume the downloading of the torrent data.
    pub async fn resume(&self) {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            inner.resume().await;
        }
    }

    /// Pause the current torrent.
    pub async fn pause(&self) {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            inner.pause().await;
        }
    }

    /// Try the read the data from the given piece.
    /// This doesn't verify if the bytes are valid and completed.
    ///
    /// # Arguments
    ///
    /// * `piece` - The index of the piece.
    ///
    /// # Returns
    ///
    /// Returns the piece data if available, else the error.
    pub async fn read_piece(&self, piece: PieceIndex) -> Result<Vec<u8>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        inner.read_piece(piece).await
    }

    /// Try to read the given piece bytes range.
    /// This doesn't verify if the bytes are valid and completed.
    ///
    /// # Arguments
    ///
    /// * `piece` - The index of the piece.
    ///
    /// # Returns
    ///
    /// Returns the piece data if available, else the error.
    pub async fn read_piece_bytes(
        &self,
        piece: PieceIndex,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        inner.read_piece_bytes(piece, range).await
    }

    /// Try to read the piece data for the given file.
    /// This will only try to read the piece data that is contained within the file, ignoring the bytes which overlap with another file.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_piece(&self, file: &File, piece: PieceIndex) -> Result<Vec<u8>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        inner.read_file_piece(file, piece).await
    }

    /// Get the piece part of the torrent based on the piece and the offset within the piece.
    /// It returns [None] if the piece part is unknown to this torrent.
    ///
    /// # Arguments
    ///
    /// * `piece` - The index of the piece.
    /// * `begin` - The offset within the piece.
    pub(crate) async fn piece_part(&self, piece: PieceIndex, begin: usize) -> Option<PiecePart> {
        if let Some(inner) = self.instance() {
            return inner.find_piece_part(piece, begin).await;
        }

        None
    }

    /// Notify this torrent about a new availability of a piece from a peer.
    /// This is a crate function to allow peers to send the torrent notifications about this event.
    pub(crate) fn notify_peer_has_piece(&self, piece: PieceIndex) {
        if let Some(inner) = self.instance() {
            inner.send_command_event(TorrentCommandEvent::PiecesAvailable(vec![piece]));
        }
    }

    /// Notify this torrent about a new availability of a piece from a peer.
    /// This is a crate function to allow peers to send the torrent notifications about this event.
    pub(crate) fn notify_peer_has_pieces(&self, pieces: Vec<PieceIndex>) {
        if let Some(inner) = self.instance() {
            inner.send_command_event(TorrentCommandEvent::PiecesAvailable(pieces));
        }
    }

    /// Notify the torrent that a peer has been closed.
    pub(crate) fn notify_peer_closed(&self, peer: PeerHandle) {
        if let Some(inner) = self.instance() {
            inner.send_command_event(TorrentCommandEvent::PeerClosed(peer));
        }
    }

    /// Notify the torrent that a pending request has been rejected by the remote peer.
    pub(crate) async fn pending_request_rejected(
        &self,
        piece: PieceIndex,
        begin: usize,
        peer: PeerHandle,
    ) {
        if let Some(inner) = self.instance() {
            if let Some(part) = inner.find_piece_part(piece, begin).await {
                inner.send_command_event(TorrentCommandEvent::PendingRequestRejected(
                    PendingRequestRejected {
                        part,
                        peer,
                        reason: RequestRejectedReason::RejectedByRemotePeer,
                    },
                ));
            } else {
                warn!(
                    "Unable to find rejected request part for piece {}, begin {} for {}",
                    piece, begin, self
                )
            }
        }
    }

    /// Notify this torrent about the completion of a piece.
    /// The torrent will then validate and store the completed piece data.
    pub(crate) fn piece_completed(&self, part: PiecePart, data: Vec<u8>) {
        if let Some(inner) = self.instance() {
            inner.send_command_event(TorrentCommandEvent::PiecePartCompleted(part, data));
        }
    }

    /// Notify the torrent of an invalid received piece part.
    pub(crate) fn invalid_piece_data_received(&self, part: PiecePart, peer: PeerHandle) {
        if let Some(inner) = self.instance() {
            inner.send_command_event(TorrentCommandEvent::PendingRequestRejected(
                PendingRequestRejected {
                    part,
                    peer,
                    reason: RequestRejectedReason::InvalidDataResponse,
                },
            ));
        }
    }

    /// Get a temporary strong reference to the inner torrent.
    pub(crate) fn instance(&self) -> Option<Arc<TorrentContext>> {
        match &self.instance {
            TorrentInstance::Owner(e) => Some(e.clone()),
            TorrentInstance::Borrowed(e) => e.upgrade(),
        }
    }
}

impl Callbacks<TorrentEvent> for Torrent {
    fn add_callback(&self, callback: TorrentCallback) -> CallbackHandle {
        self.instance()
            .map(|e| e.callbacks.add_callback(callback))
            .unwrap_or(CallbackHandle::new())
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.instance().map(|e| e.callbacks.remove_callback(handle));
    }
}

impl Clone for Torrent {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            peer_id: self.peer_id,
            peer_port: self.peer_port,
            instance: self.instance.clone(),
            runtime: self.runtime.clone(),
        }
    }
}

impl PartialEq for Torrent {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle && self.peer_id == other.peer_id
    }
}

impl Display for Torrent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle)
    }
}

impl Drop for Torrent {
    fn drop(&mut self) {
        // if the owning torrent gets dropped
        // we need to make sure that any running threads are cancelled on the inner torrent
        if let TorrentInstance::Owner(inner) = &self.instance {
            inner.cancellation_token.cancel();
            block_in_place_runtime(
                async {
                    inner.peer_pool.shutdown().await;
                },
                &self.runtime,
            );
        }
    }
}

/// The torrent instances owns the actual inner instance.
/// This prevents other [Torrent] references from keeping the torrent alive while the session has dropped it.
#[derive(Debug)]
enum TorrentInstance {
    Owner(Arc<TorrentContext>),
    Borrowed(Weak<TorrentContext>),
}

impl Clone for TorrentInstance {
    fn clone(&self) -> Self {
        match self {
            Self::Owner(inner) => Self::Borrowed(Arc::downgrade(inner)),
            Self::Borrowed(inner) => Self::Borrowed(inner.clone()),
        }
    }
}

/// The information of a pending request being rejected
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRequestRejected {
    pub part: PiecePart,
    pub peer: PeerHandle,
    pub reason: RequestRejectedReason,
}

/// The reason why a pending request was rejected
#[derive(Debug, Clone, PartialEq)]
pub enum RequestRejectedReason {
    /// Indicates that the received piece data was invalid
    InvalidDataResponse,
    /// Indicates that the remote peer rejected the request
    RejectedByRemotePeer,
}

/// The internal torrent command events which are executed on the main loop of the torrent.
/// These are triggered when certain events happen in the torrent, but are never exposed outside the [TorrentContext].
#[derive(PartialEq)]
pub enum TorrentCommandEvent {
    /// Indicates that the torrent options (flags) have changed
    OptionsChanged,
    /// Indicates that the torrent wants to connect to a new tracker
    ConnectToTracker(TrackerEntry),
    /// Indicates that the torrent wants to connect to the given peer addr
    ConnectToPeer(SocketAddr),
    /// Indicates that the given peer has been connected and needs to be managed by the torrent
    PeerConnected(Peer),
    /// Indicates that a peer has closed the connection
    PeerClosed(PeerHandle),
    /// Indicates that pieces have become available for download by a peer
    PiecesAvailable(Vec<PieceIndex>),
    /// Indicates that a piece part has been completed
    PiecePartCompleted(PiecePart, Vec<u8>),
    /// Indicates that a piece has been completed
    PieceCompleted(PieceIndex),
    /// Indicates that an invalid piece request response has been received by a peer
    PendingRequestRejected(PendingRequestRejected),
}

impl Debug for TorrentCommandEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentCommandEvent::OptionsChanged => write!(f, "OptionsChanged"),
            TorrentCommandEvent::ConnectToTracker(e) => write!(f, "ConnectToTracker({:?})", e),
            TorrentCommandEvent::ConnectToPeer(e) => write!(f, "ConnectToPeer({})", e),
            TorrentCommandEvent::PeerConnected(e) => write!(f, "PeerConnected({})", e),
            TorrentCommandEvent::PeerClosed(e) => write!(f, "PeerClosed({})", e),
            TorrentCommandEvent::PiecesAvailable(e) => {
                if e.len() <= 10 {
                    write!(f, "PiecesAvailable({:?})", e)
                } else {
                    write!(f, "PiecesAvailable([len {}])", e.len())
                }
            }
            TorrentCommandEvent::PiecePartCompleted(e, data) => {
                write!(f, "PiecePartCompleted({:?}, [size {}])", e, data.len())
            }
            TorrentCommandEvent::PieceCompleted(e) => write!(f, "PieceCompleted({})", e),
            TorrentCommandEvent::PendingRequestRejected(e) => {
                write!(f, "PendingRequestRejected({:?})", e)
            }
        }
    }
}

/// The torrent context data.
/// This context can be shared by multiple [Torrent] instances, but only one [Torrent] instance can own the context.
#[derive(Debug)]
pub struct TorrentContext {
    /// The unique immutable handle of the torrent
    handle: TorrentHandle,
    /// The unique immutable peer id of the torrent
    peer_id: PeerId,
    /// The torrent metadata information of the torrent
    /// This might still be incomplete if the torrent was created from a magnet link
    metadata: RwLock<TorrentInfo>,
    /// The manager of the trackers for the torrent
    tracker_manager: TrackerManager,

    /// The pool of peer connections
    peer_pool: PeerPool,

    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: RwLock<Vec<Piece>>,
    /// The completed pieces of the torrent
    completed_pieces: RwLock<BitVec>,
    /// The pending requests of this torrent
    pending_requests: RwLock<PendingRequestBuffer>,
    /// The pool which stores the received piece parts
    piece_chunk_pool: PieceChunkPool,

    /// The torrent files
    files: RwLock<Vec<File>>,
    /// The torrent file storage to store the data
    storage: Box<dyn TorrentFileStorage>,

    /// The immutable enabled protocol extensions for this torrent
    protocol_extensions: ProtocolExtensionFlags,
    /// The immutable extensions for this torrent
    extensions: Extensions,
    /// The torrent data processing operations chain
    operations: Vec<Box<dyn TorrentOperation>>,
    /// The torrent requests strategies
    request_strategies: Vec<Box<dyn RequestStrategy>>,

    /// The state of the torrent
    state: RwLock<TorrentState>,
    /// The torrent options that are set for this torrent
    options: RwLock<TorrentFlags>,
    /// The torrent configuration
    config: RwLock<TorrentConfig>,
    /// The data transfer stats of the torrent
    stats: RwLock<TorrentTransferStats>,
    /// The internal command event sender
    event_sender: UnboundedSender<TorrentCommandEvent>,
    /// The callbacks for the torrent events
    callbacks: CoreCallbacks<TorrentEvent>,
    cancellation_token: CancellationToken,
}

impl TorrentContext {
    /// Start the main loop of this torrent.
    /// It starts listening for events from different receivers and processes them accordingly.
    async fn start(
        &self,
        weak_ref: Torrent,
        mut command_receiver: UnboundedReceiver<TorrentCommandEvent>,
        mut tracker_receiver: Receiver<TrackerManagerEvent>,
        mut peer_receiver: Box<dyn PeerListener>,
    ) {
        // the interval used to execute periodic torrent operations
        let mut operations_tick = time::interval(Duration::from_secs(1));
        let mut cleanup_interval = time::interval(Duration::from_secs(30));

        // execute the operations at the beginning of the loop
        select! {
            _ = self.cancellation_token.cancelled() => return,
            _ = self.execute_operations_chain() => {}
        }

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                event = command_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_command_event(&weak_ref, event).await;
                    } else {
                        debug!("Torrent {} events channel closed", self);
                        break;
                    }
                }
                Some(event) = tracker_receiver.recv() => self.handle_tracker_event(event).await,
                Some(entry) = peer_receiver.recv() => self.handle_incoming_peer_connection(&weak_ref, entry).await,
                _ = operations_tick.tick() => {
                    self.execute_operations_chain().await;
                    self.update_stats().await;
                },
                _ = cleanup_interval.tick() => {
                    self.clean_peers().await;
                },
            }
        }

        trace!("Torrent {} main loop ended", self);
    }

    /// Get the peer pool of the torrent.
    pub fn peer_pool(&self) -> &PeerPool {
        &self.peer_pool
    }

    /// Get the state of the torrent.
    pub async fn state(&self) -> TorrentState {
        self.state.read().await.clone()
    }

    /// Get the options of the torrent.
    pub fn options(&self) -> &RwLock<TorrentFlags> {
        &self.options
    }

    /// Get an owned instance of the options of the torrent.
    pub async fn options_owned(&self) -> TorrentFlags {
        self.options.read().await.clone()
    }

    /// Get the currently active trackers of the torrent.
    pub async fn active_trackers(&self) -> Vec<Url> {
        self.tracker_manager.trackers().await
    }

    /// Get an owned instance of the metadata from the torrent.
    /// It returns an owned instance of the metadata.
    pub async fn metadata(&self) -> TorrentInfo {
        self.metadata.read().await.clone()
    }

    /// Get the metadata of the torrent.
    /// It returns a reference to the metadata lock.
    pub fn metadata_lock(&self) -> &RwLock<TorrentInfo> {
        &self.metadata
    }

    /// Check if the metadata of the torrent is known.
    /// It returns false when the torrent is still retrieving the metadata, else true.
    pub async fn is_metadata_known(&self) -> bool {
        self.metadata.read().await.info.is_some()
    }

    /// Get the total amount of actively connected peers.
    /// This only counts peers that have not been closed yet, so it can be smaller than the peer pool.
    pub async fn active_peer_connections(&self) -> usize {
        self.peer_pool.active_peer_connections().await
    }

    /// Get the total amount of active tracker connections.
    /// This only counts trackers which have at least made one successful announcement.
    pub async fn active_tracker_connections(&self) -> usize {
        self.tracker_manager.total_trackers().await
    }

    /// Get the total amount of pieces for this torrent.
    /// If the metadata is still being retrieved, the total pieces cannot yet be known and this will result in 0.
    ///
    /// # Returns
    ///
    /// Returns the total pieces of this torrent when known.
    pub async fn total_pieces(&self) -> usize {
        self.pieces.read().await.len()
    }

    /// Get the torrent pieces, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the current torrent pieces when known, else [None].
    pub async fn pieces(&self) -> Option<Vec<Piece>> {
        let pieces = self.pieces.read().await.clone();

        if pieces.len() > 0 {
            return Some(pieces);
        }

        None
    }

    /// Get the torrent pieces as a slice, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and an empty slice will be returned.
    pub fn pieces_lock(&self) -> &RwLock<Vec<Piece>> {
        &self.pieces
    }

    /// Get the wanted pieces for this torrent.
    /// This is based on the [PiecePriority] set within the pieces of this torrent.
    pub async fn wanted_pieces(&self) -> Vec<Piece> {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .cloned()
            .collect()
    }

    /// Get the amount of completed pieces which are wanted.
    pub async fn completed_wanted_pieces(&self) -> usize {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .filter(|e| e.is_completed())
            .count()
    }

    /// Get if the given piece is completed with downloading its data.
    /// It returns true if the piece is completed, validated and written to the storage, else false.
    pub async fn has_piece(&self, piece: PieceIndex) -> bool {
        self.completed_pieces
            .read()
            .await
            .get(piece)
            .unwrap_or(false)
    }

    /// Get if the given bytes have been completed downloading.
    /// It returns true if all bytes are completed, validated and written to the storage, else false.
    pub async fn has_bytes(&self, range: &std::ops::Range<usize>) -> bool {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| {
                let piece_range = e.torrent_byte_range();

                // check if there is any overlap with the given byte range and piece range
                piece_range.start < range.end && range.start < piece_range.end
            })
            .all(|e| e.is_completed())
    }

    /// Get the priorities of the known pieces.
    pub async fn piece_priorities(&self) -> Vec<(PieceIndex, PiecePriority)> {
        self.pieces
            .read()
            .await
            .iter()
            .map(|e| (e.index, e.priority))
            .collect()
    }

    /// Prioritize the given pieces within this torrent.
    pub async fn prioritize_pieces(&self, priorities: Vec<(PieceIndex, PiecePriority)>) {
        {
            let mut mutex = self.pieces.write().await;
            for (index, priority) in priorities {
                if let Some(piece) = mutex.get_mut(index) {
                    piece.priority = priority;
                }
            }

            let mut stats_mutex = self.stats.write().await;
            stats_mutex.total_size = mutex
                .iter()
                .filter(|e| e.priority != PiecePriority::None)
                .map(|e| e.length)
                .sum();
            stats_mutex.wanted_pieces = mutex
                .iter()
                .filter(|e| e.priority != PiecePriority::None)
                .count();
        }

        self.pending_requests.write().await.clear();
        debug!("Torrent {} piece priorities have been changed", self);
    }

    /// Get if this torrent has been completed downloading are wanted pieces.
    pub async fn is_completed(&self) -> bool {
        let mutex = self.pieces.read().await;

        trace!(
            "Checking is torrent {} is completed for {} pieces",
            self,
            mutex.len()
        );
        mutex
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .filter(|e| !e.is_completed())
            .count()
            == 0
    }

    /// Check if additional peers are wanted for the torrent.
    pub async fn is_peer_wanted(&self) -> bool {
        self.remaining_peer_connections_needed().await > 0
    }

    /// Calculate the additional wanted peer connections for the torrent.
    pub async fn remaining_peer_connections_needed(&self) -> usize {
        let options = self.options.read().await;

        if options.contains(TorrentFlags::Paused) {
            return 0;
        }

        let is_retrieving_data = options.contains(TorrentFlags::DownloadMode)
            || options.contains(TorrentFlags::Metadata);
        let peer_count = self.active_peer_connections().await;
        let config = self.config.read().await;
        let peer_lower_bound: usize = config.peers_lower_limit;
        let peer_upper_bound: usize = config.peers_upper_limit;

        if is_retrieving_data && peer_count < peer_upper_bound {
            return peer_upper_bound - peer_count;
        }

        if !is_retrieving_data && peer_count < peer_lower_bound {
            return peer_lower_bound - peer_count;
        }

        0
    }

    /// Get the related files to the given piece.
    /// This will check which file bytes are overlapping with the piece range.
    pub async fn find_relevant_files_for_piece(&self, piece: &Piece) -> Vec<File> {
        self.files
            .read()
            .await
            .iter()
            .filter(|e| e.contains(piece))
            .cloned()
            .collect::<Vec<File>>()
    }

    /// Try to find the [PiecePart] for the given piece and begin index.
    pub async fn find_piece_part(&self, piece: PieceIndex, begin: usize) -> Option<PiecePart> {
        self.pieces
            .read()
            .await
            .iter()
            .find(|e| e.index == piece)
            .and_then(|piece| piece.parts.iter().find(|part| part.begin == begin).cloned())
    }

    /// Check if the given file already exists within the storage.
    /// This doesn't verify if the file is valid and completed.
    pub fn file_exists(&self, file: &File) -> bool {
        self.storage.exists(&file.path)
    }

    /// Get the pieces for the given file.
    /// This will retrieve all overlapping pieces with the file.
    /// The last piece can be longer than the actual file if the piece overlaps with multiple files.
    ///
    /// # Returns
    ///
    /// Returns the cloned pieces for the given file.
    pub async fn file_pieces(&self, file: &File) -> Vec<Piece> {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| file.contains(e))
            .cloned()
            .collect()
    }

    /// Get the known files of the torrent.
    /// If the metadata is currently being retrieved, the returned array will be empty.
    pub async fn files(&self) -> Vec<File> {
        self.files.read().await.clone()
    }

    /// Get the currently known total files of the torrent.
    /// If the metadata is currently being retrieved, the returned result will be 0.
    pub async fn total_files(&self) -> usize {
        self.files.read().await.len()
    }

    /// Prioritize the files of the torrent.
    /// This will update the underlying piece priorities of each file.
    ///
    /// Providing all file indexes of the torrent is not required.
    pub async fn prioritize_files(&self, priorities: Vec<(FileIndex, PiecePriority)>) {
        let mut mutex = self.files.write().await;
        let mut piece_priorities = HashMap::new();

        for (file_index, priority) in priorities {
            if let Some(file) = mutex.get_mut(file_index) {
                let pieces = self.file_pieces(&file).await;

                // update the priority of the file
                file.priority = priority;

                // add the piece priorities that have to be updated
                for piece in pieces {
                    let entry = piece_priorities.entry(piece.index).or_insert(priority);
                    *entry = PiecePriority::from((*entry as u8).max(priority as u8));
                }
            } else {
                warn!(
                    "Invalid torrent file index {} given for {}",
                    file_index, self
                );
            }
        }

        self.prioritize_pieces(piece_priorities.into_iter().map(|(k, v)| (k, v)).collect())
            .await;
    }

    /// Get the list of currently discovered peers.
    pub async fn discovered_peers(&self) -> Vec<SocketAddr> {
        self.tracker_manager.discovered_peers().await
    }

    /// Try to add the given tracker to the tracker manager of this torrent.
    pub async fn add_tracker(&self, entry: TrackerEntry) -> Result<TrackerHandle> {
        let url = entry.url.clone();
        let handle = self.tracker_manager.add_tracker_entry(entry).await?;

        trace!(
            "Tracker {}({}) has been added to torrent {}",
            url,
            handle,
            self
        );
        Ok(handle)
    }

    /// Try to add the given tracker to the tracker manager of this torrent.
    /// This creates the tracker in a background task.
    pub async fn add_tracker_async(&self, entry: TrackerEntry) {
        self.tracker_manager.add_tracker_async(entry).await;
    }

    /// Add the given peer to this torrent.
    /// Duplicate peers will be ignored and dropped.
    async fn add_peer(&self, peer: Peer) {
        debug!("Adding peer {} to torrent {}", peer, self);
        if self.peer_pool.add_peer(peer).await {
            self.invoke_event(TorrentEvent::PeersChanged);
        }
    }

    /// Remove the given peer from the torrent as it has been closed.
    async fn remove_peer(&self, handle: PeerHandle) {
        trace!("Removing peer {} from torrent {}", handle, self);
        if let Some(peer) = self
            .peer_pool
            .peers
            .read()
            .await
            .iter()
            .find(|e| e.handle() == handle)
        {
            let mut mutex = self.pieces.write().await;
            let bitfield = peer.remote_piece_bitfield().await;

            // decrease the availability of the pieces that the peer had
            for (piece_index, _) in bitfield.iter().enumerate().filter(|(_, value)| *value) {
                if let Some(piece) = mutex.iter_mut().find(|e| e.index == piece_index) {
                    piece.decrease_availability();
                }
            }
        }

        self.peer_pool.remove_peer(handle).await;
    }

    async fn add_metadata(&self, metadata: TorrentMetadata) {
        let is_metadata_known: bool;
        let info_hash: InfoHash;

        {
            let mutex = self.metadata.read().await;
            is_metadata_known = mutex.info.is_some();
            info_hash = mutex.info_hash.clone();
        }

        // verify if the metadata of the torrent is already known
        // if so, we ignore this update
        if is_metadata_known {
            return;
        }

        // validate the received metadata against our info hash
        let is_metadata_invalid = metadata
            .info_hash()
            .map(|metadata_info_hash| metadata_info_hash != info_hash)
            .map_err(|e| {
                debug!(
                    "Failed to calculate the info hash from the received metadata of {}, {}",
                    self, e
                );
            })
            .unwrap_or(true);
        if is_metadata_invalid {
            debug!("Received invalid metadata for torrent {}", self);
            return;
        }

        {
            let mut mutex = self.metadata.write().await;
            (*mutex).info = Some(metadata);
            debug!("Updated metadata of {}", self);
        }

        self.invoke_event(TorrentEvent::MetadataChanged);
    }

    /// Announce the torrent to all trackers.
    /// It returns the announcement result collected from all active trackers.
    pub async fn announce_all(&self) -> Announcement {
        self.tracker_manager.announce_all().await
    }

    /// Announce to all the trackers without waiting for the results.
    pub async fn make_announce_all(&self) {
        self.tracker_manager.make_announce_all().await
    }

    /// Add the given options to the torrent.
    ///
    /// It triggers the [TorrentEvent::OptionsChanged] event if the options changed.
    /// If the options are already present, this will be a no-op.
    pub async fn add_options(&self, options: TorrentFlags) {
        // check if all the given options are already present
        // of so, this is a no-op
        if self.options.read().await.contains(options) {
            return;
        }

        {
            let mut mutex = self.options.write().await;
            *mutex |= options;
        }

        self.send_command_event(TorrentCommandEvent::OptionsChanged);
    }

    /// Remove the given options from the torrent.
    ///
    /// It triggers the [TorrentEvent::OptionsChanged] event if the options changed.
    /// If none of the given options are present, this will be a no-op.
    pub async fn remove_options(&self, options: TorrentFlags) {
        // check if any of the given options is actually present
        // of not, this is a no-op
        if !self.options.read().await.intersects(options) {
            return;
        }

        {
            let mut mutex = self.options.write().await;
            *mutex &= !options;
        }

        self.send_command_event(TorrentCommandEvent::OptionsChanged);
    }

    /// Update the state of this torrent.
    /// If the torrent is already in the given state, this will be a no-op.
    pub async fn update_state(&self, state: TorrentState) {
        // check if we're already in the expected state
        // if so, ignore this update
        if *self.state.read().await == state {
            return;
        }

        trace!("Updating state of {} to {}", self, state);
        {
            let mut mutex = self.state.write().await;
            *mutex = state.clone();
        }

        self.invoke_event(TorrentEvent::StateChanged(state));
    }

    // TODO: move this to an operation
    async fn update_stats(&self) {
        let mut peer_metrics = Vec::new();
        let mut mutex = self.stats.write().await;
        // start by resetting the rate based metrics
        mutex.reset();

        {
            let peer_mutex = self.peer_pool.peers.read().await;
            mutex.total_peers = peer_mutex.len();
            // only collect the metrics of peers that are not closed
            for peer in peer_mutex
                .iter()
                .filter(|e| block_in_place(e.state()) != PeerState::Closed)
            {
                // copy the peer metrics into a buffer to release the peers lock as soon as possible
                peer_metrics.push(peer.stats_and_reset().await);
            }
        }

        // process the collected peer metrics
        for peer_stats in peer_metrics.into_iter() {
            mutex.upload += peer_stats.upload;
            mutex.upload_rate += peer_stats.upload_rate;
            mutex.upload_useful += peer_stats.upload_useful;
            mutex.upload_useful_rate += peer_stats.upload_useful_rate;
            mutex.download += peer_stats.download;
            mutex.download_rate += peer_stats.download_rate;
            mutex.download_useful += peer_stats.download_useful;
            mutex.download_useful_rate += peer_stats.download_useful_rate;
            mutex.total_uploaded += peer_stats.upload;
            mutex.total_downloaded += peer_stats.download;
        }

        let event_metrics = mutex.clone();
        drop(mutex);
        self.invoke_event(TorrentEvent::Stats(event_metrics));
    }

    pub async fn update_pieces_availability(&self, pieces: Vec<PieceIndex>) {
        // check if the metadata is known and the pieces info has been created
        // if not, ignore this update
        if !self.is_metadata_known().await || self.total_pieces().await == 0 {
            return;
        }

        let mut mutex = self.pieces.write().await;
        for piece in pieces {
            match mutex.iter_mut().find(|e| e.index == piece) {
                None => warn!("Peer notified about an unknown piece {}", piece),
                Some(piece) => piece.increase_availability(),
            }
        }
    }

    /// Set the pieces of the torrent.
    pub async fn update_pieces(&self, pieces: Vec<Piece>) {
        trace!("Updating pieces of {}", self);
        {
            let mut mutex = self.pieces.write().await;
            *mutex = pieces;

            // update the piece availability based on the current peer connections
            let mut availability: HashMap<PieceIndex, usize> = HashMap::new();
            for peer in self.peer_pool.peers.read().await.iter() {
                let bitfield = peer.remote_piece_bitfield().await;
                for (piece_index, _) in bitfield.iter().enumerate().filter(|(_, value)| *value) {
                    *availability.entry(piece_index).or_insert(0) += 1;
                }
            }

            for piece in mutex.iter_mut() {
                if let Some(availability) = availability.get(&piece.index) {
                    piece.availability += *availability as u32;
                }
            }
            trace!(
                "Updated a total of {} piece availabilities for {}",
                availability.len(),
                self
            );
        }

        self.invoke_event(TorrentEvent::PiecesChanged);
    }

    pub async fn update_completed_pieces(&self, completed_pieces: BitVec) {
        trace!("Updating completed pieces of {}", self);
        {
            let mut mutex = self.completed_pieces.write().await;
            *mutex = completed_pieces;
        }
    }

    pub async fn update_piece_completed(&self, piece: PieceIndex) {
        trace!("Marking piece {} as completed for {}", piece, self);
        {
            let mut mutex = self.pieces.write().await;
            if let Some(piece) = mutex.get_mut(piece) {
                piece.mark_completed();
            }
        }
        {
            let mut mutex = self.completed_pieces.write().await;
            mutex.set(piece, true);
        }

        self.invoke_event(TorrentEvent::PieceCompleted(piece));

        if self.is_completed().await {
            self.update_state(TorrentState::Finished).await;
        }

        // notify the peers
        for peer in self.peer_pool.peers.read().await.iter() {
            peer.notify_have_piece(piece);
        }
    }

    /// Update the torrent files of the torrent.
    /// This replaces any existing files.
    pub async fn update_files(&self, files: Vec<File>) {
        trace!("Updating the files of {}", self);
        {
            let mut mutex = self.files.write().await;
            *mutex = files;
        }

        self.invoke_event(TorrentEvent::FilesChanged);
    }

    /// Get the pending requests of the torrent.
    /// It returns a reference to the internal pending requests buffer.
    pub fn pending_requests(&self) -> &RwLock<PendingRequestBuffer> {
        &self.pending_requests
    }

    /// Cancel all currently queued pending requests of the torrent.
    /// This will clear all pending requests from the buffer.
    pub async fn cancel_all_pending_requests(&self) {
        self.pending_requests.write().await.clear();
    }

    /// Resume the torrent operations.
    pub async fn resume(&self) {
        self.add_options(TorrentFlags::DownloadMode | TorrentFlags::Metadata)
            .await;
        self.remove_options(TorrentFlags::Paused).await;

        // announce to the trackers if we don't know any peers
        if self.peer_pool.available_peer_addrs_len().await == 0 {
            self.tracker_manager.make_announce_all().await;
        }
    }

    /// Pause the torrent operations.
    pub async fn pause(&self) {
        self.add_options(TorrentFlags::Paused).await;
        self.send_command_event(TorrentCommandEvent::OptionsChanged);
    }

    /// Handle a command event from the channel of the torrent.
    async fn handle_command_event(&self, torrent: &Torrent, event: TorrentCommandEvent) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            TorrentCommandEvent::OptionsChanged => self.process_options().await,
            TorrentCommandEvent::ConnectToTracker(e) => self.add_tracker_async(e).await,
            TorrentCommandEvent::ConnectToPeer(addr) => {
                self.create_peer_connection(torrent.clone(), addr)
            }
            TorrentCommandEvent::PeerConnected(peer) => self.add_peer(peer).await,
            TorrentCommandEvent::PeerClosed(handle) => self.remove_peer(handle).await,
            TorrentCommandEvent::PiecesAvailable(piece) => {
                self.update_pieces_availability(piece).await
            }
            TorrentCommandEvent::PiecePartCompleted(part, data) => {
                self.process_completed_piece_part(part, data).await
            }
            TorrentCommandEvent::PieceCompleted(piece) => self.process_completed_piece(piece).await,
            TorrentCommandEvent::PendingRequestRejected(request_rejection) => {
                self.process_pending_request_rejected(request_rejection)
                    .await
            }
        }
    }

    async fn handle_tracker_event(&self, event: TrackerManagerEvent) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            TrackerManagerEvent::PeersDiscovered(peers) => {
                let new_connections = self.remaining_peer_connections_needed().await;
                self.peer_pool.add_available_peer_addrs(peers).await;
                self.create_peer_connections(new_connections).await
            }
            TrackerManagerEvent::TrackerAdded(handle) => {
                let options = self.options_owned().await;
                let is_retrieving_metadata =
                    *self.state.read().await == TorrentState::RetrievingMetadata;
                let is_download_mode =
                    options & TorrentFlags::DownloadMode == TorrentFlags::DownloadMode;

                if is_retrieving_metadata || is_download_mode {
                    self.tracker_manager
                        .make_announcement(handle, AnnounceEvent::Started)
                        .await;
                }

                self.invoke_event(TorrentEvent::TrackersChanged);
            }
        }
    }

    async fn handle_incoming_peer_connection(&self, torrent: &Torrent, entry: PeerEntry) {
        trace!(
            "Trying to accept incoming {} peer connection for {}",
            entry.socket_addr,
            self
        );
        match Peer::new_inbound(
            entry.socket_addr,
            entry.stream,
            torrent.clone(),
            self.protocol_extensions,
            self.extensions(),
            torrent.runtime.clone(),
        )
        .await
        {
            Ok(peer) => {
                debug!("Established connection with peer {} for {}", peer, self);
                self.add_peer(peer).await;
            }
            Err(e) => debug!(
                "Failed to accept incoming peer connection for {}, {}",
                self, e
            ),
        }
    }

    async fn process_options(&self) {
        let mutex = self.options.read().await;
        trace!("Processing the options{:?} of {}", *mutex, self);

        if *mutex & TorrentFlags::Paused == TorrentFlags::Paused {
            // choke all the peers
            let peers = self.peer_pool.peers.read().await;
            for peer in peers.iter() {
                peer.pause();
            }
        } else if *mutex & TorrentFlags::UploadMode == TorrentFlags::UploadMode {
            // unchoke all the peers
            let peers = self.peer_pool.peers.read().await;
            for peer in peers.iter() {
                peer.resume();
            }
        }
    }

    /// Execute the torrent operations chain.
    ///
    /// This will execute the operations in order as defined by the chain.
    /// If an operation returns [None], the execution chain will be interrupted.
    async fn execute_operations_chain(&self) {
        for operation in self.operations.iter() {
            let start = Instant::now();
            let execution_result = operation.execute(&self).await;
            let elapsed = start.elapsed();
            trace!(
                "Took {} millis to execute {} for {}",
                elapsed.as_millis(),
                operation,
                self
            );
            if execution_result.is_none() {
                break;
            }
        }
    }

    async fn process_pending_request_rejected(&self, request_rejection: PendingRequestRejected) {
        trace!(
            "Retrying to request piece part {:?} for {}",
            request_rejection.part,
            self
        );

        let mutex = self.pending_requests.read().await;
        if let Some(request) = mutex
            .requests
            .iter()
            .find(|e| e.piece() == request_rejection.part.piece)
        {
            mutex.release_part(request, &request_rejection.part.part);
        }
    }

    async fn process_completed_piece_part(&self, piece_part: PiecePart, data: Vec<u8>) {
        let piece_length: usize;
        let piece_completed: bool;

        {
            let mut mutex = self.pieces.write().await;
            if let Some(piece) = mutex.iter_mut().find(|e| e.index == piece_part.piece) {
                piece.part_completed(piece_part.part);
                piece_length = piece.length;
                piece_completed = piece.is_completed();
                trace!(
                    "Piece {} (parts: {}, complete {}, completed: {:?}) updated for {}",
                    piece.index,
                    piece.parts.len(),
                    piece.is_completed(),
                    piece.completed_parts,
                    self
                );
            } else {
                warn!(
                    "Received unknown piece {} chunk data for {}",
                    piece_part.piece, self
                );
                return;
            }
        }

        match self
            .piece_chunk_pool
            .add_chunk(&piece_part, piece_length, data)
            .await
        {
            Ok(_) => {
                self.pending_requests
                    .write()
                    .await
                    .update_request_part_completed(piece_part.piece, piece_part.part);

                if piece_completed {
                    self.send_command_event(TorrentCommandEvent::PieceCompleted(piece_part.piece));
                }
            }
            Err(e) => warn!("Failed to add chunk data for {}, {}", self, e),
        }
    }

    async fn process_completed_piece(&self, piece: PieceIndex) {
        if let Some(data) = self.piece_chunk_pool.get(piece).await {
            let is_valid = self.validate_piece_data(piece, &data).await;

            if is_valid {
                self.write_piece_chunk(piece, data).await;
                self.update_piece_completed(piece).await;
                self.stats.write().await.completed_pieces = self.completed_wanted_pieces().await;
            } else {
                let mut mutex = self.pieces.write().await;
                if let Some(piece) = mutex.iter_mut().find(|e| e.index == piece) {
                    debug!(
                        "Retrying invalid received piece {} data for {}",
                        piece.index, self
                    );

                    // reset the piece completed parts
                    piece.reset_completed_parts();

                    // start the request over for the whole piece
                    self.pending_requests
                        .write()
                        .await
                        .push(PendingRequest::new(piece.index, piece.parts.clone()));
                }
            }
        } else {
            warn!(
                "Piece chunk data of {} is not available for {}",
                piece, self
            );
        }
    }

    /// Validate if the given piece data is valid.
    /// It retrieves the known piece hash from the pieces map and checks if the hash matches the data.
    ///
    /// ## Remarks
    ///
    /// If an unknown [PieceIndex] is given, it will always be assumed as invalid as there is no way to validate the data.
    pub async fn validate_piece_data(&self, piece: PieceIndex, data: &[u8]) -> bool {
        let mut is_valid = false;

        if let Some(hash) = self
            .pieces
            .read()
            .await
            .iter()
            .find(|e| e.index == piece)
            .map(|e| e.hash.clone())
        {
            let is_v2_hash = hash.has_v2();

            if is_v2_hash {
                let actual_hash = Sha256::digest(&data).to_vec();
                is_valid = hash.hash_v2().unwrap() == actual_hash;
            } else {
                let actual_hash = Sha1::digest(&data).to_vec();
                is_valid = hash.hash_v1().unwrap().as_slice() == actual_hash.as_slice();
            }
        } else {
            warn!(
                "Unable to validate piece data, piece {} is unknown within {}",
                piece, self
            );
        }

        is_valid
    }

    /// Try to read the piece data for the given file.
    /// This will only try to read the piece data that is contained within the file, ignoring the bytes which overlap with another file.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_piece(&self, file: &File, piece: PieceIndex) -> Result<Vec<u8>> {
        if let Some(piece) = self.pieces.read().await.get(piece) {
            if let Some((file_offset, piece_range)) = file.torrent_piece_byte_range(piece) {
                let file_range = file_offset..(file_offset + piece_range.len());

                return self.read_file_bytes(file, file_range).await;
            }
        }

        Err(TorrentError::DataUnavailable)
    }

    /// Try to read the given byte range from the torrent file.
    ///
    /// # Arguments
    ///
    /// * `file` - The torrent file to read from.
    /// * `range` - The file byte range to read.
    ///
    /// ## Remarks
    ///
    /// This doesn't check if the piece was completed or not.
    pub async fn read_file_bytes(
        &self,
        file: &File,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>> {
        if range.end > file.length {
            return Err(TorrentError::InvalidRange(range));
        }

        Ok(self.storage.read(&file.path, range).await?)
    }

    /// Try to read the given piece bytes.
    /// It will read the bytes from all relevant files which overlap with the given piece.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_piece(&self, piece: PieceIndex) -> Result<Vec<u8>> {
        if let Some(piece) = self.pieces.read().await.get(piece) {
            let files = self.find_relevant_files_for_piece(&piece).await;
            let mut buffer = Vec::with_capacity(piece.len());

            for file in files {
                if let Some((file_offset, piece_range)) = file.torrent_piece_byte_range(&piece) {
                    let file_range = file_offset..(file_offset + piece_range.len());

                    buffer.extend_from_slice(&self.read_file_bytes(&file, file_range).await?)
                } else {
                    // this should normally never occur
                    warn!("Unable to find the file bytes for the given piece");
                    return Err(TorrentError::DataUnavailable);
                }
            }

            return Ok(buffer);
        }

        Err(TorrentError::DataUnavailable)
    }

    /// Try to read the given piece bytes range.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_piece_bytes(
        &self,
        piece: PieceIndex,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>> {
        self.read_piece(piece).await.map(|e| e[range].to_vec())
    }

    /// Create new peer connections for the available peer addresses and the number of wanted new connections.
    pub async fn create_peer_connections(&self, wanted_connections: usize) {
        if wanted_connections == 0 {
            return;
        }

        let peer_addrs = self
            .peer_pool
            .take_available_peer_addrs(wanted_connections)
            .await;
        let request_peers = peer_addrs.len();

        for peer_addr in peer_addrs {
            self.send_command_event(TorrentCommandEvent::ConnectToPeer(peer_addr));
        }

        debug!(
            "Requested a total of {} new peer connections for {}",
            request_peers, self
        );
    }

    /// Create a new peer connection for the torrent and the given address.
    /// This process is spawned into a new thread.
    ///
    /// The passed [Torrent] should always be a weak reference to this instance.
    fn create_peer_connection(&self, torrent: Torrent, peer_addr: SocketAddr) {
        let protocol_extensions = self.protocol_extensions.clone();
        let extensions = self.extensions();
        let event_sender = self.event_sender.clone();
        let runtime = torrent.runtime.clone();
        debug!(
            "Trying to create a new peer connection {} for {}",
            peer_addr, self
        );
        runtime.spawn(async move {
            let handle_info = torrent.handle();
            match Self::try_create_peer_connection(
                torrent,
                peer_addr,
                protocol_extensions,
                extensions,
            )
            .await
            {
                Ok(peer) => {
                    let _ = event_sender.send(TorrentCommandEvent::PeerConnected(peer));
                }
                Err(e) => debug!(
                    "Failed to create peer connection for torrent {}, {}",
                    handle_info, e
                ),
            }
        });
    }

    /// Cleanup the peer resources which have been closed or are no longer valid.
    async fn clean_peers(&self) {
        trace!("Executing peer cleanup cycle for {}", self);
        self.peer_pool.clean().await;
    }

    async fn write_piece_chunk(&self, piece_index: PieceIndex, data: Vec<u8>) {
        let pieces = self.pieces.read().await;
        let piece = pieces
            .iter()
            .find(|e| e.index == piece_index)
            .expect("expected the piece index to be valid");

        // get all files that have this piece
        let relevant_files = self.find_relevant_files_for_piece(piece).await;

        if relevant_files.is_empty() {
            warn!(
                "Unable to find the files relevant to piece {} for {}",
                piece_index, self
            );
        }

        trace!(
            "Writing piece {} to {} relevant files",
            piece_index,
            relevant_files.len()
        );
        for file in relevant_files {
            // if the file priority is none, then skip writing to this file
            // this can happen if a piece overlaps with multiple files and only one file has a priority
            if file.priority == FilePriority::None {
                trace!(
                    "Ignoring file {:?} chunk write as it has priority None",
                    file
                );
                continue;
            }

            if let Some((offset, range)) = file.data_byte_range(piece) {
                if data.len() < range.end {
                    error!(
                        "Data range is out of bounds for piece {}, data size: {}, range end: {}",
                        piece_index,
                        data.len(),
                        range.end
                    );
                    continue;
                }

                if let Err(_) = self.storage.write(&file.path, offset, &data[range]).await {
                    error!(
                        "Failed to write the piece chunk data of {} for {}",
                        piece_index, self
                    );
                    self.update_state(TorrentState::Error).await;
                    return;
                }
            }
        }
    }

    /// Get the known extensions of the torrent.
    /// It returns owned instance of the extensions.
    pub fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_boxed()).collect()
    }

    /// Get the known request strategies of the torrent.
    /// These can be used to prioritize the requests in a particular order.
    pub fn request_strategies_ref<'a>(&'a self) -> &'a [Box<dyn RequestStrategy>] {
        self.request_strategies.as_slice()
    }

    /// Send an internal command event for this torrent.
    /// It will queue the command for execution on the main loop thread.
    pub fn send_command_event(&self, event: TorrentCommandEvent) {
        if let Err(e) = self.event_sender.send(event) {
            warn!(
                "Failed to send command event of {} to the main loop, {}",
                self, e
            );
        }
    }

    /// Invoke the given torrent event for all registered callbacks.
    pub fn invoke_event(&self, event: TorrentEvent) {
        self.callbacks.invoke(event)
    }

    async fn try_create_peer_connection(
        torrent: Torrent,
        peer_addr: SocketAddr,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
    ) -> Result<Peer> {
        let runtime = torrent.runtime.clone();
        Ok(
            Peer::new_outbound(peer_addr, torrent, protocol_extensions, extensions, runtime)
                .await?,
        )
    }
}

impl Callbacks<TorrentEvent> for TorrentContext {
    fn add_callback(&self, callback: CoreCallback<TorrentEvent>) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.callbacks.remove_callback(handle)
    }
}

impl Display for TorrentContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle)
    }
}

impl PartialEq for TorrentContext {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Drop for TorrentContext {
    fn drop(&mut self) {
        trace!("Torrent {} is being dropped", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::operations::{
        TorrentFileValidationOperation, TorrentFilesOperation, TorrentPendingRequestsOperation,
        TorrentPiecesOperation, TorrentRetrievePendingRequestsOperation,
    };
    use crate::torrents::peers::extensions::metadata::MetadataExtension;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file_to_bytes};
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;

    #[test]
    fn test_torrent_announce() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            DEFAULT_TORRENT_OPERATIONS(),
        );

        let result = torrent.runtime.block_on(torrent.announce()).unwrap();

        assert_ne!(
            0, result.total_seeders,
            "expected seeders to have been found"
        );
        assert_ne!(0, result.peers.len(), "expected peers to have been found");
    }

    #[test]
    fn test_torrent_metadata() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "debian-udp.torrent";
        let torrent_info_data = read_test_file_to_bytes(filename);
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = create_torrent_from_uri(filename, temp_path, TorrentFlags::None, vec![]);

        let metadata = torrent.runtime.block_on(torrent.metadata()).unwrap();

        assert_eq!(torrent_info, metadata);
    }

    #[test]
    fn test_retrieve_metadata() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:2C6B6858D61DA9543D4231A71DB4B1C9264B0685&dn=Ubuntu%2022.04%20LTS&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let port = available_port!(9000, 31000).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = channel();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(port)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .extensions(vec![Box::new(MetadataExtension::new())])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::MetadataChanged = event {
                tx.send(event).unwrap();
            }
        }));

        let _ = rx.recv_timeout(Duration::from_secs(30)).unwrap();
        let result = runtime.block_on(torrent.metadata()).unwrap();

        assert_ne!(
            None, result.info,
            "expected the metadata to have been present"
        );
    }

    #[test]
    fn test_prioritize_pieces() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx_pieces_event, rx_pieces_event) = channel();
        let torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentPiecesOperation::new()),
                Box::new(TorrentFilesOperation::new()),
            ],
        );

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::PiecesChanged = event {
                tx_pieces_event.send(event).unwrap();
            }
        }));

        // wait for the pieces to be created before trying to download the data
        let _ = rx_pieces_event
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the pieces to have been created");

        // only request the first piece
        let mut priorities = torrent.runtime.block_on(torrent.piece_priorities());
        for priority in &mut priorities[1..] {
            priority.1 = PiecePriority::None;
        }
        torrent
            .runtime
            .block_on(torrent.prioritize_pieces(priorities));

        let result = torrent
            .runtime
            .block_on(torrent.pieces())
            .expect("expected the pieces to be present");
        assert_eq!(PiecePriority::Normal, result[0].priority);
        assert_eq!(PiecePriority::None, result[1].priority);
    }

    #[test]
    fn test_piece_part() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_piece_part = PiecePart {
            piece: 0,
            part: 1,
            begin: 16384,
            length: 16384,
        };
        let (tx, rx) = channel();
        let torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentPiecesOperation::new()),
                Box::new(TorrentFilesOperation::new()),
            ],
        );

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::PiecesChanged = event {
                tx.send(()).unwrap();
            }
        }));

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the pieces to have been created");

        let result = torrent.runtime.block_on(torrent.piece_part(0, 16000));
        assert_eq!(
            None, result,
            "expected no piece part to be returned for invalid begin"
        );

        let result = torrent.runtime.block_on(torrent.piece_part(0, 16384));
        assert_eq!(Some(expected_piece_part), result, "expected the piece part");
    }

    #[test]
    fn test_resume() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let num_of_pieces = 10;
        let torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::Metadata,
            DEFAULT_TORRENT_OPERATIONS(),
        );

        torrent.runtime.block_on(async {
            let (tx_state, rx_state) = channel();
            let (tx_ready, rx_pieces_event) = channel();

            torrent.add_callback(Box::new(move |event| {
                if let TorrentEvent::StateChanged(state) = event {
                    if state == TorrentState::Finished {
                        tx_state.send(()).unwrap();
                    }
                } else if let TorrentEvent::PiecesChanged = event {
                    tx_ready.send(event).unwrap();
                }
            }));

            // wait for the pieces to be created before trying to download the data
            let _ = rx_pieces_event
                .recv_timeout(Duration::from_millis(1500))
                .expect("expected the pieces to have been created");

            // only request the first 2 pieces
            let mut priorities = torrent.piece_priorities().await;
            for priority in &mut priorities[num_of_pieces..] {
                priority.1 = PiecePriority::None;
            }
            torrent.prioritize_pieces(priorities).await;
            torrent.resume().await;

            // wait for a piece to be completed
            let _ = rx_state
                .recv_timeout(Duration::from_secs(180))
                .expect("expected the torrent to enter the FINISHED state");

            let pieces = torrent.pieces().await.unwrap();
            let pieces_bitfield = torrent.piece_bitfield().await;

            info!("Checking pieces stored in {}", temp_path);
            for piece in &pieces[0..num_of_pieces] {
                let piece_index = piece.index;
                assert_eq!(
                    true,
                    piece.is_completed(),
                    "expected piece {} to have been completed",
                    piece_index
                );
                assert_eq!(
                    Some(true),
                    pieces_bitfield.get(piece_index),
                    "expected piece bitfield bit {} to be set",
                    piece_index
                );
            }
        });
    }

    #[test]
    fn test_resume_internal() {
        init_logger();
        let temp_dir_source = tempdir().unwrap();
        let temp_path_source = temp_dir_source.path().to_str().unwrap();
        let temp_dir_target = tempdir().unwrap();
        let temp_path_target = temp_dir_target.path().to_str().unwrap();
        let num_of_pieces = 10;
        copy_test_file(
            temp_path_source,
            "piece-1_10.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let source_torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path_source,
            TorrentFlags::DownloadMode | TorrentFlags::UploadMode | TorrentFlags::Metadata,
            vec![
                Box::new(TorrentPiecesOperation::new()),
                Box::new(TorrentFilesOperation::new()),
                Box::new(TorrentFileValidationOperation::new()),
                Box::new(TorrentPendingRequestsOperation::new()),
            ],
        );
        let target_torrent = create_torrent_from_uri(
            "debian-udp.torrent",
            temp_path_target,
            TorrentFlags::Metadata,
            vec![
                Box::new(TorrentPiecesOperation::new()),
                Box::new(TorrentFilesOperation::new()),
                Box::new(TorrentPendingRequestsOperation::new()),
                Box::new(TorrentRetrievePendingRequestsOperation::new()),
            ],
        );

        target_torrent.runtime.block_on(async {
            let (tx_state, rx_state) = channel();
            let (tx_ready, rx_ready) = channel();

            let tx_source_ready = tx_ready.clone();
            source_torrent.add_callback(Box::new(move |event| {
                if let TorrentEvent::StateChanged(state) = event {
                    if state != TorrentState::Initializing && state != TorrentState::CheckingFiles {
                        tx_source_ready.send(()).unwrap();
                    }
                }
            }));
            target_torrent.add_callback(Box::new(move |event| {
                if let TorrentEvent::StateChanged(state) = event {
                    if state == TorrentState::Finished {
                        tx_state.send(()).unwrap();
                    }
                } else if let TorrentEvent::PiecesChanged = event {
                    tx_ready.send(()).unwrap();
                }
            }));

            // wait for the pieces to be created before trying to download the data
            let _ = rx_ready
                .recv_timeout(Duration::from_millis(2000))
                .expect("expected the files to have been created");
            let _ = rx_ready
                .recv_timeout(Duration::from_millis(2000))
                .expect("expected the files to have been created");

            // only request the first 2 pieces
            let mut priorities = target_torrent.piece_priorities().await;
            for priority in &mut priorities[num_of_pieces..] {
                priority.1 = PiecePriority::None;
            }
            target_torrent.prioritize_pieces(priorities).await;
            target_torrent.resume().await;

            // connect the source torrent to the target torrent
            let source_peer_addr = SocketAddr::from(([127, 0, 0, 1], target_torrent.peer_port()));
            let source_peer_runtime = source_torrent.runtime.clone();
            let source_peer = Peer::new_outbound(
                source_peer_addr,
                source_torrent,
                DEFAULT_TORRENT_PROTOCOL_EXTENSIONS(),
                DEFAULT_TORRENT_EXTENSIONS(),
                source_peer_runtime,
            )
            .await
            .expect("expected the outgoing peer to be created");

            source_peer.resume();

            // wait for a piece to be completed
            let _ = rx_state
                .recv_timeout(Duration::from_secs(60))
                .expect("expected the torrent to enter the FINISHED state");

            let pieces = target_torrent
                .pieces()
                .await
                .expect("expected the pieces to have been created");
            let pieces_bitfield = target_torrent.piece_bitfield().await;

            for piece in &pieces[0..num_of_pieces] {
                let piece_index = piece.index;
                assert_eq!(
                    true,
                    piece.is_completed(),
                    "expected piece {} to have been completed",
                    piece_index
                );
                assert_eq!(
                    Some(true),
                    pieces_bitfield.get(piece_index),
                    "expected piece bitfield bit {} to be set",
                    piece_index
                );
            }
        });
    }

    #[test]
    fn test_torrent_create_pieces() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let port = available_port!(6881, 31000).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(port)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let (tx, rx) = channel();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::PiecesChanged = event {
                tx.send(event).unwrap();
            }
        }));

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(1500)).unwrap();
        let inner = torrent.instance();
        let pieces = runtime.block_on(torrent.pieces()).unwrap();
        let completed_pieces = runtime.block_on(inner.as_ref().unwrap().completed_pieces.read());

        assert_ne!(0, pieces.len(), "expected the pieces to have been created");
        assert_ne!(
            0,
            completed_pieces.len(),
            "expected the completed pieces to have been created"
        );
    }

    #[test]
    fn test_torrent_create_files() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(6881, 31000).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(port)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let (tx, rx) = channel();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::FilesChanged = event {
                tx.send(event).unwrap();
            }
        }));

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(1500)).unwrap();
        let files = runtime.block_on(torrent.files());

        assert_eq!(1, files.len(), "expected the files to have been created");
    }

    /// Create a new torrent instance from the given uri.
    /// The uri can either be a [Magnet] uri or a filename to a torrent file within the testing resources.
    fn create_torrent_from_uri(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        operations: TorrentOperations,
    ) -> Torrent {
        let torrent_info: TorrentInfo;

        if uri.starts_with("magnet:") {
            let magnet = Magnet::from_str(uri).unwrap();
            torrent_info = TorrentInfo::try_from(magnet).unwrap();
        } else {
            let torrent_info_data = read_test_file_to_bytes(uri);
            torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        }

        let port = available_port!(6881, 31000).unwrap();

        Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(port)
            .options(options)
            .operations(operations)
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_dir)))
            .runtime(Arc::new(Runtime::new().unwrap()))
            .build()
            .unwrap()
    }
}
