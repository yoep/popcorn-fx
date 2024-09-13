use crate::torrents::channel::ChannelError;
use std::io;
use thiserror::Error;
use tokio::time::error::Elapsed;

/// The peer operation specific [std::result::Result] type
pub type Result<T> = std::result::Result<T, PeerError>;

/// Indicates that an error occurred while communicating with a peer
#[derive(Debug, Clone, Error, PartialEq)]
pub enum PeerError {
    #[error("peer id is invalid")]
    InvalidPeerId,
    #[error("unsupported message type {0}")]
    UnsupportedMessage(u8),
    #[error("an error occurred during the handshake, {0}")]
    Handshake(String),
    #[error("failed to parse message, {0}")]
    Parsing(String),
    #[error("an io error occurred, {0}")]
    Io(String),
    #[error("the peer is no longer available")]
    Closed,
}

impl From<io::Error> for PeerError {
    fn from(error: io::Error) -> Self {
        PeerError::Io(error.to_string())
    }
}

impl From<Elapsed> for PeerError {
    fn from(error: Elapsed) -> Self {
        PeerError::Io(error.to_string())
    }
}

impl From<ChannelError> for PeerError {
    fn from(error: ChannelError) -> Self {
        if let ChannelError::Closed = error {
            return PeerError::Closed;
        }

        PeerError::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_channel_error() {
        let error = ChannelError::Closed;

        let result = PeerError::from(error);

        assert_eq!(PeerError::Closed, result);
    }
}
