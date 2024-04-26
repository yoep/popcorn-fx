use std::fmt::{Debug, Display};
#[cfg(any(test, feature = "testing"))]
use std::fmt::Formatter;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use tokio_util::sync::CancellationToken;

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
    /// An error has occurred during the loading process.
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
    /// * `event_channel` - A sender channel to communicate loading events.
    /// * `cancel` - A cancellation token that can be checked to determine if the loading process should be canceled.
    ///
    /// # Returns
    ///
    /// A `LoadingResult` indicating the outcome of processing.
    async fn process(
        &self,
        data: LoadingData,
        event_channel: Sender<LoadingEvent>,
        cancel: CancellationToken,
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
