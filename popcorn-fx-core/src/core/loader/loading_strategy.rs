#[cfg(any(test, feature = "testing"))]
use std::fmt::Formatter;
use std::fmt::{Debug, Display};

use async_trait::async_trait;
use derive_more::Display;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{LoadingData, LoadingError, LoadingProgress, LoadingState};

/// An event representing a change in the loading process.
///
/// Loading events can be used to communicate changes in the loading state or progress of media items in a playlist. This enum defines different types of loading events that can occur during the loading process.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum LoadingEvent {
    /// The loading state of a media item has changed.
    #[display(fmt = "Loading state changed to {:?}", _0)]
    StateChanged(LoadingState),
    /// The loading progress of a media item has changed.
    #[display(fmt = "Loading progress changed to {:?}", _0)]
    ProgressChanged(LoadingProgress),
    /// Indicates that the loading task has been cancelled.
    #[display(fmt = "Loading cancelled")]
    Cancelled,
    /// Indicates that the loading task has finished/completed.
    #[display(fmt = "Loading finished")]
    Completed,
    /// Indicates that the loading task has encountered an error.
    #[display(fmt = "Loading failed, {:?}", _0)]
    LoadingError(LoadingError),
}

/// A trait for defining loading strategies for media items in a playlist.
///
/// Loading strategies are used to process and prepare media items in a playlist before playback. These strategies can produce loading events and support cancellation of the loading process.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait LoadingStrategy: Debug + Display + Send + Sync {
    /// Processes the given `data` and communicates loading events through the provided event channel.
    ///
    /// # Arguments
    ///
    /// * `data` - The `LoadingData` to be processed by the loading strategy.
    /// * `context` - The context of the loading task.
    ///
    /// # Returns
    ///
    /// A `LoadingResult` indicating the outcome of processing.
    async fn process(
        &self,
        data: &mut LoadingData,
        context: &LoadingTaskContext,
    ) -> crate::core::loader::LoadingResult;

    /// Cancels the loading process associated with the given `data`.
    ///
    /// # Arguments
    ///
    /// * `data` - The `LoadingData` to be canceled.
    ///
    /// # Returns
    ///
    /// A `CancellationResult` indicating the outcome of the cancellation operation.
    async fn cancel(&self, data: LoadingData) -> crate::core::loader::CancellationResult;
}

#[cfg(any(test, feature = "testing"))]
impl Display for MockLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockLoadingStrategy")
    }
}
