use crate::torrent::dht::DhtTracker;
use crate::torrent::errors::Result;
use crate::torrent::file::File;
use crate::torrent::metrics::Metric;
use crate::torrent::operation::{
    TorrentConnectPeersOperation, TorrentCreateFilesOperation, TorrentCreatePiecesOperation,
    TorrentDhtNodesOperation, TorrentDhtPeersOperation, TorrentFileValidationOperation,
    TorrentMetadataOperation, TorrentTrackersOperation,
};
use crate::torrent::peer::extension::Extension;
use crate::torrent::peer::{
    BitTorrentPeer, Peer, PeerClientInfo, PeerDiscovery, PeerEntry, PeerEvent, PeerHandle, PeerId,
    ProtocolExtensionFlags,
};
use crate::torrent::storage::{Storage, StorageParams};
use crate::torrent::torrent_config::TorrentConfig;
use crate::torrent::tracker::{
    AnnounceEvent, AnnouncementResult, TrackerEntry, TrackerHandle, TrackerManager,
    TrackerManagerEvent,
};
use crate::torrent::{
    FileAttributeFlags, FileIndex, FilePool, Metrics, PeerPool, Piece, PieceChunkPool, PieceIndex,
    PiecePart, PiecePool, PiecePriority, TorrentError, TorrentFlags, TorrentMetadata,
    TorrentMetadataInfo, TorrentPeer, DEFAULT_TORRENT_EXTENSIONS,
    DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
};
use async_trait::async_trait;
use derive_more::Display;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, error, info, trace, warn};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Notify, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::{select, time};
use tokio_util::sync::{
    CancellationToken, WaitForCancellationFuture, WaitForCancellationFutureOwned,
};
use url::Url;

const PEER_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const OPERATIONS_INTERVAL: Duration = Duration::from_secs(1);

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// The [Torrent] operation factory.
/// This factory will create a new instance of an [TorrentOperation] for each new torrent.
pub type TorrentOperationFactory = fn() -> Box<dyn TorrentOperation>;

/// The chain of torrent operations that are executed for each torrent.
pub type TorrentOperations = Vec<Box<dyn TorrentOperation>>;

/// A [Torrent] extension factory.
/// This factory will create a new instance of an [Extension] for each new torrent.
pub type ExtensionFactory = fn() -> Box<dyn Extension>;

/// A list of [Torrent] extension factories.
pub type ExtensionFactories = Vec<ExtensionFactory>;

/// Creates a new torrent [Storage] instance.
pub type StorageFactory = dyn FnOnce(StorageParams) -> Box<dyn Storage> + Send + Sync;

/// The states of the torrent
#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum TorrentState {
    /// The torrent is being initialized
    #[display(fmt = "initializing")]
    Initializing,
    /// The torrent is trying to retrieve the metadata from peers.
    #[display(fmt = "retrieving metadata")]
    RetrievingMetadata,
    /// The torrent has not started its download yet, and is currently checking existing files.
    #[display(fmt = "validating files")]
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

/// Requests a new torrent creation based on the given data.
/// This is the **recommended** way to create new torrents.
///
/// # Examples
///
/// ```rust,no_run
/// use popcorn_fx_torrent::torrent::{Torrent, TorrentFlags, TorrentMetadata, TorrentRequest, MagnetResult, ExtensionFactories, Result};
/// use popcorn_fx_torrent::torrent::storage::{DiskStorage};
/// use popcorn_fx_torrent::torrent::peer::extension::Extensions;
/// use popcorn_fx_torrent::torrent::peer::{PeerDiscovery, TcpPeerDiscovery};
///
/// fn create_new_torrent(
///     metadata: TorrentMetadata,
///     extensions: ExtensionFactories,
/// ) -> Result<Torrent> {
///     // create a tcp peer discovery for dialing and accepting tpc connections
///     let peer_discovery = TcpPeerDiscovery::new();
///
///     Torrent::request()
///         .metadata(metadata)
///         .options(TorrentFlags::AutoManaged)
///         .extensions(extensions)
///         .storage(|params| {
///             Box::new(DiskStorage::new(params.info_hash, params.path, params.files))
///         })
///         .peer_discovery(Box::new(peer_discovery))
///         .build()
/// }
/// ```
#[derive(Default)]
pub struct TorrentRequest {
    /// The torrent metadata information
    metadata: Option<TorrentMetadata>,
    /// The torrent options
    options: Option<TorrentFlags>,
    /// The torrent configuration
    config: Option<TorrentConfig>,
    /// The discovery strategies for peer connections.
    peer_discoveries: Option<Vec<Box<dyn PeerDiscovery>>>,
    /// The protocol extensions that should be enabled
    protocol_extensions: Option<ProtocolExtensionFlags>,
    /// The factories for creating the peer extensions that should be enabled for this torrent
    extensions: Option<ExtensionFactories>,
    /// The storage strategy to use for the torrent data
    storage: Option<Box<StorageFactory>>,
    /// The operations used by the torrent for processing data
    operations: Option<Vec<Box<dyn TorrentOperation>>>,
    /// The DHT node server to use for discovering peers
    dht: Option<DhtTracker>,
    /// The peer tracker manager for the torrent
    tracker_manager: Option<TrackerManager>,
}

impl TorrentRequest {
    /// Set the torrent metadata
    pub fn metadata(&mut self, metadata: TorrentMetadata) -> &mut Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the torrent options
    pub fn options(&mut self, options: TorrentFlags) -> &mut Self {
        self.options = Some(options);
        self
    }

    /// Set the torrent configuration
    pub fn config(&mut self, config: TorrentConfig) -> &mut Self {
        self.config = Some(config);
        self
    }

    /// Add the given peer dialer to the torrent.
    pub fn peer_discovery(&mut self, dialer: Box<dyn PeerDiscovery>) -> &mut Self {
        self.peer_discoveries.get_or_insert(Vec::new()).push(dialer);
        self
    }

    /// Set the given peer dialers of the torrent.
    pub fn peer_discoveries(&mut self, dialers: Vec<Box<dyn PeerDiscovery>>) -> &mut Self {
        self.peer_discoveries = Some(dialers);
        self
    }

    /// Set the protocol extensions that should be enabled
    pub fn protocol_extensions(&mut self, extensions: ProtocolExtensionFlags) -> &mut Self {
        self.protocol_extensions = Some(extensions);
        self
    }

    /// Add the given extension factory that should be activated.
    pub fn extension(&mut self, extension: ExtensionFactory) -> &mut Self {
        self.extensions.get_or_insert(Vec::new()).push(extension);
        self
    }

    /// Set the extension factories that should be activated for this torrent
    pub fn extensions(&mut self, extensions: ExtensionFactories) -> &mut Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set the underlying storage for storing the torrent file data.
    pub fn storage<F>(&mut self, storage: F) -> &mut Self
    where
        F: FnOnce(StorageParams) -> Box<dyn Storage> + Send + Sync + 'static,
    {
        self.storage = Some(Box::new(storage));
        self
    }

    /// Add the operation to the torrent for processing data.
    pub fn operation(&mut self, operation: Box<dyn TorrentOperation>) -> &mut Self {
        self.operations.get_or_insert(Vec::new()).push(operation);
        self
    }

    /// Set the operations used by the torrent for processing data
    pub fn operations(&mut self, operations: Vec<Box<dyn TorrentOperation>>) -> &mut Self {
        self.operations = Some(operations);
        self
    }

    /// Set the DHT node server to use for discovering peers.
    pub fn dht(&mut self, dht: DhtTracker) -> &mut Self {
        self.dht = Some(dht);
        self
    }

    /// Set the optional DHT node server to use for discovering peers.
    /// This will override any previously configured DHT node server, even if the value is [None].
    pub fn dht_option(&mut self, dht: Option<DhtTracker>) -> &mut Self {
        self.dht = dht;
        self
    }

    /// Set the tracker manager for discovering peers.
    pub fn tracker_manager(&mut self, tracker_manager: TrackerManager) -> &mut Self {
        self.tracker_manager.get_or_insert(tracker_manager);
        self
    }

    /// Build the torrent from the given data.
    /// This is the same as calling `Torrent::try_from(self)`.
    pub fn build(&mut self) -> Result<Torrent> {
        Torrent::try_from(self)
    }
}

impl Debug for TorrentRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentRequest")
            .field("metadata", &self.metadata)
            .field("options", &self.options)
            .field("config", &self.config)
            .field("peer_discoveries", &self.peer_discoveries)
            .field("protocol_extensions", &self.protocol_extensions)
            .field("operations", &self.operations)
            .field("dht", &self.dht)
            .field("tracker_manager", &self.tracker_manager)
            .finish()
    }
}

impl TryFrom<&mut TorrentRequest> for Torrent {
    type Error = TorrentError;

