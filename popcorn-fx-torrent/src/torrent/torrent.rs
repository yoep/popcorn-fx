use bit_vec::BitVec;
use bitmask_enum::bitmask;
use derive_more::Display;
use log::{debug, error, info, trace, warn};

use crate::torrent::file::{File, FilePriority};
use crate::torrent::fs::TorrentFileStorage;
use crate::torrent::peer::extension::{Extension, Extensions};
use crate::torrent::peer::{
    ConnectionType, DefaultPeerListener, Peer, PeerEntry, PeerEvent, PeerHandle, PeerId,
    PeerListener, PeerState, ProtocolExtensionFlags,
};
use crate::torrent::peer_pool::PeerPool;
use crate::torrent::tracker::{
    AnnounceEvent, Announcement, TrackerEntry, TrackerHandle, TrackerManager, TrackerManagerEvent,
};
use crate::torrent::{
    FileIndex, InfoHash, Piece, PieceChunkPool, PieceIndex, PiecePart, PiecePriority, Result,
    TorrentError, TorrentInfo, TorrentMetadata, DEFAULT_TORRENT_EXTENSIONS,
    DEFAULT_TORRENT_OPERATIONS, DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
};
use async_trait::async_trait;
use itertools::Itertools;
use popcorn_fx_core::available_port;
use popcorn_fx_core::core::callback::{Callback, MultiCallback, Subscriber, Subscription};
use popcorn_fx_core::core::Handle;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Filter;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::slice::Iter;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Notify, OwnedSemaphorePermit, RwLock, RwLockReadGuard, Semaphore};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

const DEFAULT_PEER_TIMEOUT_SECONDS: u64 = 6;
const DEFAULT_TRACKER_TIMEOUT_SECONDS: u64 = 3;
const DEFAULT_PEER_LOWER_LIMIT: usize = 10;
const DEFAULT_PEER_UPPER_LIMIT: usize = 100;
const DEFAULT_PEER_IN_FLIGHT: usize = 25;
const DEFAULT_PEER_UPLOAD_SLOTS: usize = 25;
const DEFAULT_MAX_IN_FLIGHT_PIECES: usize = 100;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// The chain of torrent operations that are executed for each torrent.
pub type TorrentOperations = Vec<Box<dyn TorrentOperation>>;

/// A [Torrent] extension factory.
/// This factory will create a new instance of an [Extension] for each new torrent.
pub type ExtensionFactory = fn() -> Box<dyn Extension>;

/// A list of [Torrent] extension factories.
pub type ExtensionFactories = Vec<ExtensionFactory>;

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
    /// The torrent is trying to retrieve the metadata from peers.
    #[display(fmt = "retrieving metadata")]
    RetrievingMetadata,
    /// The torrent has not started its download yet, and is currently checking existing files.
    #[display(fmt = "checking files")]
    CheckingFiles,
    /// The torrent is being downloaded. This is the state most torrents will be in most of the time.
    #[display(fmt = "downloading")]
    Downloading,
    /// In this state the torrent has finished downloading but still doesn't have the entire torrent.
    #[display(fmt = "finished")]
    Finished,
    /// In this state the torrent has finished downloading and is a pure seeder.
    #[display(fmt = "seeding")]
    Seeding,
    /// The torrent is currently paused and no longer executing any operations.
    #[display(fmt = "paused")]
    Paused,
    /// The torrent encountered an unrecoverable error.
    #[display(fmt = "error")]
    Error,
}

impl TorrentState {
    /// Check if the current state is an initialization phase state.
    pub fn is_initializing_phase(&self) -> bool {
        self == &TorrentState::Initializing
            || self == &TorrentState::RetrievingMetadata
            || self == &TorrentState::CheckingFiles
    }
}

impl Default for TorrentState {
    fn default() -> Self {
        Self::Initializing
    }
}

/// The torrent data transfer statistics.
/// These statics both include rate based- and lifetime metrics.
#[derive(Debug, Default, Clone, PartialEq)]
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
    /// The total bytes of piece data that have downloaded.
    pub total_downloaded_useful: usize,
    /// The total amount of pieces which are wanted by the torrent
    pub wanted_pieces: usize,
    /// The amount of pieces which have been completed by the torrent
    pub completed_pieces: usize,
    /// The total size, in bytes, of all interested files of the torrent.
    pub total_size: usize,
    /// The size in bytes of the pieces that have already been completed.
    pub total_completed_size: usize,
    /// The currently total active peer connections.
    pub total_peers: usize,
}

impl TorrentTransferStats {
    /// Get the progress, as a percentage, of the torrent download.
    /// The value returned is between 0.0 and 1.0.
    pub fn progress(&self) -> f32 {
        let progress: f32;
        if self.total_size == 0 {
            if self.wanted_pieces == 0 {
                return 100.0;
            }

            progress = self.completed_pieces as f32 / self.wanted_pieces as f32;
        } else {
            progress = self.total_completed_size as f32 / self.total_size as f32;
        }

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
    pub peers_in_flight: usize,
    pub peers_upload_slots: usize,
    pub peer_connection_timeout: Duration,
    pub tracker_connection_timeout: Duration,
    pub max_in_flight_pieces: usize,
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
    peers_in_flight: Option<usize>,
    peers_upload_slots: Option<usize>,
    peer_connection_timeout: Option<Duration>,
    tracker_connection_timeout: Option<Duration>,
    max_in_flight_pieces: Option<usize>,
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

    /// Set the max number of peer upload slots.
    pub fn peers_upload_slots(mut self, slots: usize) -> Self {
        self.peers_upload_slots = Some(slots);
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

    /// Set the maximum number of in flight pieces which can be requested in parallel from peers.
    pub fn max_in_flight_pieces(mut self, limit: usize) -> Self {
        self.max_in_flight_pieces = Some(limit);
        self
    }

    /// Build the torrent configuration.
    pub fn build(self) -> TorrentConfig {
        let peers_lower_limit = self.peers_lower_limit.unwrap_or(DEFAULT_PEER_LOWER_LIMIT);
        let peers_upper_limit = self.peers_upper_limit.unwrap_or(DEFAULT_PEER_UPPER_LIMIT);
        let peers_in_flight = self.peers_in_flight.unwrap_or(DEFAULT_PEER_IN_FLIGHT);
        let peers_upload_slots = self.peers_upload_slots.unwrap_or(DEFAULT_PEER_UPLOAD_SLOTS);
        let peer_connection_timeout = self
            .peer_connection_timeout
            .unwrap_or(Duration::from_secs(DEFAULT_PEER_TIMEOUT_SECONDS));
        let tracker_connection_timeout = self
            .tracker_connection_timeout
            .unwrap_or(Duration::from_secs(DEFAULT_TRACKER_TIMEOUT_SECONDS));
        let max_in_flight_pieces = self
            .max_in_flight_pieces
            .unwrap_or(DEFAULT_MAX_IN_FLIGHT_PIECES);

        TorrentConfig {
            peers_lower_limit,
            peers_upper_limit,
            peers_in_flight,
            peers_upload_slots,
            peer_connection_timeout,
            tracker_connection_timeout,
            max_in_flight_pieces,
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
/// use popcorn_fx_torrent::torrent::{Torrent, TorrentFlags, TorrentInfo, TorrentRequest, Result};
/// use popcorn_fx_torrent::torrent::fs::TorrentFileStorage;
/// use popcorn_fx_torrent::torrent::peer::extension::Extensions;
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
            .unwrap_or_else(|| DEFAULT_TORRENT_EXTENSIONS().iter().map(|e| e()).collect());
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
            runtime,
        ))
    }
}

/// A torrent operation which is executed in a chain during the lifetime of the torrent.
/// It provides a specific operation to be executed on the torrent in a sequential order.
///
/// The operation is always specific to one torrent, but should be allowed to create a new instance of the operation.
/// This allows the operation to store data which is specific to the torrent.
#[async_trait]
pub trait TorrentOperation: Debug + Send + Sync {
    /// Get the unique name of the operation.
    fn name(&self) -> &str;

