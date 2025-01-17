use crate::torrent::PieceIndex;
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::time::error::Elapsed;

/// The peer operation specific [std::result::Result] type
pub type Result<T> = std::result::Result<T, Error>;

/// Indicates that an error occurred while communicating with a peer
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    /// Indicates that the peer id is invalid
    #[error("peer id is invalid")]
    InvalidPeerId,
    /// Indicates that an invalid message length has been received
    #[error("invalid message length, expected {0} but got {1}")]
    InvalidLength(u32, u32),
    /// Indicates that an invalid piece index has been received
    #[error("piece index {0} is invalid")]
    InvalidPiece(PieceIndex),
    /// Indicates that a received message is unsupported
    #[error("unsupported message type {0}")]
    UnsupportedMessage(u8),
    /// Indicates that the protocol version is unsupported
    #[error("unsupported version {0}")]
    UnsupportedVersion(u32),
    /// Indicates that a given extensions is not supported
    #[error("extension number {0} is not supported")]
    UnsupportedExtensions(u8),
    /// Indicates that the handshake failed
    #[error("handshake with {0} failed, {1}")]
    Handshake(SocketAddr, String),
    /// Indicates that a received message couldn't be parsed
    #[error("failed to parse message, {0}")]
    Parsing(String),
    /// Indicates that an io error occurred
    #[error("an io error occurred, {0}")]
    Io(String),
    /// Indicates that the given port number is in use
    #[error("port {0} is already in use")]
    PortUnavailable(u16),
    /// Indicates that the peer connection is closed
    #[error("the peer connection is closed")]
    Closed,
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
    fn test_error_from_io() {
        let error = io::Error::from(io::ErrorKind::UnexpectedEof);

        let result = Error::from(error);

        assert_eq!(Error::Io(io::ErrorKind::UnexpectedEof.to_string()), result);
    }
}
