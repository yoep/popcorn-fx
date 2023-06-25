use thiserror::Error;

/// The `ImageError` enum represents the possible errors that can occur during image operations.
#[derive(Debug, Clone, Error)]
pub enum ImageError {
    /// Failed to parse the image URL.
    #[error("failed to parse image url \"{0}\", error: {1}")]
    ParseUrl(String, String),
    /// Failed to load the image data.
    #[error("failed to load image data: {0}")]
    Load(String),
}