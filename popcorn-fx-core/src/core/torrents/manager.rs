use std::fmt::{Debug, Display, Formatter};
use std::sync::Weak;

use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::torrents::{Torrent, TorrentFileInfo, TorrentHealth, TorrentInfo};
use crate::core::{torrents, CoreCallback};

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
    StateChanged(TorrentManagerState),
}

impl Display for TorrentManagerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentManagerEvent::StateChanged(state) => {
                write!(f, "Manager state changed to {}", state)
            }
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

    /// Retrieve the health of the torrent based on the given magnet link.
    ///
    /// # Arguments
    ///
    /// * `url` - The magnet link of the torrent
    ///
    /// # Returns
    ///
    /// The torrent health on success, or a [torrent::TorrentError] if there was an error.
    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth>;

    /// Create a new torrent session based on the provided file information.
    ///
    /// # Arguments
    ///
    /// * `file_info` - The file information for the torrent.
    /// * `torrent_directory` - The directory where the torrent files will be stored.
    /// * `auto_download` - A flag indicating whether the torrent should start downloading automatically.
    ///
    /// # Returns
    ///
    /// A `Result` containing a weak reference to the created torrent session on success,
    /// or a [torrent::TorrentError] on failure.
    async fn create(
        &self,
        file_info: &TorrentFileInfo,
        torrent_directory: &str,
        auto_download: bool,
    ) -> torrents::Result<Weak<Box<dyn Torrent>>>;

    /// Retrieve a torrent session by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a weak reference to the torrent session if found, or `None` if not found.
    fn by_handle(&self, handle: &str) -> Option<Weak<Box<dyn Torrent>>>;

    /// Remove a torrent session by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to remove.
    fn remove(&self, handle: &str);

    /// Calculate the health of the torrent based on the given seed count and peer count.
    ///
    /// # Arguments
    ///
    /// * `seeds` - The number of seeds the torrent has (completed peers).
    /// * `leechers` - The number of leechers the torrent has (incomplete peers).
    ///
    /// # Returns
    ///
    /// Returns the calculated torrent health.
    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth;

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
