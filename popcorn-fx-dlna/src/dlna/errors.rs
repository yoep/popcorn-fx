use thiserror::Error;

use crate::dlna::DlnaServerState;

#[derive(Debug, Clone, Error)]
pub enum DlnaError {
    #[error("Failed to discover devices: {0}")]
    Discovery(String),
    #[error("Server state is invalid: {0}")]
    InvalidState(DlnaServerState),
    #[error("A device error occurred: {0}")]
    Device(String),
    #[error("Invalid device uri: {0}")]
    Uri(String),
}

pub type Result<T> = std::result::Result<T, DlnaError>;