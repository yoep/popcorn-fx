use bit_vec::BitVec;
use bitmask_enum::bitmask;
use derive_more::Display;
use futures::future::join_all;
use itertools::Itertools;
use log::{debug, error, trace, warn};

use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::{Peer, PeerId, PeerState};
use popcorn_fx_core::core::torrents::stream::Range;
use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle,
};

use crate::torrents::file::{File, FilePriority};
use crate::torrents::fs::TorrentFileStorage;
use crate::torrents::torrent_request_buffer::{PendingRequest, PendingRequestBuffer};
use crate::torrents::trackers::{
    AnnounceEvent, Announcement, TrackerError, TrackerHandle, TrackerManager, TrackerManagerEvent,
};
use crate::torrents::{
    InfoHash, PartIndex, Piece, PieceChunkPool, PieceError, PieceIndex, PiecePart, PiecePriority,
    Result, TorrentError, TorrentInfo, TorrentMetadata,
};

const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
const RETRIEVE_METADATA_CONNECTIONS: usize = 15;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// Possible flags which can be attached to a [Torrent].
///
/// The default value for the flag options is [TorrentFlags::AutoManaged],
/// which will retrieve the metadata if needed and automatically start the download.
#[bitmask(u16)]
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
    SequentialDownload = 0b0000000010000000,
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

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub piece_count: usize,
    pub piece_len: u32,
    pub last_piece_len: u32,
    pub download_len: u64,
    pub download_dir: PathBuf,
}

/// The torrent data transfer statistics.
/// These statics both include rate based- and lifetime metrics.
#[derive(Debug, Display, Default, Clone, PartialEq)]
#[display(
    fmt = "upload: {}, upload_rate: {}, download: {}, download_rate: {}, total_uploaded: {}, total_downloaded: {}, peers: {}",
    upload,
    upload_rate,
    download,
    download_rate,
    total_uploaded,
    total_downloaded,
    total_peers
)]
pub struct TorrentTransferStats {
    /// The bytes that have been transferred from the peer.
    pub upload: usize,
    /// The bytes per second that have been transferred from the peer.
    pub upload_rate: u64,
    /// The bytes that have been transferred to the peer.
    pub download: usize,
    /// The bytes per second that the downloaded from the peer.
    pub download_rate: u64,
    /// The total bytes that have been uploaded during the lifetime of the torrent.
    pub total_uploaded: usize,
    /// The total bytes that have been downloaded during the lifetime of the torrent.
    pub total_downloaded: usize,
    /// The currently total active peer connections.
    pub total_peers: usize,
}

impl TorrentTransferStats {
    /// Reset the rate- & second based metrics within the statistics.
    fn reset(&mut self) {
        self.upload = 0;
        self.upload_rate = 0;
        self.download = 0;
        self.download_rate = 0;
    }
}

/// Requests a new torrent creation based on the given data.
/// This is the **recommended** way to create new torrents.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use popcorn_fx_torrent::torrents::{Torrent, TorrentFlags, TorrentInfo, TorrentRequest, Result};
/// use popcorn_fx_torrent::torrents::fs::TorrentFileStorage;
///
/// fn create_new_torrent(metadata: TorrentInfo, storage: Box<dyn TorrentFileStorage>) -> Result<Torrent> {
///     let request = TorrentRequest {
///         metadata,
///         options: TorrentFlags::default(),
///         peer_listener_port: 6881,
///         extensions: vec![],
///         storage,
///         peer_timeout: Some(Duration::from_secs(10)),
///         tracker_timeout: Some(Duration::from_secs(3)),
///         runtime: None, // optional shared runtime between torrents
///     };
///
///     Torrent::try_from(request)
/// }
///
/// ```
///
#[derive(Debug)]
pub struct TorrentRequest {
    /// The torrent metadata information
    pub metadata: TorrentInfo,
    /// The torrent options
    pub options: TorrentFlags,
    /// The port on which the torrent session is listening for new incoming peer connections
    pub peer_listener_port: u16,
    /// The extensions that should be enabled for this torrent
    pub extensions: Extensions,
    /// The storage strategy to use for the torrent data
    pub storage: Box<dyn TorrentFileStorage>,
    /// The maximum amount of time to wait for a response from peers
    pub peer_timeout: Option<Duration>,
    /// The maximum amount of time to wait for a response from trackers
    pub tracker_timeout: Option<Duration>,
    /// The underlying Tokio runtime to use for asynchronous operations
    pub runtime: Option<Arc<Runtime>>,
}

/// The torrent callbacks which are invoked when certain events occur.
pub type TorrentCallback = CoreCallback<TorrentEvent>;

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
    /// The inner torrent instance reference holder
    instance: TorrentInstance,
    runtime: Arc<Runtime>,
}

