use thiserror::Error;

use crate::core::torrents::TorrentStreamState;

/// The torrent package specific results.
pub type Result<T> = std::result::Result<T, Error>;

/// The torrent error describes exceptions which have occurred when handling
/// torrent actions.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    #[error("Torrent url {0} is invalid")]
    InvalidUrl(String),
    #[error("Torrent file {0} cannot be found")]
    FileNotFound(String),
    #[error("Torrent stream has invalid state {0}")]
    InvalidStreamState(TorrentStreamState),
    #[error("Torrent handle {0} is not valid")]
    InvalidHandle(String),
    #[error("The torrent info couldn't be resolved, {0}")]
    TorrentResolvingFailed(String),
    #[error("Failed to load the torrent collection, {0}")]
    TorrentCollectionLoadingFailed(String),
    #[error("{0}")]
    TorrentError(String),
    #[error("an io error occurred, {0}")]
    Io(String),
}