    /// Execute the operation for the given torrent.
    /// The [TorrentContext] structure exposes additional internal data of the torrent which is otherwise not exposed on the [Torrent].
    ///
    /// # Returns
    ///
    /// It returns how the chain should proceed.
    async fn execute(&self, torrent: &TorrentContext) -> TorrentOperationResult;

    /// Clone this operation into a new boxed instance.
    ///
    /// The new boxed instance should have a clean state if it stores data.
    fn clone_boxed(&self) -> Box<dyn TorrentOperation>;
}

/// The result of executing a torrent operation.
#[derive(Debug, PartialEq)]
pub enum TorrentOperationResult {
    /// Continue the operations chain
    Continue,
    /// Stop the operations chain
    Stop,
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentEvent {
    /// Invoked when the status of the torrent has changed
    #[display(fmt = "torrent state has changed to {}", _0)]
    StateChanged(TorrentState),
    /// Invoked when the torrent metadata has been changed
    #[display(fmt = "torrent metadata has been changed")]
    MetadataChanged,
    /// Invoked when a new peer connection has been established
    #[display(fmt = "peer {} has been connected", _0)]
    PeerConnected(PeerInfo),
    /// Invoked when an existing peer connection has closed.
    #[display(fmt = "peer {} has been disconnected", _0)]
    PeerDisconnected(PeerInfo),
    /// Invoked when the active trackers have been changed
    #[display(fmt = "trackers have changed")]
    TrackersChanged,
    /// Invoked when the pieces have changed of the torrent
    #[display(fmt = "torrent pieces have changed")]
    PiecesChanged,
    /// Invoked when the priorities of the torrent pieces have changed
    #[display(fmt = "torrent piece priorities have changed")]
    PiecePrioritiesChanged,
    /// Invoked when a piece has been completed.
    #[display(fmt = "piece {} has been completed", _0)]
    PieceCompleted(PieceIndex),
    /// Invoked when the files have changed of the torrent
    #[display(fmt = "torrent files have changed")]
    FilesChanged,
    /// Invoked when the options of the torrent have been changed
    #[display(fmt = "torrent options have changed")]
    OptionsChanged,
    /// Invoked when the torrent stats have been updated
    #[display(fmt = "torrent stats changed {:?}", _0)]
    Stats(TorrentTransferStats),
}

#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "handle {}", handle)]
pub struct PeerInfo {
    /// The handle of the peer
    pub handle: PeerHandle,
    /// The remote address of the connected peer
    pub addr: SocketAddr,
    /// The indication of the connection direction
    pub connection_type: ConnectionType,
}

/// A torrent is an actual tracked torrent which is communicating with one or more trackers and peers.
///
/// Use [crate::torrent::TorrentInfo] if you only want to retrieve the metadata of a torrent.
#[derive(Debug)]
pub struct Torrent {
    handle: TorrentHandle,
    /// The unique peer id of this torrent
    /// This id is used as our client id when connecting to peers
    peer_id: PeerId,
    /// The port on which the torrent is listening for incoming peer connections
    peer_port: u16,
    /// The reference info of the torrent
    /// If the torrent reference is the original owner, then dropping this instance will stop the torrent
    ref_type: TorrentRefType,
    /// The inner torrent instance reference holder
    instance: Weak<TorrentContext>,
}

impl Torrent {
    /// Create a new request builder for creating a new torrent.
    pub fn request() -> TorrentRequest {
        TorrentRequest::default()
    }

