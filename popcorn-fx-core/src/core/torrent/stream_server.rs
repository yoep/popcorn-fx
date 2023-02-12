use std::fmt::Debug;
use std::sync::Arc;

use crate::core::torrent;
use crate::core::torrent::{Torrent, TorrentStream};

/// The torrent stream server allows a [Torrent] to be streamed over the HTTP protocol.
pub trait TorrentStreamServer: Debug + Send + Sync {
    /// Start a new stream for the given [Torrent] info.
    ///
    /// * `torrent` - The torrent info for which a stream should be started.
    ///
    /// It returns a reference to the started stream on success, else the [torrent::TorrentError].
    fn start_stream(&self, torrent: Box<dyn Torrent>) -> torrent::Result<Arc<dyn TorrentStream>>;

    /// Stop the given torrent stream on the server.
    fn stop_stream(&self, stream: &Arc<dyn TorrentStream>);
}