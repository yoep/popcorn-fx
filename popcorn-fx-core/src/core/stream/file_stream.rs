use crate::core::stream;
use crate::core::stream::{
    Error, Stream, StreamBytesResult, StreamEvent, StreamRange, StreamState, StreamingResource,
};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, trace};
use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

const DEFAULT_BUFFER_SIZE: usize = 256 * 1000; // 256KB

#[derive(Debug)]
pub struct FileStreamingResource {
    inner: Arc<InnerFileStreamingResource>,
}

impl FileStreamingResource {
    pub fn new<P: AsRef<Path>>(filepath: P) -> stream::Result<Self> {
        let filepath = filepath.as_ref().to_path_buf();
        let filename = filepath
            .file_name()
            .map(|e| e.to_string_lossy().to_string())
            .ok_or(stream::Error::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "filepath is invalid",
            )))?;

        Ok(Self {
            inner: Arc::new(InnerFileStreamingResource {
                filename,
                filepath,
                state: Mutex::new(StreamState::Streaming),
                callbacks: MultiThreadedCallback::new(),
            }),
        })
    }
}

#[async_trait]
impl StreamingResource for FileStreamingResource {
    fn filename(&self) -> &str {
        self.inner.filename.as_str()
    }

    async fn stream(&self) -> stream::Result<Box<dyn Stream>> {
        self.stream_range(0, None).await
    }

    async fn stream_range(&self, start: u64, end: Option<u64>) -> stream::Result<Box<dyn Stream>> {
        self.inner.assert_state().await?;
        let stream = FileStream::new(&self.inner.filepath, start, end)?;
        Ok(Box::new(stream))
    }

    async fn state(&self) -> StreamState {
        *self.inner.state.lock().await
    }

    async fn stop(&self) {
        self.inner.update_state(StreamState::Stopped).await;
    }
}

impl Callback<StreamEvent> for FileStreamingResource {
    fn subscribe(&self) -> Subscription<StreamEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<StreamEvent>) {
        self.inner.callbacks.subscribe_with(subscriber);
    }
}

#[derive(Debug)]
struct InnerFileStreamingResource {
    filename: String,
    filepath: PathBuf,
    state: Mutex<StreamState>,
    callbacks: MultiThreadedCallback<StreamEvent>,
}

impl InnerFileStreamingResource {
    /// Asserts that the current state is streaming, returning an error if not
    async fn assert_state(&self) -> stream::Result<()> {
        if *self.state.lock().await != StreamState::Streaming {
            return Err(stream::Error::InvalidState);
        }
        Ok(())
    }

    /// Updates the state of the resource.
    /// This informs subscribers of the state change.
    async fn update_state(&self, new_state: StreamState) {
        let mut state = self.state.lock().await;
        if *state == new_state {
            return;
        }

        *state = new_state;
        self.callbacks.invoke(StreamEvent::StateChanged(new_state));
    }
}

#[derive(Debug, Display)]
#[display("{:?}", filepath)]
pub struct FileStream {
    file: File,
    filepath: PathBuf,
    /// The total length of the file resource
    resource_length: u64,
    /// The cursor position within the stream range
    cursor: usize,
    /// The range of bytes that will be streamed
    stream_range: StreamRange,
}

impl FileStream {
    fn new<P: AsRef<Path>>(filepath: P, start: u64, end: Option<u64>) -> stream::Result<Self> {
        let filepath = filepath.as_ref();
        let file = OpenOptions::new().read(true).open(filepath)?;
        let resource_length = Self::resource_len(&file).ok_or(Error::Io(io::Error::new(
            io::ErrorKind::Unsupported,
            "failed to get file length",
        )))? as u64;
        let stream_start = start as usize;
        let stream_end = min(end.unwrap_or(resource_length), resource_length) as usize;

        Ok(Self {
            file,
            filepath: filepath.to_path_buf(),
            resource_length,
            stream_range: stream_start..stream_end,
            cursor: 0,
        })
    }

    /// Returns the next buffer range for the stream.
    fn next_buffer(&self) -> Range<usize> {
        let buffer_start = self.stream_range.start + self.cursor;
        let buffer_end = min(buffer_start + DEFAULT_BUFFER_SIZE, self.stream_range.end);
        buffer_start..buffer_end
    }

