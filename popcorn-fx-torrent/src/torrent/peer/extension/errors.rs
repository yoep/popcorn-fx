use thiserror::Error;

/// The extension specific result type
pub type Result<T> = std::result::Result<T, Error>;

/// The errors which may occur within extensions
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("failed to parse extension payload, {0}")]
    Parsing(String),
    #[error("failed to execute extension operation, {0}")]
    Operation(String),
    #[error("an io error occurred, {0}")]
    Io(String),
    #[error("the payload or operation is not supported")]
    Unsupported,
}

impl From<serde_bencode::error::Error> for Error {
    fn from(error: serde_bencode::error::Error) -> Self {
        Self::Parsing(error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
