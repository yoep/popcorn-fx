use thiserror::Error;

/// The result type of event operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The errors that can occur during event operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("the event publisher has been closed")]
    Closed,
}
