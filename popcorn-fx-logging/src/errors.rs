use std::result;
use thiserror::Error;

/// The result type of logging operations.
pub type Result<T> = result::Result<T, Error>;

/// The errors of the logging crate.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("a logger instance has already been initialized")]
    AlreadyInitialized,
    #[error("path does not exist")]
    NotFound,
    #[error("configuration file is invalid, {0}")]
    InvalidConfig(String),
}