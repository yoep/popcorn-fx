use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

use derive_more::Display;
use mockall::automock;

use crate::core::CoreCallback;

const TORRENT_STATES: [TorrentState; 7] = [
    TorrentState::Creating,
    TorrentState::Ready,
    TorrentState::Starting,
    TorrentState::Downloading,
    TorrentState::Paused,
    TorrentState::Completed,
    TorrentState::Error,
];

/// The callback type for all torrent events.
pub type TorrentCallback = CoreCallback<TorrentEvent>;

/// The torrent event which occurred for the torrent.
#[derive(Debug, Clone)]
pub enum TorrentEvent {
    /// The new state of the torrent
    StateChanged(TorrentState),
    /// The piece that has finished downloading
    PieceFinished(u32),
}

impl Display for TorrentEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentEvent::StateChanged(state) => write!(f, "Torrent state changed to {}", state),
            TorrentEvent::PieceFinished(piece_index) => write!(f, "Torrent piece {} finished downloading", piece_index),
        }
    }
}

/// The state of a [Torrent] which is represented as a [i32].
/// This state is abi compatible to be used over [std::ffi].
#[repr(i32)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum TorrentState {
    /// The initial phase of the torrent in which it's still being created.
    /// This is the state where the metadata of the torrent is retrieved.
    Creating = 0,
    /// The torrent is ready to be downloaded (metadata is available).
    Ready = 1,
    /// The download of the torrent is starting.
    Starting = 2,
    /// The torrent is being downloaded.
    Downloading = 3,
    /// The torrent download has been paused.
    Paused = 4,
    /// The torrent download has completed.
    Completed = 5,
    /// The torrent encountered an error and cannot be downloaded.
    Error = -1,
}

impl From<i32> for TorrentState {
    fn from(value: i32) -> Self {
        for state in TORRENT_STATES {
            let ordinal = state.clone() as i32;

            if ordinal == value {
                return state;
            }
        }

        panic!("Ordinal {} is out of range for TorrentState", value)
    }
}

/// The torrent describes the meta-info of a shared file that can be queried over the network.
/// It allows for action such as downloading the shared file to the local system.
#[automock]
pub trait Torrent: Display + Debug + Send + Sync {
    /// The absolute path to this torrent file.
    fn file(&self) -> PathBuf;

    /// Verify if the given bytes are available for this [Torrent].
    ///
    /// It returns true when the bytes are available, else false.
    fn has_bytes(&self, bytes: &[u64]) -> bool;

    /// Prioritize the given bytes to be downloaded.
    fn prioritize_bytes(&self, bytes: &[u64]);

    /// Prioritize the given piece indexes.
    fn prioritize_pieces(&self, pieces: &[u32]);

    /// The total number of pieces that are available for download.
    fn total_pieces(&self) -> i32;

    /// Update the download mode of the torrent to sequential.
    fn sequential_mode(&self);

    /// Register a new callback for the [TorrentEvent]'s.
    /// The callback will be triggered when a new event occurs within the torrent.
    fn register(&self, callback: TorrentCallback);
}

impl Display for MockTorrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTorrent")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_torrent_state_from() {
        let error = TorrentState::from(-1);
        let creating = TorrentState::from(0);
        let ready = TorrentState::from(1);
        let starting = TorrentState::from(2);
        let downloading = TorrentState::from(3);
        let paused = TorrentState::from(4);

        assert_eq!(TorrentState::Error, error);
        assert_eq!(TorrentState::Creating, creating);
        assert_eq!(TorrentState::Ready, ready);
        assert_eq!(TorrentState::Starting, starting);
        assert_eq!(TorrentState::Downloading, downloading);
        assert_eq!(TorrentState::Paused, paused);
    }
}