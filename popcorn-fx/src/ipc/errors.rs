use std::io;
use thiserror::Error;
use tokio::sync::oneshot;

/// The result type of IPC message operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The IPC message operations related errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid message type {0}")]
    InvalidMessage(String),
    #[error("message \"{0}\" is not supported")]
    UnsupportedMessage(String),
    #[error("enum value is not supported")]
    UnsupportedEnum,
    #[error("missing required field")]
    MissingField,
    #[error("message type cannot be empty")]
    MissingMessageType,
    #[error("a protobuf error occurred, {0}")]
    Proto(protobuf::Error),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<protobuf::Error> for Error {
    fn from(error: protobuf::Error) -> Self {
        Error::Proto(error)
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(value: oneshot::error::RecvError) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::Other, value))
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::InvalidLength, Error::InvalidLength) => true,
            (Error::Proto(_), Error::Proto(_)) => true,
            (Error::Io(_), Error::Io(_)) => true,
            _ => false,
        }
    }
}
