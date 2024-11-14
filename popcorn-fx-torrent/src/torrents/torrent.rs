use std::fmt::{Debug, Display};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc::{channel, RecvError};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use bitmask_enum::bitmask;
use derive_more::Display;
use futures::executor::block_on;
use futures::future;
use futures::future::join_all;
use log::{debug, error, trace, warn};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

use popcorn_fx_core::core::{CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};

use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::{Peer, PeerId, PeerState};

use crate::torrents::trackers::{AnnounceEvent, Announcement, TrackerError, TrackerManager};
use crate::torrents::{
    InfoHash, Piece, PieceError, Result, TorrentError, TorrentInfo, TorrentMetadata,
};

const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

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
    SeedMode = 0b00000001,
    /// Indicates upload mode.
    UploadMode = 0b00000010,
    /// Indicates share mode.
    ShareMode = 0b00000100,
    /// Applies an IP filter.
    ApplyIpFilter = 0b00001000,
    /// Torrent is paused.
    Paused = 0b00010000,
    /// Complete the torrent metadata from peers if needed.
    Metadata = 0b00100000,
    /// Sequential download is enabled.
    SequentialDownload = 0b01000000,
    /// Torrent should stop when ready.
    StopWhenReady = 0b10000000,
    /// Torrent is auto-managed.
    /// This means that the torrent may be resumed at any point in time.
    AutoManaged = 0b100100000,
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

/// The torrent statistics
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentStats {
    /// The total rates for all peers for this torrent.
    /// The rates are given as the number of bytes per second.
    pub uploaded: f64,
    /// The total rates for all peers for this torrent.
    /// The rates are given as the number of bytes per second.
    pub downloaded: f64,
}

