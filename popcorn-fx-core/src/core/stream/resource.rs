use crate::core::stream::Result;
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::Callback;
use fx_handle::Handle;
use std::fmt::Debug;

/// The unique identifier handle of a stream.
pub type StreamHandle = Handle;

/// The stream bytes available to be used for the [StreamingResource].
pub type StreamBytes = Vec<u8>;

/// The streaming result of a read operation on the [StreamingResource].
pub type StreamBytesResult = Result<StreamBytes>;

/// The state of a [Stream].
#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum StreamState {
    /// The initial state of the stream.
    /// This state indicates that the stream is preparing.
    Preparing = 0,
    /// The resource can be streamed over HTTP.
    Streaming = 1,
    /// The resource has been stopped and can no longer be streamed.
    Stopped = 2,
}

/// The statistics of the stream.
#[derive(Debug, Copy, Clone)]
pub struct StreamStats {
    pub progress: f32,
    pub connections: usize,
    pub download_speed: u32,
    pub upload_speed: u32,
    pub downloaded: usize,
    pub total_size: usize,
}

/// The events that can be emitted by a [StreamingResource].
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Invoked when the state of the stream has changed.
    StateChanged(StreamState),
    /// Invoked when the statistics of the stream have changed.
    StatsChanged(StreamStats),
}

impl PartialEq for StreamEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StateChanged(_), Self::StateChanged(_)) => true,
            (Self::StatsChanged(_), Self::StatsChanged(_)) => true,
            _ => false,
        }
    }
}

/// Represents a resource that can be streamed as a video format.
#[async_trait]
pub trait StreamingResource: Debug + Callback<StreamEvent> + Send + Sync {
    /// Returns the filename of the resource.
    fn filename(&self) -> &str;

    /// Try to create a new stream for this streaming resource.
    /// It returns a new [Stream] resource if available, else the error.
    async fn stream(&self) -> Result<Box<dyn Stream>>;

    /// Try to create a new stream for this streaming resource for the specified byte range.
    /// It returns a new [Stream] resource if available, else the error.
    async fn stream_range(&self, start: u64, end: Option<u64>) -> Result<Box<dyn Stream>>;

    /// Returns the state of the streaming resource.
    async fn state(&self) -> StreamState;

    /// Stop the streaming resource.
    /// This should release any resources held by the streaming resource.
    async fn stop(&self);
}

/// The range of bytes that will be streamed from the resource.
pub type StreamRange = std::ops::Range<usize>;

/// The stream created by a [StreamingResource].
/// It extends the [futures::Stream] trait to provide additional information about the stream.
pub trait Stream: Debug + futures::Stream<Item = StreamBytesResult> + Send {
    /// Returns the range of bytes that will be streamed from the resource.
    fn range(&self) -> StreamRange;

    /// Returns the total number of bytes in the resource.
    ///
    /// This is different from the total number of bytes that will be returned by the stream.
    /// The stream length is determined by the [Stream::range].
    fn resource_len(&self) -> u64;

    /// The HTTP content range that will be provided by this stream.
    fn content_range(&self) -> String;
}
