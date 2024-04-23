extern crate core;

use std::fmt::Display;

use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;

#[cfg(feature = "chromecast")]
pub mod chromecast;
#[cfg(feature = "dlna")]
pub mod dlna;
#[cfg(feature = "vlc")]
pub mod vlc;

/// Errors that can occur during the discovery process.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DiscoveryError {
    /// Indicates that the discovery service is in an invalid state.
    #[error("Discovery service is in invalid state: {0}")]
    InvalidState(DiscoveryState),
    /// Indicates a failure to initialize the discovery service.
    #[error("Failed to initialize discovery service: {0}")]
    Initialization(String),
    /// Indicates a failure to terminate the discovery service.
    #[error("Failed to terminate discovery service: {0}")]
    Terminate(String),
}

/// A specialized `Result` type for discovery operations.
pub type Result<T> = std::result::Result<T, DiscoveryError>;

/// Represents the states of a discovery process.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum DiscoveryState {
    /// Indicates that the discovery process is running.
    #[display(fmt = "Running")]
    Running,
    /// Indicates that the discovery process is stopped.
    #[display(fmt = "Stopped")]
    Stopped,
    /// Indicates that an error occurred during the discovery process.
    #[display(fmt = "Error")]
    Error,
}

/// This trait defines a generic interface for discovering media players.
#[async_trait]
pub trait Discovery: Display + Send + Sync {
    /// Returns the current state of the discovery process.
    fn state(&self) -> DiscoveryState;
    
    /// Starts the discovery process.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the discovery process started successfully, otherwise an error indicating the reason.
    async fn start_discovery(&self) -> Result<()>;

    /// Stops the discovery process.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the discovery process stopped successfully, otherwise an error indicating the reason.
    fn stop_discovery(&self) -> Result<()>;
}
