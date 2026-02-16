use std::{io, result};
use thiserror::Error;

/// The result type of logging operations.
pub type Result<T> = result::Result<T, Error>;

/// The errors of the logging crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("a logger instance has already been initialized")]
    AlreadyInitialized,
    #[error("path does not exist")]
    NotFound,
    #[error("configuration file is invalid, {0}")]
    InvalidConfig(String),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::AlreadyInitialized, Error::AlreadyInitialized) => true,
            (Error::NotFound, Error::NotFound) => true,
            (Error::InvalidConfig(_), Error::InvalidConfig(_)) => true,
            (Error::Io(_), Error::Io(_)) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_eq() {
        assert_eq!(Error::AlreadyInitialized, Error::AlreadyInitialized);
        assert_eq!(Error::NotFound, Error::NotFound);
        assert_eq!(
            Error::InvalidConfig("Foo".to_string()),
            Error::InvalidConfig("Bar".to_string())
        );
        assert_eq!(
            Error::Io(io::Error::from(io::ErrorKind::AlreadyExists)),
            Error::Io(io::Error::from(io::ErrorKind::AlreadyExists))
        );
        assert_ne!(Error::AlreadyInitialized, Error::NotFound);
    }

    #[test]
    fn test_from_io() {
        let err = io::Error::from(io::ErrorKind::AlreadyExists);
        let expected_result = Error::Io(io::Error::from(io::ErrorKind::AlreadyExists));

        let result = Error::from(err);

        assert_eq!(expected_result, result);
    }
}
