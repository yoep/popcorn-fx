use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// The result type of file system operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The file system specific errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("piece data is unavailable")]
    Unavailable,
    #[error("the requested range is out-of-bounds")]
    OutOfBounds,
    #[error("the torrent filepath {0} is invalid")]
    InvalidFilepath(PathBuf),
    #[error("an io error occurred, {0}")]
    Io(io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Unavailable, Error::Unavailable) => true,
            (Error::OutOfBounds, Error::OutOfBounds) => true,
            (Error::InvalidFilepath(_), Error::InvalidFilepath(_)) => true,
            (Error::Io(_), Error::Io(_)) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
