use thiserror::Error;

use crate::core::torrents::{TorrentManagerState, TorrentStreamState};

/// The torrent package specific results.
pub type Result<T> = std::result::Result<T, TorrentError>;

/// The torrent error describes exceptions which have occurred when handling
/// torrent actions.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TorrentError {
    #[error("Torrent url {0} is invalid")]
    InvalidUrl(String),
    #[error("Torrent file {0} cannot be found")]
    FileNotFound(String),
    #[error("Torrent file encountered an error, {0}")]
    FileError(String),
    #[error("Torrent stream has invalid state {0}")]
    InvalidStreamState(TorrentStreamState),
    #[error("Torrent manager has invalid state {0}")]
    InvalidManagerState(TorrentManagerState),
    #[error("Torrent handle {0} is not valid")]
    InvalidHandle(String),
    #[error("The torrent info couldn't be resolved, {0}")]
    TorrentResolvingFailed(String),
    #[error("Failed to load the torrent collection, {0}")]
    TorrentCollectionLoadingFailed(String),
}