    /// Reads the next buffer from the file and returns the bytes read.
    fn read_data(&mut self) -> stream::Result<Vec<u8>> {
        let buffer = self.next_buffer();
        let mut bytes = vec![0u8; buffer.len()];
        if let Err(e) = self.file.seek(io::SeekFrom::Start(buffer.start as u64)) {
            return Err(Error::Io(e));
        }

        let len = self.file.read(&mut bytes)?;
        if len == 0 {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "stream reached unexpected EOF",
            )));
        }

        self.cursor += len;
        bytes.truncate(len);
        Ok(bytes)
    }

    /// Returns the length of the resource, if known.
    fn resource_len(file: &File) -> Option<usize> {
        file.metadata().map(|e| e.len() as usize).ok()
    }
}

impl Stream for FileStream {
    fn range(&self) -> StreamRange {
        self.stream_range.clone()
    }

    fn resource_len(&self) -> u64 {
        self.resource_length
    }

    fn content_range(&self) -> String {
        let range = format!(
            "bytes {}-{}/{}",
            self.stream_range.start,
            self.stream_range.end.saturating_sub(1),
            self.resource_len()
        );

        trace!("File stream {} has content range {{{}}}", self, &range);
        range
    }
}

impl futures::Stream for FileStream {
    type Item = StreamBytesResult;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // check if the current cursor position is out-of-bounds,
        // if so, return additional bytes
        if self.cursor >= self.stream_range.end {
            return Poll::Ready(None);
        }

        Poll::Ready(Some(self.as_mut().read_data().map_err(|e| {
            debug!("File stream failed to read data, {}", e);
            e
        })))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Self::resource_len(&self.file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use crate::testing::copy_test_file;
    use std::time::Duration;
    use tempfile::{tempdir, TempDir};

    mod filename {
        use super::*;

        #[test]
        fn test_filename() {
            init_logger!();
            let filename = "large-[123].txt";
            let (resource, _temp_dir) = create_new_resource(filename);

            let result = resource.filename();

            assert_eq!(result, filename);
        }
    }

    mod stream {
        use super::*;

        #[tokio::test]
        async fn test_stream_range() {
            init_logger!();
            let (resource, _temp_dir) = create_new_resource("large-[123].txt");

            // create a stream within the resource bounds
            let stream = resource
                .stream_range(1000, Some(2800))
                .await
                .expect("expected a stream");
            let result = stream.range();
            assert_eq!(1000..2800, result);

            // create a stream exceeding the resource bounds
            let stream = resource
                .stream_range(3000, Some(38000))
                .await
                .expect("expected a stream");
            let len = stream.resource_len();
            let result = stream.range();
            assert_eq!(3000..len as usize, result);
        }
    }

    mod stop {
        use super::*;
        use crate::recv_timeout;

        #[tokio::test]
        async fn test_stop() {
            init_logger!();
            let (resource, _temp_dir) = create_new_resource("large-[123].txt");

            // subscribe to the stream resource
            let mut receiver = resource.subscribe();

            // stop the stream
            resource.stop().await;
            let result = resource.state().await;
            assert_eq!(StreamState::Stopped, result);

            // wait for the stream event
            let event = recv_timeout!(&mut receiver, Duration::from_millis(250));
            match &*event {
                StreamEvent::StateChanged(state) => assert_eq!(state, &StreamState::Stopped),
                _ => assert!(
                    false,
                    "expected StreamEvent::StateChanged, but got {:?}",
                    event
                ),
            }
        }
    }

    mod poll_next {
        use super::*;
        use crate::core::stream::tests::read_stream;
        use crate::testing::read_test_file_to_string;

        #[tokio::test]
        async fn test_poll_next() {
            init_logger!();
            let (resource, _temp_dir) = create_new_resource("large-[123].txt");
            let expected_result =
                read_test_file_to_string(resource.inner.filepath.to_str().unwrap());
            let stream = resource.stream().await.expect("expected a stream");

            let result = read_stream(stream).await;

            assert_eq!(expected_result, result);
        }

        #[tokio::test]
        async fn test_poll_next_range() {
            init_logger!();
            let (resource, _temp_dir) = create_new_resource("large-[123].txt");
            let expected_result = "Nullam consequat elit ut ornare scelerisque. Nullam sodales pretium sem sit amet efficitur. Sed feugiat sapien lorem, in rhoncus".to_string();
            let stream = resource
                .stream_range(0, Some(128))
                .await
                .expect("expected a stream");

            let result = read_stream(stream).await;

            assert_eq!(expected_result, result);
        }
    }

    fn create_new_resource(filename: &str) -> (FileStreamingResource, TempDir) {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filepath = PathBuf::from(copy_test_file(temp_path, filename, None));

        (FileStreamingResource::new(&filepath).unwrap(), temp_dir)
    }
}
