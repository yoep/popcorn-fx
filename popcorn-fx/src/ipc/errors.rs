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
            (Error::InvalidMessage(_), Error::InvalidMessage(_)) => true,
            (Error::UnsupportedMessage(_), Error::UnsupportedMessage(_)) => true,
            (Error::UnsupportedEnum, Error::UnsupportedEnum) => true,
            (Error::MissingField, Error::MissingField) => true,
            (Error::MissingMessageType, Error::MissingMessageType) => true,
            (Error::Proto(_), Error::Proto(_)) => true,
            (Error::Io(_), Error::Io(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io = io::Error::new(io::ErrorKind::Other, "Some IO error");
        let result = Error::from(io);
        if let Error::Io(error) = result {
            assert_eq!(error.kind(), io::ErrorKind::Other);
        } else {
            assert!(false, "expected Error::Io, but got {:?} instead", result);
        }
    }

    #[test]
    fn test_from_protobuf_error() {
        let io = io::Error::new(io::ErrorKind::Other, "Some IO error");
        let proto_err = protobuf::Error::from(io);
        let result = Error::from(proto_err);
        if let Error::Proto(_) = result {
        } else {
            assert!(false, "expected Error::Proto, but got {:?} instead", result);
        }
    }
}
