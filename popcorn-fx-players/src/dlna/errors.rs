use thiserror::Error;

use crate::DiscoveryState;

/// Errors that can occur during DLNA operations.
#[derive(Debug, Clone, Error)]
pub enum DlnaError {
    /// Indicates a failure to discover devices.
    #[error("failed to discover devices: {0}")]
    Discovery(String),
    /// Indicates an invalid server state.
    #[error("server state is invalid: {0}")]
    InvalidState(DiscoveryState),
    /// Indicates an invalid transport state for UPnP.
    #[error("transport state is invalid: {0}")]
    InvalidTransportState(String),
    /// Indicates an error related to a specific device.
    #[error("a device error occurred: {0}")]
    Device(String),
    /// Indicates an invalid device URI.
    #[error("invalid device URI: {0}")]
    Uri(String),
    /// Indicates command for the device service failed.
    #[error("failed to execute service command")]
    ServiceCommand,
}

/// Result type for DLNA operations.
pub type Result<T> = std::result::Result<T, DlnaError>;
