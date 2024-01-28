use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::loader::LoadingState;
use crate::core::playlists::PlaylistItem;

/// A type representing a function that updates the playlist state.
pub type UpdateState = Box<dyn Fn(LoadingState) + Send + Sync>;

/// A trait for defining loading strategies for media items in a playlist.
///
/// Loading strategies are used to process and prepare media items in a playlist before playback.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait LoadingStrategy: Debug + Display + Send + Sync {
    fn on_state_update(&self, state_update: UpdateState);

    /// Process the given `item` and optionally update the playlist state using `state_updater`.
    ///
    /// # Arguments
    ///
    /// * `item` - The `PlaylistItem` to be processed by the loading strategy.
    /// * `state_updater` - A function to update the playlist state if needed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the outcome of processing.
    async fn process(&self, item: PlaylistItem) -> crate::core::loader::LoadingResult;
}

#[cfg(any(test, feature = "testing"))]
impl Display for MockLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockLoadingStrategy")
    }
}