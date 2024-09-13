use thiserror::Error;

/// The extension specific result type
pub type Result<T> = std::result::Result<T, ExtensionError>;

/// The errors which may occur within extensions
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ExtensionError {
    #[error("failed to parse extension payload, {0}")]
    Parsing(String),
    #[error("the extension is not supported")]
    Unsupported,
}
