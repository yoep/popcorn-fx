use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ChromecastError {
    #[error("failed to discover Chromecast devices, {0}")]
    Discovery(String),
    #[error("failed to establish connection with Chromecast device, {0}")]
    Connection(String),
    #[error("failed to initialize default media receiver app, {0}")]
    AppInitializationFailed(String),
    #[error("the default media receiver app is not running")]
    AppNotInitialized,
    #[error("failed to stop the default media receiver app, {0}")]
    AppTerminationFailed(String),
    #[error("failed to parse chromecast message, {0}")]
    Parsing(String),
    #[error("command {0} timed out")]
    CommandTimeout(String),
}

pub type Result<T> = std::result::Result<T, ChromecastError>;
