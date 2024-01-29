use std::fmt::Debug;
use std::sync::Weak;

use derive_more::Display;

use crate::core::torrents;
use crate::core::torrents::{Torrent, TorrentStream};

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
pub trait TorrentStreamServer: Debug + Send + Sync {
    /// Get the current state of the torrent stream server.
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
    fn start_stream(&self, torrent: Weak<Box<dyn Torrent>>) -> torrents::Result<Weak<dyn TorrentStream>>;

    /// Stop a torrent stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A weak reference to the torrent stream to be stopped.
    fn stop_stream(&self, stream: Weak<dyn TorrentStream>);
}