impl Torrent {
    fn new(
        metadata: TorrentInfo,
        peer_listener_port: u16,
        extensions: Extensions,
        flags: TorrentFlags,
        peer_timeout: Option<Duration>,
        tracker_timeout: Option<Duration>,
        storage: Box<dyn TorrentFileStorage>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let info_hash = metadata.info_hash.clone();
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(20);
        let (tracker_sender, tracker_receiver) = tokio::sync::mpsc::channel(3);
        let want_metadata = flags.contains(TorrentFlags::Metadata) && metadata.info.is_none();
        let cancellation_token = CancellationToken::new();
        let inner = Arc::new(InnerTorrent {
            handle,
            metadata: RwLock::new(metadata),
            peer_id,
            tracker_manager: TrackerManager::new(
                peer_id,
                peer_listener_port,
                info_hash,
                tracker_timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)),
                tracker_sender,
                runtime.clone(),
            ),
            peers: RwLock::new(Vec::with_capacity(0)),
            wanted_peers: Default::default(),
            pieces: RwLock::new(Vec::with_capacity(0)),
            completed_pieces: RwLock::new(BitVec::with_capacity(0)),
            pending_requests: PendingRequestBuffer::new(20),
            piece_chunk_pool: PieceChunkPool::new(),
            files: RwLock::new(Vec::with_capacity(0)),
            storage,
            extensions,
            state: RwLock::new(Default::default()),
            options: RwLock::new(flags),
            stats: RwLock::new(TorrentTransferStats::default()),
            event_sender,
            callbacks: Default::default(),
            timeout: peer_timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)),
            cancellation_token,
        });

        let inner_main_loop = inner.clone();
        let torrent = Self {
            handle,
            peer_id,
            instance: TorrentInstance::Owner(inner),
            runtime,
        };
        let torrent_main_loop = torrent.clone();

        // create a new separate thread which manages the internal torrent resources
        // this thread is automatically cancelled when the torrent is dropped
        torrent.runtime.spawn(async move {
            if want_metadata {
                inner_main_loop
                    .send_internal_event(InternalEvent::WantMetadata)
                    .await;
            } else {
                inner_main_loop
                    .send_internal_event(InternalEvent::WantPieces)
                    .await;
                inner_main_loop
                    .send_internal_event(InternalEvent::WantFiles)
                    .await;
            }

            inner_main_loop
                .start(torrent_main_loop, event_receiver, tracker_receiver)
                .await;
        });

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

    /// Retrieve the total amount of pieces for this torrent.
    /// If the metadata is still being retrieved, the total pieces cannot yet be known and this will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the total pieces of this torrent when known, otherwise [None].
    pub async fn total_pieces(&self) -> Option<usize> {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            return inner.metadata.read().await.total_pieces();
        }

        None
    }

    /// Retrieve the torrent pieces, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the current torrent pieces when known, else [None].
    pub async fn pieces(&self) -> Option<Vec<Piece>> {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            let pieces = inner.pieces.read().await.clone();

            if pieces.len() > 0 {
                return Some(pieces);
            }
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
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
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
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            return inner.completed_pieces.read().await.clone();
        }

        BitVec::with_capacity(0)
    }

    /// Get the priorities of the pieces.
    /// It might return an empty array if the metadata is still being retrieved.
    pub async fn piece_priorities(&self) -> Vec<(PieceIndex, PiecePriority)> {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            return inner.piece_priorities().await;
        }

        Vec::with_capacity(0)
    }

    /// Set the priorities of the pieces.
    /// Use [Torrent::piece_priorities] to get the current priorities with its [PieceIndex].
    ///
    /// Providing all piece indexes of the torrent is not required.
    pub async fn prioritize_pieces(&self, priorities: Vec<(PieceIndex, PiecePriority)>) {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            inner.prioritize_pieces(priorities).await;
        }
    }

    /// Get of the given piece index has completed downloading, validating and written to the storage.
    ///
    /// # Returns
    ///
    /// Returns true if the piece has been downloaded, validated and written to storage, else false.
    pub async fn has_piece(&self, piece: PieceIndex) -> bool {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
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
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            return inner.has_bytes(range).await;
        }

        false
    }

    /// Get the torrent files, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the current torrent files when known, else [None].
    pub async fn files(&self) -> Option<Vec<File>> {
        if let Ok(inner) = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))
        {
            let files = inner.files.read().await.clone();

            if files.len() > 0 {
                return Some(files);
            }
        }

        None
    }

    /// Start announcing the torrent to its known trackers.
    /// This will start a period announcement for all active trackers.
    ///
    /// # Returns
    ///
    /// Returns a [TorrentError] when the announcement of the torrent couldn't be started.
    pub async fn start_announcing(&self) -> Result<()> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;
        let trackers = inner.active_trackers().await;

        if trackers.is_empty() {
            inner.add_known_trackers().await?;
        }

        inner.start_announcing().await;
        Ok(())
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
        let trackers = inner.active_trackers().await;

        if trackers.is_empty() {
            Ok(inner.add_and_announce_trackers().await?)
        } else {
            Ok(inner.announce_all().await)
        }
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
            inner
                .add_options(TorrentFlags::DownloadMode | TorrentFlags::Metadata)
                .await;
            inner.remove_options(TorrentFlags::Paused).await;
            inner
                .send_internal_event(InternalEvent::OptionsChanged)
                .await;

            if !inner.metadata().await.info.is_none() {
                inner.send_internal_event(InternalEvent::WantMetadata).await;
            }
            if inner.tracker_manager.trackers().await.len() == 0 {
                inner.send_internal_event(InternalEvent::WantTracker).await;
            }

            inner.update_wanted_peers(20).await;
            inner.send_internal_event(InternalEvent::WantPeer).await;

            if !inner.is_completed().await {
                inner.send_internal_event(InternalEvent::WantData).await;
            }
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

    /// Try to read the given piece bytes range.
    pub async fn read_piece_bytes(
        &self,
        piece: PieceIndex,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<u8>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;

        if let Some(piece) = inner.pieces.read().await.get(piece).cloned() {
            let files = inner.find_relevant_files_for_piece(&piece).await;
            let mut buffer = Vec::with_capacity(range.len());

            for file in files {
                if let Some((_offset, file_range)) = file.byte_range(&piece) {
                    buffer.extend_from_slice(&inner.storage.read_bytes(&file, file_range).await?)
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

    /// Get the piece part of the torrent based on the piece and the offset within the piece.
    /// It returns [None] if the piece part is unknown to this torrent.
    ///
    /// # Argument
    ///
    /// * `piece` - The index of the piece.
    /// * `begin` - The offset within the piece.
    pub(crate) async fn piece_part(&self, piece: PartIndex, begin: usize) -> Option<PiecePart> {
        if let Some(inner) = self.instance() {
            return inner
                .pieces
                .read()
                .await
                .iter()
                .find(|e| e.index == piece)
                .and_then(|piece| piece.parts.iter().find(|part| part.begin == begin).cloned());
        }

        None
    }

    /// Notify this torrent about a new availability of a piece from a peer.
    /// This is a crate function to allow peers to send the torrent notifications about this event.
    pub(crate) async fn notify_peer_has_piece(&self, piece: PieceIndex) {
        if let Some(inner) = self.instance() {
            inner.update_piece_availability(piece).await;
        }
    }

    /// Notify this torrent about the completion of a piece.
    /// The torrent will then validate and store the completed piece data.
    pub(crate) async fn piece_completed(&self, part: PiecePart, data: Vec<u8>) {
        if let Some(inner) = self.instance() {
            inner
                .send_internal_event(InternalEvent::PiecePartCompleted(part, data))
                .await;
        }
    }

    async fn send_internal_event(&self, event: InternalEvent) {
        if let Some(inner) = self.instance() {
            inner.send_internal_event(event).await;
        }
    }

    /// Get a temporary strong reference to the inner torrent.
    fn instance(&self) -> Option<Arc<InnerTorrent>> {
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
            instance: self.instance.clone(),
            runtime: self.runtime.clone(),
        }
    }
}

impl TryFrom<TorrentRequest> for Torrent {
    type Error = TorrentError;

    fn try_from(request: TorrentRequest) -> Result<Self> {
        let metadata = request.metadata;
        let runtime = request
            .runtime
            .unwrap_or_else(|| Arc::new(Runtime::new().expect("expected a new runtime")));

        // validate the given metadata before creating the torrent
        metadata.validate()?;

        Ok(Self::new(
            metadata,
            request.peer_listener_port,
            request.extensions,
            request.options,
            request.peer_timeout,
            request.tracker_timeout,
            request.storage,
            runtime,
        ))
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
        }
    }
}

/// The torrent instances owns the actual inner instance.
/// This prevents other [Torrent] references from keeping the torrent alive while the session has dropped it.
#[derive(Debug)]
enum TorrentInstance {
    Owner(Arc<InnerTorrent>),
    Borrowed(Weak<InnerTorrent>),
}

impl Clone for TorrentInstance {
    fn clone(&self) -> Self {
        match self {
            Self::Owner(inner) => Self::Borrowed(Arc::downgrade(inner)),
            Self::Borrowed(inner) => Self::Borrowed(inner.clone()),
        }
    }
}

/// The events for internal torrent commands.
/// These are triggered when certain events happen in the torrent, but are never exposed outside the torrent.
#[derive(PartialEq)]
enum InternalEvent {
    /// Indicates that the torrent options (flags) have changed
    OptionsChanged,
    /// Indicates that trackers are wanted for the torrent
    WantTracker,
    /// Indicates that an additional peer is wanted for the torrent
    WantPeer,
    /// Indicates that the metadata is wanted for the torrent
    WantMetadata,
    /// Indicates that the torrent wants data from peers
    WantData,
    /// Indicates that the torrent wants the pieces to be created
    WantPieces,
    /// Indicates that the torrent wants the files to be created
    WantFiles,
    /// Indicates that the given peer has been connected and needs to be managed by the torrent
    PeerConnected(Peer),
    /// Indicates that the torrent wants to connect to the given peer addr
    ConnectToPeer(SocketAddr),
    /// Indicates that a piece part has been completed
    PiecePartCompleted(PiecePart, Vec<u8>),
    /// Indicates that a piece has been completed
    PieceCompleted(PieceIndex),
}

impl Debug for InternalEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalEvent::OptionsChanged => write!(f, "OptionsChanged"),
            InternalEvent::WantTracker => write!(f, "WantTracker"),
            InternalEvent::WantPeer => write!(f, "WantPeer"),
            InternalEvent::WantMetadata => write!(f, "WantMetadata"),
            InternalEvent::WantData => write!(f, "WantData"),
            InternalEvent::WantPieces => write!(f, "WantPieces"),
            InternalEvent::WantFiles => write!(f, "WantFiles"),
            InternalEvent::PeerConnected(e) => write!(f, "PeerConnected({})", e),
            InternalEvent::ConnectToPeer(e) => write!(f, "ConnectToPeer({})", e),
            InternalEvent::PiecePartCompleted(e, data) => {
                write!(f, "PiecePartCompleted({:?}, [size {}])", e, data.len())
            }
            InternalEvent::PieceCompleted(e) => write!(f, "PieceCompleted({})", e),
        }
    }
}

