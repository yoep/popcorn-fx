use std::fmt::Debug;
use std::sync::Arc;

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

/// The torrent stream server allows a [Torrent] to be streamed over the HTTP protocol.
pub trait TorrentStreamServer: Debug + Send + Sync {
    /// The state of the torrent stream server.
    fn state(&self) -> TorrentStreamServerState;

    /// Start a new stream for the given [Torrent] info.
    ///
    /// * `torrent` - The torrent info for which a stream should be started.
    ///
    /// It returns a reference to the started stream on success, else the [torrent::TorrentError].
    fn start_stream(&self, torrent: Box<dyn Torrent>) -> torrents::Result<Arc<dyn TorrentStream>>;

    /// Stop the given torrent stream on the server.
    fn stop_stream(&self, stream: &Arc<dyn TorrentStream>);
}