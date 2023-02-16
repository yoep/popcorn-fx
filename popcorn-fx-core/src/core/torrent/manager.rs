use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use derive_more::Display;

use crate::core::CoreCallback;

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
#[async_trait]
pub trait TorrentManager {
    /// Retrieve the current state of the torrent manager.
    ///
    /// It returns an owned instance of the state.
    fn state(&self) -> TorrentManagerState;

    /// Register a new callback to this manager.
    /// The callback will receive events when an action occurs in this manager.
    fn register(&self, callback: TorrentManagerCallback);
}

#[cfg(test)]
mod test {
    use crate::core::torrent::{TorrentManagerEvent, TorrentManagerState};

    #[test]
    fn test_torrent_manager_event_display() {
        let error = TorrentManagerEvent::StateChanged(TorrentManagerState::Error).to_string();
        let running = TorrentManagerEvent::StateChanged(TorrentManagerState::Running).to_string();

        assert_eq!("Manager state changed to Error".to_string(), error);
        assert_eq!("Manager state changed to Running".to_string(), running);
    }
}