    fn new(
        metadata: TorrentInfo,
        peer_listener: Box<dyn PeerListener>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        options: TorrentFlags,
        config: TorrentConfig,
        storage: Box<dyn TorrentFileStorage>,
        operations: Vec<Box<dyn TorrentOperation>>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let info_hash = metadata.info_hash.clone();
        let (event_sender, command_receiver) = unbounded_channel();
        let (peer_event_sender, peer_event_receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let location = storage.path().to_path_buf();
        let context = Arc::new(TorrentContext {
            handle,
            metadata: RwLock::new(metadata),
            peer_id,
            peer_port: peer_listener.port(),
            tracker_manager: TrackerManager::new(
                peer_id,
                peer_listener.port(),
                info_hash,
                config.tracker_connection_timeout.clone(),
                runtime.clone(),
            ),
            peer_pool: PeerPool::new(handle, config.peers_upper_limit, config.peers_in_flight),
            peer_subscriber: peer_event_sender,
            pieces: RwLock::new(Vec::with_capacity(0)),
            piece_chunk_pool: PieceChunkPool::new(),
            request_download_permits: Arc::new(Semaphore::new(config.max_in_flight_pieces)),
            request_upload_permits: Arc::new(Semaphore::new(config.peers_upload_slots)),
            files: RwLock::new(Vec::with_capacity(0)),
            protocol_extensions,
            extensions,
            storage,
            state: RwLock::new(Default::default()),
            options: RwLock::new(options),
            config: RwLock::new(config),
            stats: RwLock::new(TorrentTransferStats::default()),
            event_sender,
            callbacks: MultiCallback::new(runtime.clone()),
            cancellation_token,
            runtime,
        });

        let torrent = Self {
            handle,
            peer_id,
            peer_port: peer_listener.port(),
            ref_type: TorrentRefType::Owner,
            instance: Arc::downgrade(&context),
        };

        // create a new separate thread which manages the internal torrent resources
        // this thread is automatically cancelled when the torrent is dropped
        let loop_runtime = context.runtime.clone();
        loop_runtime.spawn(async move {
            // start the main loop of the torrent
            context
                .start(
                    &context,
                    operations,
                    command_receiver,
                    peer_event_receiver,
                    peer_listener,
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

    /// Get the unique peer id of this torrent as a reference.
    /// This id is used within the peer clients to identify with remote peers.
    ///
    /// # Returns
    ///
    /// Returns the unique peer id of this torrent.
    pub fn peer_id_as_ref(&self) -> &PeerId {
        &self.peer_id
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

    /// Check if the torrent has completed downloading all wanted pieces.
    pub async fn is_completed(&self) -> bool {
        if let Some(inner) = self.instance() {
            return inner.is_completed().await;
        }

        false
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
            let notifier = Arc::new(Notify::new());
            let mut receiver = inner.subscribe();
            let cancellation_token = CancellationToken::new();

            let inner_cancel = cancellation_token.clone();
            let inner_notifier = notifier.clone();
            inner.runtime.spawn(async move {
                loop {
                    select! {
                        _ = inner_cancel.cancelled() => break,
                        Some(event) = receiver.recv() => {
                            if let TorrentEvent::TrackersChanged = *event {
                                inner_notifier.notify_one();
                            }
                        }
                    }
                }
            });

            loop {
                notifier.notified().await;
                if inner.active_tracker_connections().await >= 2 {
                    break;
                }
            }

            cancellation_token.cancel();
        }

        Ok(inner.announce_all().await)
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

    /// Try to read the bytes from the given torrent file.
    /// This reads all available bytes of the file stored within the [Storage].
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_to_end(&self, file: &File) -> Result<Vec<u8>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        inner.read_file_to_end(file).await
    }

    /// Get a temporary strong reference to the inner torrent.
    pub(crate) fn instance(&self) -> Option<Arc<TorrentContext>> {
        self.instance.upgrade()
    }
}

impl Callback<TorrentEvent> for Torrent {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        if let Some(inner) = self.instance() {
            return inner.subscribe();
        }

        error!(
            "Unable to subscribe to torrent events for {}, handle has been invalidated",
            self
        );
        let (_, rx) = unbounded_channel();
        rx
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        if let Some(inner) = self.instance() {
            return inner.subscribe_with(subscriber);
        }

        error!(
            "Unable to subscribe to torrent events for {}, handle has been invalidated",
            self
        );
    }
}

impl Clone for Torrent {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            peer_id: self.peer_id,
            peer_port: self.peer_port,
            ref_type: TorrentRefType::Borrowed,
            instance: self.instance.clone(),
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
        if self.ref_type == TorrentRefType::Owner {
            if let Some(context) = self.instance.upgrade() {
                context.cancellation_token.cancel();
            }
        }
    }
}

/// The torrent instances owns the actual inner instance.
/// This prevents other [Torrent] references from keeping the torrent alive while the session has dropped it.
#[derive(Debug, PartialEq)]
enum TorrentRefType {
    Owner,
    Borrowed,
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
    /// Indicates that a piece part has been completed
    PiecePartCompleted(PiecePart, Vec<u8>),
    /// Indicates that a piece has been completed
    PieceCompleted(PieceIndex),
    /// Indicates that an invalid piece request response has been received by a peer
    PendingRequestRejected(PendingRequestRejected),
    /// Notify the peers about the availability of the given pieces
    NotifyPeersHavePieces(Vec<PieceIndex>),
    /// Indicates that the torrent state needs to be changed
    State(TorrentState),
}

impl Debug for TorrentCommandEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentCommandEvent::OptionsChanged => write!(f, "OptionsChanged"),
            TorrentCommandEvent::ConnectToTracker(e) => write!(f, "ConnectToTracker({:?})", e),
            TorrentCommandEvent::ConnectToPeer(e) => write!(f, "ConnectToPeer({})", e),
            TorrentCommandEvent::PeerConnected(e) => write!(f, "PeerConnected({})", e),
            TorrentCommandEvent::PeerClosed(e) => write!(f, "PeerClosed({})", e),
            TorrentCommandEvent::PiecePartCompleted(e, data) => {
                write!(f, "PiecePartCompleted({:?}, [size {}])", e, data.len())
            }
            TorrentCommandEvent::PieceCompleted(e) => write!(f, "PieceCompleted({})", e),
            TorrentCommandEvent::PendingRequestRejected(e) => {
                write!(f, "PendingRequestRejected({:?})", e)
            }
            TorrentCommandEvent::NotifyPeersHavePieces(pieces) => {
                write!(f, "NotifyPeersHavePieces(len {})", pieces.len())
            }
            TorrentCommandEvent::State(state) => write!(f, "State({:?})", state),
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
    /// The port the peer listener is listerning on for accepting incoming connections
    peer_port: u16,
    /// The torrent metadata information of the torrent
    /// This might still be incomplete if the torrent was created from a magnet link
    metadata: RwLock<TorrentInfo>,
    /// The manager of the trackers for the torrent
    tracker_manager: TrackerManager,

    /// The pool of peer connections
    peer_pool: PeerPool,
    /// The sender which is shared between all peers to inform the torrent of a [PeerEvent].
    peer_subscriber: Subscriber<PeerEvent>,

    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: RwLock<Vec<Piece>>,
    /// The pool which stores the received piece parts
    piece_chunk_pool: PieceChunkPool,

    /// The permit counter for requesting pieces from remote peers
    request_download_permits: Arc<Semaphore>,
    /// The permit counter for uploading pieces to remote peers
    request_upload_permits: Arc<Semaphore>,

    /// The torrent files
    files: RwLock<Vec<File>>,
    /// The torrent file storage to store the data
    storage: Box<dyn TorrentFileStorage>,

    /// The immutable enabled protocol extensions for this torrent
    protocol_extensions: ProtocolExtensionFlags,
    /// The immutable extensions for this torrent
    extensions: Extensions,

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
    callbacks: MultiCallback<TorrentEvent>,
    /// The main loop cancellation token
    cancellation_token: CancellationToken,
    /// The shared runtime used by the torrent
    runtime: Arc<Runtime>,
}

impl TorrentContext {
    /// Start the main loop of this torrent.
    /// It starts listening for events from different receivers and processes them accordingly.
    async fn start(
        &self,
        context: &Arc<TorrentContext>,
        operations: Vec<Box<dyn TorrentOperation>>,
        mut command_receiver: UnboundedReceiver<TorrentCommandEvent>,
        mut peer_event_receiver: UnboundedReceiver<Arc<PeerEvent>>,
        mut peer_receiver: Box<dyn PeerListener>,
    ) {
        let mut tracker_event_receiver = self.tracker_manager.subscribe();
        // the interval used to execute periodic torrent operations
        let mut operations_tick = time::interval(Duration::from_secs(1));
        let mut cleanup_interval = time::interval(Duration::from_secs(30));

        // execute the operations at the beginning of the loop
        select! {
            _ = self.cancellation_token.cancelled() => return,
            _ = self.execute_operations_chain(&operations) => {}
        }

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                event = command_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_command_event(&context, event).await;
                    } else {
                        debug!("Torrent {} events channel closed", self);
                        break;
                    }
                }
                Some(event) = tracker_event_receiver.recv() => self.handle_tracker_event((*event).clone()).await,
                Some(entry) = peer_receiver.recv() => self.handle_incoming_peer_connection(&context, entry).await,
                Some(event) = peer_event_receiver.recv() => self.handle_peer_event((*event).clone()).await,
                _ = operations_tick.tick() => {
                    self.execute_operations_chain(&operations).await;
                    self.update_stats().await;
                },
                _ = cleanup_interval.tick() => {
                    self.clean_peers().await;
                },
            }
        }

        self.peer_pool.shutdown().await;
        trace!("Torrent {} main loop ended", self);
    }

    /// Get the peer pool of the torrent.
    pub fn peer_pool(&self) -> &PeerPool {
        &self.peer_pool
    }

    /// Get the peer port this torrent is listening on for incoming peer connections.
    pub fn peer_port(&self) -> u16 {
        self.peer_port
    }

    /// Get the shared runtime used by the torrent.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Get the enabled protocol extensions for the torrent.
    pub fn protocols(&self) -> &ProtocolExtensionFlags {
        &self.protocol_extensions
    }

    /// Get the state of the torrent.
    pub async fn state(&self) -> TorrentState {
        self.state.read().await.clone()
    }

    /// Get the known torrent transfer stats.
    pub async fn stats(&self) -> TorrentTransferStats {
        self.stats.read().await.clone()
    }

    /// Get the options of the torrent.
    pub fn options(&self) -> &RwLock<TorrentFlags> {
        &self.options
    }

    /// Get an owned instance of the options of the torrent.
    pub async fn options_owned(&self) -> TorrentFlags {
        self.options.read().await.clone()
    }

    /// Get the configuration of the torrent.
    pub async fn config(&self) -> TorrentConfig {
        self.config.read().await.clone()
    }

