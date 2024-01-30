use std::fmt::{Debug, Display};
#[cfg(any(test, feature = "testing"))]
use std::fmt::Formatter;

use async_trait::async_trait;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::loader::{LoadingData, LoadingProgress, LoadingState};

/// A type representing a function that updates the playlist state.
pub type UpdateState = Box<dyn Fn(LoadingState) + Send + Sync>;

/// A type representing a function that updates the playlist progress.
pub type UpdateProgress = Box<dyn Fn(LoadingProgress) + Send + Sync>;

/// A trait for defining loading strategies for media items in a playlist.
///
/// Loading strategies are used to process and prepare media items in a playlist before playback.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait LoadingStrategy: Debug + Display + Send + Sync {
    /// Sets the state updater function for the loading strategy.
    ///
    /// # Arguments
    ///
    /// * `state_updater` - A function that updates the playlist state when called.
    fn state_updater(&self, state_updater: UpdateState);

    /// Sets the progress updater function for the loading strategy.
    ///
    /// # Arguments
    ///
    /// * `progress_updater` - A function that updates the playlist progress when called.
    fn progress_updater(&self, progress_updater: UpdateProgress);

    /// Process the given `data` and optionally update the playlist state and progress using the
    /// provided updater functions.
    ///
    /// # Arguments
    ///
    /// * `data` - The `LoadingData` to be processed by the loading strategy.
    ///
    /// # Returns
    ///
    /// A `LoadingResult` indicating the outcome of processing.
    async fn process(&self, data: LoadingData) -> crate::core::loader::LoadingResult;
}

#[cfg(any(test, feature = "testing"))]
impl Display for MockLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockLoadingStrategy")
    }
}