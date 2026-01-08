use crate::core::torrents;
use crate::core::torrents::{Torrent, TorrentStream, TorrentStreamEvent};
use async_trait::async_trait;
use downcast_rs::{impl_downcast, DowncastSync};
use fx_callback::Subscription;
use fx_handle::Handle;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use std::fmt::Debug;

/// A trait for a torrent stream server that allows streaming torrents over HTTP.
///
/// This trait defines methods for managing the state of the torrent stream server and starting/stopping torrent streams.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait TorrentStreamServer: Debug + DowncastSync {
    /// Start streaming a torrent.
    ///
    /// # Arguments
    ///
    /// * `torrent` - A boxed trait object implementing `Torrent` to be streamed.
    /// * `filename` - The filename within the torrent to start streaming.
    ///
    /// # Returns
    ///
    /// A result containing a weak reference to the started torrent stream, or an error if the stream could not be started.
    async fn start_stream(
        &self,
        torrent: Box<dyn Torrent>,
        filename: &str,
    ) -> torrents::Result<Box<dyn TorrentStream>>;

    /// Stop a torrent stream.
    ///
    /// # Arguments
    ///
    /// * `handle` - An identifier for the torrent stream to stop.
    async fn stop_stream(&self, handle: Handle);

    /// Subscribe to events from a torrent stream.
    ///
    /// # Arguments
    ///
    /// * `handle` - An identifier for the torrent stream to subscribe to.
    /// * `callback` - A closure that will be called when events occur on the torrent stream.
    ///
    /// # Returns
    ///
    /// An optional callback handle that can be used to unsubscribe from the event stream.
    ///
    /// # Remarks
    ///
    /// This method allows subscribing to events from the specified torrent stream using a callback function.
    /// The callback function will be called whenever events occur on the torrent stream.
    /// It returns an optional callback handle that can be used to unsubscribe from the event stream later.
    async fn subscribe(&self, handle: Handle) -> Option<Subscription<TorrentStreamEvent>>;
}
impl_downcast!(sync TorrentStreamServer);
