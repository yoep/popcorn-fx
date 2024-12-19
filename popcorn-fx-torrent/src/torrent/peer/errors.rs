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
    #[error("peer id is invalid")]
    InvalidPeerId,
    #[error("invalid message length, expected {0} but got {1}")]
    InvalidLength(u32, u32),
    #[error("piece index {0} is invalid")]
    InvalidPiece(PieceIndex),
    #[error("unsupported message type {0}")]
    UnsupportedMessage(u8),
    #[error("handshake with {0} failed, {1}")]
    Handshake(SocketAddr, String),
    /// Indicates that a received message couldn't be parsed
    #[error("failed to parse message, {0}")]
    Parsing(String),
    /// Indicates that an io error occurred
    #[error("an io error occurred, {0}")]
    Io(String),
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
