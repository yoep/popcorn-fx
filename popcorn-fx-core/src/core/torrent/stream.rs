use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use url::Url;

use crate::core::torrent;
use crate::core::torrent::Torrent;

/// The stream bytes that are available to be used for the [TorrentStream].
pub type StreamBytes = Vec<u8>;

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
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper] as most streaming servers
    /// require the [Stream] to have a known size.
    ///
    /// It returns the stream of the torrent bytes, else the [torrent::TorrentError] that occurred.
    fn stream(&self) -> torrent::Result<TorrentStreamingResourceWrapper>;

    /// Stream the torrent contents as a byte array with the given offset.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper] as most streaming servers
    /// require the [Stream] to have a known size.
    ///
    /// It returns the stream of the torrent bytes, else the [torrent::TorrentError] that occurred.
    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrent::Result<TorrentStreamingResourceWrapper>;
}

/// The streaming resource of a [TorrentStream].
/// It allows a [Torrent] to be streamed over HTTP.
pub trait TorrentStreamingResource: Stream<Item=StreamBytesResult> + Send + 'static {
    /// The starting offset of the stream in regards to the resource bytes.
    /// This will be the initial seek offset within the resource bytes and is 0 index based.
    fn offset(&self) -> u64;

    /// The total length of the stream resource.
    /// This length will not be provided by the [TorrentStream] if a range
    /// has been configured.
    ///
    /// It returns the total length of the resource.
    fn total_length(&self) -> u64;

    /// The content length the stream will provide of the resource.
    fn content_length(&self) -> u64;

    /// The HTTP content range that will be provided by this stream.
    fn content_range(&self) -> String;
}

/// Wrapper around a dyn [Stream] which allows for a sized return value.
pub struct TorrentStreamingResourceWrapper {
    inner: Pin<Box<dyn TorrentStreamingResource<Item=StreamBytesResult>>>,
}

impl TorrentStreamingResourceWrapper {
    pub fn new<T>(stream: T) -> Self
        where T: TorrentStreamingResource<Item=StreamBytesResult> {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Retrieve the wrapped [TorrentStreamingResource] resource.
    pub fn resource(&self) -> &Pin<Box<dyn TorrentStreamingResource<Item=StreamBytesResult>>> {
        &self.inner
    }
}

impl Stream for TorrentStreamingResourceWrapper {
    type Item = StreamBytesResult;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}