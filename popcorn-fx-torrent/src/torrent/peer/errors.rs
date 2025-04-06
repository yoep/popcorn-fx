use crate::torrent::PieceIndex;
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::time::error::Elapsed;

/// The peer operation specific [std::result::Result] type
pub type Result<T> = std::result::Result<T, Error>;

/// Indicates that an error occurred while communicating with a peer
#[derive(Debug, Error)]
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
    /// Indicates that the peer is in an invalid state.
    #[error("invalid state {0}")]
    InvalidState(String),
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
    Io(io::Error),
    /// Indicates that the given payload is too large
    #[error("the payload exceeds the maximum size of {0}")]
    TooLarge(usize),
    /// Indicates that the peer connection is closed
    #[error("the peer connection is closed")]
    Closed,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::InvalidPeerId, Error::InvalidPeerId) => true,
            (Error::InvalidLength(_, _), Error::InvalidLength(_, _)) => true,
            (Error::InvalidPiece(_), Error::InvalidPiece(_)) => true,
            (Error::InvalidState(_), Error::InvalidState(_)) => true,
            (Error::UnsupportedMessage(_), Error::UnsupportedMessage(_)) => true,
            (Error::UnsupportedVersion(_), Error::UnsupportedVersion(_)) => true,
            (Error::UnsupportedExtensions(_), Error::UnsupportedExtensions(_)) => true,
            (Error::Handshake(_, _), Error::Handshake(_, _)) => true,
            (Error::Parsing(_), Error::Parsing(_)) => true,
            (Error::Io(_), Error::Io(_)) => true,
            (Error::Closed, Error::Closed) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<Elapsed> for Error {
    fn from(error: Elapsed) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::TimedOut, error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_io() {
        let error = io::Error::from(io::ErrorKind::UnexpectedEof);

        let result = Error::from(error);

        if let Error::Io(_) = result {
            return;
        } else {
            assert!(false, "expected Error::Io, got {:?} instead", result)
        }
    }
}
