use thiserror::Error;

use crate::core::torrent::{TorrentManagerState, TorrentStreamState};

/// The torrent package specific results.
pub type Result<T> = std::result::Result<T, TorrentError>;

/// The torrent error describes exceptions which have occurred when handling
/// torrent actions.
#[derive(Debug, Clone, Error)]
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
}