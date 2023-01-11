use std::fmt::{Display, Formatter};

/// The result type for storage actions.
pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(PartialEq, Debug)]
pub enum StorageError {
    FileNotFound(String),
    CorruptData(String, String),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::FileNotFound(filename) => write!(f, "filename {} not found", filename),
            StorageError::CorruptData(filename, error) => write!(f, "filename {} is corrupt and cannot be read, {}", filename, error),
        }
    }
}