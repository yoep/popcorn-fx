use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use derive_more::Display;
use futures::Stream;
use mockall::mock;
use url::Url;

use crate::core::{CoreCallback, torrents};
use crate::core::torrents::{Torrent, TorrentCallback, TorrentState};

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

/// A trait for a torrent stream that provides access to torrent streaming information.
///
/// This trait defines methods for retrieving stream details, streaming torrent content,
/// and managing the stream state.
pub trait TorrentStream: Torrent {
    /// Get the stream handle of this stream.
    ///
    /// Returns the stream handle of this stream.
    fn stream_handle(&self) -> i64;

    /// Get the endpoint URL where the stream is available.
    ///
    /// Returns an owned instance of the URL.
    fn url(&self) -> Url;

    /// Stream the torrent contents as a byte array.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper],
    /// as most streaming servers require the [Stream] to have a known size.
    ///
    /// Returns the stream of the torrent bytes or the [torrents::TorrentError] that occurred.
    fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// Stream the torrent contents as a byte array with the given offset and length.
    /// The actual [Stream] implementation is wrapped in the [TorrentStreamingResourceWrapper],
    /// as most streaming servers require the [Stream] to have a known size.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset within the torrent to start streaming from.
    /// * `len` - The length of the content to stream (optional).
    ///
    /// Returns the stream of the torrent bytes or the [torrents::TorrentError] that occurred.
    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrents::Result<TorrentStreamingResourceWrapper>;

    /// Get the current state of the stream.
    fn stream_state(&self) -> TorrentStreamState;

    /// Subscribe to stream events with the provided callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - A callback function to handle stream events.
    fn subscribe_stream(&self, callback: TorrentStreamCallback) -> i64;

    /// Unsubscribe from stream events with the provided callback ID.
    ///
    /// # Arguments
    ///
    /// * `callback_id` - The unique identifier of the callback to unsubscribe.
    fn unsubscribe_stream(&self, callback_id: i64);

    /// Stop the stream, preventing new streaming resources from being created,
    /// and stopping the underlying [Torrent] process.
    fn stop_stream(&self);
}

mock! {
    #[derive(Debug)]
    pub TorrentStream {}

    impl Torrent for TorrentStream {
        fn handle(&self) -> &str;

        fn file(&self) -> PathBuf;

        fn has_bytes(&self, bytes: &[u64]) -> bool;

        fn has_piece(&self, piece: u32) -> bool;

        fn prioritize_bytes(&self, bytes: &[u64]);

        fn prioritize_pieces(&self, pieces: &[u32]);

        fn total_pieces(&self) -> i32;

        fn sequential_mode(&self);

        fn state(&self) -> TorrentState;

        fn subscribe(&self, callback: TorrentCallback) -> i64;
    }

    impl TorrentStream for TorrentStream {
        fn stream_handle(&self) -> i64;

        fn url(&self) -> Url;

        fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper>;

        fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrents::Result<TorrentStreamingResourceWrapper>;

        fn stream_state(&self) -> TorrentStreamState;

        fn subscribe_stream(&self, callback: TorrentStreamCallback) -> i64;

        fn unsubscribe_stream(&self, callback_id: i64);

        fn stop_stream(&self);
    }
}

impl Display for MockTorrentStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTorrentStream")
    }
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