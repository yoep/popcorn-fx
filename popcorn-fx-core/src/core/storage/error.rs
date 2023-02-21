use thiserror::Error;

/// The result type for storage actions.
pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(PartialEq, Debug, Error)]
pub enum StorageError {
    /// The given `filename` couldn't be found within the storage.
    #[error("filename {0} not found")]
    FileNotFound(String),
    /// The given `filename` contains invalid data, it returned `error_message` while reading/parsing.
    #[error("filename {0} is corrupt and cannot be read, {1}")]
    ReadingFailed(String, String),
    /// The given `file_path` couldn't be written, it returned `error_message` while writing.
    #[error("failed to write to {0}, {1}")]
    WritingFailed(String, String),
}