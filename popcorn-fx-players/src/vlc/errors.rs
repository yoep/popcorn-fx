use thiserror::Error;

/// The VLC player result type.
pub type Result<T> = std::result::Result<T, VlcError>;

/// The VLC player errors.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum VlcError {
    #[error("failed to send request, {0}")]
    Request(String),
    #[error("failed to parse response, {0}")]
    Parsing(String),
}
