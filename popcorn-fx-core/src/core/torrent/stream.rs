use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use url::Url;

use crate::core::torrent;
use crate::core::torrent::Torrent;

/// The stream bytes that are available to be used for the [TorrentStream].
pub type StreamBytes = bytes::Bytes;

/// The streaming result of a read operation on the [TorrentStream] resource.
pub type StreamBytesResult = Result<StreamBytes, torrent::TorrentError>;

/// The torrent stream contains the information of a [Torrent] that is being streamed
/// over the [TorrentStreamServer].
pub trait TorrentStream: Torrent {
    /// Retrieve the endpoint url on which the stream is available.
    ///
    /// It returns an owned instance of the url.
    fn url(&self) -> Url;

    /// Stream the torrent contents as a byte array.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResource] as most streaming servers
    /// require the [Stream] to have a known size.
    ///
    /// It returns the stream of the torrent bytes, else the [torrent::TorrentError] that occurred.
    fn stream(&self) -> torrent::Result<TorrentStreamingResource>;
}

/// Wrapper around a dyn [Stream] which allows for a sized returns value.
pub struct TorrentStreamingResource {
    inner: Pin<Box<dyn Stream<Item=StreamBytesResult> + Send + 'static>>,
}

impl TorrentStreamingResource {
    pub fn new<T>(stream: T) -> Self
        where T: Stream<Item=StreamBytesResult> + Send + 'static {
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for TorrentStreamingResource {
    type Item = StreamBytesResult;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}