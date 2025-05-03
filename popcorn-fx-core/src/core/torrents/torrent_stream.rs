use crate::core::torrents;
use crate::core::torrents::Torrent;
use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
use futures::Stream;
use fx_callback::Callback;
use fx_handle::Handle;
use popcorn_fx_torrent::torrent::TorrentStats;
use std::pin::Pin;
use std::task::{Context, Poll};
use url::Url;

/// The unique identifier handle of a stream.
pub type StreamHandle = Handle;

/// The stream bytes that are available to be used for the [TorrentStream].
pub type StreamBytes = Vec<u8>;

/// The streaming result of a read operation on the [TorrentStream] resource.
pub type StreamBytesResult = Result<StreamBytes, torrents::Error>;

/// The state of the [TorrentStream].
#[derive(Debug, Copy, Clone, Display, PartialEq)]
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
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TorrentStreamEvent {
    /// The new state of the torrent stream.
    ///
    /// # Arguments
    ///
    /// * `StateChanged` - The new state of the torrent stream.
    #[display(fmt = "Torrent stream state changed to {}", _0)]
    StateChanged(TorrentStreamState),
    /// Download status update for the torrent stream.
    ///
    /// # Arguments
    ///
    /// * `DownloadStatus` - The download status of the torrent stream.
    #[display(fmt = "Torrent stream download status changed to {}", _0)]
    DownloadStatus(TorrentStats),
}

/// A trait for a torrent stream that provides access to torrent streaming information.
///
/// This trait defines methods for retrieving stream details, streaming torrent content,
/// and managing the stream state.
#[async_trait]
pub trait TorrentStream: Torrent + Callback<TorrentStreamEvent> + DowncastSync {
    /// Get the endpoint URL where the stream is available.
    ///
    /// Returns an owned instance of the URL.
    fn url(&self) -> Url;

    /// Stream the torrent contents as a byte array.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper],
    /// as most streaming servers require the [Stream] to have a known size.
    ///
    /// Returns the stream of the torrent bytes or the [torrents::Error] that occurred.
    async fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// Stream the torrent contents as a byte array with the given offset and length.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper],
    /// as most streaming servers require the [Stream] to have a known size.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset within the torrent to start streaming from.
    /// * `len` - The length of the content to stream (optional).
    ///
    /// Returns the stream of the torrent bytes or the [torrents::Error] that occurred.
    async fn stream_offset(
        &self,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// Get the current state of the stream.
    async fn stream_state(&self) -> TorrentStreamState;

    /// Stop the stream, preventing new streaming resources from being created,
    /// and stopping the underlying [Torrent] process.
    fn stop_stream(&self);
}
impl_downcast!(sync TorrentStream);

/// The streaming resource of a [TorrentStream].
/// It allows a [Torrent] to be streamed over HTTP.
pub trait TorrentStreamingResource: Stream<Item = StreamBytesResult> + Send {
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
    inner: Pin<Box<dyn TorrentStreamingResource<Item = StreamBytesResult>>>,
}

impl TorrentStreamingResourceWrapper {
    pub fn new<T>(stream: T) -> Self
    where
        T: TorrentStreamingResource<Item = StreamBytesResult> + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Retrieve the wrapped [TorrentStreamingResource] resource.
    pub fn resource(&self) -> &Pin<Box<dyn TorrentStreamingResource<Item = StreamBytesResult>>> {
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
