use thiserror::Error;

/// The result type for the updater package.
pub type Result<T> = std::result::Result<T, UpdateError>;

/// These error indicate that an issue arose while handling an update action.
#[derive(Debug, Clone, Error)]
pub enum UpdateError {
    #[error("The update channel \"{0}\" is invalid and cannot be queried")]
    InvalidUpdateChannel(String),
    #[error("Received invalid update channel response, {0}")]
    Response(String)
}