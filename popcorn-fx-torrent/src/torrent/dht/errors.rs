use crate::torrent::dht::krpc::ErrorMessage;
use std::{io, result};
use thiserror::Error;

/// The result type of the DHT package.
pub type Result<T> = result::Result<T, Error>;

/// The errors that can occur within DHT operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("krpc message is invalid, {0}")]
    InvalidMessage(String),
    #[error("transaction id is invalid")]
    InvalidTransactionId,
    #[error("invalid node id")]
    InvalidNodeId,
    #[error("address is invalid")]
    InvalidAddr,
    #[error("token is invalid")]
    InvalidToken,
    #[error("response error code {0}, {1}")]
    Response(u16, String),
    #[error("timed-out while waiting for a response from the node")]
    Timeout,
    #[error("failed to parse message, {0}")]
    Parse(String),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
    #[error("node server has been stopped")]
    Closed,
}

impl From<&ErrorMessage> for Error {
    fn from(value: &ErrorMessage) -> Self {
        Self::Response(value.code(), value.description().to_string())
    }
}

impl From<serde_bencode::error::Error> for Error {
    fn from(e: serde_bencode::error::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidMessage(_), Self::InvalidMessage(_)) => true,
            (Self::InvalidTransactionId, Self::InvalidTransactionId) => true,
            (Self::InvalidNodeId, Self::InvalidNodeId) => true,
            (Self::InvalidAddr, Self::InvalidAddr) => true,
            (Self::InvalidToken, Self::InvalidToken) => true,
            (Self::Response(code, _), Self::Response(other_code, _)) => code == other_code,
            (Self::Timeout, Self::Timeout) => true,
            (Self::Parse(_), Self::Parse(_)) => true,
            (Self::Io(_), Self::Io(_)) => true,
            (Self::Closed, Self::Closed) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_error_message() {
        let msg = "A Generic error occurred";
        let error = ErrorMessage::Generic(msg.to_string());
        let expected_result = Error::Response(201, msg.to_string());

        let result = Error::from(&error);

        assert_eq!(expected_result, result);
    }
}
