use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("the requested torrent piece data is unavailable")]
    Unavailable,
    #[error("the torrent filepath {0} is invalid")]
    InvalidFilepath(PathBuf),
    #[error("an io error occurred, {0}")]
    Io(String),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error.to_string())
    }
}
