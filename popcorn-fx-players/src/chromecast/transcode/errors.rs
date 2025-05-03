use thiserror::Error;

/// Represents errors that can occur during transcoding.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TranscodeError {
    /// The transcoder is unavailable for transcoding the given media.
    #[error("transcoder is unavailable for transcoding the given media")]
    Unavailable,
    /// Transcoding of the given media is not supported.
    #[error("transcoding of the given media is not supported")]
    Unsupported,
    /// The transcoder failed to initialize.
    #[error("transcoder failed to initialize: {0}")]
    Initialization(String),
    /// The transcoder failed to transcode the given media.
    #[error("transcoder failed to transcode the given media: {0}")]
    Transcode(String),
}

/// A specialized `Result` type for transcoding operations.
pub type Result<T> = std::result::Result<T, TranscodeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcode_error_initialization() {
        let error = TranscodeError::Initialization("lorem ipsum".to_string());

        let result = error.to_string();

        assert_eq!("transcoder failed to initialize: lorem ipsum", result);
    }
}