    /// Get the configuration lock of the torrent.
    pub fn config_lock(&self) -> &RwLock<TorrentConfig> {
        &self.config
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

    /// Get the bitfield of the pieces indicating if a piece has been completed or not.
    pub async fn piece_bitfield(&self) -> BitVec {
        let mutex = self.pieces.read().await;
        let mut bitfield = BitVec::from_elem(mutex.len(), false);

        for piece in mutex.iter().filter(|e| e.is_completed()) {
            bitfield.set(piece.index, true);
        }

        bitfield
    }

    /// Get the interested pieces of this torrent.
    /// It returns all piece indexes for which the priority is not [PiecePriority::None], even if the piece is already completed.
    ///
    /// If you want the pieces which need to be downloaded, use [TorrentContext::wanted_pieces] instead.
    pub async fn interested_pieces(&self) -> Vec<PieceIndex> {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .map(|e| e.index)
            .collect()
    }

    /// Get all wanted pieces by the torrent ordered by [PiecePriority].
    /// Pieces with the highest priority will be first.
    ///
    /// It returns all piece indexes for which the priority is not [PiecePriority::None] and the piece has not been completed.
    pub async fn wanted_pieces(&self) -> Vec<PieceIndex> {
        let mutex = self.pieces.read().await;
        self.wanted_pieces_iter(&mutex)
            .await
            .sorted_by(|a, b| a.priority.cmp(&b.priority))
            .map(|e| e.index)
            .collect()
    }

    /// Get the total amount of wanted pieces by the torrent.
    pub async fn total_wanted_pieces(&self) -> usize {
        let mutex = self.pieces.read().await;
        self.wanted_pieces_iter(&mutex).await.count()
    }

    /// Get the total size in bytes of all interested pieces by the torrent.
    /// It returns the file size in bytes of the files which have a priority other than [PiecePriority::None].
    pub async fn interested_piece_size(&self) -> usize {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .map(|e| e.length)
            .sum()
    }

    /// Check if the given piece is wanted by the torrent.
    pub async fn is_piece_wanted(&self, piece: &PieceIndex) -> bool {
        self.pieces
            .read()
            .await
            .get(*piece)
            .map(|e| !e.is_completed() && e.priority != PiecePriority::None)
            .unwrap_or(false)
    }

    /// Get if the given piece is completed with downloading its data.
    /// It returns true if the piece is completed, validated and written to the storage, else false.
    pub async fn has_piece(&self, piece: PieceIndex) -> bool {
        self.pieces
            .read()
            .await
            .get(piece)
            .map(|e| e.is_completed())
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
        }

        self.update_interested_pieces_stats().await;
        debug!("Torrent {} piece priorities have been changed", self);
        self.invoke_event(TorrentEvent::PiecePrioritiesChanged);

        // update the state of the torrent based on the new priorities
        let new_state = self.determine_state().await;
        self.update_state(new_state).await;
    }

    /// Check if the torrent has completed downloading all wanted pieces.
    pub async fn is_completed(&self) -> bool {
        let mutex = self.pieces.read().await;
        mutex
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .all(|e| e.is_completed())
    }

    /// Check if downloading piece data is allowed by the torrent.
    pub async fn is_download_allowed(&self) -> bool {
        let options = self.options.read().await;
        let state = self.state.read().await;
        let is_download_mode = options.contains(TorrentFlags::DownloadMode);
        let is_not_paused = !options.contains(TorrentFlags::Paused);
        let is_not_init_state = !state.is_initializing_phase();

        is_download_mode && is_not_paused && is_not_init_state
    }

    /// Check if uploading piece data is allowed by the torrent.
    pub async fn is_upload_allowed(&self) -> bool {
        let options = self.options.read().await;
        trace!("Torrent options {:?}", options);
        (options.contains(TorrentFlags::UploadMode) || options.contains(TorrentFlags::SeedMode))
            && !options.contains(TorrentFlags::Paused)
    }

    /// Check if the torrent is a partial seed.
    /// A partial seed is a torrent that is seeding only a selection of a multi file torrent.
    pub async fn is_partial_seed(&self) -> bool {
        // check if this a multi file torrent
        if self.total_files().await <= 1 {
            return false;
        }

        // check if all wanted pieces have been downloaded
        self.total_wanted_pieces().await == 0
    }

    /// Calculate the additionally wanted peer connections by the torrent.
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

