use std::io;
use thiserror::Error;
use url::ParseError;

/// The result type for dns operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The dns package errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("url is invalid, {0}")]
    InvalidUrl(String),
    #[error("{0}")]
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidUrl(_), Self::InvalidUrl(_)) => true,
            (Self::Io(_), Self::Io(_)) => true,
            _ => false,
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::InvalidUrl(e.to_string())
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
    fn test_from_io_error() {
        let err_msg = "FooBar";
        let error = io::Error::new(io::ErrorKind::TimedOut, err_msg);
        let expected_result = Error::Io(io::Error::new(io::ErrorKind::TimedOut, err_msg));

        let result = Error::from(error);

        assert_eq!(expected_result, result);
    }
}
