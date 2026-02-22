use std::{io, result};
use thiserror::Error;

/// The result type of the stream module.
pub type Result<T> = result::Result<T, Error>;

/// Errors that can occur during streaming operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("a stream for \"{0}\" already exists")]
    AlreadyExists(String),
    #[error("the stream for \"{0}\" was not found")]
    NotFound(String),
    #[error("the stream is in an invalid state")]
    InvalidState,
    #[error("invalid stream range")]
    InvalidRange,
    #[error("stream parsing error occurred, {0}")]
    Parse(String),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AlreadyExists(_), Self::AlreadyExists(_)) => true,
            (Self::NotFound(_), Self::NotFound(_)) => true,
            (Self::InvalidState, Self::InvalidState) => true,
            (Self::InvalidRange, Self::InvalidRange) => true,
            (Self::Parse(_), Self::Parse(_)) => true,
            (Self::Io(_), Self::Io(_)) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_eq() {
        assert_eq!(
            Error::AlreadyExists("Foo".to_string()),
            Error::AlreadyExists("Foo".to_string())
        );
        assert_eq!(
            Error::NotFound("Foo".to_string()),
            Error::NotFound("Foo".to_string())
        );
        assert_eq!(Error::InvalidState, Error::InvalidState);
        assert_eq!(Error::InvalidRange, Error::InvalidRange);
        assert_eq!(Error::Io(new_io_error()), Error::Io(new_io_error()));
        assert_ne!(
            Error::AlreadyExists("Foo".to_string()),
            Error::NotFound("Bar".to_string())
        );
    }

    #[test]
    fn test_from_io_error() {
        let result = Error::from(new_io_error());
        assert_eq!(Error::Io(new_io_error()), result);
    }

    fn new_io_error() -> io::Error {
        io::Error::new(io::ErrorKind::Other, "Foo")
    }
}
