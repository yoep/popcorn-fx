use std::{fs, thread};
use std::borrow::BorrowMut;
use std::fs::File;
use std::future::Future;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Once};
use std::task::{Context, Poll};
use std::time::Duration;

use derive_more::Display;
use futures::Stream;
use log::{debug, error, trace, warn};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use tokio::runtime;
use url::Url;

use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{MockTorrent, StreamBytesResult, Torrent, TorrentError, TorrentStream, TorrentStreamingResource, TorrentStreamingResourceWrapper};

/// The default buffer size used while streaming in bytes
const BUFFER_SIZE: usize = 10000;
const BUFFER_AVAILABILITY_CHECK: usize = 100;

#[derive(Debug, Display)]
#[display(fmt = "url: {}", url)]
pub struct DefaultTorrentStream {
    /// The backing torrent of this stream.
    torrent: Arc<Box<dyn Torrent>>,
    /// The url on which this stream is being hosted.
    url: Url,
}

impl DefaultTorrentStream {
    pub fn new(url: Url, torrent: Box<dyn Torrent>) -> Self {
        Self {
            torrent: Arc::new(torrent),
            url,
        }
    }
}

impl Torrent for DefaultTorrentStream {
    fn file(&self) -> PathBuf {
        self.torrent.file()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        self.torrent.has_bytes(bytes)
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {
        self.torrent.prioritize_bytes(bytes)
    }
}

impl TorrentStream for DefaultTorrentStream {
    fn url(&self) -> Url {
        self.url.clone()
    }

    fn stream(&self) -> torrent::Result<TorrentStreamingResourceWrapper> {
        DefaultTorrentStreamingResource::new(&self.torrent)
            .map(|e| TorrentStreamingResourceWrapper::new(e))
    }

    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrent::Result<TorrentStreamingResourceWrapper> {
        DefaultTorrentStreamingResource::new_offset(&self.torrent, offset, len)
            .map(|e| TorrentStreamingResourceWrapper::new(e))
    }
}

/// The default implementation of a [Stream] for torrents.
#[derive(Debug, Display)]
#[display(fmt = "torrent: {:?}, file: {:?}, cursor: {}", torrent, filepath, cursor)]
pub struct DefaultTorrentStreamingResource {
    torrent: Arc<Box<dyn Torrent>>,
    /// The open reader handle to the torrent file
    file: File,
    filepath: PathBuf,
    /// The total length of the file resource.
    resource_length: u64,
    /// The current reading cursor for the stream
    cursor: u64,
    /// The starting offset of the stream
    offset: u64,
    /// The total len of the stream
    len: u64,
}

impl DefaultTorrentStreamingResource {
    /// Create a new streaming resource which will read the full [Torrent].
    pub fn new(torrent: &Arc<Box<dyn Torrent>>) -> torrent::Result<Self> {
        Self::new_offset(torrent, 0, None)
    }

    /// Create a new streaming resource for the given offset.
    /// If no `len` is given, the streaming resource will be read till it's end.
    pub fn new_offset(torrent: &Arc<Box<dyn Torrent>>, offset: u64, len: Option<u64>) -> torrent::Result<Self> {
        let torrent = torrent.clone();

        debug!("Creating a new streaming resource for torrent {:?}", torrent);
        futures::executor::block_on(async {
            let filepath = torrent.file();

            trace!("Opening torrent file {:?}", &filepath);
            fs::OpenOptions::new()
                .read(true)
                .open(&filepath)
                .map(|mut file| {
                    let resource_length = Self::file_bytes(&mut file).expect("expected a file length");
                    let mut stream_length = match len {
                        None => resource_length,
                        Some(e) => e
                    };
                    let stream_end = offset + stream_length;

                    if stream_end > resource_length {
                        warn!("Requested stream range ({}-{}) is larger than {} resource length", &offset, &stream_end, &resource_length);
                        stream_length = resource_length - offset;
                    }

                    Self {
                        torrent,
                        file,
                        filepath: filepath.clone(),
                        resource_length,
                        cursor: offset,
                        offset,
                        len: stream_length,
                    }
                })
                .map_err(|e| {
                    warn!("Failed to open torrent file {:?}, {}", &filepath, e);
                    let file = filepath;
                    let filepath = file.as_path().to_str().expect("expected a valid path");
                    TorrentError::FileNotFound(filepath.to_string())
                })
        })
    }