#[derive(Debug)]
struct InnerTorrent {
    /// The unique immutable handle of the torrent
    handle: TorrentHandle,
    /// The unique immutable peer id of the torrent
    peer_id: PeerId,
    /// The torrent metadata information of the torrent
    /// This might still be incomplete if the torrent was created from a magnet link
    metadata: RwLock<TorrentInfo>,
    /// The manager of the trackers for the torrent
    tracker_manager: TrackerManager,

    /// The established peer connections
    peers: RwLock<Vec<Peer>>,
    /// The lower bound of peers which are at least wanted
    wanted_peers: RwLock<usize>,

    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: RwLock<Vec<Piece>>,
    /// The completed pieces of the torrent
    completed_pieces: RwLock<BitVec>,
    /// The pending requests of this torrent
    pending_requests: PendingRequestBuffer,
    /// The pool which stores the received piece parts
    piece_chunk_pool: PieceChunkPool,

    /// The torrent files
    files: RwLock<Vec<File>>,
    /// The torrent file storage to store the data
    storage: Box<dyn TorrentFileStorage>,

    /// The immutable extensions for this torrent
    extensions: Extensions,
    /// The state of the torrent
    state: RwLock<TorrentState>,
    /// The torrent options that are set for this torrent
    options: RwLock<TorrentFlags>,
    /// The data transfer stats of the torrent
    stats: RwLock<TorrentTransferStats>,
    /// The internal command event sender
    event_sender: Sender<InternalEvent>,
    /// The callbacks for the torrent events
    callbacks: CoreCallbacks<TorrentEvent>,
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerTorrent {
    /// Start the main loop of this torrent.
    /// It starts listening for events from different receivers and processes them accordingly.
    async fn start(
        &self,
        weak_ref: Torrent,
        mut internal_events: Receiver<InternalEvent>,
        mut tracker_receiver: Receiver<TrackerManagerEvent>,
    ) {
        let mut stats_interval = time::interval(Duration::from_secs(1));
        let mut cleanup_interval = time::interval(Duration::from_secs(30));

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                event = internal_events.recv() => {
                    if let Some(event) = event {
                        self.handle_command_event(&weak_ref, event).await;
                    } else {
                        debug!("Torrent {} events channel closed", self);
                        break;
                    }
                }
                request = self.pending_requests.next() => {
                    if let Some(request) = request {
                        self.process_pending_request(request).await;
                    }
                }
                event = tracker_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_tracker_event(event).await;
                    }
                }
                _ = stats_interval.tick() => self.update_stats().await,
                _ = cleanup_interval.tick() => {
                    self.clean_peers().await;
                    self.pending_requests.retry_timed_out_requests().await;
                },
            }
        }

        trace!("Torrent {} main loop ended", self);
    }

    async fn state(&self) -> TorrentState {
        self.state.read().await.clone()
    }

    async fn options(&self) -> TorrentFlags {
        self.options.read().await.clone()
    }

    async fn active_trackers(&self) -> Vec<Url> {
        self.tracker_manager.trackers().await
    }

    async fn metadata(&self) -> TorrentInfo {
        self.metadata.read().await.clone()
    }

    /// Get the wanted pieces for this torrent.
    /// This is based on the [PiecePriority] set within the pieces of this torrent.
    async fn wanted_pieces(&self) -> Vec<PiecePart> {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| e.priority != PiecePriority::None)
            .map(|e| e.parts.clone())
            .flatten()
            .collect()
    }

    /// Get if the given piece is completed with downloading its data.
    /// It returns true if the piece is completed, validated and written to the storage, else false.
    async fn has_piece(&self, piece: PieceIndex) -> bool {
        self.completed_pieces
            .read()
            .await
            .get(piece)
            .unwrap_or(false)
    }

    /// Get if the given bytes have been completed downloading.
    /// It returns true if all bytes are completed, validated and written to the storage, else false.
    async fn has_bytes(&self, range: &std::ops::Range<usize>) -> bool {
        self.pieces
            .read()
            .await
            .iter()
            .filter(|e| {
                let piece_range = e.range();

                // check if there is any overlap with the given byte range and piece range
                piece_range.start < range.end && range.start < piece_range.end
            })
            .all(|e| e.is_completed())
    }

    /// Get the priorities of the known pieces.
    async fn piece_priorities(&self) -> Vec<(PieceIndex, PiecePriority)> {
        self.pieces
            .read()
            .await
            .iter()
            .map(|e| (e.index, e.priority))
            .collect()
    }

    /// Set the priorities of the pieces.
    async fn prioritize_pieces(&self, priorities: Vec<(PieceIndex, PiecePriority)>) {
        let mut mutex = self.pieces.write().await;

        for (index, priority) in priorities {
            if let Some(piece) = mutex.get_mut(index) {
                piece.priority = priority;
            }
        }

        debug!("Torrent {} piece priorities have been changed", self);
    }

    /// Get if this torrent has been completed downloading are wanted pieces.
    async fn is_completed(&self) -> bool {
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

    /// Get the tiered trackers from the metadata of this torrent.
    async fn tiered_trackers(&self) -> Result<BTreeMap<u8, Vec<Url>>> {
        let metadata = self.metadata().await;
        let tiered_trackers = metadata.tiered_trackers();

        if tiered_trackers.is_empty() {
            return Err(TorrentError::Tracker(TrackerError::Unavailable));
        }

        Ok(tiered_trackers)
    }

    /// Get the number of active peers for this torrent.
    /// This excludes the closed peer connections which have not yet been cleaned.
    async fn active_peers_len(&self) -> usize {
        self.peers
            .read()
            .await
            .iter()
            .filter(|e| block_in_place(e.state()) != PeerState::Closed)
            .count()
    }

    /// Get the related files to the given piece.
    /// This will check which file bytes are overlapping with the piece range.
    async fn find_relevant_files_for_piece(&self, piece: &Piece) -> Vec<File> {
        self.files
            .read()
            .await
            .iter()
            .filter(|e| e.contains(piece))
            .cloned()
            .collect::<Vec<File>>()
    }

    /// Try to add the given tracker to the tracker manager of this torrent.
    async fn add_tracker(&self, url: Url, tier: u8) -> Result<TrackerHandle> {
        let handle = self.tracker_manager.add_tracker(&url, tier).await?;

        debug!(
            "Tracker {}({}) has been added to torrent {}",
            url, handle, self
        );
        Ok(handle)
    }

    /// Add the given peer to this torrent.
    /// Duplicate peers will be ignored and dropped.
    async fn add_peer(&self, peer: Peer) {
        trace!("Adding peer {} to torrent {}", peer, self);
        {
            let mut mutex = self.peers.write().await;
            // verify that the peer address is unique
            if mutex.iter().any(|e| e.addr() == peer.addr()) {
                warn!("Duplicate peer {} detected for torrent {}", peer, self);
                return;
            }
            mutex.push(peer);
        }

        self.invoke_event(TorrentEvent::PeersChanged);
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
        self.create_pieces().await;
        self.create_files().await;

        if *self.options.read().await & TorrentFlags::DownloadMode == TorrentFlags::DownloadMode {
            self.create_pending_requests().await;
        }
    }

    async fn start_announcing(&self) {
        self.tracker_manager.start_announcing();
    }

    async fn announce_all(&self) -> Announcement {
        self.tracker_manager.announce_all().await
    }

    /// Add the given options to the torrent.
    async fn add_options(&self, options: TorrentFlags) {
        {
            let mut mutex = self.options.write().await;
            *mutex |= options;
        }
        self.send_internal_event(InternalEvent::OptionsChanged)
            .await;
    }

    /// Remove the given options from the torrent.
    async fn remove_options(&self, options: TorrentFlags) {
        {
            let mut mutex = self.options.write().await;
            *mutex &= !options;
        }
        self.send_internal_event(InternalEvent::OptionsChanged)
            .await;
    }

    async fn update_state(&self, state: TorrentState) {
        {
            let mut mutex = self.state.write().await;
            *mutex = state.clone();
        }

        self.invoke_event(TorrentEvent::StateChanged(state));
    }

    async fn update_stats(&self) {
        let mut peer_metrics = Vec::new();
        let mut mutex = self.stats.write().await;
        // start by resetting the rate based metrics
        mutex.reset();

        {
            let peer_mutex = self.peers.read().await;
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
            mutex.download += peer_stats.download;
            mutex.download_rate += peer_stats.download_rate;
            mutex.total_uploaded += peer_stats.upload;
            mutex.total_downloaded += peer_stats.download;
        }

        let event_metrics = mutex.clone();
        drop(mutex);
        self.invoke_event(TorrentEvent::Stats(event_metrics));
    }

    async fn update_piece_availability(&self, piece: PieceIndex) {
        trace!("Updating piece {} availability for {}", piece, self);
        let mut mutex = self.pieces.write().await;
        match mutex.iter_mut().find(|e| e.index == piece) {
            None => warn!("Peer notified about an unknown piece {}", piece),
            Some(piece) => piece.increase_availability(),
        }
    }

    async fn update_wanted_peers(&self, num_of_peers: usize) {
        *self.wanted_peers.write().await = num_of_peers;
        debug!("Wanted peers set to {} for {}", num_of_peers, self);

        if self.active_peers_len().await < num_of_peers {
            trace!(
                "There are not enough peers for torrent {}, requesting additional peers",
                self
            );
            self.send_internal_event(InternalEvent::WantPeer).await;
        }
    }

    async fn update_piece_completion(&self, piece: PieceIndex) {
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

        // notify the peers
        for peer in self.peers.read().await.iter() {
            peer.notify_have_piece(piece).await;
        }
    }

    /// Pause the torrent.
    async fn pause(&self) {
        self.pending_requests.clear().await;
        self.add_options(TorrentFlags::Paused).await;
        self.send_internal_event(InternalEvent::OptionsChanged)
            .await;
    }

    async fn handle_command_event(&self, torrent: &Torrent, event: InternalEvent) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            InternalEvent::OptionsChanged => self.process_options().await,
            InternalEvent::WantTracker => self.connect_to_known_trackers(&torrent),
            InternalEvent::WantPeer => self.process_peer_wanted().await,
            InternalEvent::WantData => self.process_data_wanted().await,
            InternalEvent::WantMetadata => self.request_metadata().await,
            InternalEvent::WantPieces => self.create_pieces().await,
            InternalEvent::WantFiles => self.create_files().await,
            InternalEvent::PeerConnected(peer) => self.add_peer(peer).await,
            InternalEvent::ConnectToPeer(addr) => {
                self.create_peer_connection(torrent.clone(), addr)
            }
            InternalEvent::PiecePartCompleted(part, data) => {
                self.process_received_part(part, data).await
            }
            InternalEvent::PieceCompleted(piece) => self.process_completed_piece(piece).await,
        }
    }

    async fn handle_tracker_event(&self, event: TrackerManagerEvent) {
        trace!("Handling event {:?} for torrent {}", event, self);
        match event {
            TrackerManagerEvent::PeersDiscovered(peers) => {
                // calculate the number of newly wanted connections
                let new_connections =
                    self.wanted_peers.read().await.clone() - self.active_peers_len().await;
                let peers = self.unique_peer_addresses(peers).await;

                self.create_peer_connections(peers, new_connections).await
            }
            TrackerManagerEvent::TrackerAdded(handle) => {
                let options = self.options().await;
                let is_retrieving_metadata =
                    *self.state.read().await == TorrentState::RetrievingMetadata;
                let is_download_mode =
                    options & TorrentFlags::DownloadMode == TorrentFlags::DownloadMode;

                if is_retrieving_metadata || is_download_mode {
                    self.tracker_manager
                        .make_announcement(handle, AnnounceEvent::Started)
                        .await;
                }
            }
        }
    }

    async fn process_options(&self) {
        let mutex = self.options.read().await;
        if *mutex & TorrentFlags::Paused == TorrentFlags::Paused {
            // choke all the peers
            let peers = self.peers.read().await;
            let mut futures = Vec::new();
            for peer in peers.iter() {
                futures.push(peer.pause());
            }
            futures::future::join_all(futures).await;
        } else if *mutex & TorrentFlags::UploadMode == TorrentFlags::UploadMode {
            // unchoke all the peers
            let peers = self.peers.read().await;
            let mut futures = Vec::new();
            for peer in peers.iter() {
                futures.push(peer.resume());
            }
            futures::future::join_all(futures).await;
        }
    }

    /// Process the wanted peer event and try to establish one or more new peer connections.
    async fn process_peer_wanted(&self) {
        let peer_addresses = self
            .unique_peer_addresses(self.tracker_manager.discovered_peers().await)
            .await;
        let num_of_peers = (*self.wanted_peers.read().await - self.active_peers_len().await).max(1);

        self.create_peer_connections(peer_addresses, num_of_peers)
            .await
    }

    /// Processes the wanted pieces and creates pending requests if needed.
    async fn process_data_wanted(&self) {
        // check if the torrent is completed
        // if so, we don't need to do anything
        if self.is_completed().await {
            return;
        }

        self.create_pending_requests().await;
    }

    async fn process_pending_request(&self, request: PendingRequest) {
        let peers = self.peers.read().await;
        let futures: Vec<_>;
        let piece_part = request.part;
        let piece = piece_part.piece.clone();
        let mut pending_peers = Vec::new();

        // try to find 3 peers that have the piece
        futures = peers
            .iter()
            // filter out any peers that don't have the piece available
            .filter(|peer| block_in_place(peer.remote_has_piece(piece)))
            // sort the peers by choke state
            .sorted_by(|a, b| {
                block_in_place(a.remote_choke_state()).cmp(&block_in_place(b.remote_choke_state()))
            })
            .take(3)
            .map(|peer| {
                pending_peers.push(peer.handle());
                peer.request_piece_part(piece_part.clone())
            })
            .collect();

        futures::future::join_all(futures).await;

        self.pending_requests
            .update_pending_peers(piece_part, &pending_peers)
            .await;
    }

    async fn process_received_part(&self, piece_part: PiecePart, data: Vec<u8>) {
        let piece_length: usize;
        let piece_completed: bool;

        {
            let mut mutex = self.pieces.write().await;
            if let Some(piece) = mutex.iter_mut().find(|e| e.index == piece_part.piece) {
                piece.part_completed(piece_part.part);
                piece_length = piece.length;
                piece_completed = piece.is_completed();
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
                self.pending_requests.remove_by_part(&piece_part).await;

                if piece_completed {
                    self.send_internal_event(InternalEvent::PieceCompleted(piece_part.piece))
                        .await;
                }
            }
            Err(e) => warn!("Failed to add chunk data for {}, {}", self, e),
        }
    }

    async fn process_completed_piece(&self, piece: PieceIndex) {
        if let Some(data) = self.piece_chunk_pool.get(piece).await {
            if let Some(hash) = self
                .pieces
                .read()
                .await
                .iter()
                .find(|e| e.index == piece)
                .map(|e| e.hash.clone())
            {
                let is_valid: bool;
                let is_v2_hash = hash.has_v2();

                if is_v2_hash {
                    let actual_hash = Sha256::digest(&data).to_vec();
                    is_valid = hash.hash_v2().unwrap() == actual_hash;
                } else {
                    let actual_hash = Sha1::digest(&data).to_vec();
                    is_valid = hash.hash_v1().unwrap().as_slice() == actual_hash.as_slice();
                }

                if is_valid {
                    self.write_piece_chunk(piece, data).await;
                    self.update_piece_completion(piece).await;
                } else {
                    debug!(
                        "Retrying invalid received piece {} data for {}",
                        piece, self
                    );
                    // start the request over for the whole piece
                    self.create_pending_request_for_piece(piece).await;
                }
            } else {
                warn!("Received unknown complete piece {} for {}", piece, self);
            }
        } else {
            warn!(
                "Piece chunk data of {} is not available for {}",
                piece, self
            );
        }
    }

    async fn add_known_trackers(&self) -> Result<()> {
        let futures: Vec<_> = self
            .get_missing_trackers()
            .await?
            .into_iter()
            .map(|(tier, url)| self.add_tracker(url, tier))
            .collect();

        // extract all individual announcements that have been made to trackers
        let start_time = Instant::now();
        join_all(futures).await;
        let time_taken = start_time.elapsed();
        trace!(
            "Took {}.{:03} seconds to add trackers",
            time_taken.as_secs(),
            time_taken.subsec_millis()
        );

        Ok(())
    }

    async fn add_and_announce_trackers(&self) -> Result<Announcement> {
        let futures: Vec<_> = self
            .get_missing_trackers()
            .await?
            .into_iter()
            .map(|(tier, url)| async move {
                match self.add_tracker(url, tier).await {
                    Ok(handle) => Ok(self
                        .tracker_manager
                        .announce(handle, AnnounceEvent::Started)
                        .await?),
                    Err(e) => Err(e),
                }
            })
            .collect();

        // extract all individual announcements that have been made to trackers
        let start_time = Instant::now();
        let announcements: Vec<Announcement> = join_all(futures)
            .await
            .into_iter()
            .filter_map(|e| match e {
                Ok(e) => Some(e),
                Err(e) => {
                    debug!("Failed to add torrent {} tracker, {}", self, e);
                    None
                }
            })
            .collect();
        let time_taken = start_time.elapsed();
        trace!(
            "Took {}.{:03} seconds to add trackers",
            time_taken.as_secs(),
            time_taken.subsec_millis()
        );

        let mut result = Announcement::default();
        for announcement in announcements {
            result.total_leechers += announcement.total_leechers;
            result.total_seeders += announcement.total_seeders;
            result.peers.extend(announcement.peers);
        }

        Ok(result)
    }

    async fn get_missing_trackers(&self) -> Result<Vec<(u8, Url)>> {
        let tiered_trackers = self.tiered_trackers().await?;

        Ok(tiered_trackers
            .into_iter()
            .map(|(tier, trackers)| {
                trackers
                    .into_iter()
                    .map(|url| (tier, url))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .filter(|(_, url)| !block_in_place(self.tracker_manager.is_tracker_url_known(url)))
            .collect())
    }

    /// Connect to trackers which are currently known within the torrent.
    /// This operation is executed on a new thread to unblock the caller.
    fn connect_to_known_trackers(&self, torrent: &Torrent) {
        // move this operation to a new thread to unblock the main loop
        let inner_tracker = torrent.instance().expect("expected a valid instance");
        torrent.runtime.spawn(async move {
            if let Err(e) = inner_tracker.add_known_trackers().await {
                warn!("Failed to add known trackers for {}, {}", inner_tracker, e);
            }
        });
    }

    async fn request_metadata(&self) {
        if self.metadata().await.info.is_some() {
            debug!(
                "Metadata of {} is already known, ignoring want metadata",
                self
            );
            return;
        }

        self.update_state(TorrentState::RetrievingMetadata).await;

        // update the wanted peers count
        self.update_wanted_peers(RETRIEVE_METADATA_CONNECTIONS)
            .await;

        // if there are currently not any known peers
        // then we'll make an announcement request
        if self.tracker_manager.discovered_peers().await.len() == 0 {
            trace!("There are currently no peers discovered for {}", self);
            self.send_internal_event(InternalEvent::WantTracker).await;
        } else {
            self.send_internal_event(InternalEvent::WantPeer).await;
        }
    }

    /// Create the pieces information for the torrent.
    /// This operation can only be done when the metadata of the torrent is known.
    async fn create_pieces(&self) {
        // check if the pieces have already been created
        // if so, ignore this operation
        if self.pieces.read().await.len() > 0 {
            return;
        }

        match self.try_create_pieces().await {
            Ok(pieces) => {
                let total_pieces = pieces.len();
                {
                    let mut mutex = self.pieces.write().await;
                    *mutex = pieces;
                }
                {
                    let mut mutex = self.completed_pieces.write().await;
                    *mutex = BitVec::from_elem(total_pieces, false);
                }

                debug!(
                    "A total of {} pieces have been created for {}",
                    total_pieces, self
                );
                self.invoke_event(TorrentEvent::PiecesChanged);
            }
            Err(e) => warn!("Failed to create torrent pieces of {}, {}", self, e),
        }
    }

    /// Try to create the pieces of the torrent.
    /// This operation doesn't store the pieces results.
    ///
    /// # Returns
    ///
    /// Returns the pieces result for the torrent if available, else the error.
    async fn try_create_pieces(&self) -> Result<Vec<Piece>> {
        let info_hash: InfoHash;
        let num_pieces: usize;
        let metadata: TorrentMetadata;

        {
            let mutex = self.metadata.read().await;
            info_hash = mutex.info_hash.clone();
            metadata = mutex
                .info
                .clone()
                .ok_or(PieceError::UnableToDeterminePieces(
                    "metadata is unavailable".to_string(),
                ))?;
            num_pieces = mutex
                .total_pieces()
                .ok_or(PieceError::UnableToDeterminePieces(
                    "failed to calculate number of pieces".to_string(),
                ))?;
        }

        let sha1_pieces = if info_hash.has_v1() {
            metadata.sha1_pieces()
        } else {
            Vec::new()
        };
        let sha256_pieces = if info_hash.has_v2() {
            metadata.sha256_pieces()
        } else {
            Vec::new()
        };
        let mut pieces = Vec::with_capacity(num_pieces);
        let total_file_size = metadata.total_size();
        let piece_length = metadata.piece_length as usize;
        let mut last_piece_length = total_file_size % piece_length;
        let mut offset = 0;

        if last_piece_length == 0 {
            last_piece_length = piece_length;
        }

        for piece_index in 0..num_pieces {
            let hash = if info_hash.has_v2() {
                InfoHash::try_from_bytes(sha256_pieces.get(piece_index).unwrap())?
            } else {
                InfoHash::try_from_bytes(sha1_pieces.get(piece_index).unwrap())?
            };
            let length = if piece_index != num_pieces - 1 {
                piece_length
            } else {
                last_piece_length
            };

            pieces.push(Piece::new(hash, piece_index as PieceIndex, offset, length));
            offset += length;
        }

        Ok(pieces)
    }

    /// Create the torrent files information.
    /// This can only be executed when the torrent metadata is known.
    async fn create_files(&self) {
        match self.try_create_files().await {
            Ok(files) => {
                let total_files = files.len();
                {
                    let mut mutex = self.files.write().await;
                    trace!("Created torrent {} files {:?}", self, files);
                    *mutex = files;
                }
                debug!(
                    "A total of {} files have been created for {}",
                    total_files, self
                );
                self.invoke_event(TorrentEvent::FilesChanged);
            }
            Err(e) => warn!("Failed to create torrent files of {}, {}", self, e),
        }
    }

    /// Try to create the files of the torrent.
    /// This operation doesn't store the created files within this torrent.
    async fn try_create_files(&self) -> Result<Vec<File>> {
        let is_v2_metadata: bool;
        let metadata: TorrentMetadata;

        {
            let mutex = self.metadata.read().await;
            is_v2_metadata = mutex.info_hash.has_v2();
            metadata = mutex
                .info
                .as_ref()
                .cloned()
                .ok_or(TorrentError::InvalidMetadata(
                    "metadata is missing".to_string(),
                ))?;
        }

        let mut offset = 0;
        let mut files = vec![];

        for file in metadata.files() {
            let file_length = file.length as usize;
            let mut path = PathBuf::new().join(metadata.name());

            for path_section in file.path() {
                path = path.join(path_section);
            }

            files.push(File {
                path,
                offset,
                length: file.length as usize,
                info: file,
                priority: Default::default(),
            });

            if is_v2_metadata {
                offset = (offset + metadata.piece_length as usize - 1)
                    / metadata.piece_length as usize
                    * metadata.piece_length as usize;
            } else {
                offset += file_length;
            }
        }

        Ok(files)
    }

    /// Create the requests for the pieces that are not completed yet.
    /// These requests will be added to the `pending_requests`.
    async fn create_pending_requests(&self) {
        match self.try_create_pending_requests().await {
            Ok(requests) => {
                let requests_len = requests.len();
                self.pending_requests.push_all(requests).await;
                debug!(
                    "A total of {} pending requests have been created for {}",
                    requests_len, self
                );
            }
            Err(e) => debug!("Failed to create pending requests for {}, {}", self, e),
        }
    }

    async fn try_create_pending_requests(&self) -> Result<Vec<PendingRequest>> {
        // check if the torrent pieces are known
        if self.pieces.read().await.len() == 0 || self.completed_pieces.read().await.len() == 0 {
            return Err(PieceError::Unavailable)?;
        }

        let wanted_pieces = self.wanted_pieces().await;
        let completed_pieces = self.completed_pieces.read().await;
        let pending_requested_parts = self.pending_requests.pending_parts().await;
        let pieces = self.pieces.read().await;

        trace!("Wants {} pieces for {}", wanted_pieces.len(), self);
        Ok(wanted_pieces
            .into_iter()
            // sort the pieces by availability
            .sorted_by(|a, b| {
                let a = pieces
                    .iter()
                    .find(|e| e.index == a.piece)
                    .map(|e| e.availability());
                let b = pieces
                    .iter()
                    .find(|e| e.index == b.piece)
                    .map(|e| e.availability());

                // sort by availability decreasing
                b.cmp(&a)
            })
            .filter(|piece_part| {
                let piece_index = piece_part.piece.clone();
                !completed_pieces.get(piece_index).unwrap_or(false)
            })
            .filter(|piece_part| !pending_requested_parts.contains(piece_part))
            .map(|part| PendingRequest::new(part))
            .collect())
    }

    /// Create the requests for the given piece.
    /// It will create multiple pending requests for the given piece based on its parts.
    ///
    /// This doesn't check if the piece was completed or not.
    async fn create_pending_request_for_piece(&self, piece: PieceIndex) {
        if let Some(parts) = self
            .pieces
            .read()
            .await
            .iter()
            .find(|e| e.index == piece)
            .map(|e| e.parts.clone())
        {
            self.pending_requests
                .push_all(parts.into_iter().map(|e| PendingRequest::new(e)).collect())
                .await;
        }
    }

    /// Create new peer connections for the available peer addresses and the number of wanted new connections.
    async fn create_peer_connections(
        &self,
        peer_addresses: Vec<SocketAddr>,
        wanted_connections: usize,
    ) {
        if peer_addresses.is_empty() {
            return;
        }

        let mut requested_peers = 0;

        for peer_addr in peer_addresses.into_iter().take(wanted_connections) {
            self.send_internal_event(InternalEvent::ConnectToPeer(peer_addr))
                .await;
            requested_peers += 1;
        }

        debug!(
            "Requested a total of {} new peer connections for {}",
            requested_peers, self
        );
    }

    /// Create a new peer connection for the torrent and the given address.
    /// This process is spawned into a new thread.
    ///
    /// The passed [Torrent] should always be a weak reference to this instance.
    fn create_peer_connection(&self, torrent: Torrent, peer_addr: SocketAddr) {
        let extensions = self.extensions();
        let event_sender = self.event_sender.clone();
        let runtime = torrent.runtime.clone();
        debug!(
            "Trying to create a new peer connection {} for {}",
            peer_addr, self
        );
        runtime.spawn(async move {
            let handle_info = torrent.handle();
            match Self::try_create_peer_connection(torrent, peer_addr, extensions).await {
                Ok(peer) => {
                    let _ = event_sender.send(InternalEvent::PeerConnected(peer)).await;
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
        let mut mutex = self.peers.write().await;
        let mut indexes_to_remove = vec![];

        for (index, peer) in mutex.iter().enumerate() {
            let peer_state = peer.state().await;
            if peer_state == PeerState::Closed || peer_state == PeerState::Error {
                indexes_to_remove.push(index);
                // make sure we're correctly closing the peer connection
                peer.close().await;
            }
        }

        let total_peers_removed = indexes_to_remove.len();
        for index in indexes_to_remove {
            mutex.remove(index);
        }

        drop(mutex);
        debug!(
            "Cleaned a total of {} peers for torrent {}",
            total_peers_removed, self
        );

        // check if new peers are wanted
        if *self.wanted_peers.read().await > self.active_peers_len().await {
            self.send_internal_event(InternalEvent::WantPeer).await;
        }
    }

    /// Filter out any peer address which currently have a peer connection established.
    async fn unique_peer_addresses(&self, peer_addresses: Vec<SocketAddr>) -> Vec<SocketAddr> {
        let mutex = self.peers.read().await;
        peer_addresses
            .into_iter()
            .filter(|addr| !mutex.iter().any(|peer| peer.addr() != *addr))
            .collect()
    }

    async fn send_internal_event(&self, event: InternalEvent) {
        if let Err(e) = self.event_sender.send(event).await {
            warn!("Failed to send command event to torrent {}, {}", self, e);
        }
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

            if let Some((offset, range)) = file.byte_range(piece) {
                if let Err(e) = self.storage.write_piece(&file, offset, &data[range]).await {
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

    fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_box()).collect()
    }

    fn invoke_event(&self, event: TorrentEvent) {
        self.callbacks.invoke(event)
    }

    async fn try_create_peer_connection(
        torrent: Torrent,
        peer_addr: SocketAddr,
        extensions: Extensions,
    ) -> Result<Peer> {
        let runtime = torrent.runtime.clone();
        Ok(Peer::new_outbound(peer_addr, torrent, extensions, runtime).await?)
    }
}

impl Callbacks<TorrentEvent> for InnerTorrent {
    fn add_callback(&self, callback: CoreCallback<TorrentEvent>) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.callbacks.remove_callback(handle)
    }
}

impl Display for InnerTorrent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle)
    }
}

impl Drop for InnerTorrent {
    fn drop(&mut self) {
        trace!("Torrent {} is being dropped", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::peers::extensions::metadata::MetadataExtension;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use tempfile::tempdir;

    #[test]
    fn test_torrent_start_announcing() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info,
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();

        let result = runtime.block_on(torrent.start_announcing());

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_torrent_announce() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info,
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();

        let result = runtime.block_on(torrent.announce()).unwrap();

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
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();

        let metadata = runtime.block_on(torrent.metadata()).unwrap();

        assert_eq!(torrent_info, metadata);
    }

    #[test]
    fn test_retrieve_metadata() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = channel();
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![Box::new(MetadataExtension::new())],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: Some(Duration::from_secs(2)),
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
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
    fn test_resume() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: TorrentFlags::None,
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: Some(Duration::from_secs(2)),
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();
        let (tx_piece_completed, rx_piece_completed) = channel();
        let (tx_pieces_event, rx_pieces_event) = channel();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::PieceCompleted(_) = event {
                tx_piece_completed.send(event).unwrap();
            } else if let TorrentEvent::PiecesChanged = event {
                tx_pieces_event.send(event).unwrap();
            }
        }));

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
        runtime.block_on(torrent.resume());

        // wait for a piece to be completed
        let _ = rx_piece_completed
            .recv_timeout(Duration::from_secs(120))
            .unwrap();
    }

    #[test]
    fn test_torrent_create_pieces() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();
        let (tx, rx) = channel();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::PiecesChanged = event {
                tx.send(event).unwrap();
            }
        }));

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(200)).unwrap();
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
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            storage: Box::new(DefaultTorrentFileStorage::new(temp_path)),
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();
        let (tx, rx) = channel();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::FilesChanged = event {
                tx.send(event).unwrap();
            }
        }));

        // wait for the pieces changed event
        let _ = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        let files = runtime.block_on(torrent.files()).unwrap();

        assert_eq!(1, files.len(), "expected the files to have been created");
    }
}
