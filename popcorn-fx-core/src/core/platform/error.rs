use thiserror::Error;

/// The platform package specific result type.
/// This result will always return [PlatformError]. 
pub type Result<T> = std::result::Result<T, PlatformError>;

/// The platform specific errors.
#[derive(Debug, Error)]
pub enum PlatformError {
    /// The screensaver specific error.
    /// `String` contains the specific error message.
    #[error("Screensaver error occurred, {0}")]
    Screensaver(String)
}