use std::error::Error;

use thiserror::Error;

/// The result type used in cache operations, containing the successful value or a `CacheError` on failures.
pub type Result<T> = std::result::Result<T, CacheError>;

/// An error that occurred during the execution of a cache operation.
#[derive(Debug, Clone, Error)]
pub enum CacheExecutionError<T>
    where T: Error {
    /// An error occurred while executing the operation.
    #[error("an error occurred while executing the operation: {0}")]
    Operation(T),
    /// An error occurred while mapping the binary data.
    #[error("an error occurred while mapping the data: {0}")]
    Mapping(T),
    /// An error occurred while handling the cache data.
    #[error("an error occurred while handling the cache data: {0}")]
    Cache(CacheError),
}

/// An error related to cache handling.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CacheError {
    #[error("cache data location {0} not found")]
    NotFound(String),
    #[error("failed to parse cache data, {0}")]
    Parsing(String),
    /// An IO error occurred while handling cache data.
    #[error("an IO error occurred while handling cache data: {0}")]
    Io(String),
}

#[derive(Debug, Clone, Error)]
pub enum CacheParserError<T>
    where T: Error {
    #[error("an error occurred while executing the operation: {0}")]
    Operation(T),
    #[error("an error occurred while parsing the cache data, {0}")]
    Parsing(String),
}