    /// Wait for the current cursor to become available.
    fn wait_for(&self, cx: &mut Context) -> Poll<Option<StreamBytesResult>> {
        let torrent = self.torrent.clone();
        let waker = cx.waker().clone();
        let buffer = self.next_buffer();
        let buffer_length = (buffer.end - buffer.start) as usize;
        let mut bytes: Vec<u64> = vec![0; buffer_length];

        for i in 0..buffer_length {
            bytes[i] = i as u64 + buffer.start;
        }
        torrent.prioritize_bytes(&bytes[..]);

        tokio::spawn(async move {
            let log = Once::new();

            while !Self::is_buffer_available_(&torrent, &buffer) {
                log.call_once(|| {
                    debug!("Waiting for buffer {{{}-{}}} to be available", &buffer.start, &buffer.end);
                });
                thread::sleep(Duration::from_millis(10))
            }

            debug!("Buffer {{{}-{}}} became available", &buffer.start, &buffer.end);
            waker.wake();
        });

        return Poll::Pending;
    }

    /// Read the data of the stream at the current cursor.
    fn read_data(&mut self) -> Option<StreamBytesResult> {
        let buffer_size = self.calculate_buffer_size();
        let reader = &mut self.file;
        let cursor = self.cursor.clone();
        let mut buffer = vec![0; buffer_size];

        match reader.seek(SeekFrom::Start(cursor)) {
            Err(e) => {
                error!("Failed to modify the file cursor to {}, {}", &self.cursor, e);
                return None;
            }
            Ok(_) => {}
        }

        match reader.read(&mut buffer) {
            Err(e) => {
                error!("Failed to read the file cursor data, {}", e);
                None
            }
            Ok(size) => {
                if size == 0 {
                    trace!("Reached EOF for {:?}", &self.filepath);
                    return None;
                }

                self.cursor += size as u64;

                if buffer_size != BUFFER_SIZE {
                    trace!("Reached EOF for {:?} with {} remaining bytes (cursor {})", &self.filepath, size, &self.cursor)
                }
                Some(Ok(buffer))
            }
        }
    }

    fn calculate_buffer_size(&self) -> usize {
        let cursor = self.cursor.clone();
        let range_end = self.offset + self.len;

        if cursor as usize + BUFFER_SIZE <= range_end as usize {
            BUFFER_SIZE
        } else {
            (range_end - cursor) as usize
        }
    }

    /// Verify if the [Torrent] resource has loaded the next buffer to provide to the [Stream::poll_next].
    ///
    /// It returns true when all bytes for the next poll buffer are present, else false.
    fn is_buffer_available(&self) -> bool {
        let buffer = self.next_buffer();

        Self::is_buffer_available_(&self.torrent, &buffer)
    }

    /// Retrieve the next buffer byte range.
    ///
    /// It returns the [Buffer] range.
    fn next_buffer(&self) -> Buffer {
        let mut buffer_end_byte = self.cursor + BUFFER_SIZE as u64;
        let stream_end = self.offset() + self.content_length();

        if buffer_end_byte > stream_end {
            buffer_end_byte = stream_end;
        }

        Buffer {
            start: self.cursor,
            end: buffer_end_byte,
        }
    }

    /// Retrieve the last byte for the given file.
    fn file_bytes(file: &mut File) -> torrent::Result<u64> {
        match file.seek(SeekFrom::End(0)) {
            Ok(e) => Ok(e),
            Err(e) => {
                error!("Failed determining the file len, {}", e);
                Err(TorrentError::FileError(e.to_string()))
            }
        }
    }

