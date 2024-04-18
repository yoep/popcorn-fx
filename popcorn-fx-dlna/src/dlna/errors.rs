use thiserror::Error;

use crate::dlna::DlnaServerState;

/// Errors that can occur during DLNA operations.
#[derive(Debug, Clone, Error)]
pub enum DlnaError {
    /// Indicates a failure to discover devices.
    #[error("Failed to discover devices: {0}")]
    Discovery(String),
    /// Indicates an invalid server state.
    #[error("Server state is invalid: {0}")]
    InvalidState(DlnaServerState),
    /// Indicates an invalid transport state for UPnP.
    #[error("Transport state is invalid: {0}")]
    InvalidTransportState(String),
    /// Indicates an error related to a specific device.
    #[error("A device error occurred: {0}")]
    Device(String),
    /// Indicates an invalid device URI.
    #[error("Invalid device URI: {0}")]
    Uri(String),
    /// Indicates command for the device service failed.
    #[error("Failed to execute service command")]
    ServiceCommand,
}

/// Result type for DLNA operations.
pub type Result<T> = std::result::Result<T, DlnaError>;