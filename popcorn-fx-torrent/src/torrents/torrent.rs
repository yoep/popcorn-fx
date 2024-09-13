use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bitmask_enum::bitmask;
use derive_more::Display;
use futures::future::join_all;
use log::{debug, error, trace, warn};
use tokio::runtime::Runtime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use url::Url;

use popcorn_fx_core::core::{CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};

use crate::torrents::channel::{new_command_channel, ChannelError};
use crate::torrents::peers::{Peer, PeerId};
use crate::torrents::torrent_commands::{
    TorrentCommand, TorrentCommandInstruction, TorrentCommandReceiver, TorrentCommandResponse,
    TorrentCommandSender,
};
use crate::torrents::trackers::{Announcement, Tracker, TrackerError, TrackerManager};
use crate::torrents::{Pieces, Result, TorrentError, TorrentInfo};

const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// Possible flags which can be attached to a [Torrent].
#[bitmask(u8)]
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
    /// Torrent is auto-managed.
    /// This means that the torrent may be resumed at any point in time.
    AutoManaged = 0b00100000,
    /// Sequential download is enabled.
    SequentialDownload = 0b01000000,
    /// Torrent should stop when ready.
    StopWhenReady = 0b10000000,
}

impl Default for TorrentFlags {
    fn default() -> Self {
        TorrentFlags::None
    }
}

#[derive(Debug, Clone)]
pub enum TorrentStatus {
    /// The torrent has not started its download yet, and is currently checking existing files.
    CheckingFiles,
    /// The torrent is trying to download metadata from peers.
    DownloadingMetadata,
    /// The torrent is being downloaded. This is the state most torrents will be in most of the time.
    Downloading,
    /// In this state the torrent has finished downloading but still doesn't have the entire torrent.
    Finished,
    /// In this state the torrent has finished downloading and is a pure seeder.
    Seeding,
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub piece_count: usize,
    pub piece_len: u32,
    pub last_piece_len: u32,
    pub download_len: u64,
    pub download_dir: PathBuf,
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
///         timeout: Some(Duration::from_secs(10)),
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
    /// The maximum amount of time to wait for a response from trackers & peers
    pub timeout: Option<Duration>,
    /// The underlying Tokio runtime to use for asynchronous operations
    pub runtime: Option<Arc<Runtime>>,
}

pub type TorrentCallback = CoreCallback<TorrentEvent>;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentEvent {}

/// A torrent is an actual tracked torrent which is communicating with one or more trackers and peers.
///
/// Use [crate::torrents::TorrentInfo] if you only want to retrieve the metadata of a torrent.
#[derive(Debug)]
pub struct Torrent {
    handle: TorrentHandle,
    peer_id: PeerId,
    command_sender: TorrentCommandSender,
    runtime: Arc<Runtime>,
    cancellation_token: CancellationToken,
}

impl Torrent {
    /// Retrieve the unique handle of this torrent.
    /// This handle identifies the torrent within a session.
    ///
    /// # Returns
    ///
    /// Returns the unique handle of this torrent.
    pub fn handle(&self) -> TorrentHandle {
        self.handle
    }

    /// Retrieve the command sender of this torrent.
    ///
    /// # Returns
    ///
    /// Returns the command sender of this torrent.
    pub(crate) fn command_sender(&self) -> TorrentCommandSender {
        self.command_sender.clone()
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

    /// Start announcing the torrent to its known trackers.
    ///
    /// # Returns
    ///
    /// nothing if the announcing process has started for this torrent, else the [TorrentError].
    pub async fn start_announcing(&self) -> Result<()> {
        let trackers = self.retrieve_active_trackers().await?;

        if trackers.is_empty() {
            self.add_known_torrent_trackers().await?;
        }

        self.command_sender
            .send_void(TorrentCommand::StartAnnouncing)?;

        Ok(())
    }

    pub async fn announce(&self) -> Result<Announcement> {
        let trackers = self.retrieve_active_trackers().await?;
        if trackers.is_empty() {
            self.add_known_torrent_trackers().await?;
        }

        self.command_sender
            .send(TorrentCommand::AnnounceAll)
            .await
            .map(|response| {
                if let TorrentCommandResponse::AnnounceAll(announce) = response {
                    return Ok(announce);
                }

                Err(ChannelError::UnexpectedCommandResponse(
                    "TorrentCommandResponse::AnnounceAll".to_string(),
                    format!("{:?}", response),
                ))?
            })?
    }

    async fn add_known_torrent_trackers(&self) -> Result<()> {
        let metadata = self.retrieve_metadata().await?;
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
                let sender = self.command_sender.clone();
                let task = self.runtime.spawn(async move {
                    if let Err(e) = sender.send_void(TorrentCommand::AddTracker(url, tier)) {
                        error!("Failed to add tracker to torrent, {}", e);
                    }
                });
                tracker_futures.push(task);
            }
        }

        join_all(tracker_futures).await;
        let time_taken = start_time.elapsed();
        trace!(
            "Took {}.{:03} seconds to add trackers",
            time_taken.as_secs(),
            time_taken.subsec_millis()
        );
        Ok(())
    }

    async fn retrieve_metadata(&self) -> Result<TorrentInfo> {
        self.command_sender
            .send(TorrentCommand::Metadata)
            .await
            .map(|response| {
                if let TorrentCommandResponse::Metadata(metadata) = response {
                    return Ok(metadata);
                }

                Err(ChannelError::UnexpectedCommandResponse(
                    "TorrentCommandResponse::Metadata".to_string(),
                    format!("{:?}", response),
                ))?
            })?
    }

    async fn retrieve_active_trackers(&self) -> Result<Vec<Url>> {
        self.command_sender
            .send(TorrentCommand::ActiveTrackers)
            .await
            .map(|response| {
                if let TorrentCommandResponse::ActiveTrackers(trackers) = response {
                    return Ok(trackers);
                }

                Err(ChannelError::UnexpectedCommandResponse(
                    "TorrentCommandResponse::ActiveTrackers".to_string(),
                    format!("{:?}", response),
                ))?
            })?
    }
}

