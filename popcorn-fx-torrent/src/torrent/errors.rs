use crate::torrent::storage::Error;
use crate::torrent::tracker::TrackerError;
use crate::torrent::{dht, peer, storage, SessionState, TorrentHandle};
use std::io;
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

#[derive(Debug, Error)]
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
    #[error("torrent handle {0} is no longer valid or invalid")]
    InvalidHandle(TorrentHandle),
    #[error("session is in invalid state {0}")]
    InvalidSessionState(SessionState),
    #[error("torrent request is invalid, {0}")]
    InvalidRequest(String),
    #[error("session is invalid, {0}")]
    InvalidSession(String),
    #[error("the specified range {0:?} is invalid")]
    InvalidRange(std::ops::Range<usize>),
    #[error("tracker error: {0}")]
    Tracker(TrackerError),
    #[error("peer error: {0}")]
    Peer(peer::Error),
    #[error("dht error: {0}")]
    Dht(dht::Error),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
    #[error("the torrent operation has timed out")]
    Timeout,
    #[error("a torrent piece error occurred, {0}")]
    Piece(PieceError),
    #[error("the requested data is unavailable")]
    DataUnavailable,
}

impl PartialEq for TorrentError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Magnet(_), Self::Magnet(_)) => true,
            (Self::TorrentParse(_), Self::TorrentParse(_)) => true,
            (Self::AddressParse(_), Self::AddressParse(_)) => true,
            (Self::InvalidMetadata(_), Self::InvalidMetadata(_)) => true,
            (Self::InvalidTopic(_), Self::InvalidTopic(_)) => true,
            (Self::InvalidInfoHash(le), Self::InvalidInfoHash(re)) => le == re,
            (Self::InvalidHandle(_), Self::InvalidHandle(_)) => true,
            (Self::InvalidSessionState(_), Self::InvalidSessionState(_)) => true,
            (Self::InvalidRequest(_), Self::InvalidRequest(_)) => true,
            (Self::InvalidSession(_), Self::InvalidSession(_)) => true,
            (Self::InvalidRange(_), Self::InvalidRange(_)) => true,
            (Self::Tracker(le), Self::Tracker(re)) => le == re,
            (Self::Peer(le), Self::Peer(re)) => le == re,
            (Self::Dht(le), Self::Dht(re)) => le == re,
            (Self::Io(_), Self::Io(_)) => true,
            (Self::Timeout, Self::Timeout) => true,
            (Self::Piece(_), Self::Piece(_)) => true,
            (Self::DataUnavailable, Self::DataUnavailable) => true,
            _ => false,
        }
    }
}

impl From<MagnetError> for TorrentError {
    fn from(err: MagnetError) -> Self {
        TorrentError::Magnet(err)
    }
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

impl From<io::Error> for TorrentError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
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

impl From<storage::Error> for TorrentError {
    fn from(err: storage::Error) -> Self {
        match err {
            Error::Unavailable | Error::OutOfBounds => {
                Self::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
            Error::InvalidFilepath(e) => {
                Self::Io(io::Error::new(io::ErrorKind::NotFound, format!("{:?}", e)))
            }
            Error::Io(e) => Self::Io(e),
        }
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
    fn test_torrent_error_from_magnet_error() {
        let err_text = "lorem ipsum dolor";
        let err = MagnetError::Parse(err_text.to_string());
        let expected_result = TorrentError::Magnet(err.clone());

        let result: TorrentError = err.into();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_torrent_error_from_tracker_error() {
        let err_text = "foo bar";
        let err = TrackerError::Connection(err_text.to_string());
        let expected_result = TorrentError::Tracker(TrackerError::Connection(err_text.to_string()));

        let result: TorrentError = err.into();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_error_from_peer_error() {
        let err = peer::Error::InvalidPeerId;
        let expected_result = TorrentError::Peer(peer::Error::InvalidPeerId);

        let result: TorrentError = err.into();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_error_from_io_error() {
        let error = "foo bar";
        let io_err = io::Error::new(io::ErrorKind::Other, error);

        let result: TorrentError = io_err.into();

        let err_text = result.to_string();
        assert_eq!(format!("an io error occurred, {}", error), err_text);
    }
}