    fn try_from(request: &mut TorrentRequest) -> Result<Self> {
        let metadata = request.metadata.take().ok_or(TorrentError::InvalidRequest(
            "metadata is missing".to_string(),
        ))?;
        let peer_discoveries = request
            .peer_discoveries
            .take()
            .unwrap_or(Vec::with_capacity(0));
        let protocol_extensions = request
            .protocol_extensions
            .unwrap_or_else(DEFAULT_TORRENT_PROTOCOL_EXTENSIONS);
        let extensions = request
            .extensions
            .take()
            .unwrap_or_else(|| DEFAULT_TORRENT_EXTENSIONS());
        let options = request.options.unwrap_or(TorrentFlags::default());
        let config = request
            .config
            .take()
            .unwrap_or_else(|| TorrentConfig::builder().build());
        let piece_pool = PiecePool::new();
        let file_pool = FilePool::new(piece_pool.clone());
        let storage = request.storage.take().ok_or(TorrentError::InvalidRequest(
            "file storage is missing".to_string(),
        ))?;
        let storage_params = StorageParams {
            info_hash: metadata.info_hash.clone(),
            path: config.path().to_path_buf(),
            files: file_pool.clone(),
        };
        let operations = request.operations.take().unwrap_or_else(|| {
            vec![
                Box::new(TorrentTrackersOperation::new()),
                #[cfg(feature = "dht")]
                Box::new(TorrentDhtNodesOperation::new()),
                #[cfg(feature = "dht")]
                Box::new(TorrentDhtPeersOperation::new()),
                Box::new(TorrentConnectPeersOperation::new()),
                Box::new(TorrentMetadataOperation::new()),
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
                Box::new(TorrentFileValidationOperation::new()),
            ]
        });
        let dht = request.dht.take();
        let tracker_manager =
            request
                .tracker_manager
                .take()
                .ok_or(TorrentError::InvalidRequest(
                    "tracker_manager is missing".to_string(),
                ))?;

        Ok(Self::new(
            metadata,
            peer_discoveries,
            protocol_extensions,
            extensions,
            options,
            config,
            piece_pool,
            file_pool,
            storage(storage_params),
            operations,
            dht,
            tracker_manager,
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
    /// The [TorrentContext] reference exposes additional internal data of the torrent which is otherwise not exposed on the [Torrent].
    ///
    /// ## Remarks
    ///
    /// The shared reference shouldn't be kept alive for too long, but can be used for spawning async tasks.
    /// It's recommended to cancel the spawned task when the operation is being dropped.
    ///
    /// # Returns
    ///
    /// It returns how the chain should proceed.
    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult;
}

/// The result of executing a torrent operation.
#[derive(Debug, PartialEq)]
pub enum TorrentOperationResult {
    /// Continue the operations chain
    Continue,
    /// Stop the operations chain
    Stop,
}

/// The result metrics from a tracker scrape.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrapeMetrics {
    /// The number of active peers that have completed downloading.
    pub complete: u32,
    /// The number of active peers that have not completed downloading.
    pub incomplete: u32,
    /// The number of peers that have ever completed downloading.
    pub downloaded: u32,
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentEvent {
    /// Invoked when the status of the torrent has changed
    #[display(fmt = "torrent state has changed to {}", _0)]
    StateChanged(TorrentState),
    /// Invoked when the torrent metadata has been changed
    #[display(fmt = "torrent metadata has been changed")]
    MetadataChanged(TorrentMetadata),
    /// Invoked when a new peer connection has been established
    #[display(fmt = "peer {} has been connected", _0)]
    PeerConnected(PeerClientInfo),
    /// Invoked when an existing peer connection has closed.
    #[display(fmt = "peer {} has been disconnected", _0)]
    PeerDisconnected(PeerClientInfo),
    /// Invoked when the active trackers have been changed
    #[display(fmt = "trackers have changed")]
    TrackersChanged,
    /// Invoked when the pieces have changed of the torrent
    #[display(fmt = "torrent pieces have changed to {}", _0)]
    PiecesChanged(usize),
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
    /// Invoked when the torrent metrics have been updated
    #[display(fmt = "torrent stats changed {:?}", _0)]
    Stats(Metrics),
}

/// A torrent is an actual tracked torrent which is communicating with one or more trackers and peers.
///
/// Use [TorrentMetadata] if you only want to retrieve the metadata of a torrent.
#[derive(Debug)]
pub struct Torrent {
    handle: TorrentHandle,
    /// The unique peer id of this torrent
    /// This id is used as our client id when connecting to peers
    peer_id: PeerId,
    /// The metric stats of the torrent which is a reference.
    metrics: Metrics,
    /// The reference info of the torrent
    /// If the torrent reference is the original owner, then dropping this instance will stop the torrent
    ref_type: TorrentRefType,
    /// The inner torrent instance reference holder
    instance: Weak<TorrentContext>,
}

impl Torrent {
    /// Create a new request builder for creating a new torrent.
    /// See [TorrentRequest] for more information.
    pub fn request() -> TorrentRequest {
        TorrentRequest::default()
    }

    fn new(
        metadata: TorrentMetadata,
        peer_discoveries: Vec<Box<dyn PeerDiscovery>>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: ExtensionFactories,
        options: TorrentFlags,
        config: TorrentConfig,
        piece_pool: PiecePool,
        file_pool: FilePool,
        storage: Box<dyn Storage>,
        operations: Vec<Box<dyn TorrentOperation>>,
        dht: Option<DhtTracker>,
        tracker_manager: TrackerManager,
    ) -> Self {
        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let peer_discovery_addrs: Vec<SocketAddr> =
            peer_discoveries.iter().map(|e| e.addr()).cloned().collect();
        let info_hash = metadata.info_hash.clone();
        let (event_sender, command_receiver) = unbounded_channel();
        let (peer_subscriber, peer_event_receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let location = config.path().to_path_buf();
        let metrics = Metrics::new();
        let context = Arc::new(TorrentContext {
            handle,
            metadata: RwLock::new(metadata),
            peer_id,
            peer_discovery_addrs: peer_discovery_addrs.clone(),
            tracker_manager,
            dht,
            peer_pool: PeerPool::new(handle, config.peers_upper_limit),
            peer_subscriber,
            peer_discoveries: Arc::new(peer_discoveries),
            pieces: piece_pool,
            piece_chunk_pool: PieceChunkPool::new(),
            pending_piece_requests: Default::default(),
            request_download_permits: Arc::new(Semaphore::new(config.max_in_flight_pieces)),
            request_upload_permits: Arc::new(Semaphore::new(config.peers_upload_slots)),
            files: file_pool,
            protocol_extensions,
            extensions,
            storage,
            state: RwLock::new(Default::default()),
            options: RwLock::new(options),
            config: RwLock::new(config),
            metrics: metrics.clone(),
            event_sender,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token,
        });

        let torrent = Self {
            handle,
            peer_id,
            metrics,
            ref_type: TorrentRefType::Owner,
            instance: Arc::downgrade(&context),
        };

        // create a new separate thread which manages the internal torrent resources
        // this thread is automatically cancelled when the torrent is dropped
        tokio::spawn(async move {
            // start the main loop of the torrent
            context
                .start(&context, operations, command_receiver, peer_event_receiver)
                .await;
        });

        info!(
            "Torrent {} (info hash {}) created with storage location {:?}",
            torrent, info_hash, location
        );
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

    /// Get the port of one of the listeners for accepting incoming peer connections.
    /// This is most of the time the port of the first listener.
    ///
    /// # Returns
    ///
    /// It returns the port number of one of its listeners or [None] if the torrent is not listening for peer connections.
    pub fn peer_port(&self) -> Option<u16> {
        if let Some(inner) = self.instance() {
            return inner.peer_port();
        }

        None
    }

    /// Get all ports the torrent is listening on for accepting incoming peer connections.
    /// It returns the slice of ports or an empty slice if the torrent is not listening for peer connections.
    pub fn peer_ports(&self) -> Vec<u16> {
        if let Some(inner) = self.instance() {
            return inner.peer_ports();
        }

        vec![]
    }

    /// Check if this torrent handle is still valid.
    ///
    /// # Returns
    ///
    /// Returns true if the handle is still valid, else false.
    pub fn is_valid(&self) -> bool {
        self.instance().is_some()
    }

    /// Get the absolute filesystem path to a given file in the torrent.
    ///
    /// This combines the torrent's storage path with the file's [`torrent_path`]
    /// to produce a full path on the local filesystem.
    pub async fn absolute_file_path(&self, file: &File) -> PathBuf {
        if let Some(inner) = self.instance() {
            return inner.absolute_file_path(file).await;
        }

        PathBuf::new()
    }

    /// Get the absolute path to the torrent location.
    /// This can either be a file or directory to the torrent depending on the type of the torrent.
    ///
    /// The path is only available when the `metadata` of the torrent is known.
    /// See [Torrent::is_metadata_known].
    ///
    /// # Returns
    ///
    /// It returns the location of the torrent if the metadata is known, else [None].
    pub async fn path(&self) -> Option<PathBuf> {
        if let Some(inner) = self.instance() {
            return inner.path().await;
        }

        None
    }

    /// Get the current state of the torrent.
    ///
    /// # Returns
    ///
    /// It returns the state of this torrent.
    pub async fn state(&self) -> TorrentState {
        match self.instance() {
            None => TorrentState::Error,
            Some(e) => e.state().await,
        }
    }

    /// Get the metric statics of the torrent.
    /// These are collected from each active peer connection within the torrent and are periodically scraped.
    ///
    /// # Returns
    ///
    /// It returns the statics of this torrent.
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Get the metadata of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the metadata of the torrent, or [TorrentError::InvalidHandle] when the torrent is invalid.
    pub async fn metadata(&self) -> Result<TorrentMetadata> {
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

        TorrentFlags::none()
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
    /// It returns the total pieces of this torrent when known.
    pub async fn total_pieces(&self) -> usize {
        if let Some(inner) = self.instance() {
            return inner.piece_pool().len().await;
        }

        0
    }

    /// Get the total number of completed pieces for this torrent.
    ///
    /// # Returns
    ///
    /// It returns the total amount of completed pieces of this torrent when known.
    pub async fn total_completed_pieces(&self) -> usize {
        if let Some(inner) = self.instance() {
            return inner.total_completed_pieces().await;
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
            let pieces = inner.pieces.pieces().await;
            return Some(pieces).filter(|e| e.len() > 0);
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
    pub async fn piece_info(&self, piece: &PieceIndex) -> Option<Piece> {
        if let Some(inner) = self.instance() {
            return inner.pieces.get(&piece).await;
        }

        None
    }

    /// Get the priorities of the pieces.
    /// It might return an empty array if the metadata is still being retrieved.
    pub async fn piece_priorities(&self) -> BTreeMap<PieceIndex, PiecePriority> {
        if let Some(inner) = self.instance() {
            return inner.pieces.priorities().await;
        }

        BTreeMap::new()
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
    pub async fn has_piece(&self, piece: &PieceIndex) -> bool {
        if let Some(inner) = self.instance() {
            return inner.pieces.contains(piece).await;
        }

        false
    }

    /// Prioritize the given bytes within the torrent.
    /// This will match the bytes against the relevant pieces, and prioritize those pieces.
    pub async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>, priority: PiecePriority) {
        if let Some(inner) = self.instance() {
            inner.prioritize_bytes(bytes, priority).await;
        }
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

    /// Set the priorities of the torrent files.
    /// Use [Torrent::files] to get the current files with their respective [FileIndex].
    ///
    /// Providing all file indexes of the torrent is not required.
    pub async fn prioritize_files(&self, priorities: Vec<(FileIndex, PiecePriority)>) {
        if let Some(inner) = self.instance() {
            inner.prioritize_files(priorities).await;
        }
    }

    /// Get the total amount of active peer connections of the torrent.
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
    pub async fn announce(&self) -> Result<AnnouncementResult> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        // try to wait for at least 2 connections
        if inner.active_tracker_connections().await == 0 {
            Self::wait_for_trackers(&inner, 2).await;
        }

        Ok(inner.announce_all().await)
    }

    /// Get a "weak" reference to a peer in this torrent identified by `handle`.
    ///
    /// This looks up the `handle` within the peer pool of the torrent.
    /// When found, it will create a weak reference to the [Peer].
    /// Before calling a method, make sure to check if the reference is still valid by calling [TorrentPeer::is_valid].
    ///
    /// # Arguments
    ///
    /// * `handle` — The [`PeerHandle`] reference to look up.
    ///
    /// # Returns
    ///
    /// It returns the torrent peer (weak reference) when found, else [None].
    pub async fn peer(&self, handle: &PeerHandle) -> Option<TorrentPeer> {
        if let Some(inner) = self.instance() {
            return inner.peer_pool.get(&handle).await;
        }

        None
    }

    /// Scrape the trackers of the torrent to retrieve the metrics.
    pub async fn scrape(&self) -> Result<ScrapeMetrics> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        // try to wait for at least 2 connections
        if inner.active_tracker_connections().await == 0 {
            Self::wait_for_trackers(&inner, 2).await;
        }

        inner.scrape().await
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
    pub async fn read_piece(&self, piece: &PieceIndex) -> Result<Vec<u8>> {
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

    /// Try to read the bytes from the given torrent file.
    /// This reads all available bytes of the file stored within the [Storage].
    ///
    /// Returns the amount of bytes read and the byte buffer.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_to_end(&self, file: &File) -> Result<(usize, Vec<u8>)> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        inner.read_file_to_end(file).await
    }

    /// Get a temporary strong reference to the inner torrent.
    pub(crate) fn instance(&self) -> Option<Arc<TorrentContext>> {
        self.instance.upgrade()
    }

    /// Wait for the given number of active trackers.
    async fn wait_for_trackers(inner: &Arc<TorrentContext>, num_of_trackers: usize) {
        let notifier = Arc::new(Notify::new());
        let mut receiver = inner.subscribe();
        let cancellation_token = CancellationToken::new();

        let inner_cancel = cancellation_token.clone();
        let inner_notifier = notifier.clone();
        tokio::spawn(async move {
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
            if inner.active_tracker_connections().await >= num_of_trackers {
                break;
            }
        }

        cancellation_token.cancel();
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
            metrics: self.metrics.clone(),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle)
    }
}

impl Drop for Torrent {
    fn drop(&mut self) {
        // if the owning torrent gets dropped
        // we need to make sure that any running threads are cancelled on the inner torrent
        if self.ref_type == TorrentRefType::Owner {
            if let Some(context) = self.instance.upgrade() {
                trace!("Dropping torrent {}", self.handle);
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
    pub peer: PeerClientInfo,
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
pub enum TorrentCommandEvent {
    /// Indicates that the torrent options (flags) have changed
    OptionsChanged,
    /// Indicates that the torrent wants to connect to a new tracker
    ConnectToTracker(TrackerEntry),
    /// Indicates that the given peer has been connected and needs to be managed by the torrent
    PeerConnected(Box<dyn Peer>),
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

impl PartialEq for TorrentCommandEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TorrentCommandEvent::OptionsChanged, TorrentCommandEvent::OptionsChanged) => true,
            (
                TorrentCommandEvent::ConnectToTracker(_),
                TorrentCommandEvent::ConnectToTracker(_),
            ) => true,
            (TorrentCommandEvent::PeerConnected(_), TorrentCommandEvent::PeerConnected(_)) => true,
            (TorrentCommandEvent::PeerClosed(_), TorrentCommandEvent::PeerClosed(_)) => true,
            (
                TorrentCommandEvent::PiecePartCompleted(_, _),
                TorrentCommandEvent::PiecePartCompleted(_, _),
            ) => true,
            (TorrentCommandEvent::PieceCompleted(_), TorrentCommandEvent::PieceCompleted(_)) => {
                true
            }
            (
                TorrentCommandEvent::PendingRequestRejected(_),
                TorrentCommandEvent::PendingRequestRejected(_),
            ) => true,
            (
                TorrentCommandEvent::NotifyPeersHavePieces(_),
                TorrentCommandEvent::NotifyPeersHavePieces(_),
            ) => true,
            (TorrentCommandEvent::State(_), TorrentCommandEvent::State(_)) => true,
            _ => false,
        }
    }
}

impl Debug for TorrentCommandEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentCommandEvent::OptionsChanged => write!(f, "OptionsChanged"),
            TorrentCommandEvent::ConnectToTracker(e) => write!(f, "ConnectToTracker({:?})", e),
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
    /// The addresses on which the torrent is listening for incoming peer connections
    peer_discovery_addrs: Vec<SocketAddr>,
    /// The torrent metadata information of the torrent
    /// This might still be incomplete if the torrent was created from a magnet link
    metadata: RwLock<TorrentMetadata>,
    /// The manager of the trackers for the torrent
    tracker_manager: TrackerManager,
    /// The dht server of the torrent
    dht: Option<DhtTracker>,

    /// The pool of peer connections
    peer_pool: PeerPool,
    /// The sender which is shared between all peers to inform the torrent of a [PeerEvent].
    peer_subscriber: Subscriber<PeerEvent>,
    /// The peer discoveries for creating outgoing and accepting incoming connections
    peer_discoveries: Arc<Vec<Box<dyn PeerDiscovery>>>,

    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: PiecePool,
    /// The pool which stores the received piece parts
    piece_chunk_pool: PieceChunkPool,
    /// The in-flight pending requests of pieces by peers
    pending_piece_requests: RwLock<HashMap<PieceIndex, Instant>>,

    /// The permit counter for requesting pieces from remote peers
    request_download_permits: Arc<Semaphore>,
    /// The permit counter for uploading pieces to remote peers
    request_upload_permits: Arc<Semaphore>,

    /// The torrent files
    files: FilePool,
    /// The storage interface of the torrent
    storage: Box<dyn Storage>,

    /// The immutable enabled protocol extensions for this torrent
    protocol_extensions: ProtocolExtensionFlags,
    /// The immutable peer extension factories for this torrent.
    /// These factories create the extensions for each established peer connection.
    extensions: ExtensionFactories,

    /// The state of the torrent
    state: RwLock<TorrentState>,
    /// The torrent options that are set for this torrent
    options: RwLock<TorrentFlags>,
    /// The torrent configuration
    config: RwLock<TorrentConfig>,
    /// The metrics of the torrent
    metrics: Metrics,
    /// The internal command event sender
    event_sender: UnboundedSender<TorrentCommandEvent>,
    /// The callbacks for the torrent events
    callbacks: MultiThreadedCallback<TorrentEvent>,
    /// The main loop cancellation token
    cancellation_token: CancellationToken,
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
    ) {
        let mut tracker_event_receiver = self.tracker_manager.subscribe();
        // the interval used to execute periodic torrent operations
        let mut operations_tick = time::interval(OPERATIONS_INTERVAL);
        let mut cleanup_interval = time::interval(Duration::from_secs(30));

        // register the torrent within the tracker
        if !self.add_torrent_to_tracker().await {
            return;
        }

        // execute the operations at the beginning of the loop
        select! {
            _ = self.cancellation_token.cancelled() => return,
            _ = Self::execute_operations_chain(context, &operations) => {}
        }

        let mut peer_connections = FuturesUnordered::from_iter(
            self.peer_discoveries
                .iter()
                .map(|e| self.accept_connections(e, context)),
        )
        .fuse();
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                event = command_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_command_event(event).await;
                    } else {
                        debug!("Torrent {} events channel closed", self);
                        break;
                    }
                }
                Some(event) = tracker_event_receiver.recv() => self.handle_tracker_event((*event).clone()).await,
                Some(event) = peer_event_receiver.recv() => self.handle_peer_event((*event).clone()).await,
                Some(_) = peer_connections.next() => {},
                _ = operations_tick.tick() => {
                    Self::execute_operations_chain(context, &operations).await;
                    self.update_stats().await;
                },
                _ = cleanup_interval.tick() => {
                    self.clean_peers().await;
                },
            }
        }

        // shutdown the peer pool
        self.peer_pool.shutdown().await;
        // inform the tracker the torrent is being stopped
        let metadata = self.metadata.read().await;
        self.tracker_manager
            .announce_all(&metadata.info_hash, AnnounceEvent::Stopped)
            .await;
        self.tracker_manager.remove_torrent(&metadata.info_hash);
        trace!("Torrent {} main loop ended", self);
    }

    /// Get the unique handle of the torrent.
    /// It returns an owned instance of the torrent handle.
    pub fn handle(&self) -> TorrentHandle {
        self.handle
    }

    /// Get the address on which the torrent is listening for new incoming connections.
    pub fn addr(&self) -> Option<SocketAddr> {
        self.peer_discovery_addrs.first().cloned()
    }

    /// Get the peer pool of the torrent.
    pub fn peer_pool(&self) -> &PeerPool {
        &self.peer_pool
    }

    /// Get the peer id of the torrent.
    /// This is the unique peer ID that is used within the communication with remote peers for this torrent.
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get the port of one of the listeners for accepting incoming peer connections.
    /// This is most of the time the port of the first listener.
    ///
    /// # Returns
    ///
    /// It returns the port number of one of its listeners or [None] if the torrent is not listening for peer connections.
    pub fn peer_port(&self) -> Option<u16> {
        self.peer_discovery_addrs
            .first()
            .map(|e| e.port())
            .filter(|e| *e != 0)
    }

    /// Get all ports the torrent is listening on for accepting incoming peer connections.
    /// It returns the slice of ports or an empty slice if the torrent is not listening for peer connections.
    pub fn peer_ports(&self) -> Vec<u16> {
        self.peer_discovery_addrs.iter().map(|e| e.port()).collect()
    }

    /// Get the peer dialers for establishing outgoing peer connections of the torrent.
    pub fn peer_dialers(&self) -> &Arc<Vec<Box<dyn PeerDiscovery>>> {
        &self.peer_discoveries
    }

    /// Returns a Future that gets fulfilled when the torrent is being cancelled/stopped.
    /// The future will complete immediately if the torrenbt is already cancelled when this method is called.
    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.cancellation_token.cancelled()
    }

    /// Returns a Future that gets fulfilled when the torrent is being cancelled/stopped.
    /// The future will complete immediately if the torrenbt is already cancelled when this method is called.
    pub fn cancelled_owned(&self) -> WaitForCancellationFutureOwned {
        self.cancellation_token.clone().cancelled_owned()
    }

    /// Get the enabled protocol extensions for the torrent.
    pub fn protocol_extensions(&self) -> ProtocolExtensionFlags {
        self.protocol_extensions
    }

    /// Get the tracker manager for the torrent.
    pub fn tracker_manager(&self) -> &TrackerManager {
        &self.tracker_manager
    }

    /// Get the absolute path to the torrent location.
    /// This can either be a file or directory to the torrent depending on the type of the torrent.
    pub async fn path(&self) -> Option<PathBuf> {
        let metadata = self.metadata.read().await;
        if let Some(info) = &metadata.info {
            let config = self.config.read().await;
            return Some(config.path().join(info.name()));
        }

        None
    }

    /// Get the state of the torrent.
    pub async fn state(&self) -> TorrentState {
        self.state.read().await.clone()
    }

    /// Get the known torrent transfer stats.
    pub async fn metrics(&self) -> &Metrics {
        &self.metrics
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
        self.tracker_manager.tracker_urls().await
    }

    /// Get an owned instance of the metadata from the torrent.
    /// It returns an owned instance of the metadata.
    pub async fn metadata(&self) -> TorrentMetadata {
        self.metadata.read().await.clone()
    }

    /// Get the metadata of the torrent.
    /// It returns a reference to the metadata lock.
    pub fn metadata_lock(&self) -> &RwLock<TorrentMetadata> {
        &self.metadata
    }

    /// Check if the metadata of the torrent is known.
    /// It returns false when the torrent is still retrieving the metadata, else true.
    pub async fn is_metadata_known(&self) -> bool {
        self.metadata.read().await.info.is_some()
    }

    /// Get an instance of the torrent command event sender.
    /// It returns an owned sender for the torrent command events.
    pub fn event_sender(&self) -> UnboundedSender<TorrentCommandEvent> {
        self.event_sender.clone()
    }

    /// Get the total amount of actively connected peers.
    /// This only counts peers that have not been closed yet, so it can be smaller than the peer pool.
    pub async fn active_peer_connections(&self) -> usize {
        self.peer_pool.active_peer_connections().await
    }

    /// Get the total amount of active tracker connections.
    /// This only counts trackers which have at least made one successful announcement.
    pub async fn active_tracker_connections(&self) -> usize {
        self.tracker_manager.trackers_len().await
    }

    /// Get the DHT tracker of the torrent.
    pub fn dht(&self) -> Option<&DhtTracker> {
        self.dht.as_ref()
    }

    /// Get the piece pool of the torrent.
    pub fn piece_pool(&self) -> &PiecePool {
        &self.pieces
    }

    /// Get the file pool of the torrent.
    pub fn file_pool(&self) -> &FilePool {
        &self.files
    }

    pub fn storage(&self) -> &Box<dyn Storage> {
        &self.storage
    }

    /// Get all wanted pieces which are currently not being requested by a [Peer].
    /// Pieces with the highest priority will be first.
    ///
    /// It returns all piece indexes for which the priority is not [PiecePriority::None], the piece has not been completed and
    /// no peer is requesting the data.
    ///
    /// ## Sorting
    ///
    /// The pieces are **sorted** by their priorities, meaning that pieces with [PiecePriority::High] will come before [PiecePriority::Normal].
    pub async fn wanted_request_pieces(&self) -> Vec<PieceIndex> {
        let piece_requests = self.pending_piece_requests.read().await;
        let is_end_game = self.is_end_game().await;

        self.pieces
            .wanted_pieces()
            .await
            .into_iter()
            .map(|e| e.index)
            // don't allow duplicate piece requests which have not timed out
            // the exclusion on this is only during the end-game phase of the torrent
            .filter(|e| {
                let should_request_piece = piece_requests
                    .get(e)
                    .filter(|e| e.elapsed() <= PEER_REQUEST_TIMEOUT)
                    .is_none();

                is_end_game || should_request_piece
            })
            .collect()
    }

    /// Get the total amount of wanted pieces by the torrent.
    pub async fn total_wanted_pieces(&self) -> usize {
        self.pieces.wanted_pieces().await.len()
    }

    /// Get if the given bytes have been completed downloading.
    /// It returns true if all bytes are completed, validated and written to the storage, else false.
    pub async fn has_bytes(&self, range: &std::ops::Range<usize>) -> bool {
        self.pieces
            .pieces()
            .await
            .iter()
            .filter(|e| {
                let piece_range = e.torrent_range();

                // check if there is any overlap with the given byte range and piece range
                piece_range.start < range.end && range.start < piece_range.end
            })
            .all(|e| e.is_completed())
    }

    /// Prioritize the given pieces within this torrent.
    pub async fn prioritize_pieces(&self, priorities: Vec<(PieceIndex, PiecePriority)>) {
        trace!("Torrent {} is prioritizing pieces {:?}", self, priorities);
        self.pieces.set_priorities(priorities.as_slice()).await;
        self.update_interested_pieces_stats().await;

        debug!("Torrent {} piece priorities have been changed", self);
        self.invoke_event(TorrentEvent::PiecePrioritiesChanged);

        // update the state of the torrent based on the new priorities
        // this can only be done after the init phase to not disrupt the init operations
        let is_not_init_state = !self.state.read().await.is_initializing_phase();
        if is_not_init_state {
            let new_state = self.determine_state().await;
            self.update_state(new_state).await;
        }
    }

    /// Prioritize the given bytes within the torrent.
    /// This will match the bytes against the relevant pieces, and prioritize those pieces.
    pub async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>, priority: PiecePriority) {
        let piece_priorities = self
            .find_relevant_pieces_for_bytes(bytes)
            .await
            .into_iter()
            .map(|piece| (piece.index, priority))
            .collect();
        self.prioritize_pieces(piece_priorities).await;
    }

    /// Check if the torrent has completed downloading all wanted pieces.
    pub async fn is_completed(&self) -> bool {
        self.pieces.is_completed().await
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
        let is_not_paused = !options.contains(TorrentFlags::Paused);
        let is_uploading_mode =
            options.contains(TorrentFlags::UploadMode) || options.contains(TorrentFlags::SeedMode);

        is_uploading_mode && is_not_paused
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

    /// Check if the torrent is currently paused.
    pub async fn is_paused(&self) -> bool {
        self.options.read().await.contains(TorrentFlags::Paused)
    }

    /// Check if the torrent is currently in the end-game phase.
    /// It returns true if it entered the end-game mode.
    pub async fn is_end_game(&self) -> bool {
        let interested_pieces = self.pieces.interested_pieces().await.len();
        if interested_pieces == 0 {
            return true;
        }

        let total_completed_pieces = self.pieces.bitfield().await.count_ones();
        // if only 3 percent, counted with a precision of 2 decimals, of the pieces are left to be completed
        // then we enter the end-game phase
        let percentage_completed_pieces =
            ((total_completed_pieces as f32 / interested_pieces as f32) * 10_000.0).round() / 100.0;

        percentage_completed_pieces >= 97.0
    }

    /// Determines the number of additional peer connections needed for the torrent.
    ///
    /// This function calculates how many more peer connections are required based on the
    /// current torrent state, configuration limits, and active connections. It ensures
    /// the number of connections stays within defined thresholds.
    ///
    /// # Returns
    ///
    /// It returns a number of additionally wanted connection, ensuring the total
    /// stays within the configured peer connection limits.
    pub async fn remaining_peer_connections_needed(&self) -> usize {
        let options = self.options.read().await;
        if options.contains(TorrentFlags::Paused) {
            return 0;
        }

        let state = *self.state.read().await;
        // if the torrent is validating files, then don't open any new peer connections during the process
        // if the torrent is finished, then don't actively reach out to new peers
        if matches!(
            state,
            TorrentState::CheckingFiles | TorrentState::Finished | TorrentState::Seeding
        ) {
            return 0;
        }

        let currently_active_peers = self.active_peer_connections().await;
        let config = self.config.read().await;

        let is_retrieving_data = options.contains(TorrentFlags::DownloadMode);
        let is_retrieving_metadata =
            options.contains(TorrentFlags::Metadata) && state == TorrentState::RetrievingMetadata;

        let peer_lower_bound = config.peers_lower_limit;
        let peer_upper_bound = config.peers_upper_limit;

        // if we're downloading or retrieving metadata, aim for the upper bound
        if is_retrieving_metadata || is_retrieving_data {
            return peer_upper_bound.saturating_sub(currently_active_peers);
        }

        // if we're not actively requesting any data, aim for the lower bound
        peer_lower_bound.saturating_sub(currently_active_peers)
    }

    /// Get all relevant pieces for the given torrent byte range.
    ///
    /// # Arguments
    ///
    /// * `torrent_bytes` - The torrent byte range to retrieve the relevant pieces of.
    ///
    /// # Returns
    ///
    /// It returns all pieces with at least 1 byte overlapping with the given range.
    pub async fn find_relevant_pieces_for_bytes(
        &self,
        torrent_bytes: &std::ops::Range<usize>,
    ) -> Vec<Piece> {
        self.pieces
            .pieces()
            .await
            .into_iter()
            .filter(|e| e.contains(torrent_bytes))
            .collect()
    }

    /// Try to find the [PiecePart] for the given piece and begin index.
    pub async fn find_piece_part(&self, piece: PieceIndex, begin: usize) -> Option<PiecePart> {
        self.pieces
            .pieces()
            .await
            .into_iter()
            .find(|e| e.index == piece)
            .and_then(|piece| piece.parts.into_iter().find(|part| part.begin == begin))
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
            .pieces()
            .await
            .into_iter()
            .filter(|piece| file.contains(&piece.torrent_range()))
            .collect()
    }

    /// Get the list of non-padding files contained in the torrent.
    ///
    /// This method filters out any files marked with the [`FileAttributeFlags::PaddingFile`] attribute,
    /// so padding files will **not** be included in the returned list.
    ///
    /// ## Remarks
    ///
    /// If the torrent's metadata has not yet been fully retrieved, this method will return an empty vector.
    pub async fn files(&self) -> Vec<File> {
        self.files
            .files()
            .await
            .into_iter()
            // filter out any padding files
            .filter_map(|file| {
                if !file.attributes().contains(FileAttributeFlags::PaddingFile) {
                    Some(file)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of non-padding files currently known in the torrent.
    ///
    /// Files marked with the [`FileAttributeFlags::PaddingFile`] attribute are excluded from the count.
    ///
    /// ## Remarks
    ///
    /// If the torrent's metadata has not yet been fully retrieved, this method will return `0`.
    pub async fn total_files(&self) -> usize {
        self.files.len().await
    }

    /// Prioritize the files of the torrent.
    /// This will update the underlying piece priorities of each file.
    ///
    /// Providing all file indexes of the torrent is not required.
    pub async fn prioritize_files(&self, priorities: Vec<(FileIndex, PiecePriority)>) {
        trace!("Torrent {} is prioritizing files {:?}", self, priorities);
        self.files.set_priorities(priorities.as_slice()).await;
    }

    /// Get the absolute filesystem path to a given file in the torrent.
    ///
    /// This combines the torrent's storage path with the file's [`torrent_path`]
    /// to produce a full path on the local filesystem.
    pub async fn absolute_file_path(&self, file: &File) -> PathBuf {
        self.config
            .read()
            .await
            .path()
            .join(file.torrent_path.as_path())
    }

    /// Get the total byte length of the torrent.
    ///
    /// # Returns
    ///
    /// It returns the total bytes of all files within the torrent.
    pub async fn len(&self) -> Option<usize> {
        self.metadata_lock()
            .read()
            .await
            .info
            .as_ref()
            .map(|e| e.len())
    }

    /// Get the list of currently discovered peers.
    pub async fn discovered_peers(&self) -> Vec<SocketAddr> {
        let info_hash = self.metadata.read().await.info_hash.clone();
        self.tracker_manager
            .discovered_peers(&info_hash)
            .await
            .unwrap_or_else(Vec::new)
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
    async fn add_peer(&self, peer: Box<dyn Peer>) {
        trace!("Torrent {} is trying to add new peer {}", self, peer);
        let info = peer.client();
        let subscriber = self.peer_subscriber.clone();
        peer.subscribe_with(subscriber);

        match self.peer_pool.add_peer(peer).await {
            Ok(_) => {
                debug!("Torrent {} added peer {}", self, info);
                self.invoke_event(TorrentEvent::PeerConnected(info));
            }
            Err(e) => {
                debug!("Torrent {} failed to add peer {}, {}", self, info, e);
            }
        }
    }

    /// Remove the given peer from the torrent as it has been closed.
    async fn remove_peer(&self, handle: &PeerHandle) {
        trace!("Removing peer {} from torrent {}", handle, self);
        if let Some(peer) = self.peer_pool.remove_peer(handle).await {
            let bitfield = peer.remote_piece_bitfield().await;

            // decrease the availability of the pieces that the peer had
            for (piece_index, _) in bitfield.iter().enumerate().filter(|(_, value)| *value) {
                self.pieces.update_availability(&piece_index, -1).await;
            }

            self.invoke_event(TorrentEvent::PeerDisconnected(peer.client()));
        }
    }

    /// Add the given metadata to the torrent.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// If the data was already known, this method does nothing.
    pub async fn add_metadata(&self, metadata_info: TorrentMetadataInfo) {
        let mut metadata = self.metadata.write().await;

        // verify if the metadata of the torrent is already known
        // if so, we ignore this update
        if metadata.info.is_some() {
            return;
        }

        // validate the received metadata against our info hash
        let info_hash = metadata.info_hash.clone();
        let is_metadata_invalid = metadata_info
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
            debug!("Torrent {} received invalid metadata", self);
            return;
        }

        (*metadata).info = Some(metadata_info);
        debug!("Torrent {} updated metadata of {}", self, info_hash);
        self.invoke_event(TorrentEvent::MetadataChanged(metadata.clone()));
    }

    /// Announce the torrent to all trackers.
    /// It returns the announcement result collected from all active trackers.
    pub async fn announce_all(&self) -> AnnouncementResult {
        let metadata = self.metadata.read().await;
        self.tracker_manager
            .announce_all(&metadata.info_hash, AnnounceEvent::Started)
            .await
    }

    /// Announce to all the trackers without waiting for the results.
    pub async fn make_announce_all(&self) {
        let metadata = self.metadata.read().await;
        self.tracker_manager
            .make_announcement_to_all(&metadata.info_hash, AnnounceEvent::Started)
    }

    /// Get the scrape metrics result from scraping all trackers for this torrent.
    pub async fn scrape(&self) -> Result<ScrapeMetrics> {
        trace!("Torrent {} is scraping trackers", self);
        let metadata = self.metadata.read().await;
        match self.tracker_manager.scrape(&metadata.info_hash).await {
            Ok(result) => {
                let info_hash = self.metadata.read().await.info_hash.clone();
                if let Some(metrics) = result.files.get(&info_hash) {
                    Ok(ScrapeMetrics {
                        complete: metrics.complete,
                        incomplete: metrics.incomplete,
                        downloaded: metrics.downloaded,
                    })
                } else {
                    Err(TorrentError::InvalidInfoHash(format!(
                        "info hash {} not found in scrape result",
                        info_hash
                    )))
                }
            }
            Err(e) => Err(TorrentError::Tracker(e)),
        }
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
        let metadata = self.metadata.read().await;
        match &state {
            TorrentState::Downloading => self
                .tracker_manager
                .make_announcement_to_all(&metadata.info_hash, AnnounceEvent::Started),
            TorrentState::Seeding | TorrentState::Finished => self
                .tracker_manager
                .make_announcement_to_all(&metadata.info_hash, AnnounceEvent::Completed),
            TorrentState::Paused => self
                .tracker_manager
                .make_announcement_to_all(&metadata.info_hash, AnnounceEvent::Paused),
            _ => {}
        }

        debug!("Torrent {} state updated to {:?}", self, state);
        self.invoke_event(TorrentEvent::StateChanged(state));
    }

    async fn update_stats(&self) {
        self.invoke_event(TorrentEvent::Stats(self.metrics.snapshot()));
        info!("Torrent {} stats {}", self, self.metrics);

        self.metrics.tick(OPERATIONS_INTERVAL)
    }

    /// Update the availability of the given piece indexes.
    /// This will increase or decrease the availability of the torrent pieces.
    ///
    /// This method can be used ot both increase and decrease the availability information
    /// to correctly establish the rarity of pieces.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The piece indexes that need to be updated.
    ///* `available` - Indicates if the pieces become available or unavailable.
    pub async fn update_piece_availabilities(&self, pieces: Vec<PieceIndex>, available: bool) {
        // check if the metadata is known and the pieces have been created
        if !self.is_metadata_known().await || self.piece_pool().len().await == 0 {
            trace!(
                "Torrent {} is unable to update piece availabilities, metadata or pieces are unknown",
                self
            );
            return;
        }

        for piece in pieces {
            let change = if available { 1 } else { -1 };
            self.pieces.update_availability(&piece, change).await;
        }
    }

    /// Set the pieces of the torrent.
    pub async fn update_pieces(&self, pieces: Vec<Piece>) {
        let total_pieces = pieces.len();
        trace!("Torrent {} updating {} pieces", self, total_pieces);

        self.pieces.set_pieces(pieces).await;

        {
            // update the piece availability based on the current peer connections
            let mut availability: BTreeMap<PieceIndex, u32> = BTreeMap::new();
            let mut peer_count = 0u32;

            {
                for peer in self
                    .peer_pool
                    .peers()
                    .await
                    .into_iter()
                    .filter_map(|peer| peer.upgrade())
                {
                    peer_count += 1;
                    for (piece_index, _) in peer
                        .remote_piece_bitfield()
                        .await
                        .into_iter()
                        .enumerate()
                        .filter(|(_, value)| *value)
                    {
                        *availability.entry(piece_index).or_insert(0) += 1;
                    }
                }
            }

            let availability_len = availability.len();
            if availability_len > 0 {
                for (piece, availability) in availability {
                    self.pieces
                        .update_availability(&piece, availability as i32)
                        .await;
                }
                debug!(
                    "Torrent {} updated {} piece availabilities from {} peers",
                    self, availability_len, peer_count
                );
            }
        }

        debug!(
            "Torrent {} updated pieces to a total of {}",
            self, total_pieces
        );
        self.update_interested_pieces_stats().await;
        self.invoke_event(TorrentEvent::PiecesChanged(total_pieces));
    }

    /// Set the given piece as completed.
    /// This can be called by file validation operations to indicate that a piece has been stored in the storage.
    ///
    /// ## Remark
    ///
    /// This function doesn't verify if the piece is actually valid.
    pub async fn piece_completed(&self, piece: PieceIndex) {
        self.pieces_completed(vec![piece]).await;
    }

    /// Set the given pieces as completed.
    /// This can be called by file validation operations to indicate that a piece has been stored in the storage.
    ///
    /// ## Remark
    ///    
    /// This function doesn't verify if the pieces are actually valid.
    pub async fn pieces_completed(&self, pieces: Vec<PieceIndex>) {
        trace!("Torrent {} marking pieces {:?} as completed", self, pieces);
        let mut total_wanted_completed_size = 0;
        let mut total_completed_pieces_size = 0;
        let mut total_wanted_completed_pieces = 0;
        let mut total_completed_pieces = 0;

        {
            let mut pending_requests = self.pending_piece_requests.write().await;
            for piece in pieces.iter() {
                self.pieces.set_completed(piece, true).await;
                if let Some(piece) = self.pieces.get(piece).await {
                    total_completed_pieces_size += piece.length;
                    total_completed_pieces += 1;

                    if piece.priority != PiecePriority::None {
                        total_wanted_completed_size += piece.length;
                        total_wanted_completed_pieces += 1;
                    }
                } else {
                    warn!(
                        "Torrent {} received unknown completed piece {}",
                        self, piece
                    );
                }

                // remove the pending request
                pending_requests.remove(&piece);
            }
        }

        self.metrics.completed_pieces.inc_by(total_completed_pieces);
        self.metrics
            .wanted_completed_pieces
            .inc_by(total_wanted_completed_pieces);
        self.metrics
            .completed_size
            .inc_by(total_completed_pieces_size as u64);
        self.metrics
            .wanted_completed_size
            .inc_by(total_wanted_completed_size as u64);

        // inform the subscribers about each completed piece
        for piece in pieces.iter() {
            debug!("Torrent {} piece {} has been completed", self, piece);
            self.invoke_event(TorrentEvent::PieceCompleted(*piece));
        }

        // check if the all wanted pieces have been completed
        let is_completed = self.is_completed().await;
        if is_completed {
            // offload the state change to the main loop
            self.send_command_event(TorrentCommandEvent::State(TorrentState::Finished));
        }

        // notify the connected peers about the completed pieces
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
        self.files.set_files(files).await;
        self.invoke_event(TorrentEvent::FilesChanged);
    }

    /// Update the stats info of all interested pieces by the torrent.
    async fn update_interested_pieces_stats(&self) {
        let mut wanted_pieces = 0;
        let mut wanted_completed_pieces = 0;
        let mut wanted_size = 0;
        let mut wanted_completed_size = 0;

        {
            for piece_index in self.pieces.interested_pieces().await {
                if let Some(piece) = self.pieces.get(&piece_index).await {
                    wanted_pieces += 1;
                    wanted_size += piece.length;

                    if piece.is_completed() {
                        wanted_completed_pieces += 1;
                        wanted_completed_size += piece.length;
                    }
                }
            }
        }

        self.metrics.wanted_pieces.set(wanted_pieces);
        self.metrics
            .wanted_completed_pieces
            .set(wanted_completed_pieces);
        self.metrics.wanted_size.set(wanted_size as u64);
        self.metrics
            .wanted_completed_size
            .set(wanted_completed_size as u64);
    }

    /// Cancel all currently queued pending requests of the torrent.
    /// This will clear all pending requests from the buffer.
    pub async fn cancel_all_pending_requests(&self) {
        // TODO: cancel pending requests in the peer
    }

    /// Resume the torrent.
    /// This will put the torrent back into [TorrentFlags::DownloadMode], trying to download any missing pieces.
    pub async fn resume(&self) {
        self.add_options(TorrentFlags::DownloadMode | TorrentFlags::Metadata)
            .await;
        self.remove_options(TorrentFlags::Paused).await;

        // announce to the trackers if we don't know any peers
        if self.peer_pool.num_connect_candidates().await == 0 {
            let metadata = self.metadata.read().await;
            self.tracker_manager
                .make_announcement_to_all(&metadata.info_hash, AnnounceEvent::Started);
        }

        let wanted_pieces = self.total_wanted_pieces().await;
        debug!(
            "Torrent {} is resuming with {} wanted remaining pieces",
            self, wanted_pieces
        );
    }

    /// Pause the torrent operations.
    pub async fn pause(&self) {
        self.add_options(TorrentFlags::Paused).await;
        self.send_command_event(TorrentCommandEvent::OptionsChanged);
        self.send_command_event(TorrentCommandEvent::State(TorrentState::Paused));
    }

    /// Add the specified peer addresses to the peer pool of the torrent.
    ///
    /// These peers will be considered as potential connection targets in the future,
    /// particularly when the torrent requires additional connections.
    /// The provided addresses are queued for possible use; there is no immediate
    /// guarantee that connections will be attempted right away.
    pub async fn add_peer_addresses(&self, peer_addrs: Vec<SocketAddr>) {
        let addr = self.peer_discovery_addrs.first().cloned();
        self.peer_pool.add_peer_addresses(peer_addrs, addr).await;
    }

    /// Handle a command event from the channel of the torrent.
    async fn handle_command_event(&self, event: TorrentCommandEvent) {
        trace!("Torrent {} handling command event {:?}", self, event);
        match event {
            TorrentCommandEvent::OptionsChanged => self.options_changed().await,
            TorrentCommandEvent::ConnectToTracker(e) => self.add_tracker_async(e).await,
            TorrentCommandEvent::PeerConnected(peer) => self.add_peer(peer).await,
            TorrentCommandEvent::PeerClosed(handle) => self.remove_peer(&handle).await,
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
            TrackerManagerEvent::PeersDiscovered(info_hash, peers) => {
                if info_hash == self.metadata.read().await.info_hash {
                    self.add_peer_addresses(peers).await
                }
            }
            TrackerManagerEvent::TrackerAdded(handle) => {
                let is_paused = self.options.read().await.contains(TorrentFlags::Paused);
                let is_pieces_known = self.pieces.len().await > 0;
                let is_completed = self.is_completed().await;
                let mut event = AnnounceEvent::Started;

                if is_paused {
                    event = AnnounceEvent::Paused;
                } else if is_pieces_known && is_completed {
                    event = AnnounceEvent::Completed;
                }

                {
                    let metadata = self.metadata.read().await;
                    self.tracker_manager
                        .make_announcement(handle, &metadata.info_hash, event);
                }

                self.invoke_event(TorrentEvent::TrackersChanged);
            }
            _ => {}
        }
    }

    async fn handle_incoming_peer_connection(
        &self,
        torrent: &Arc<TorrentContext>,
        entry: PeerEntry,
    ) {
        trace!(
            "Torrent {} is trying to accept incoming {} peer connection",
            self,
            entry.socket_addr
        );
        let extensions = self.extensions();
        let timeout = self.config.read().await.peer_connection_timeout;

        match BitTorrentPeer::new_inbound(
            self.peer_id,
            entry.socket_addr,
            entry.stream,
            torrent.clone(),
            self.protocol_extensions,
            extensions,
            timeout,
        )
        .await
        {
            Ok(peer) => {
                debug!("Torrent {} established connection with peer {}", self, peer);
                self.add_peer(Box::new(peer)).await;
            }
            Err(e) => debug!(
                "Torrent {} failed to accept incoming peer connection {}, {}",
                self, entry.socket_addr, e
            ),
        }
    }

    /// Handle the given peer event.
    /// This will update the torrent context info based on an event that occurred within one of its peers.
    async fn handle_peer_event(&self, event: PeerEvent) {
        match event {
            PeerEvent::PeersDiscovered(peers) => self.add_peer_addresses(peers).await,
            PeerEvent::PeersDropped(peers) => self.decrease_peer_addr_priority(peers).await,
            PeerEvent::RemoteAvailablePieces(pieces) => {
                self.update_piece_availabilities(pieces, true).await
            }
            PeerEvent::RemoteUnavailablePieces(pieces) => {
                self.update_piece_availabilities(pieces, false).await
            }
            PeerEvent::Stats(metrics) => {
                self.metrics.upload.inc_by(metrics.bytes_out.get());
                self.metrics
                    .upload_useful
                    .inc_by(metrics.bytes_out_useful.get());
                self.metrics.download.inc_by(metrics.bytes_in.get());
                self.metrics
                    .download_useful
                    .inc_by(metrics.bytes_in_useful.get());
            }
            _ => {}
        }
    }

    async fn decrease_peer_addr_priority(&self, peers: Vec<SocketAddr>) {
        for peer in peers {
            self.peer_pool.update_peer_rank(&peer, -1).await;
        }
    }

    async fn add_torrent_to_tracker(&self) -> bool {
        let info_hash = self.metadata.read().await.info_hash.clone();
        let peer_port = self.peer_port().unwrap_or(6881);

        if let Err(e) = self
            .tracker_manager
            .add_torrent(self.peer_id, peer_port, info_hash, self.metrics.clone())
            .await
        {
            error!(
                "Torrent {} failed to register with tracker manager, {}",
                self, e
            );
            self.update_state(TorrentState::Error).await;
            return false;
        }

        true
    }

    async fn process_pending_request_rejected(&self, request_rejection: PendingRequestRejected) {
        debug!(
            "Torrent {} received rejected request for {:?}, reason {:?}",
            self, request_rejection.part, request_rejection.reason
        );

        // release the pending request to be retried by another peer
        self.pending_piece_requests
            .write()
            .await
            .remove(&request_rejection.part.piece);
    }

    async fn process_completed_piece_part(&self, piece_part: PiecePart, data: Vec<u8>) {
        let piece = match self.pieces.get(&piece_part.piece).await {
            Some(piece) => piece,
            None => return,
        };

        // check if the piece has already been completed
        // this can happen "end game" as the same piece & parts are requested from multiple torrents
        if piece.is_completed() {
            debug!(
                "Torrent {} received piece {} part {} data which has already been completed",
                self, piece_part.piece, piece_part.part
            );
            return;
        }

        trace!(
            "Torrent {} writing piece {} part {} data (size {}) to chunk pool",
            self,
            piece_part.piece,
            piece_part.part,
            data.len()
        );
        match self
            .piece_chunk_pool
            .add_chunk(&piece_part, piece.len(), data)
            .await
        {
            Ok(_) => {
                // update the piece info
                self.pieces
                    .set_part_completed(&piece.index, &piece_part.part)
                    .await;
                self.pending_piece_requests
                    .write()
                    .await
                    .insert(piece.index, Instant::now());

                if self.pieces.is_piece_completed(&piece.index).await {
                    self.send_command_event(TorrentCommandEvent::PieceCompleted(piece_part.piece));
                }
            }
            Err(e) => warn!("Failed to add chunk data for {}, {}", self, e),
        }
    }

    async fn process_completed_piece(&self, piece: PieceIndex) {
        if let Some(data) = self.piece_chunk_pool.get(piece).await {
            let data_size = data.len();
            trace!(
                "Torrent {} is validating piece {} data (size {})",
                self,
                piece,
                data_size
            );
            let is_valid = self.validate_piece_index_data(&piece, &data).await;

            if is_valid {
                debug!(
                    "Torrent {} validated piece {} data (size {}) with success",
                    self, piece, data_size
                );

                match self.storage.write(&data, &piece, 0).await {
                    Ok(len) => {
                        trace!("Torrent {} wrote piece {} ({} bytes)", self, piece, len);
                        self.piece_completed(piece).await
                    }
                    Err(e) => {
                        error!(
                            "Torrent {} failed to write piece {} data, {}",
                            self, piece, e
                        );
                        // reset the pending piece to be retried
                        self.pending_piece_requests.write().await.remove(&piece);
                        self.metrics.wasted.inc_by(data_size as u64);
                    }
                }
            } else {
                trace!(
                    "Torrent {} validated piece {} data (size {}) as failure",
                    self,
                    piece,
                    data_size
                );
                self.pieces.set_completed(&piece, false).await;
                self.metrics.wasted.inc_by(data_size as u64);
            }
        } else {
            warn!(
                "Torrent {} received piece completion of {}, but no data is available in the chunk pool",
                self, piece
            );
        }
    }

    /// Process the new options of the torrent.
    async fn options_changed(&self) {
        // update the state of the torrent based on the new options
        // this can only be done after the init phase to not disrupt the init operations
        let is_not_init_state = !self.state.read().await.is_initializing_phase();
        if is_not_init_state {
            let state = self.determine_state().await;
            self.update_state(state).await;
        }
    }

    /// Try to determine the state the torrent currently has.
    /// It returns the expected state of the torrent without actually updating the state.
    pub async fn determine_state(&self) -> TorrentState {
        let is_paused: bool;
        let is_download_mode: bool;

        {
            let options = self.options.read().await;
            is_paused = options.contains(TorrentFlags::Paused);
            is_download_mode = options.contains(TorrentFlags::DownloadMode);
        }

        if is_paused {
            return TorrentState::Paused;
        }

        let total_pieces = self.pieces.len().await;
        if total_pieces == 0 {
            return TorrentState::Initializing;
        }

        if is_download_mode {
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
    pub async fn validate_piece_index_data(&self, piece: &PieceIndex, data: &[u8]) -> bool {
        if let Some(piece) = self.pieces.get(piece).await {
            return Self::validate_piece_data(&piece, data);
        } else {
            warn!(
                "Unable to validate piece data, piece {} is unknown within {}",
                piece, self
            );
        }

        false
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
        peer: &PeerClientInfo,
    ) {
        if let Some(part) = self.find_piece_part(piece, begin).await {
            debug!(
                "Torrent {} received rejected piece {} part {} from {:?}",
                self, piece, part.part, peer
            );
            self.send_command_event(TorrentCommandEvent::PendingRequestRejected(
                PendingRequestRejected {
                    part,
                    peer: peer.clone(),
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

    /// Get the total amount of completed pieces for the torrent.
    pub async fn total_completed_pieces(&self) -> usize {
        self.pieces.num_completed().await
    }

    /// Notify the torrent of an invalid received piece part.
    pub fn invalid_piece_data_received(&self, part: PiecePart, peer: &PeerClientInfo) {
        self.send_command_event(TorrentCommandEvent::PendingRequestRejected(
            PendingRequestRejected {
                part,
                peer: peer.clone(),
                reason: RequestRejectedReason::InvalidDataResponse,
            },
        ));
    }

    /// Get a request permit for the given piece to download piece data from a remote peer.
    /// A permit should be retrieved for each piece that is being requested from a peer.
    pub async fn request_download_permit(
        &self,
        piece: &PieceIndex,
    ) -> Option<OwnedSemaphorePermit> {
        if !self.is_download_allowed().await {
            return None;
        }

        // check if the request is already in-flight and not timed-out
        let is_end_game = self.is_end_game().await;
        let is_piece_download_allowed = self
            .pending_piece_requests
            .read()
            .await
            .get(piece)
            .filter(|e| e.elapsed() <= PEER_REQUEST_TIMEOUT)
            .is_none();
        if !is_end_game && !is_piece_download_allowed {
            trace!(
                "Torrent {} is already requesting piece {} data",
                self,
                piece
            );
            return None;
        }

        if let Some(permit) = self
            .request_download_permits
            .clone()
            .try_acquire_owned()
            .ok()
        {
            let mut piece_requests = self.pending_piece_requests.write().await;
            piece_requests.insert(*piece, Instant::now());
            return Some(permit);
        }

        None
    }

    /// Get a request permit to upload piece data to a remote peer.
    /// A permit is peer based and should only be requested when trying to unchoke the client peer.
    pub async fn request_upload_permit(&self) -> Option<OwnedSemaphorePermit> {
        if !self.is_upload_allowed().await {
            return None;
        }

        self.request_upload_permits.clone().try_acquire_owned().ok()
    }

    /// Try to read the bytes from the given torrent file.
    /// This reads all available bytes of the file stored within the [Storage].
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_file_to_end(&self, file: &File) -> Result<(usize, Vec<u8>)> {
        if let Some(piece) = self.pieces.get(&file.pieces.start).await {
            let len = file.len();
            let mut buffer = vec![0; len];
            let file_offset = file.torrent_offset.saturating_sub(piece.offset);

            let bytes_read = self
                .storage
                .read(&mut buffer, &piece.index, file_offset)
                .await?;

            return Ok((bytes_read, buffer[..bytes_read].to_vec()));
        }

        Err(TorrentError::DataUnavailable)
    }

    /// Try to read the given piece bytes.
    /// It will read the bytes from all relevant files which overlap with the given piece.
    ///
    /// ## Remarks
    ///
    /// This doesn't verify if the bytes are valid and completed.
    pub async fn read_piece(&self, piece: &PieceIndex) -> Result<Vec<u8>> {
        match self.pieces.get(piece).await {
            None => Err(TorrentError::DataUnavailable),
            Some(piece) => {
                let mut buffer = vec![0; piece.length];
                let bytes_read = self.storage.read(&mut buffer, &piece.index, 0).await?;
                if bytes_read != piece.len() {
                    return Err(TorrentError::Io(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!(
                            "wanted {} bytes, but got {} instead",
                            piece.len(),
                            bytes_read
                        ),
                    )));
                }

                Ok(buffer)
            }
        }
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
        // TODO: improve the retrieval of bytes
        self.read_piece(&piece).await.map(|e| e[range].to_vec())
    }

    /// Try to read the given bytes from the torrent.
    /// This reads all available bytes of one or more files from the torrent stored within the [Storage].
    /// The returned bytes will be padded with 0 if the available data is smaller than the requested range.
    ///
    /// # Arguments
    ///
    /// * `torrent_range` - The byte range within the torrent to read.
    ///
    /// # Returns
    ///
    /// It returns the bytes read from the torrent, padding the bytes with `0` if the data was not available.
    pub async fn read_bytes_with_padding(
        &self,
        torrent_range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>> {
        self.internal_read_bytes(torrent_range, true).await
    }

    /// Try to read the given bytes from the torrent.
    /// This reads all bytes of one or more files from the torrent stored within the [Storage].
    ///
    /// # Arguments
    ///
    /// * `torrent_range` - The byte range within the torrent to read.
    ///
    /// # Returns
    ///
    /// It returns the bytes read from the torrent, returning a [TorrentError] if data was not available.
    pub async fn read_bytes(&self, torrent_range: std::ops::Range<usize>) -> Result<Vec<u8>> {
        self.internal_read_bytes(torrent_range, false).await
    }

    async fn internal_read_bytes(
        &self,
        torrent_range: std::ops::Range<usize>,
        with_padding: bool,
    ) -> Result<Vec<u8>> {
        // verify that the given range is not longer than the total torrent size
        let length = self.len().await.ok_or(TorrentError::InvalidMetadata(
            "metadata is unknown".to_string(),
        ))?;
        if torrent_range.is_empty() || torrent_range.end > length {
            return Err(TorrentError::InvalidRange(torrent_range));
        }

        let pieces = self.pieces.pieces().await;
        let starting_piece = pieces
            .iter()
            .find(|piece| {
                torrent_range.start >= piece.offset
                    && torrent_range.start <= piece.offset + piece.length
            })
            .ok_or(TorrentError::DataUnavailable)?;
        let offset = torrent_range.start.saturating_sub(starting_piece.offset);
        let mut buffer = vec![0u8; torrent_range.len()];

        let bytes_read = self
            .storage
            .read(&mut buffer, &starting_piece.index, offset)
            .await?;
        if bytes_read < torrent_range.len() && !with_padding {
            return Err(TorrentError::DataUnavailable);
        }

        Ok(buffer)
    }

    /// Cleanup the peer resources which have been closed or are no longer valid.
    async fn clean_peers(&self) {
        trace!("Torrent {} is executing peer cleanup cycle", self);
        for peer in self.peer_pool.clean().await {
            self.callbacks
                .invoke(TorrentEvent::PeerDisconnected(peer.client()));
        }
    }

    /// Notify the peers about the pieces that have become available.
    async fn notify_peers_have_pieces(&self, pieces: Vec<PieceIndex>) {
        for peer in self.peer_pool.peers.read().await.values() {
            peer.notify_piece_availability(pieces.clone());
        }
    }

    /// Accept incoming connections discovered by the peer discovery.
    async fn accept_connections(
        &self,
        discovery: &Box<dyn PeerDiscovery>,
        context: &Arc<TorrentContext>,
    ) {
        while let Some(entry) = discovery.recv().await {
            self.handle_incoming_peer_connection(context, entry).await;
        }
    }

    /// Get the peer extensions of the torrent.
    /// These extensions should be activated for each established peer connection of the torrent.
    pub fn extensions(&self) -> Vec<Box<dyn Extension>> {
        self.extensions.iter().map(|e| e()).collect()
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

    /// Validate the given piece data.
    /// The data will be validated against the underlying hash of the piece.
    ///
    /// # Important
    ///
    /// This is computationally expensive operation and should be executed on a thread pool.
    ///
    /// # Returns
    ///
    /// It returns `true` if the data is valid for the given piece, else `false`.
    pub fn validate_piece_data(piece: &Piece, data: &[u8]) -> bool {
        let hash = &piece.hash;

        if hash.has_v2() {
            let actual_hash = Sha256::digest(&data);
            hash.hash_v2()
                .map_or(false, |v2_hash| v2_hash == actual_hash.as_slice())
        } else {
            let actual_hash = Sha1::digest(&data);
            hash.hash_v1()
                .map_or(false, |v1_hash| v1_hash == actual_hash.as_slice())
        }
    }

    /// Execute the torrent operations chain.
    ///
    /// This will execute the operations in order as defined by the chain.
    /// If an operation returns [None], the execution chain will be interrupted.
    async fn execute_operations_chain(
        context: &Arc<TorrentContext>,
        operations: &Vec<Box<dyn TorrentOperation>>,
    ) {
        for operation in operations.iter() {
            let start = Instant::now();
            let execution_result = operation.execute(context).await;
            let elapsed = start.elapsed();
            trace!(
                "Operation {} resulted in {:?} after {} millis for {}",
                operation.name(),
                execution_result,
                elapsed.as_millis(),
                context
            );
            if execution_result == TorrentOperationResult::Stop {
                break;
            }
        }
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

    use crate::init_logger;
    use crate::torrent::operation::{
        TorrentConnectPeersOperation, TorrentCreateFilesOperation, TorrentCreatePiecesOperation,
        TorrentFileValidationOperation,
    };
    use crate::torrent::InfoHash;
    use crate::{create_torrent, timeout};

    use crate::torrent::tests::{copy_test_file, read_test_file_to_bytes};
    use log::LevelFilter;
    use std::ops::Sub;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_torrent_announce() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default()
        );

        let result = torrent.announce().await.unwrap();

        assert_ne!(
            0, result.total_seeders,
            "expected seeders to have been found"
        );
        assert_ne!(0, result.peers.len(), "expected peers to have been found");
    }

    #[tokio::test]
    async fn test_torrent_metadata() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "debian-udp.torrent";
        let torrent_info_data = read_test_file_to_bytes(filename);
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = create_torrent!(
            filename,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );

        let metadata = torrent.metadata().await.unwrap();

        assert_eq!(torrent_info, metadata);
    }

    #[tokio::test]
    async fn test_torrent_retrieve_metadata() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let uri = "magnet:?xt=urn:btih:2C6B6858D61DA9543D4231A71DB4B1C9264B0685&dn=Ubuntu%2022.04%20LTS&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let torrent = create_torrent!(uri, temp_path, TorrentFlags::Metadata);

        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::MetadataChanged(_) = *event {
                        tx.send(()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        timeout!(
            rx.recv(),
            Duration::from_secs(30),
            "expected to receive a MetadataChanged event"
        )
        .unwrap();
        let result = torrent.metadata().await.unwrap();

        assert_ne!(
            None, result.info,
            "expected the metadata to have been present"
        );
    }

    #[tokio::test]
    async fn test_torrent_total_wanted_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = 75;
        let operation = TorrentCreatePiecesOperation::new();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();

        // create the torrent pieces
        operation.execute(&context).await;

        // only request the first piece
        let total_pieces = torrent.total_pieces().await;
        let priorities = (0..total_pieces)
            .into_iter()
            .map(|i| {
                if i < expected_result {
                    (i, PiecePriority::Normal)
                } else {
                    (i, PiecePriority::None)
                }
            })
            .collect();
        torrent.prioritize_pieces(priorities).await;

        // check the total wanted pieces
        let result = context.total_wanted_pieces().await;
        assert_eq!(expected_result, result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_torrent_resume_internal() {
        init_logger!(LevelFilter::Debug);
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
        let (tx_state, mut rx_state) = unbounded_channel();
        let source_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path_source,
            TorrentFlags::UploadMode,
            TorrentConfig::default(),
            vec![]
        );
        let target_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path_target,
            TorrentFlags::DownloadMode | TorrentFlags::Paused,
            TorrentConfig::default(),
            vec![|| Box::new(TorrentConnectPeersOperation::new()),]
        );
        let source_context = source_torrent.instance().unwrap();

        // initialize the source torrent
        let operation = TorrentCreatePiecesOperation::new();
        let result = operation.execute(&source_context).await;
        assert_eq!(TorrentOperationResult::Continue, result);
        let operation = TorrentCreateFilesOperation::new();
        let result = operation.execute(&source_context).await;
        assert_eq!(TorrentOperationResult::Continue, result);
        let operation = TorrentFileValidationOperation::new();
        let result = select! {
            _ = time::sleep(Duration::from_secs(10)) => TorrentOperationResult::Stop,
            result = async {
                loop {
                    let result = operation.execute(&source_context).await;
                    if result == TorrentOperationResult::Continue {
                        return result;
                    }
                }
            } => result,
        };
        assert_eq!(TorrentOperationResult::Continue, result);

        // initialize the target torrent
        let target_context = target_torrent.instance().unwrap();
        let operation = TorrentCreatePiecesOperation::new();
        let result = operation.execute(&target_context).await;
        assert_eq!(TorrentOperationResult::Continue, result);
        let operation = TorrentCreateFilesOperation::new();
        let result = operation.execute(&target_context).await;
        assert_eq!(TorrentOperationResult::Continue, result);
        target_context
            .update_state(target_context.determine_state().await)
            .await;

        // only request the first X amount of pieces
        let total_pieces = target_torrent.total_pieces().await;
        target_torrent
            .prioritize_pieces(
                (num_of_pieces..total_pieces)
                    .into_iter()
                    .map(|piece| (piece, PiecePriority::None))
                    .collect(),
            )
            .await;

        // resume the target torrent to fetch data from the source torrent
        target_torrent.resume().await;

        // listen to the finished event
        let mut receiver = target_torrent.subscribe();
        tokio::spawn(async move {
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

        // connect the target torrent to the source torrent
        // do not connect the source torrent to the target, as the source torrent is seeding and won't actively create new connections
        target_context
            .peer_pool()
            .add_peer_addresses(
                vec![SocketAddr::from((
                    [127, 0, 0, 1],
                    source_torrent.peer_port().unwrap(),
                ))],
                None,
            )
            .await;

        // wait for all pieces to be completed (finished state)
        timeout!(
            rx_state.recv(),
            Duration::from_secs(90),
            "expected the torrent to enter the FINISHED state"
        )
        .unwrap();

        // validate the pieces and received data
        let pieces = target_torrent
            .pieces()
            .await
            .expect("expected the pieces to have been created");
        let target_context = target_torrent.instance().unwrap();
        let pieces_bitfield = target_context.piece_pool().bitfield().await;

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
        match target_torrent
            .read_file_to_end(files.get(0).expect("expected file index 0 to be present"))
            .await
        {
            Ok((bytes_read, bytes)) => {
                assert_eq!(
                    expected_file_data.len(),
                    bytes_read,
                    "expected the available data to have been read"
                );
                assert_eq!(
                    expected_file_data, bytes,
                    "expected the available data to have been read"
                );
            }
            Err(e) => assert!(false, "failed to read torrent data, {}", e),
        }
    }

    #[tokio::test]
    async fn test_torrent_piece_part() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_piece_part = PiecePart {
            piece: 0,
            part: 1,
            begin: 16384,
            length: 16384,
        };
        let (tx, mut rx) = unbounded_channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new()), || {
                Box::new(TorrentCreateFilesOperation::new())
            },]
        );
        let context = torrent.instance().unwrap();

        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PiecesChanged(_) = *event {
                        tx.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        timeout!(
            rx.recv(),
            Duration::from_millis(200),
            "expected the pieces to be created"
        )
        .unwrap();

        let result = context.piece_part(0, 16000).await;
        assert_eq!(
            None, result,
            "expected no piece part to be returned for invalid begin"
        );

        let result = context.piece_part(0, 16384).await;
        assert_eq!(Some(expected_piece_part), result, "expected the piece part");
    }

    #[tokio::test]
    async fn test_torrent_create_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new())]
        );
        let (tx, mut rx) = unbounded_channel();

        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PiecesChanged(_) = *event {
                        tx.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // wait for the pieces changed event
        timeout!(
            rx.recv(),
            Duration::from_millis(200),
            "expected the pieces to be created"
        )
        .unwrap();
        let pieces = torrent.pieces().await.unwrap();

        assert_ne!(0, pieces.len(), "expected the pieces to have been created");
    }

    #[tokio::test]
    async fn test_torrent_create_files() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new()), || {
                Box::new(TorrentCreateFilesOperation::new())
            },]
        );
        let (tx, mut rx) = unbounded_channel();

        // wait for the pieces changed event
        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = *event {
                        tx.send(()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        let _ = timeout!(
            rx.recv(),
            Duration::from_millis(200),
            "expected the files to be created"
        )
        .unwrap();
        let files = torrent.files().await;

        assert_eq!(1, files.len(), "expected the files to have been created");
    }

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test]
    async fn test_torrent_is_completed() {
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
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![
                || Box::new(TorrentCreatePiecesOperation::new()),
                || Box::new(TorrentCreateFilesOperation::new()),
                || Box::new(TorrentFileValidationOperation::new()),
            ]
        );
        let (tx, mut rx) = unbounded_channel();

        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
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
        timeout!(
            rx.recv(),
            Duration::from_secs(8),
            "expected the torrent to be initialized"
        )
        .unwrap();

        // prioritize the first 30 pieces
        let total_pieces = torrent.total_pieces().await;
        let priorities = (30..total_pieces)
            .into_iter()
            .map(|i| (i, PiecePriority::None))
            .collect();
        torrent.prioritize_pieces(priorities).await;

        let result = torrent.is_completed().await;
        assert_eq!(true, result, "expected the torrent to be completed");
    }

    #[tokio::test]
    async fn test_torrent_is_download_allowed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new())]
        );
        let context = torrent.instance().unwrap();

        let mut receiver = torrent.subscribe();

        let result = context.is_download_allowed().await;
        assert_eq!(false, result, "expected downloading to not be allowed");

        let result = async {
            context.add_options(TorrentFlags::DownloadMode).await;
            // wait for the state change event
            let _ = receiver.recv().await;
            context.is_download_allowed().await
        }
        .await;
        assert_eq!(false, result, "expected downloading to not be allowed");

        let result = async {
            context.update_state(TorrentState::Finished).await;
            context.is_download_allowed().await
        }
        .await;
        assert_eq!(true, result, "expected downloading to be allowed");

        let result = async {
            context.add_options(TorrentFlags::Paused).await;
            context.is_download_allowed().await
        }
        .await;
        assert_eq!(false, result, "expected downloading to not be allowed");
    }