    /// Get an iterator over the pieces that are wanted and not completed for the torrent.
    async fn wanted_pieces_iter<'a>(
        &'a self,
        mutex: &'a RwLockReadGuard<'a, Vec<Piece>>,
    ) -> Filter<Iter<Piece>, fn(&&'a Piece) -> bool> {
        mutex
            .iter()
            .filter(|e| e.priority != PiecePriority::None && !e.is_completed())
    }

    /// Add the given peer to this torrent.
    /// Duplicate peers will be ignored and dropped.
    async fn add_peer(&self, peer: Peer) {
        debug!("Adding peer {} to torrent {}", peer, self);
        let info = PeerInfo {
            handle: peer.handle(),
            addr: peer.addr(),
            connection_type: peer.connection_type(),
        };
        let subscriber = self.peer_subscriber.clone();
        peer.subscribe_with(subscriber);

        if self.peer_pool.add_peer(peer).await {
            self.invoke_event(TorrentEvent::PeerConnected(info));
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

        if let Some(peer) = self.peer_pool.remove_peer(handle).await {
            self.invoke_event(TorrentEvent::PeerDisconnected(PeerInfo {
                handle: peer.handle(),
                addr: peer.addr(),
                connection_type: peer.connection_type(),
            }));
        }
    }

    /// Add the given metadata to the torrent.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// If the data was already known, this method does nothing.
    pub async fn add_metadata(&self, metadata: TorrentMetadata) {
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
        self.tracker_manager
            .make_announcement_to_all(AnnounceEvent::Started)
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
        self.invoke_event(TorrentEvent::OptionsChanged);
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
        self.invoke_event(TorrentEvent::OptionsChanged);
    }

    /// Update the state of this torrent.
    /// If the torrent is already in the given state, this will be a no-op.
    pub async fn update_state(&self, state: TorrentState) {
        // check if we're already in the expected state
        // if so, ignore this update
        if *self.state.read().await == state {
            return;
        }

        {
            let mut mutex = self.state.write().await;
            *mutex = state.clone();
        }

        // inform the trackers about the new state
        match &state {
            TorrentState::Downloading => self
                .tracker_manager
                .make_announcement_to_all(AnnounceEvent::Started),
            TorrentState::Seeding | TorrentState::Finished => self
                .tracker_manager
                .make_announcement_to_all(AnnounceEvent::Completed),
            TorrentState::Paused => self
                .tracker_manager
                .make_announcement_to_all(AnnounceEvent::Paused),
            _ => {}
        }

        debug!("Updated torrent state to {:?} for {}", state, self);
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
            for peer in peer_mutex.iter() {
                let state = peer.state().await;
                if state == PeerState::Closed || state == PeerState::Error {
                    continue;
                }

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
            mutex.total_downloaded_useful += peer_stats.download_useful;
        }

        let event_metrics = mutex.clone();
        drop(mutex);
        debug!("Torrent {} stats: {:?}", self, event_metrics);
        self.invoke_event(TorrentEvent::Stats(event_metrics));
    }

    /// Increase the availability of the given piece indexes.
    pub async fn update_piece_availabilities(&self, pieces: Vec<PieceIndex>) {
        // check if the metadata is known and the pieces have been created
        if !self.is_metadata_known().await || self.total_pieces().await == 0 {
            debug!(
                "Unable to update piece availabilities for {}, torrent metadata or pieces are unknown",
                self
            );
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
        trace!("Updating torrent pieces of {}", self);
        {
            let mut mutex = self.pieces.write().await;
            *mutex = pieces;

            // update the piece availability based on the current peer connections
            let mut availability: HashMap<PieceIndex, usize> = HashMap::new();
            let peer_count: usize;

            {
                let peer_mutex = self.peer_pool.peers.read().await;

                peer_count = peer_mutex.len();
                for peer in peer_mutex.iter() {
                    let bitfield = peer.remote_piece_bitfield().await;
                    for (piece_index, _) in bitfield.iter().enumerate().filter(|(_, value)| *value)
                    {
                        *availability.entry(piece_index).or_insert(0) += 1;
                    }
                }
            }

            if availability.len() > 0 {
                for piece in mutex.iter_mut() {
                    if let Some(availability) = availability.get(&piece.index) {
                        piece.availability += *availability as u32;
                    }
                }
                debug!(
                    "Updated a total of {} torrent piece availabilities from {} peers for {}",
                    availability.len(),
                    peer_count,
                    self
                );
            }
        }

        self.update_interested_pieces_stats().await;
        self.invoke_event(TorrentEvent::PiecesChanged);
    }

    /// Set the given piece as completed.
    /// This can be called by file validation operations to indicate that a piece has been stored in the storage.
    ///
    /// ## Remark
    ///
    /// This function doesn't verify if the piece is valid.
    pub async fn piece_completed(&self, piece: PieceIndex) {
        self.pieces_completed(vec![piece]).await;
    }

    /// Set the given pieces as completed.
    ///
    /// ## Remark
    ///    
    /// This function doesn't verify if the pieces are valid.
    pub async fn pieces_completed(&self, pieces: Vec<PieceIndex>) {
        trace!("Marking pieces {:?} as completed for {}", pieces, self);
        let mut total_pieces_size = 0;
        let mut total_completed_pieces = 0;

        {
            let mut mutex = self.pieces.write().await;
            for piece in pieces.iter() {
                if let Some(piece) = mutex.get_mut(*piece) {
                    piece.mark_completed();
                    total_pieces_size = piece.length;
                    total_completed_pieces += 1;
                } else {
                    warn!("Received unknown completed piece {} for {}", piece, self);
                }
            }
        }

        {
            let mut stats_mutex = self.stats.write().await;
            stats_mutex.completed_pieces += total_completed_pieces;
            stats_mutex.total_completed_size += total_pieces_size;
        }

        if self.is_completed().await {
            // offload the state change to the main loop
            self.send_command_event(TorrentCommandEvent::State(TorrentState::Finished));
        }

        for piece in pieces.iter() {
            self.invoke_event(TorrentEvent::PieceCompleted(*piece));
        }

        // notify the connected peers about the completed piece
        self.send_command_event(TorrentCommandEvent::NotifyPeersHavePieces(pieces));
    }

    /// Update the torrent files of the torrent.
    /// This replaces any existing files.
    pub async fn update_files(&self, files: Vec<File>) {
        trace!(
            "Updating a total of {} torrent files for {}",
            files.len(),
            self
        );
        {
            let mut mutex = self.files.write().await;
            *mutex = files;
        }

        self.invoke_event(TorrentEvent::FilesChanged);
    }

    /// Update the stats info of all interested pieces by the torrent.
    async fn update_interested_pieces_stats(&self) {
        let mut stats_mutex = self.stats.write().await;
        stats_mutex.total_size = self.interested_piece_size().await;
        stats_mutex.wanted_pieces = self.interested_pieces().await.len();
    }

    /// Cancel all currently queued pending requests of the torrent.
    /// This will clear all pending requests from the buffer.
    pub async fn cancel_all_pending_requests(&self) {
        for peer in self.peer_pool.peers.read().await.iter() {
            // TODO: cancel pending requests in the peer
        }
    }

    /// Resume the torrent.
    /// This will put the torrent back into [TorrentFlags::DownloadMode], trying to download any missing pieces.
    pub async fn resume(&self) {
        self.add_options(TorrentFlags::DownloadMode | TorrentFlags::Metadata)
            .await;
        self.remove_options(TorrentFlags::Paused).await;

        // announce to the trackers if we don't know any peers
        if self.peer_pool.available_peer_addrs_len().await == 0 {
            self.tracker_manager
                .make_announcement_to_all(AnnounceEvent::Started);
        }
    }

    /// Pause the torrent operations.
    pub async fn pause(&self) {
        self.add_options(TorrentFlags::Paused).await;
        self.send_command_event(TorrentCommandEvent::OptionsChanged);
        self.send_command_event(TorrentCommandEvent::State(TorrentState::Paused));
    }

    /// Handle a command event from the channel of the torrent.
    async fn handle_command_event(
        &self,
        context: &Arc<TorrentContext>,
        event: TorrentCommandEvent,
    ) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            TorrentCommandEvent::OptionsChanged => self.options_changed().await,
            TorrentCommandEvent::ConnectToTracker(e) => self.add_tracker_async(e).await,
            TorrentCommandEvent::ConnectToPeer(addr) => {
                self.create_peer_connection(context.clone(), addr).await
            }
            TorrentCommandEvent::PeerConnected(peer) => self.add_peer(peer).await,
            TorrentCommandEvent::PeerClosed(handle) => self.remove_peer(handle).await,
            TorrentCommandEvent::PiecePartCompleted(part, data) => {
                self.process_completed_piece_part(part, data).await
            }
            TorrentCommandEvent::PieceCompleted(piece) => self.process_completed_piece(piece).await,
            TorrentCommandEvent::PendingRequestRejected(request_rejection) => {
                self.process_pending_request_rejected(request_rejection)
                    .await
            }
            TorrentCommandEvent::NotifyPeersHavePieces(pieces) => {
                self.notify_peers_have_pieces(pieces).await
            }
            TorrentCommandEvent::State(state) => self.update_state(state).await,
        }
    }

    async fn handle_tracker_event(&self, event: TrackerManagerEvent) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            TrackerManagerEvent::PeersDiscovered(peers) => {
                self.handle_discovered_peers(peers).await
            }
            TrackerManagerEvent::TrackerAdded(handle) => {
                let is_retrieving_metadata =
                    *self.state.read().await == TorrentState::RetrievingMetadata;
                let is_download_allowed = self.is_download_allowed().await;
                let is_upload_allowed = self.is_upload_allowed().await;

                if is_retrieving_metadata || is_download_allowed {
                    self.tracker_manager
                        .make_announcement(handle, AnnounceEvent::Started);
                } else if is_upload_allowed {
                    self.tracker_manager
                        .make_announcement(handle, AnnounceEvent::Completed);
                }

                self.invoke_event(TorrentEvent::TrackersChanged);
            }
        }
    }

    async fn handle_incoming_peer_connection(
        &self,
        torrent: &Arc<TorrentContext>,
        entry: PeerEntry,
    ) {
        trace!(
            "Trying to accept incoming {} peer connection for {}",
            entry.socket_addr,
            self
        );
        let timeout = self.config.read().await.peer_connection_timeout;
        match Peer::new_inbound(
            self.peer_id,
            entry.socket_addr,
            entry.stream,
            torrent.clone(),
            self.protocol_extensions,
            self.extensions(),
            timeout,
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

    async fn handle_peer_event(&self, event: PeerEvent) {
        match event {
            PeerEvent::PeersDiscovered(peers) => self.handle_discovered_peers(peers).await,
            PeerEvent::PeersDropped(peers) => self.handle_dropped_peers(peers).await,
            PeerEvent::RemoteAvailablePieces(pieces) => {
                self.update_piece_availabilities(pieces).await
            }
            _ => {}
        }
    }

    async fn handle_discovered_peers(&self, peer_addrs: Vec<SocketAddr>) {
        self.peer_pool.add_available_peer_addrs(peer_addrs).await;
    }

    async fn handle_dropped_peers(&self, peers: Vec<SocketAddr>) {
        self.peer_pool.remove_available_peer_addrs(peers).await;
    }

    /// Execute the torrent operations chain.
    ///
    /// This will execute the operations in order as defined by the chain.
    /// If an operation returns [None], the execution chain will be interrupted.
    async fn execute_operations_chain(&self, operations: &Vec<Box<dyn TorrentOperation>>) {
        for operation in operations.iter() {
            let start = Instant::now();
            let execution_result = operation.execute(&self).await;
            let elapsed = start.elapsed();
            trace!(
                "Operation {} resulted in {:?} after {} millis for {}",
                operation.name(),
                execution_result,
                elapsed.as_millis(),
                self
            );
            if execution_result == TorrentOperationResult::Stop {
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

        // TODO: check if still needed
    }

    async fn process_completed_piece_part(&self, piece_part: PiecePart, data: Vec<u8>) {
        let piece_length: usize;
        let mut mutex = self.pieces.write().await;
        let piece: &mut Piece;

        if let Some(piece_ref) = mutex.iter_mut().find(|e| e.index == piece_part.piece) {
            piece = piece_ref;
            piece_length = piece.length;

            // check if the piece has already been completed
            // this can happen "end game" as the same piece & parts are requested from multiple torrents
            if piece.is_completed() {
                trace!(
                    "Received already completed piece {} part {} for {}",
                    piece_part.piece,
                    piece_part.part,
                    self
                );
                return;
            }
        } else {
            warn!(
                "Received unknown piece {} chunk data for {}",
                piece_part.piece, self
            );
            return;
        }

        match self
            .piece_chunk_pool
            .add_chunk(&piece_part, piece_length, data)
            .await
        {
            Ok(_) => {
                // update the piece info
                piece.part_completed(piece_part.part);

                if piece.is_completed() {
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
                self.piece_completed(piece).await;
                debug!("Piece {} for {} has been completed", piece, self);
            } else {
                let mut mutex = self.pieces.write().await;
                if let Some(piece) = mutex.iter_mut().find(|e| e.index == piece) {
                    debug!(
                        "Retrying invalid received piece {} data for {}",
                        piece.index, self
                    );

                    // reset the piece completed parts as the parts should be fetched again
                    piece.reset_completed_parts();

                    // TODO inform peers about the piece wanted again
                }
            }
        } else {
            warn!(
                "Piece chunk data of {} is not available for {}",
                piece, self
            );
        }
    }

    /// Process the new options of the torrent.
    async fn options_changed(&self) {
        let state = self.determine_state().await;
        self.update_state(state).await;
    }

    /// Try to determine the state the torrent currently has.
    /// It returns the expected state of the torrent without actually updating the state.
    pub async fn determine_state(&self) -> TorrentState {
        let is_paused = self.options.read().await.contains(TorrentFlags::Paused);

        if is_paused {
            return TorrentState::Paused;
        }

        let is_download_allowed = self.is_download_allowed().await;
        if is_download_allowed && self.total_wanted_pieces().await > 0 {
            return TorrentState::Downloading;
        }

        if self.is_upload_allowed().await {
            return TorrentState::Seeding;
        }

        TorrentState::Finished
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

    /// Get the piece part of the torrent based on the piece and the offset within the piece.
    /// It returns [None] if the piece part is unknown to this torrent.
    ///
    /// # Arguments
    ///
    /// * `piece` - The index of the piece.
    /// * `begin` - The offset within the piece.
    pub async fn piece_part(&self, piece: PieceIndex, begin: usize) -> Option<PiecePart> {
        self.find_piece_part(piece, begin).await
    }

    /// Notify the torrent that a peer has been closed.
    pub fn notify_peer_closed(&self, peer: PeerHandle) {
        self.send_command_event(TorrentCommandEvent::PeerClosed(peer));
    }

    /// Notify the torrent that a pending request has been rejected by the remote peer.
    pub async fn pending_request_rejected(
        &self,
        piece: PieceIndex,
        begin: usize,
        peer: PeerHandle,
    ) {
        if let Some(part) = self.find_piece_part(piece, begin).await {
            self.send_command_event(TorrentCommandEvent::PendingRequestRejected(
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

    /// Notify this torrent about the completion of a piece.
    /// The torrent will then validate and store the completed piece data.
    pub fn piece_part_completed(&self, part: PiecePart, data: Vec<u8>) {
        self.send_command_event(TorrentCommandEvent::PiecePartCompleted(part, data));
    }

    /// Get the completed pieces of the torrent.
    pub async fn completed_pieces(&self) -> Vec<PieceIndex> {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.is_completed())
            .map(|e| e.index)
            .collect()
    }

    /// Get the total amount of completed pieces for the torrent.
    pub async fn total_completed_pieces(&self) -> usize {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.is_completed())
            .count()
    }

    /// Notify the torrent of an invalid received piece part.
    pub fn invalid_piece_data_received(&self, part: PiecePart, peer: PeerHandle) {
        self.send_command_event(TorrentCommandEvent::PendingRequestRejected(
            PendingRequestRejected {
                part,
                peer,
                reason: RequestRejectedReason::InvalidDataResponse,
            },
        ));
    }

    /// Get a request permit to download piece data from a remote peer.
    /// A permit should be retrieved for each piece that is being requested from a peer.
    pub async fn request_download_permit(&self) -> Option<OwnedSemaphorePermit> {
        if !self.is_download_allowed().await {
            return None;
        }

        if self.request_download_permits.available_permits() == 0 {
            return None;
        }

        self.request_download_permits
            .clone()
            .acquire_owned()
            .await
            .ok()
    }

    /// Get a request permit to upload piece data to a remote peer.
    /// A permit is peer based and should only be requested when trying to unchoke the client peer.
    pub async fn request_upload_permit(&self) -> Option<OwnedSemaphorePermit> {
        if !self.is_upload_allowed().await {
            return None;
        }

        if self.request_upload_permits.available_permits() == 0 {
            return None;
        }

        self.request_upload_permits
            .clone()
            .acquire_owned()
            .await
            .ok()
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

    /// Try to read the bytes from the given torrent file.
    /// This reads all available bytes of the file stored within the [Storage].
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_to_end(&self, file: &File) -> Result<Vec<u8>> {
        Ok(self.storage.read_to_end(&file.path).await?)
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

    /// Try to create a new peer connection with the given peer address.
    /// This is a non-blocking process and will be executed in the background.
    ///
    /// The passed [Torrent] should always be a weak reference to this instance.
    async fn create_peer_connection(&self, torrent: Arc<TorrentContext>, peer_addr: SocketAddr) {
        if let Some(permit) = self.peer_pool.permit().await {
            let protocol_extensions = self.protocol_extensions.clone();
            let extensions = self.extensions();
            let event_sender = self.event_sender.clone();
            let peer_id = self.peer_id.clone();
            let runtime = torrent.runtime.clone();
            debug!(
                "Trying to create a new peer connection {} for {}",
                peer_addr, self
            );
            runtime.spawn(async move {
                let handle_info = torrent.handle.clone();
                match Self::try_create_peer_connection(
                    torrent,
                    peer_id,
                    peer_addr,
                    protocol_extensions,
                    extensions,
                )
                .await
                {
                    Ok(peer) => {
                        drop(permit);
                        let _ = event_sender.send(TorrentCommandEvent::PeerConnected(peer));
                    }
                    Err(e) => {
                        debug!(
                            "Failed to create peer connection for torrent {}, {}",
                            handle_info, e
                        );
                        drop(permit);
                    }
                }
            });
        } else {
            // put the address back into the peer pool as no permit was granted from making the connection
            self.peer_pool
                .add_available_peer_addrs(vec![peer_addr])
                .await;
        }
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

                let range_len = range.len();
                let start_time = Instant::now();
                match self.storage.write(&file.path, offset, &data[range]).await {
                    Ok(_) => {
                        let elapsed = start_time.elapsed();
                        let path = PathBuf::from(self.storage.path()).join(file.path);
                        debug!(
                            "Wrote piece {} data (size {}) to {:?} in {}.{:03}s",
                            piece.index,
                            range_len,
                            path,
                            elapsed.as_secs(),
                            elapsed.subsec_millis()
                        )
                    }
                    Err(e) => {
                        error!(
                            "Failed to write piece {} data for {}, {}",
                            piece_index, self, e
                        );
                        self.update_state(TorrentState::Error).await;
                        return;
                    }
                }
            }
        }
    }

    /// Notify the peers about the pieces that have become available.
    async fn notify_peers_have_pieces(&self, pieces: Vec<PieceIndex>) {
        for peer in self.peer_pool.peers.read().await.iter() {
            peer.notify_has_pieces(pieces.clone());
        }
    }

    /// Get the known extensions of the torrent.
    /// It returns owned instance of the extensions.
    pub fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_boxed()).collect()
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
        torrent: Arc<TorrentContext>,
        peer_id: PeerId,
        peer_addr: SocketAddr,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
    ) -> Result<Peer> {
        let timeout = torrent.config.read().await.peer_connection_timeout;
        let runtime = torrent.runtime.clone();
        Ok(Peer::new_outbound(
            peer_id,
            peer_addr,
            torrent,
            protocol_extensions,
            extensions,
            timeout,
            runtime,
        )
        .await?)
    }
}

impl Callback<TorrentEvent> for TorrentContext {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        self.callbacks.subscribe_with(subscriber)
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
    use crate::create_torrent;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::operation::{
        TorrentCreateFilesOperation, TorrentCreatePiecesOperation, TorrentFileValidationOperation,
    };
    use crate::torrent::peer::extension::metadata::MetadataExtension;
    use log::LevelFilter;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::{copy_test_file, read_test_file_to_bytes};
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;

    #[test]
    fn test_torrent_announce() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            DEFAULT_TORRENT_OPERATIONS()
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let result = runtime.block_on(torrent.announce()).unwrap();

        assert_ne!(
            0, result.total_seeders,
            "expected seeders to have been found"
        );
        assert_ne!(0, result.peers.len(), "expected peers to have been found");
    }

    #[test]
    fn test_torrent_metadata() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "debian-udp.torrent";
        let torrent_info_data = read_test_file_to_bytes(filename);
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = create_torrent!(filename, temp_path, TorrentFlags::None, vec![]);
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let metadata = runtime.block_on(torrent.metadata()).unwrap();

        assert_eq!(torrent_info, metadata);
    }

    #[test]
    fn test_retrieve_metadata() {
        init_logger!();
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

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::MetadataChanged = *event {
                        tx.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        let _ = rx.recv_timeout(Duration::from_secs(30)).unwrap();
        let result = runtime.block_on(torrent.metadata()).unwrap();

        assert_ne!(
            None, result.info,
            "expected the metadata to have been present"
        );
    }

    #[test]
    fn test_prioritize_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx_pieces_event, rx_pieces_event) = channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PiecesChanged = *event {
                        tx_pieces_event.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the pieces to be created before trying to download the data
        let _ = rx_pieces_event
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the pieces to have been created");

        // only request the first piece
        let mut priorities = runtime.block_on(torrent.piece_priorities());
        for priority in &mut priorities[1..] {
            priority.1 = PiecePriority::None;
        }
        runtime.block_on(torrent.prioritize_pieces(priorities));

        let result = runtime
            .block_on(torrent.pieces())
            .expect("expected the pieces to be present");
        assert_eq!(PiecePriority::Normal, result[0].priority);
        assert_eq!(PiecePriority::None, result[1].priority);
    }

    #[test]
    fn test_resume() {
        init_logger!(LevelFilter::Debug);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let num_of_pieces = 80;
        let (tx_state, rx_state) = channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::Metadata,
            DEFAULT_TORRENT_OPERATIONS()
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        if state == &TorrentState::Finished {
                            tx_state.send(()).unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the first "finished" state to be received after validating the torrent files
        let _ = rx_state
            .recv_timeout(Duration::from_millis(1500))
            .expect("expected the pieces to have been created");

        // only request the first X amount of pieces
        runtime.block_on(async {
            let mut priorities = torrent.piece_priorities().await;
            for priority in &mut priorities[num_of_pieces..] {
                priority.1 = PiecePriority::None;
            }
            torrent.prioritize_pieces(priorities).await;
            torrent.resume().await;
        });
        let options = runtime.block_on(torrent.options());
        assert_eq!(
            true,
            options.contains(TorrentFlags::DownloadMode),
            "expected the download flag to have been set"
        );

        // wait for the wanted pieces to be finished
        let _ = rx_state
            .recv_timeout(Duration::from_secs(180))
            .expect("expected the torrent to enter the FINISHED state");

        runtime.block_on(async {
            // check if the expected file exists
            let filepath = PathBuf::from(temp_path).join("debian-12.4.0-amd64-DVD-1.iso");
            assert_eq!(
                true,
                filepath.exists(),
                "expected the file {} to exist on the file system",
                filepath.display()
            );

            let pieces = torrent.pieces().await.unwrap();
            let pieces_bitfield = context.piece_bitfield().await;

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
    fn test_resume_magnet() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let num_of_pieces = 20;
        let (tx_state, rx_state) = channel();
        let (tx_ready, rx_pieces_event) = channel();
        let torrent = create_torrent!(
            "magnet:?xt=urn:btih:6C73EB4F6F62CACB7D8BA6872F849D7658BE3061&tr=udp://tracker.opentrackr.org:1337&tr=udp://tracker.tiny-vps.com:6969&tr=udp://tracker.openbittorrent.com:1337&tr=udp://tracker.coppersurfer.tk:6969&tr=udp://tracker.leechers-paradise.org:6969&tr=udp://p4p.arenabg.ch:1337&tr=udp://p4p.arenabg.com:1337&tr=udp://tracker.internetwarriors.net:1337&tr=udp://9.rarbg.to:2710&tr=udp://9.rarbg.me:2710&tr=udp://exodus.desync.com:6969&tr=udp://tracker.cyberia.is:6969&tr=udp://tracker.torrent.eu.org:451&tr=udp://open.stealth.si:80&tr=udp://tracker.moeking.me:6969&tr=udp://tracker.zerobytes.xyz:1337",
            temp_path,
            TorrentFlags::Metadata,
            DEFAULT_TORRENT_OPERATIONS()
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        if state == &TorrentState::Finished {
                            tx_state.send(()).unwrap();
                        }
                    } else if let TorrentEvent::PiecesChanged = *event {
                        tx_ready.send(event).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the pieces to be created before trying to download the data
        let _ = rx_pieces_event
            .recv_timeout(Duration::from_secs(15))
            .expect("expected the pieces to have been created");

        runtime.block_on(async {
            // only request the first 2 pieces
            let mut priorities = torrent.piece_priorities().await;
            for priority in &mut priorities[num_of_pieces..] {
                priority.1 = PiecePriority::None;
            }
            torrent.prioritize_pieces(priorities).await;
            torrent.resume().await;

            // wait for a piece to be completed
            let _ = rx_state
                .recv_timeout(Duration::from_secs(300))
                .expect("expected the torrent to enter the FINISHED state");

            let pieces = torrent.pieces().await.unwrap();
            let pieces_bitfield = context.piece_bitfield().await;

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
        init_logger!();
        let temp_dir_source = tempdir().unwrap();
        let temp_path_source = temp_dir_source.path().to_str().unwrap();
        let temp_dir_target = tempdir().unwrap();
        let temp_path_target = temp_dir_target.path().to_str().unwrap();
        let num_of_pieces = 30;
        copy_test_file(
            temp_path_source,
            "piece-1_30.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let expected_file_data = read_test_file_to_bytes("piece-1_30.iso");
        let (tx_state, rx_state) = channel();
        let source_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path_source,
            TorrentFlags::UploadMode,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
                Box::new(TorrentFileValidationOperation::new()),
            ]
        );
        let target_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path_target,
            TorrentFlags::DownloadMode | TorrentFlags::Paused,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ]
        );
        let source_context = source_torrent.instance().unwrap();
        let runtime = source_context.runtime();

        runtime
            .block_on(async {
                let mut attempts = 0;

                loop {
                    let source_state = source_torrent.state().await;
                    let source_files = source_torrent.total_files().await.unwrap_or(0);
                    let target_files = target_torrent.total_files().await.unwrap_or(0);

                    if source_state != TorrentState::Initializing
                        && source_state != TorrentState::CheckingFiles
                    {
                        if source_files > 0 && target_files > 0 {
                            return Ok(());
                        }
                    }

                    if attempts > 100 {
                        return Err(TorrentError::Timeout);
                    }

                    time::sleep(Duration::from_millis(100)).await;
                    attempts += 1;
                }
            })
            .expect("expected the file to have been created");

        // only request the X amount of pieces
        runtime.block_on(async {
            let mut priorities = target_torrent.piece_priorities().await;
            for priority in &mut priorities[num_of_pieces..] {
                priority.1 = PiecePriority::None;
            }
            target_torrent.prioritize_pieces(priorities).await;
        });

        // resume the target torrent to fetch data from the source torrent
        runtime.block_on(target_torrent.resume());

        // listen to the finished event
        let mut receiver = target_torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        if state == &TorrentState::Finished {
                            tx_state.send(()).unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // connect the source torrent to the target torrent
        let source_context = source_torrent.instance().unwrap();
        let source_peer_addr = SocketAddr::from(([127, 0, 0, 1], target_torrent.peer_port()));
        runtime.block_on(
            source_context.create_peer_connection(source_context.clone(), source_peer_addr),
        );

        // wait for all pieces to be completed (finished state)
        let _ = rx_state
            .recv_timeout(Duration::from_secs(60))
            .expect("expected the torrent to enter the FINISHED state");

        // validate the pieces and received data
        runtime.block_on(async {
            let pieces = target_torrent
                .pieces()
                .await
                .expect("expected the pieces to have been created");
            let target_context = target_torrent.instance().unwrap();
            let pieces_bitfield = target_context.piece_bitfield().await;

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

            // read the torrent file
            let files = target_torrent.files().await;
            let result = target_torrent
                .read_file_to_end(files.get(0).expect("expected file index 0 to be present"))
                .await
                .unwrap();
            assert_eq!(expected_file_data, result);
        });
    }

    #[test]
    fn test_torrent_fast() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx_state, rx_state) = channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::DownloadMode | TorrentFlags::Metadata,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let ready_tx = tx_state.clone();
        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = &*event {
                        ready_tx.send(()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the files to have been created
        let _ = rx_state
            .recv_timeout(Duration::from_secs(2))
            .expect("expected the torrent to have invoked FilesChanged");

        // connect the source torrent to the target torrent
        let go_tor = SocketAddr::from(([127, 0, 0, 1], 42069));
        runtime.block_on(context.create_peer_connection(context.clone(), go_tor));

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        if state == &TorrentState::Finished {
                            tx_state.send(()).unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // wait for all pieces to be completed (finished state)
        let _ = rx_state
            .recv_timeout(Duration::from_secs(180))
            .expect("expected the torrent to enter the FINISHED state");
    }

    #[test]
    fn test_torrent_piece_part() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_piece_part = PiecePart {
            piece: 0,
            part: 1,
            begin: 16384,
            length: 16384,
        };
        let (tx, rx) = channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PiecesChanged = *event {
                        tx.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the pieces to have been created");

        let result = runtime.block_on(context.piece_part(0, 16000));
        assert_eq!(
            None, result,
            "expected no piece part to be returned for invalid begin"
        );

        let result = runtime.block_on(context.piece_part(0, 16384));
        assert_eq!(Some(expected_piece_part), result, "expected the piece part");
    }

    #[test]
    fn test_torrent_create_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![Box::new(TorrentCreatePiecesOperation::new())]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let (tx, rx) = channel();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PiecesChanged = *event {
                        tx.send(event).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(250)).unwrap();
        let pieces = runtime.block_on(torrent.pieces()).unwrap();

        assert_ne!(0, pieces.len(), "expected the pieces to have been created");
    }

    #[test]
    fn test_torrent_create_files() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let (tx, rx) = channel();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = *event {
                        tx.send(event).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(250)).unwrap();
        let files = runtime.block_on(torrent.files());

        assert_eq!(1, files.len(), "expected the files to have been created");
    }

    #[test]
    fn test_torrent_is_completed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(
            temp_path,
            "piece-1_30.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::None,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
                Box::new(TorrentFileValidationOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let (tx, rx) = channel();

        let mut receiver = torrent.subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        if state != &TorrentState::Initializing
                            && state != &TorrentState::CheckingFiles
                        {
                            tx.send(()).unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the expected state
        let _ = rx.recv_timeout(Duration::from_millis(5000)).unwrap();

        // prioritize the first 30 pieces
        let mut priorities = runtime.block_on(torrent.piece_priorities());
        for priority in &mut priorities[30..] {
            priority.1 = PiecePriority::None;
        }
        runtime.block_on(torrent.prioritize_pieces(priorities));

        let result = runtime.block_on(torrent.is_completed());
        assert_eq!(true, result, "expected the torrent to be completed");
    }

    #[test]
    fn test_is_download_allowed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!("debian-udp.torrent", temp_path, TorrentFlags::None, vec![]);
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let result = runtime.block_on(context.is_download_allowed());
        assert_eq!(false, result, "expected downloading to not be allowed");

        let result = runtime.block_on(async {
            context.add_options(TorrentFlags::DownloadMode).await;
            context.is_download_allowed().await
        });
        assert_eq!(false, result, "expected downloading to not be allowed");

        let result = runtime.block_on(async {
            context.update_state(TorrentState::Finished).await;
            context.is_download_allowed().await
        });
        assert_eq!(true, result, "expected downloading to be allowed");

        let result = runtime.block_on(async {
            context.add_options(TorrentFlags::Paused).await;
            context.is_download_allowed().await
        });
        assert_eq!(false, result, "expected downloading to not be allowed");
    }

    #[test]
    fn test_is_upload_allowed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::UploadMode,
            vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
                Box::new(TorrentFileValidationOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        let result = runtime.block_on(context.is_upload_allowed());
        assert_eq!(true, result, "expected uploading to be allowed");

        runtime.block_on(torrent.add_options(TorrentFlags::Paused));
        let result = runtime.block_on(context.is_upload_allowed());
        assert_eq!(false, result, "expected uploading to not be allowed");

        runtime.block_on(torrent.remove_options(TorrentFlags::Paused | TorrentFlags::UploadMode));
        let result = runtime.block_on(context.is_upload_allowed());
        assert_eq!(false, result, "expected uploading to not be allowed");
    }

    #[test]
    fn test_torrent_transfer_stats_progress() {
        let stats = TorrentTransferStats {
            upload: 0,
            upload_rate: 0,
            upload_useful: 0,
            upload_useful_rate: 0,
            download: 0,
            download_rate: 0,
            download_useful: 0,
            download_useful_rate: 0,
            total_uploaded: 0,
            total_downloaded: 0,
            total_downloaded_useful: 0,
            wanted_pieces: 100,
            completed_pieces: 20,
            total_size: 0,
            total_completed_size: 0,
            total_peers: 0,
        };
        let result = stats.progress();
        assert_eq!(0.20, result);

        let stats = TorrentTransferStats {
            upload: 0,
            upload_rate: 0,
            upload_useful: 0,
            upload_useful_rate: 0,
            download: 0,
            download_rate: 0,
            download_useful: 0,
            download_useful_rate: 0,
            total_uploaded: 0,
            total_downloaded: 0,
            total_downloaded_useful: 0,
            wanted_pieces: 100,
            completed_pieces: 20,
            total_size: 1024,
            total_completed_size: 512,
            total_peers: 0,
        };
        let result = stats.progress();
        assert_eq!(0.50, result);
    }
}
