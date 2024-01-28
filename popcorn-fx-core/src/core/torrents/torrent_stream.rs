use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

use derive_more::Display;
use futures::Stream;
use url::Url;

use crate::core::{CoreCallback, torrents};
use crate::core::torrents::Torrent;

/// The stream bytes that are available to be used for the [TorrentStream].
pub type StreamBytes = Vec<u8>;

/// The streaming result of a read operation on the [TorrentStream] resource.
pub type StreamBytesResult = Result<StreamBytes, torrents::TorrentError>;

/// The callback type for all torrent stream events.
pub type TorrentStreamCallback = CoreCallback<TorrentStreamEvent>;

/// The state of the [TorrentStream].
#[repr(i32)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum TorrentStreamState {
    /// The initial state of the torrent stream.
    /// This state indicates that the stream is preparing the initial pieces.
    Preparing = 0,
    /// The torrent can be streamed over HTTP.
    Streaming = 1,
    /// The torrent has been stopped and can not longer be streamed.
    Stopped = 2,
}

/// The torrent stream event which occurred for the [TorrentStream].
#[derive(Debug, Clone)]
pub enum TorrentStreamEvent {
    /// The new state of the torrent stream
    StateChanged(TorrentStreamState)
}

impl Display for TorrentStreamEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentStreamEvent::StateChanged(state) => write!(f, "Torrent stream state changed to {}", state),
        }
    }
}

/// The torrent stream contains the information of a [Torrent] that is being streamed
/// over the [TorrentStreamServer].
///
/// Use [TorrentStream::stream] or [TorrentStream::stream_offset] to retrieve a [Stream] resource.
/// Once a [TorrentStreamingResource] is started, the offset and length can't be changed anymore.
/// If you require another range, please create a new stream and drop the previous one.
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
    fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// Stream the torrent contents as a byte array with the given offset.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper] as most streaming servers
    /// require the [Stream] to have a known size.
    ///
    /// It returns the stream of the torrent bytes, else the [torrent::TorrentError] that occurred.
    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// The current state of the stream.
    fn stream_state(&self) -> TorrentStreamState;

    /// Register a new callback for the stream events.
    fn register_stream(&self, callback: TorrentStreamCallback);

    /// Stop the stream which will prevent new streaming resources to be created.
    /// It will also stop the underlying [Torrent] process.
    fn stop_stream(&self);
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