/// Requests a new torrent creation based on the given data.
/// This is the **recommended** way to create new torrents.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use popcorn_fx_torrent::torrents::{Torrent, TorrentFlags, TorrentInfo, TorrentRequest, Result};
///
/// fn create_new_torrent(metadata: TorrentInfo) -> Result<Torrent> {
///     let request = TorrentRequest {
///         metadata,
///         options: TorrentFlags::default(),
///         peer_listener_port: 6881,
///         extensions: vec![],
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
    /// The maximum amount of time to wait for a response from peers
    pub peer_timeout: Option<Duration>,
    /// The maximum amount of time to wait for a response from trackers
    pub tracker_timeout: Option<Duration>,
    /// The underlying Tokio runtime to use for asynchronous operations
    pub runtime: Option<Arc<Runtime>>,
}

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
    /// Invoked when the torrent stats have been updated
    #[display(fmt = "torrent stats have been updated")]
    Stats(TorrentStats),
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
        runtime: Arc<Runtime>,
    ) -> Self {
        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let info_hash = metadata.info_hash.clone();
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
                runtime.clone(),
            ),
            peers: RwLock::new(Vec::<Peer>::new()),
            pieces: RwLock::new(None),
            extensions,
            state: RwLock::new(Default::default()),
            options: RwLock::new(flags),
            callbacks: Default::default(),
            timeout: peer_timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)),
            cancellation_token,
        });

        // create a new separate thread which manages the internal torrent resources
        // this thread is automatically cancelled when the torrent is dropped
        let inner_schedule = inner.clone();
        runtime.spawn(async move {
            inner_schedule.start().await;
        });

        let torrent = Self {
            handle,
            peer_id,
            instance: TorrentInstance::Owner(inner),
            runtime,
        };
        let torrent_initialization = torrent.clone();

        // start a new thread which initializes the torrent
        torrent.runtime.spawn(async move {
            if let Err(e) = torrent_initialization.initialize().await {
                error!(
                    "Failed to initialize torrent {}, {}",
                    torrent_initialization.handle, e
                );
            }
        });

        torrent
    }

    /// Retrieve the unique handle of this torrent.
    /// This handle identifies the torrent within a session.
    ///
    /// # Returns
    ///
    /// Returns the unique handle of this torrent.
    pub fn handle(&self) -> TorrentHandle {
        self.handle
    }

    /// Retrieve the unique peer id of this torrent.
    /// This id is used within the peer clients to identify with remote peers.
    ///
    /// # Returns
    ///
    /// Returns the unique peer id of this torrent.
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Retrieve the state of this torrent.
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

    /// Retrieve the metadata of the torrent.
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

    /// Retrieve the torrent pieces, if known.
    /// If the metadata is still being retrieved, the pieces cannot yet be created and will result in [None].
    ///
    /// # Returns
    ///
    /// Returns the current torrent pieces when known, else [None].
    pub async fn pieces(&self) -> Option<Vec<Piece>> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle));

        if let Ok(inner) = inner {
            return inner.pieces.read().await.clone();
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
            self.add_known_torrent_trackers().await?;
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
            Ok(self.add_known_torrent_trackers().await?)
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

    async fn initialize(&self) -> Result<()> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;
        let metadata = inner.metadata().await;
        let torrent_flags = inner.options().await;

        // check if the metadata needs to be requested
        // this will only have an effect when the metadata extension is enabled for peers
        if torrent_flags.contains(TorrentFlags::Metadata) && metadata.info.is_none() {
            self.retrieve_metadata(&inner).await;
        }
        if metadata.info.is_some() {
            inner.create_pieces().await;
        }

        Ok(())
    }

    /// Add the known trackers to the torrent.
    /// These are extracted from the torrent info metadata.
    ///
    /// # Returns
    ///
    /// Returns the collected announcement result of the added trackers.
    async fn add_known_torrent_trackers(&self) -> Result<Announcement> {
        let inner = self
            .instance()
            .ok_or(TorrentError::InvalidHandle(self.handle))?;
        let metadata = inner.metadata().await;
        let tiered_trackers = metadata.tiered_trackers();
        let mut tracker_futures = Vec::with_capacity(tiered_trackers.values().map(Vec::len).sum());

        if tiered_trackers.is_empty() {
            return Err(TorrentError::Tracker(TrackerError::Unavailable));
        }

        // start adding trackers to the torrent
        trace!(
            "Adding torrent {} trackers for {:?}",
            self.handle,
            tiered_trackers
        );
        let start_time = Instant::now();
        for (tier, trackers) in tiered_trackers {
            for url in trackers {
                let inner_torrent_tracker = inner.clone();
                let task = self
                    .runtime
                    .spawn(async move { inner_torrent_tracker.add_tracker(url, tier).await });
                tracker_futures.push(task);
            }
        }

        let announcements: Vec<Announcement> = join_all(tracker_futures)
            .await
            .into_iter()
            .map(|e| {
                if let Err(e) = &e {
                    debug!("Failed to add torrent {} tracker, {}", self, e)
                }

                e
            })
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .map(|e| {
                if let Err(e) = &e {
                    debug!("Failed to add torrent {} tracker, {}", self, e)
                }

                e
            })
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
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

    async fn add_peers(&self, peer_addresses: Vec<SocketAddr>, extensions: Extensions) -> usize {
        let mut futures = Vec::new();
        let mut added_peers = 0;

        for peer_addr in peer_addresses {
            futures.push(self.add_peer(
                peer_addr,
                extensions.iter().map(|e| e.clone_box()).collect(),
            ))
        }

        let responses = future::join_all(futures).await;
        for response in responses {
            match response {
                Ok(_) => added_peers += 1,
                Err(e) => debug!("Failed to add peer to torrent {}, {}", self.handle, e),
            }
        }

        added_peers
    }

    async fn add_peer(&self, peer_addr: SocketAddr, extensions: Extensions) -> Result<()> {
        let peer =
            Peer::new_outbound(peer_addr, self.clone(), extensions, self.runtime.clone()).await?;

        match self.instance() {
            None => return Err(TorrentError::InvalidHandle(self.handle)),
            Some(e) => e.add_peer(peer).await,
        }

        Ok(())
    }

    // TODO: improve the performance of this
    async fn retrieve_metadata(&self, inner: &Arc<InnerTorrent>) {
        inner.update_state(TorrentState::RetrievingMetadata).await;

        const NUMBER_OF_PEERS: usize = 5;
        let metadata = inner.metadata().await;
        let cancellation_token = CancellationToken::new();
        let tiered_trackers = metadata.tiered_trackers();
        let (tx, rx) = channel();

        for (tier, urls) in tiered_trackers {
            for url in urls {
                let tx_tracker = tx.clone();
                let inner_torrent_tracker = inner.clone();
                let cancellation_token_tracker = cancellation_token.clone();
                self.runtime.spawn(async move {
                    select! {
                        _ = cancellation_token_tracker.cancelled() => return,
                        result = inner_torrent_tracker.add_tracker(url, tier) => {
                            match result.map(|e| e.peers)
                                .and_then(|peers| tx_tracker.send(peers)
                                    .map_err(|e| TorrentError::Io(e.to_string()))) {
                                Ok(_) => {},
                                Err(e) => debug!("Failed to add tracker to {}, {}", inner_torrent_tracker, e),
                            }
                        }
                    }
                });
            }
        }

        let mut added_peers = 0;

        // try to add at least NUMBER_OF_PEERS peers
        // if the first attempt failed, we'll try again
        while added_peers < NUMBER_OF_PEERS {
            match rx.recv() {
                Ok(peers) => added_peers += self.add_peers(peers, inner.extensions()).await,
                Err(_) => {
                    debug!(
                        "Reached end of available peers for {} before reaching {} peer connections",
                        self, NUMBER_OF_PEERS
                    );
                    break;
                }
            }
        }

        // stop any running tracker creation threads
        cancellation_token.cancel();
    }

    /// Retrieve a temporary strong reference to the inner torrent.
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
    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: RwLock<Option<Vec<Piece>>>,
    /// The enabled extensions for this torrent
    extensions: Extensions,
    /// The state of the torrent
    state: RwLock<TorrentState>,
    /// The torrent options that are set for this torrent
    options: RwLock<TorrentFlags>,
    /// The callbacks for the torrent events
    callbacks: CoreCallbacks<TorrentEvent>,
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerTorrent {
    /// Start the main loop of this torrent.
    async fn start(&self) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = time::sleep(Duration::from_secs(1)) => self.update_stats().await,
                _ = time::sleep(Duration::from_secs(30)) => self.clean_peers().await,
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

    /// Try to add the given tracker to the tracker manager of this torrent.
    ///
    /// # Returns
    ///
    /// Returns the announcement result that was made to the tracker.
    async fn add_tracker(&self, url: Url, tier: u8) -> Result<Announcement> {
        let handle = self.tracker_manager.add_tracker(&url, tier).await?;

        debug!("Tracker {} has been added to torrent {}", url, self);
        Ok(self
            .tracker_manager
            .announce(handle, AnnounceEvent::Started)
            .await?)
    }

    async fn add_peer(&self, peer: Peer) {
        trace!("Adding peer {} to torrent {}", peer, self);
        {
            let mut mutex = self.peers.write().await;
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
            debug!("Updated metadata of {:?}", self);
        }

        self.invoke_event(TorrentEvent::MetadataChanged);
        self.create_pieces().await;
    }

    async fn start_announcing(&self) {
        self.tracker_manager.start_announcing();
    }

    async fn announce_all(&self) -> Announcement {
        self.tracker_manager.announce_all().await
    }

    async fn update_state(&self, state: TorrentState) {
        {
            let mut mutex = self.state.write().await;
            *mutex = state.clone();
        }

        self.invoke_event(TorrentEvent::StateChanged(state));
    }

    async fn update_stats(&self) {
        let mut uploaded = 0;
        let mut downloaded = 0;

        for peer in self.peers.read().await.iter() {
            let stats = peer.stats_and_reset().await;
            uploaded += stats.upload;
            downloaded += stats.download;
        }

        self.invoke_event(TorrentEvent::Stats(TorrentStats {
            uploaded: uploaded as f64,
            downloaded: downloaded as f64,
        }))
    }

    /// Create the pieces information for the torrent.
    /// This operation can only be done when the metadata of the torrent is known.
    async fn create_pieces(&self) {
        // check if the pieces have already been created
        // if so, ignore this operation
        if self.pieces.read().await.is_some() {
            return;
        }

        match self.create_pieces_result().await {
            Ok(pieces) => {
                let total_pieces = pieces.len();
                {
                    let mut mutex = self.pieces.write().await;
                    *mutex = Some(pieces);
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
    async fn create_pieces_result(&self) -> Result<Vec<Piece>> {
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
                .num_of_pieces()
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

            pieces.push(Piece {
                hash,
                index: piece_index,
                length,
                have: false,
                priority: Default::default(),
            });
        }

        Ok(pieces)
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
    }

    fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_box()).collect()
    }

    fn invoke_event(&self, event: TorrentEvent) {
        self.callbacks.invoke(event)
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
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_torrent_start_announcing() {
        init_logger();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info,
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
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
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info,
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
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
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            peer_timeout: None,
            tracker_timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        })
        .unwrap();

        let metadata = runtime.block_on(torrent.metadata()).unwrap();

        assert_eq!(torrent_info, metadata);
    }

    #[test]
    fn test_torrent_create_pieces() {
        init_logger();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::try_from(TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
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
        let pieces = runtime.block_on(torrent.pieces()).unwrap();

        assert_ne!(0, pieces.len(), "expected the pieces to have been created");
    }
}
