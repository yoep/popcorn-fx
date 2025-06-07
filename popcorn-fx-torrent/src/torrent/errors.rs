use crate::torrent::tracker::TrackerError;
use crate::torrent::{dht, fs, peer, TorrentHandle};
use thiserror::Error;

/// The result type for the torrent package.
pub type Result<T> = std::result::Result<T, TorrentError>;

/// Represents possible errors that can occur when parsing a magnet URI.
pub type MagnetResult<T> = std::result::Result<T, MagnetError>;

/// Represents possible errors that can occur when parsing a magnet URI.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum MagnetError {
    /// Failed to parse the magnet URI.
    #[error("failed to parse magnet uri, {0}")]
    Parse(String),
    /// The specified magnet URI is invalid.
    #[error("invalid magnet uri")]
    InvalidUri,
    /// The specified file index value is invalid.
    #[error("value \"{0}\" is invalid")]
    InvalidValue(String),
    /// The specified scheme in the magnet URI is not supported.
    #[error("scheme \"{0}\" is not supported")]
    UnsupportedScheme(String),
}

/// The torrent piece specific errors.
/// These errors can occur when working with [Piece] related operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum PieceError {
    #[error("torrent pieces are currently unknown")]
    Unavailable,
    #[error("failed to calculate pieces, {0}")]
    UnableToDeterminePieces(String),
    #[error("exceeding chunk size, expected size {0} but got {1}")]
    InvalidChunkSize(usize, usize),
}

#[derive(Debug, Error, PartialEq)]
pub enum TorrentError {
    #[error("failed to parse magnet uri, {0}")]
    Magnet(MagnetError),
    #[error("failed to parse torrent data, {0}")]
    TorrentParse(String),
    #[error("failed to parse address, {0}")]
    AddressParse(String),
    #[error("the metadata of the torrent is invalid, {0}")]
    InvalidMetadata(String),
    #[error("the provided exact topic (xt) is invalid, {0}")]
    InvalidTopic(String),
    #[error("the provided info hash is invalid, {0}")]
    InvalidInfoHash(String),
    #[error("the torrent handle {0} is no longer valid or invalid")]
    InvalidHandle(TorrentHandle),
    #[error("the torrent request is invalid, {0}")]
    InvalidRequest(String),
    #[error("the session is invalid, {0}")]
    InvalidSession(String),
    #[error("the specified range {0:?} is invalid")]
    InvalidRange(std::ops::Range<usize>),
    #[error("tracker error: {0}")]
    Tracker(TrackerError),
    #[error("peer error: {0}")]
    Peer(peer::Error),
    #[error("dht error: {0}")]
    Dht(dht::Error),
    // TODO: rework to [std::io::Error]
    #[error("an io error occurred, {0}")]
    Io(String),
    #[error("the torrent operation has timed out")]
    Timeout,
    #[error("a torrent piece error occurred, {0}")]
    Piece(PieceError),
    #[error("the requested data is unavailable")]
    DataUnavailable,
}

impl From<TrackerError> for TorrentError {
    fn from(error: TrackerError) -> Self {
        Self::Tracker(error)
    }
}

impl From<peer::Error> for TorrentError {
    fn from(error: peer::Error) -> Self {
        Self::Peer(error)
    }
}

impl From<std::io::Error> for TorrentError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_bencode::Error> for TorrentError {
    fn from(error: serde_bencode::Error) -> Self {
        Self::TorrentParse(error.to_string())
    }
}

impl From<PieceError> for TorrentError {
    fn from(error: PieceError) -> Self {
        Self::Piece(error)
    }
}

impl From<fs::Error> for TorrentError {
    fn from(error: fs::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<dht::Error> for TorrentError {
    fn from(error: dht::Error) -> Self {
        Self::Dht(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

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
        let err = peer::Error::InvalidPeerId;

        let result: TorrentError = err.into();

        assert_eq!(result, TorrentError::Peer(peer::Error::InvalidPeerId));
    }

    #[test]
    fn test_torrent_error_from_io_error() {
        let err = io::Error::new(io::ErrorKind::Other, "foo bar");

        let result: TorrentError = err.into();

        assert_eq!(result, TorrentError::Io("foo bar".to_string()));
    }
}
