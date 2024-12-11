use crate::torrent::TorrentError;
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::time::error::Elapsed;

/// The peer operation specific [std::result::Result] type
pub type Result<T> = std::result::Result<T, Error>;

/// Indicates that an error occurred while communicating with a peer
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("peer id is invalid")]
    InvalidPeerId,
    #[error("invalid message length specified, expected {0} but got {1}")]
    InvalidLength(u32, u32),
    #[error("unsupported message type {0}")]
    UnsupportedMessage(u8),
    #[error("handshake with {0} failed, {1}")]
    Handshake(SocketAddr, String),
    #[error("failed to parse message, {0}")]
    Parsing(String),
    #[error("failed to execute the torrent operation, {0}")]
    Torrent(String),
    #[error("an io error occurred, {0}")]
    Io(String),
    #[error("the peer is no longer available")]
    Closed,
}

impl From<TorrentError> for Error {
    fn from(error: TorrentError) -> Self {
        Error::Torrent(error.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error.to_string())
    }
}

impl From<Elapsed> for Error {
    fn from(error: Elapsed) -> Self {
        Error::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_channel_error() {
        let error = Error::Closed;

        let result = Error::from(error);

        assert_eq!(Error::Closed, result);
    }
}
