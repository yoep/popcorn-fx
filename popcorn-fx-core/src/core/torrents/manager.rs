use std::fmt::Debug;

use crate::core::torrents::{Torrent, TorrentHandle, TorrentHealth, TorrentInfo};
use crate::core::{torrents, Callbacks, CoreCallback};
use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
use fx_callback::Subscription;
#[cfg(any(test, feature = "testing"))]
pub use mock::*;

/// The callback type for the torrent manager events.
pub type TorrentManagerCallback = CoreCallback<TorrentManagerEvent>;

/// The events of the torrent manager.
#[derive(Debug, Display, Clone)]
pub enum TorrentManagerEvent {
    #[display(fmt = "torrent {} has been added", _0)]
    TorrentAdded(TorrentHandle),
    #[display(fmt = "torrent {} has been removed", _0)]
    TorrentRemoved(TorrentHandle),
}

/// The torrent manager stores the active sessions and torrents that are being processed.
#[async_trait]
pub trait TorrentManager: Debug + DowncastSync + Callbacks<TorrentManagerEvent> {
    /// Retrieve the health of the torrent based on the given magnet link.
    ///
    /// # Arguments
    ///
    /// * `url` - The magnet link of the torrent
    ///
    /// # Returns
    ///
    /// The torrent health on success, or a [torrent::TorrentError] if there was an error.
    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth>;

    /// Create a new idle torrent within the torrent manager.
    async fn create(&self, uri: &str) -> torrents::Result<Box<dyn Torrent>>;

    /// Retrieve the metadata information of the torrent.
    async fn info(&self, handle: &TorrentHandle) -> torrents::Result<TorrentInfo>;

    /// Start the download of the given file within the torrent.
    async fn download(&self, handle: &TorrentHandle, filename: &str) -> torrents::Result<()>;

    /// Get a torrent by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a weak reference to the torrent session if found, or `None` if not found.
    async fn find_by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>>;

    /// Subscribe to the events of the given handle.
    async fn subscribe(
        &self,
        handle: &TorrentHandle,
    ) -> Option<Subscription<torrents::TorrentEvent>>;

    /// Remove a torrent session by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to remove.
    async fn remove(&self, handle: &TorrentHandle);

    /// Calculate the health of the torrent based on the given seed count and peer count.
    ///
    /// # Arguments
    ///
    /// * `seeds` - The number of seeds the torrent has (completed peers).
    /// * `leechers` - The number of leechers the torrent has (incomplete peers).
    ///
    /// # Returns
    ///
    /// Returns the calculated torrent health.
    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth;

    /// Cleanup the torrents directory.
    ///
    /// This operation removes all torrents from the filesystem.
    fn cleanup(&self);
}
impl_downcast!(sync TorrentManager);

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use fx_callback::CallbackHandle;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub TorrentManager {}

        #[async_trait]
        impl TorrentManager for TorrentManager {
            async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth>;
            async fn create(&self, uri: &str) -> torrents::Result<Box<dyn Torrent>>;
            async fn info(&self, handle: &TorrentHandle) -> torrents::Result<TorrentInfo>;
            async fn download(&self, handle: &TorrentHandle, filename: &str) -> torrents::Result<()>;
            async fn find_by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>>;
            async fn subscribe(&self, handle: &TorrentHandle) -> Option<Subscription<torrents::TorrentEvent>>;
            async fn remove(&self, handle: &TorrentHandle);
            fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth;
            fn cleanup(&self);
        }

        impl Callbacks<TorrentManagerEvent> for TorrentManager {
            fn add_callback(&self, callback: TorrentManagerCallback) -> CallbackHandle;
            fn remove_callback(&self, handle: CallbackHandle);
        }
    }
}
