use crate::torrents::channel::ChannelError;
use crate::torrents::peers::PeerError;
use crate::torrents::trackers::TrackerError;
use crate::torrents::TorrentHandle;
use thiserror::Error;

/// The result type for the torrent package.
pub type Result<T> = std::result::Result<T, TorrentError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum PieceError {
    #[error("failed to calculate pieces, {0}")]
    UnableToDeterminePieces(String),
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum TorrentError {
    #[error("failed to parse magnet uri, {0}")]
    MagnetParse(String),
    #[error("failed to parse torrent data, {0}")]
    TorrentParse(String),
    #[error("the metadata of the torrent is invalid, {0}")]
    InvalidMetadata(String),
    #[error("the provided exact topic (xt) is invalid, {0}")]
    InvalidTopic(String),
    #[error("the provided info hash is invalid, {0}")]
    InvalidInfoHash(String),
    #[error("the torrent handle {0} is no longer valid or invalid")]
    InvalidHandle(TorrentHandle),
    #[error("a tracker error occurred, {0}")]
    Tracker(TrackerError),
    #[error("a peer error occurred, {0}")]
    Peer(PeerError),
    #[error("an io error occurred, {0}")]
    Io(String),
    #[error("a torrent piece error occurred, {0}")]
    Piece(PieceError),
}

impl From<TrackerError> for TorrentError {
    fn from(error: TrackerError) -> Self {
        TorrentError::Tracker(error)
    }
}

impl From<PeerError> for TorrentError {
    fn from(error: PeerError) -> Self {
        TorrentError::Peer(error)
    }
}

impl From<std::io::Error> for TorrentError {
    fn from(error: std::io::Error) -> Self {
        TorrentError::Io(error.to_string())
    }
}

impl From<PieceError> for TorrentError {
    fn from(error: PieceError) -> Self {
        TorrentError::Piece(error)
    }
}

impl From<ChannelError> for TorrentError {
    fn from(error: ChannelError) -> Self {
        TorrentError::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_error_from_tracker_error() {
        let err = TrackerError::Connection("foo bar".to_string());

        let result: TorrentError = err.into();

        assert_eq!(
            result,
            TorrentError::Tracker(TrackerError::Connection("foo bar".to_string()))
        );
    }

    #[test]
    fn test_torrent_error_from_peer_error() {
        let err = PeerError::Io("foo bar".to_string());

        let result: TorrentError = err.into();

        assert_eq!(
            result,
            TorrentError::Peer(PeerError::Io("foo bar".to_string()))
        );
    }

    #[test]
    fn test_torrent_error_from_io_error() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "foo bar");

        let result: TorrentError = err.into();

        assert_eq!(result, TorrentError::Io("foo bar".to_string()));
    }
}
