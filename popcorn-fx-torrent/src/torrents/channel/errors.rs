use thiserror::Error;

/// The channel error result type
pub type Result<T> = std::result::Result<T, ChannelError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ChannelError {
    /// Indicates that the command channel is no longer available.
    #[error("command channel is not longer available")]
    Closed,
    /// Indicates that a command has timed out after a specified duration in milliseconds.
    #[error("command has timed out after {0} millis")]
    Timeout(u64),
    /// Indicates that a command failed to complete, with an associated error message.
    #[error("failed to complete command, {0}")]
    Failed(String),
    /// Indicates a mismatch in expected command response for the given command instruction.
    #[error("expected command response {0}, but got {1} instead")]
    UnexpectedCommandResponse(String, String),
}