    #[tokio::test]
    async fn test_torrent_is_upload_allowed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::UploadMode,
            TorrentConfig::default(),
            vec![
                || Box::new(TorrentCreatePiecesOperation::new()),
                || Box::new(TorrentCreateFilesOperation::new()),
                || Box::new(TorrentFileValidationOperation::new()),
            ]
        );
        let context = torrent.instance().unwrap();

        let result = context.is_upload_allowed().await;
        assert_eq!(true, result, "expected uploading to be allowed");

        torrent.add_options(TorrentFlags::Paused).await;
        let result = context.is_upload_allowed().await;
        assert_eq!(false, result, "expected uploading to not be allowed");

        torrent
            .remove_options(TorrentFlags::Paused | TorrentFlags::UploadMode)
            .await;
        let result = context.is_upload_allowed().await;
        assert_eq!(false, result, "expected uploading to not be allowed");
    }

    #[tokio::test]
    async fn test_torrent_is_end_game() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentCreatePiecesOperation::new();

        operation.execute(&context).await;
        let total_pieces = context.piece_pool().len().await;
        assert_ne!(0, total_pieces, "expected the pieces to have been created");

        let result = context.is_end_game().await;
        assert_eq!(
            false, result,
            "expected the torrent to not be in the end-game phase"
        );

        let completed_range_1 = (total_pieces as f64 * 0.90) as usize;
        context
            .pieces_completed(
                (0..completed_range_1)
                    .into_iter()
                    .map(|e| e as PieceIndex)
                    .collect(),
            )
            .await;

        let result = context.is_end_game().await;
        assert_eq!(
            false, result,
            "expected the torrent to not be in the end-game phase"
        );

        let completed_range_2 = (total_pieces as f64 * 0.98) as usize;
        context
            .pieces_completed(
                (completed_range_1..completed_range_2)
                    .into_iter()
                    .map(|e| e as PieceIndex)
                    .collect(),
            )
            .await;

        let result = context.is_end_game().await;
        assert_eq!(
            true, result,
            "expected the torrent to be in the end-game phase"
        );
    }

    #[tokio::test]
    async fn test_torrent_determine_state() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let pieces = vec![Piece::new(
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap(),
            0,
            0,
            1024,
        )];
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();

        let result = context.determine_state().await;
        assert_eq!(TorrentState::Initializing, result);

        context.update_pieces(pieces).await;
        let result = context.determine_state().await;
        assert_eq!(TorrentState::Finished, result);

        context.add_options(TorrentFlags::UploadMode).await;
        let result = context.determine_state().await;
        assert_eq!(TorrentState::Seeding, result);

        context.remove_options(TorrentFlags::UploadMode).await;
        context.add_options(TorrentFlags::DownloadMode).await;
        context.update_state(TorrentState::Paused).await;
        let result = context.determine_state().await;
        assert_eq!(TorrentState::Downloading, result);
    }

    #[tokio::test]
    async fn test_torrent_wanted_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentCreatePiecesOperation::new();

        operation.execute(&context).await;
        let total_pieces = context.piece_pool().len().await;
        assert_ne!(0, total_pieces, "expected the pieces to have been created");

        torrent
            .prioritize_pieces(
                (30..total_pieces)
                    .into_iter()
                    .map(|piece| (piece, PiecePriority::None))
                    .collect(),
            )
            .await;

        let expected_result: Vec<PieceIndex> = (0..30)
            .into_iter()
            .map(|piece| piece as PieceIndex)
            .collect();
        let result = context
            .piece_pool()
            .wanted_pieces()
            .await
            .into_iter()
            .map(|e| e.index)
            .collect::<Vec<_>>();
        assert_eq!(expected_result, result);

        context
            .pieces_completed((0..2).into_iter().map(|e| e as PieceIndex).collect())
            .await;
        let expected_result: Vec<PieceIndex> = (2..30)
            .into_iter()
            .map(|piece| piece as PieceIndex)
            .collect();
        let result = context
            .piece_pool()
            .wanted_pieces()
            .await
            .into_iter()
            .map(|e| e.index)
            .collect::<Vec<_>>();
        assert_eq!(expected_result, result);
    }

    #[tokio::test]
    async fn test_torrent_wanted_request_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::DownloadMode,
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentCreatePiecesOperation::new();

        operation.execute(&context).await;
        let total_pieces = context.piece_pool().len().await;
        assert_ne!(0, total_pieces, "expected the pieces to have been created");

        torrent
            .prioritize_pieces(
                (100..total_pieces)
                    .into_iter()
                    .map(|piece| (piece, PiecePriority::None))
                    .collect(),
            )
            .await;

        // acquire some locks
        let permits = async {
            // update the torrent state to a "download allowed" state
            context.update_state(TorrentState::Downloading).await;
            // start requesting permits
            let mut permits = Vec::new();
            for piece in (0..10).into_iter().map(|e| e as PieceIndex) {
                let permit = context
                    .request_download_permit(&piece)
                    .await
                    .expect(format!("expected to get a permit for {} piece", piece).as_str());
                permits.push(permit);
            }
            permits
        }
        .await;
        assert_eq!(10, permits.len(), "expected to acquire 10 permits");

        let expected_wanted_pieces: Vec<PieceIndex> =
            (10..100).into_iter().map(|e| e as PieceIndex).collect();
        let wanted_pieces = context.wanted_request_pieces().await;
        assert_eq!(expected_wanted_pieces, wanted_pieces);

        // update a piece 0 to have timed out
        context
            .pending_piece_requests
            .write()
            .await
            .insert(0, Instant::now().sub(Duration::from_secs(120)));
        let wanted_pieces = context.wanted_request_pieces().await;
        assert_eq!(
            &0,
            wanted_pieces.get(0).unwrap(),
            "expected piece 0 to be requested again after timeout"
        );
    }

    #[tokio::test]
    async fn test_torrent_update_state() {
        init_logger!();
        let expected_state = TorrentState::Paused;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();

        // subscribe to the events of the torrent
        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::StateChanged(state) = &*event {
                        tx.send(state.clone()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        context.update_state(expected_state).await;

        let result = timeout!(
            rx.recv(),
            Duration::from_millis(200),
            "expected a state change event"
        )
        .unwrap();
        assert_eq!(
            expected_state, result,
            "expected the state change event to match the new state"
        );

        let result = torrent.state().await;
        assert_eq!(
            expected_state, result,
            "expected the state function to match the new state"
        );
    }

    mod prioritize {
        use super::*;
        use crate::torrent::FilePriority;

        #[tokio::test]
        async fn test_torrent_prioritize_pieces() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let operation = TorrentCreatePiecesOperation::new();
            let torrent = create_torrent!(
                "debian-udp.torrent",
                temp_path,
                TorrentFlags::none(),
                TorrentConfig::default(),
                vec![]
            );
            let context = torrent.instance().unwrap();

            // create the pieces
            operation.execute(&context).await;

            // only request the first piece
            let mut priorities = torrent.piece_priorities().await;
            priorities.insert(8, PiecePriority::High);
            priorities.insert(9, PiecePriority::High);
            let priorities = priorities
                .into_iter()
                .map(|(i, priority)| {
                    if i < 10 {
                        (i, priority)
                    } else {
                        (i, PiecePriority::None)
                    }
                })
                .collect();

            torrent.prioritize_pieces(priorities).await;

            // check the new priorities of the pieces
            let result = torrent
                .pieces()
                .await
                .expect("expected the pieces to be present");
            for piece in 0..8 {
                let priority = PiecePriority::Normal;
                assert_eq!(
                    priority, result[piece].priority,
                    "expected piece {} to have priority {:?}",
                    piece, priority
                );
            }
            for piece in 9..10 {
                let priority = PiecePriority::High;
                assert_eq!(
                    priority, result[piece].priority,
                    "expected piece {} to have priority {:?}",
                    piece, priority
                );
            }
            for piece in 10..20 {
                let priority = PiecePriority::None;
                assert_eq!(
                    priority, result[piece].priority,
                    "expected piece {} to have priority {:?}",
                    piece, priority
                );
            }

            // check the wanted pieces
            let expected_wanted_pieces = vec![8, 9, 0, 1, 2, 3, 4, 5, 6, 7];
            let result = context
                .piece_pool()
                .wanted_pieces()
                .await
                .into_iter()
                .map(|e| e.index)
                .collect::<Vec<_>>();
            assert_eq!(
                expected_wanted_pieces, result,
                "expected only piece 0 to be wanted"
            );

            // check the interested pieces
            let expected_interested_pieces = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            let result = context.piece_pool().interested_pieces().await;
            assert_eq!(
                expected_interested_pieces, result,
                "expected only piece 0 to be interested"
            );
        }

        #[tokio::test]
        async fn test_prioritize_bytes() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let operation = TorrentCreatePiecesOperation::new();
            let torrent = create_torrent!(
                "debian-udp.torrent",
                temp_path,
                TorrentFlags::none(),
                TorrentConfig::default(),
                vec![]
            );
            let context = torrent.instance().unwrap();
            let piece_length = context.metadata().await.info.unwrap().piece_length as usize;
            let range = 0usize..(2 * piece_length);

            // create the torrent pieces
            operation.execute(&context).await;

            // prioritize the first 2 pieces through the bytes
            torrent.prioritize_bytes(&range, PiecePriority::High).await;

            let priorities = torrent.piece_priorities().await;
            assert_eq!(Some(&PiecePriority::High), priorities.get(&0));
            assert_eq!(Some(&PiecePriority::High), priorities.get(&1));
            assert_eq!(Some(&PiecePriority::Normal), priorities.get(&2));
        }

        #[tokio::test]
        async fn test_prioritize_files() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let pieces_operation = TorrentCreatePiecesOperation::new();
            let files_operation = TorrentCreateFilesOperation::new();
            let torrent = create_torrent!(
                "multifile.torrent",
                temp_path,
                TorrentFlags::none(),
                TorrentConfig::default(),
                vec![]
            );
            let context = torrent.instance().unwrap();

            // create the pieces and files of the torrent
            pieces_operation.execute(&context).await;
            files_operation.execute(&context).await;

            // prioritize only the 2nd file
            let file_priorities = torrent
                .files()
                .await
                .into_iter()
                .map(|file| {
                    if file.index == 1 {
                        (file.index, FilePriority::Normal)
                    } else {
                        (file.index, FilePriority::None)
                    }
                })
                .collect();
            torrent.prioritize_files(file_priorities).await;

            let priorities = torrent.piece_priorities().await;

            // verify that file 0 is ignored, except for the last piece
            for piece in 0usize..401usize {
                assert_eq!(
                    Some(&PiecePriority::None),
                    priorities.get(&piece),
                    "expected the first file (piece {}) to be ignored",
                    piece
                );
            }
            assert_eq!(Some(&PiecePriority::Normal), priorities.get(&401));
            // check that file 1 is wanted
            for piece in 402usize..725usize {
                assert_eq!(
                    Some(&PiecePriority::Normal),
                    priorities.get(&piece),
                    "expected the second file (piece {}) to be wanted",
                    piece
                );
            }
            // check that the remaining files are ignored
            for piece in 725usize..priorities.len() {
                assert_eq!(
                    Some(&PiecePriority::None),
                    priorities.get(&piece),
                    "expected the remaining files (piece {}) to be ignored",
                    piece
                );
            }
        }
    }
}
