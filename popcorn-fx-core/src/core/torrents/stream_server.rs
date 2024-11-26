use std::fmt::Debug;
use std::sync::Weak;

use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::torrents::{Torrent, TorrentStream, TorrentStreamCallback};
use crate::core::{torrents, CallbackHandle, Handle};

/// The state of the torrent stream server.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum TorrentStreamServerState {
    Stopped,
    Running,
    Error,
}

/// A trait for a torrent stream server that allows streaming torrents over HTTP.
///
/// This trait defines methods for managing the state of the torrent stream server and starting/stopping torrent streams.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait TorrentStreamServer: Debug + DowncastSync {
    /// Get the current state of the torrent stream server.
    ///
    /// # Returns
    ///
    /// The current state of the torrent stream server.
    fn state(&self) -> TorrentStreamServerState;

    /// Start streaming a torrent.
    ///
    /// # Arguments
    ///
    /// * `torrent` - A boxed trait object implementing `Torrent` to be streamed.
    ///
    /// # Returns
    ///
    /// A result containing a weak reference to the started torrent stream, or an error if the stream could not be started.
    fn start_stream(
        &self,
        torrent: Box<dyn Torrent>,
    ) -> torrents::Result<Weak<Box<dyn TorrentStream>>>;

    /// Stop a torrent stream.
    ///
    /// # Arguments
    ///
    /// * `handle` - An identifier for the torrent stream to stop.
    fn stop_stream(&self, handle: Handle);

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
    fn subscribe(&self, handle: Handle, callback: TorrentStreamCallback) -> Option<CallbackHandle>;

    /// Unsubscribe from events of a torrent stream.
    ///
    /// # Arguments
    ///
    /// * `handle` - An identifier for the torrent stream to unsubscribe from.
    /// * `callback_handle` - The handle returned from the `subscribe` method.
    ///
    /// # Remarks
    ///
    /// This method allows unsubscribing from events of a torrent stream previously subscribed to
    /// using the `subscribe` method. The `callback_handle` must match the handle returned when
    /// subscribing to the event stream.
    fn unsubscribe(&self, handle: Handle, callback_handle: CallbackHandle);
}
impl_downcast!(sync TorrentStreamServer);