impl Callbacks<TorrentEvent> for Torrent {
    fn add(&self, callback: TorrentCallback) -> CallbackHandle {
        let response = self.runtime.block_on(
            self.command_sender
                .send(TorrentCommand::AddCallback(callback)),
        );

        if let Err(e) = response {
            error!("Failed to add callback to torrent, {}", e);
            return Default::default();
        }

        match response {
            Ok(TorrentCommandResponse::AddCallback(handle)) => handle,
            _ => {
                error!("Failed to add callback to torrent, {:?}", response);
                Default::default()
            }
        }
    }

    fn remove(&self, handle: CallbackHandle) {
        if let Err(e) = self
            .command_sender
            .send_void(TorrentCommand::RemoveCallback(handle))
        {
            error!("Failed to remove the callback from torrent, {}", e);
        }
    }
}

impl Clone for Torrent {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            peer_id: self.peer_id,
            command_sender: self.command_sender.clone(),
            runtime: self.runtime.clone(),
            cancellation_token: self.cancellation_token.clone(),
        }
    }
}

impl TryFrom<TorrentRequest> for Torrent {
    type Error = TorrentError;

    fn try_from(value: TorrentRequest) -> Result<Self> {
        let metadata = value.metadata;
        let runtime = value
            .runtime
            .unwrap_or_else(|| Arc::new(Runtime::new().expect("expected a new runtime")));

        // validate the given metadata before creating the torrent
        metadata.validate()?;

        let handle = TorrentHandle::new();
        let peer_id = PeerId::new();
        let timeout = value
            .timeout
            .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS));
        let cancellation_token = CancellationToken::new();
        let mut inner = InnerTorrent {
            handle,
            metadata,
            peer_id,
            tracker_manager: TrackerManager::new(
                peer_id,
                value.peer_listener_port,
                timeout,
                runtime.clone(),
            ),
            peers: vec![],
            pieces: None,
            callbacks: Default::default(),
            timeout,
            cancellation_token: cancellation_token.clone(),
        };

        let (command_sender, command_receiver) = new_command_channel();

        runtime.spawn(async move {
            inner.start(command_receiver).await;
        });

        Ok(Self {
            handle,
            peer_id,
            command_sender,
            cancellation_token,
            runtime,
        })
    }
}

#[derive(Debug)]
struct InnerTorrent {
    handle: TorrentHandle,
    metadata: TorrentInfo,
    peer_id: PeerId,
    tracker_manager: TrackerManager,
    /// The established peer connections
    peers: Vec<Peer>,
    /// The pieces of the torrent, these are only known if the metadata is available
    pieces: Option<Pieces>,
    callbacks: CoreCallbacks<TorrentEvent>,
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerTorrent {
    async fn start(&mut self, mut command_receiver: TorrentCommandReceiver) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                command = command_receiver.recv() => {
                    if let Some(command) = command {
                        self.handle_command_instruction(command).await;
                    } else {
                        break;
                    }
                }
            }
        }

        debug!("Closing torrent {}", self.handle);
    }

    async fn handle_command_instruction(&mut self, mut instruction: TorrentCommandInstruction) {
        let instruction_info = format!("{:?}", instruction);
        let command_result = match instruction.command {
            TorrentCommand::Metadata => {
                instruction.respond(TorrentCommandResponse::Metadata(self.metadata.clone()))
            }
            TorrentCommand::ActiveTrackers => instruction.respond(
                TorrentCommandResponse::ActiveTrackers(self.active_trackers().await),
            ),
            TorrentCommand::StartAnnouncing => Ok(self.start_announcing().await),
            TorrentCommand::AnnounceAll => {
                instruction.respond(TorrentCommandResponse::AnnounceAll(
                    self.tracker_manager
                        .announce_all(self.metadata.info_hash.clone())
                        .await,
                ))
            }
            TorrentCommand::AddTracker(url, tier) => {
                self.add_tracker(url, tier).await;
                Ok(())
            }
            TorrentCommand::AddCallback(callback) => {
                self.callbacks.add(callback);
                Ok(())
            }
            TorrentCommand::RemoveCallback(handle) => {
                self.callbacks.remove(handle);
                Ok(())
            }
        };

        if let Err(e) = command_result {
            error!("Failed to process torrent {}, {}", instruction_info, e);
        }
    }

    async fn active_trackers(&self) -> Vec<Url> {
        self.tracker_manager.trackers().await
    }

    async fn add_tracker(&self, url: Url, tier: u8) {
        if let Err(e) = self.tracker_manager.add_tracker(&url, tier).await {
            warn!(
                "Failed to add tracker {} to torrent {}, {}",
                url, self.handle, e
            );
            return;
        }

        debug!("Tracker {} has been added to torrent {}", url, self.handle);
    }

    async fn start_announcing(&self) {
        let info_hash = self.metadata.info_hash.clone();
        self.tracker_manager.start_announcing(info_hash);
    }
}

impl Callbacks<TorrentEvent> for InnerTorrent {
    fn add(&self, callback: CoreCallback<TorrentEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
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
            timeout: Some(Duration::from_secs(1)),
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
            timeout: Some(Duration::from_secs(1)),
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
}
