use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{DowncastSync, impl_downcast};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::{CoreCallback, torrents};
use crate::core::torrents::{Torrent, TorrentFileInfo, TorrentInfo, TorrentWrapper};

/// The callback type for the torrent manager events.
pub type TorrentManagerCallback = CoreCallback<TorrentManagerEvent>;

/// The states of the [TorrentManager].
#[repr(i32)]
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentManagerState {
    /// The initial state of the torrent manager.
    /// This state builds the session and makes sure a session could be established.
    Initializing = 0,
    /// Indicates that the torrent manager is running and can start
    /// handling torrent actions
    Running = 1,
    /// Indicates that the torrent manager encountered an error and could not be started.
    /// This is most of the time related to failures when creating sessions.
    Error = 2,
}

/// The events of the torrent manager.
#[derive(Debug, Clone)]
pub enum TorrentManagerEvent {
    /// Indicates that the state of the torrent manager has changed
    /// * `TorrentManagerState` - The new state of the manager
    StateChanged(TorrentManagerState)
}

impl Display for TorrentManagerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentManagerEvent::StateChanged(state) => write!(f, "Manager state changed to {}", state),
        }
    }
}

/// The torrent manager stores the active sessions and torrents that are being processed.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait TorrentManager: Debug + DowncastSync {
    /// Retrieve the current state of the torrent manager.
    ///
    /// # Returns
    ///
    /// An owned instance of the torrent manager state.
    fn state(&self) -> TorrentManagerState;

    /// Register a new callback to this manager.
    ///
    /// The callback will receive events when an action occurs in this manager.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to register.
    fn register(&self, callback: TorrentManagerCallback);

    /// Resolve the given URL into torrent information.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to resolve into torrent information.
    ///
    /// # Returns
    ///
    /// The torrent meta information on success, or a [torrent::TorrentError] if there was an error.
    async fn info<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentInfo>;

    async fn create(&self, file_info: &TorrentFileInfo, torrent_directory: &str, auto_download: bool) -> torrents::Result<Box<dyn Torrent>>;

    /// Add a new torrent wrapper to the manager.
    ///
    /// # Arguments
    ///
    /// * `torrent` - The torrent wrapper to add to the manager.
    fn add(&self, torrent: Arc<TorrentWrapper>);

    /// Cleanup the torrents directory.
    ///
    /// This operation removes all torrents from the filesystem.
    fn cleanup(&self);
}
impl_downcast!(sync TorrentManager);

#[cfg(test)]
mod test {
    use crate::core::torrents::{TorrentManagerEvent, TorrentManagerState};

    #[test]
    fn test_torrent_manager_event_display() {
        let error = TorrentManagerEvent::StateChanged(TorrentManagerState::Error).to_string();
        let running = TorrentManagerEvent::StateChanged(TorrentManagerState::Running).to_string();

        assert_eq!("Manager state changed to Error".to_string(), error);
        assert_eq!("Manager state changed to Running".to_string(), running);
    }
}