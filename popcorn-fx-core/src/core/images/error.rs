use thiserror::Error;

/// The `ImageError` enum represents the possible errors that can occur during image operations.
#[derive(Debug, Clone, Error)]
pub enum ImageError {
    /// Failed to parse the image URL.
    #[error("Failed to parse image URL: {0}")]
    ParseUrl(String),
    /// Failed to load the image data.
    #[error("Failed to load image data: {0}")]
    Load(String),
}