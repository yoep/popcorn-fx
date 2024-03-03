use thiserror::Error;

/// Represents errors that can occur during configuration.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConfigError {
    /// Indicates an invalid value for a configuration property.
    #[error("invalid value {0} given for {1}")]
    InvalidValue(String, String),
    /// Indicates that a provider with the given name is unknown.
    #[error("provider with name \"{0}\" is unknown")]
    UnknownProvider(String),
    /// Indicates that a tracking provider with the given name is unknown.
    #[error("tracking provider with name \"{0}\" is unknown")]
    UnknownTrackingProvider(String),
}