    fn is_buffer_available_(torrent: &Arc<Box<dyn Torrent>>, buffer: &Buffer) -> bool {
        let buffer_length = (buffer.end - buffer.start) as usize;
        let total_bytes_to_check = buffer_length / BUFFER_AVAILABILITY_CHECK;
        let mut bytes: Vec<u64> = vec![0; total_bytes_to_check];

        for i in 0..total_bytes_to_check {
            let byte_check = (i * BUFFER_AVAILABILITY_CHECK) as u64;
            bytes[i] = byte_check + buffer.start;
        }

        torrent.has_bytes(&bytes[..])
    }
}

impl TorrentStreamingResource for DefaultTorrentStreamingResource {
    fn offset(&self) -> u64 {
        self.offset.clone()
    }

    fn total_length(&self) -> u64 {
        self.resource_length.clone()
    }

    fn content_length(&self) -> u64 {
        self.len.clone()
    }

    fn content_range(&self) -> String {
        let range_end = if self.content_length() == 0 {
            self.offset()
        } else {
            self.offset() + self.content_length() - 1
        };
        let range = format!("bytes {}-{}/{}", self.offset(), range_end, self.total_length());

        trace!("Stream {{{}}} has the following range {{{}}}", self, &range);
        range
    }
}

impl Stream for DefaultTorrentStreamingResource {
    type Item = StreamBytesResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_buffer_available() {
            return self.wait_for(cx);
        }

        Poll::Ready(self.as_mut().read_data())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.content_length() as f64;
        let total_buffers = length / BUFFER_SIZE as f64;

        (0, Some(total_buffers.ceil() as usize))
    }
}

struct Buffer {
    start: u64,
    end: u64,
}

#[cfg(test)]
mod test {
    use futures::{StreamExt, TryStreamExt};
    use tokio::runtime;

    use popcorn_fx_core::core::torrent::{MockTorrent, StreamBytes};
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file};

    use super::*;

    #[test]
    fn test_torrent_stream_stream() {
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock));

        let result = torrent_stream.stream()
            .expect("expected a stream wrapper");

        assert_eq!(0, result.resource().offset());
        assert_eq!(result.resource().total_length(), result.resource().content_length());
    }

    #[test]
    fn test_content_range() {
        init_logger();
        let filename = "range.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();
        let expected_result = "bytes 0-1027/1028";

        let result = stream.content_range();

        assert_eq!(expected_result, result.as_str())
    }

    #[test]
    fn test_offset() {
        init_logger();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let stream = DefaultTorrentStreamingResource::new_offset(
            &torrent,
            1,
            Some(3)).unwrap();

        let result = read_stream(stream);

        assert_eq!("ore".to_string(), result)
    }

    #[test]
    fn test_poll_mismatching_buffer_size() {
        init_logger();
        let filename = "mismatch.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let mut mock = MockTorrent::new();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .returning(move |_| {
                if a.is_some() {
                    a.take();
                    return false;
                }

                true
            });
        mock.expect_prioritize_bytes()
            .times(1)
            .return_const(());
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let expected_result = read_test_file(filename);
        let stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();

        let range = stream.content_range();
        let result = read_stream(stream);

        assert_eq!("bytes 0-29/30", range);
        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_poll_next_byte_not_present() {
        init_logger();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let mut mock = MockTorrent::new();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .returning(move |_| {
                if a.is_some() {
                    a.take();
                    return false;
                }

                true
            });
        mock.expect_prioritize_bytes()
            .return_const(());
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let expected_result = read_test_file(filename);
        let stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();

        let result = read_stream(stream);

        assert_eq!(expected_result, result)
    }

    fn read_stream(mut stream: DefaultTorrentStreamingResource) -> String {
        let runtime = runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut data: Option<StreamBytes>;
            let mut result: Vec<u8> = vec![];

            loop {
                data = stream.try_next().await.unwrap();
                if data.is_some() {
                    result.append(&mut data.unwrap().to_vec());
                } else {
                    break;
                }
            }

            String::from_utf8(result)
        }).expect("expected a valid string")
    }
}