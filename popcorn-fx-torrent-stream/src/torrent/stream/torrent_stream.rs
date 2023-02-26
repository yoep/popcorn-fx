use std::{fs, thread};
use std::cmp::{max, min};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Once};
use std::task::{Context, Poll};
use std::time::Duration;

use derive_more::Display;
use futures::Stream;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use tokio::sync::Mutex;
use url::Url;

use popcorn_fx_core::core::{CoreCallbacks, torrent};
use popcorn_fx_core::core::torrent::{StreamBytesResult, Torrent, TorrentCallback, TorrentError, TorrentEvent, TorrentState, TorrentStream, TorrentStreamCallback, TorrentStreamEvent, TorrentStreamingResource, TorrentStreamingResourceWrapper, TorrentStreamState};

/// The default buffer size used while streaming in bytes
const BUFFER_SIZE: usize = 10000;
const BUFFER_AVAILABILITY_CHECK: usize = 100;

/// The default implementation of [TorrentStream] which provides a [Stream]
/// over the [File] resource.
///
/// It uses a buffer of [BUFFER_SIZE] which is checked for availability through the
/// [Torrent] before it's returned.
#[derive(Debug)]
pub struct DefaultTorrentStream {
    internal: Arc<TorrentStreamWrapper>,
}

impl DefaultTorrentStream {
    pub fn new(url: Url, torrent: Box<dyn Torrent>) -> Self {
        let wrapper = TorrentStreamWrapper::new(url, torrent);
        let instance = Self {
            internal: Arc::new(wrapper),
        };

        TorrentStreamWrapper::start_torrent_listener(instance.instance());
        instance.instance().start_preparing_pieces();
        instance
    }

    fn instance(&self) -> Arc<TorrentStreamWrapper> {
        self.internal.clone()
    }
}

impl Torrent for DefaultTorrentStream {
    fn file(&self) -> PathBuf {
        self.internal.file()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        self.internal.has_bytes(bytes)
    }

    fn has_piece(&self, piece: u32) -> bool {
        self.internal.has_piece(piece)
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {
        self.internal.prioritize_bytes(bytes)
    }

    fn prioritize_pieces(&self, pieces: &[u32]) {
        self.internal.prioritize_pieces(pieces)
    }

    fn total_pieces(&self) -> i32 {
        self.internal.total_pieces()
    }

    fn sequential_mode(&self) {
        self.internal.sequential_mode()
    }

    fn state(&self) -> TorrentState {
        self.internal.state()
    }

    fn register(&self, callback: TorrentCallback) {
        self.internal.register(callback)
    }
}

impl TorrentStream for DefaultTorrentStream {
    fn url(&self) -> Url {
        self.internal.url()
    }

    fn stream(&self) -> torrent::Result<TorrentStreamingResourceWrapper> {
        self.internal.stream()
    }

    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrent::Result<TorrentStreamingResourceWrapper> {
        self.internal.stream_offset(offset, len)
    }

    fn stream_state(&self) -> TorrentStreamState {
        self.internal.stream_state()
    }

    fn register_stream(&self, callback: TorrentStreamCallback) {
        self.internal.register_stream(callback)
    }

    fn stop_stream(&self) {
        self.internal.stop_stream()
    }
}

impl Display for DefaultTorrentStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.internal)
    }
}

#[derive(Debug, Display)]
#[display(fmt = "url: {}, total_pieces: {}, preparing_pieces: {}", url, "self.total_pieces()", "preparing_pieces.blocking_lock().len()")]
struct TorrentStreamWrapper {
    /// The backing torrent of this stream
    torrent: Arc<Box<dyn Torrent>>,
    /// The url on which this stream is being hosted
    url: Url,
    /// The pieces which should be prepared for the stream
    preparing_pieces: Arc<Mutex<Vec<u32>>>,
    /// The state of this stream
    state: Arc<Mutex<TorrentStreamState>>,
    /// The callbacks for this stream
    callbacks: Arc<CoreCallbacks<TorrentStreamEvent>>,
}

impl TorrentStreamWrapper {
    fn new(url: Url, torrent: Box<dyn Torrent>) -> Self {
        let prepare_pieces = Self::preparation_pieces(&torrent);

        Self {
            torrent: Arc::new(torrent),
            url,
            preparing_pieces: Arc::new(Mutex::new(prepare_pieces)),
            state: Arc::new(Mutex::new(TorrentStreamState::Preparing)),
            callbacks: Arc::new(CoreCallbacks::default()),
        }
    }

    fn start_torrent_listener(instance: Arc<TorrentStreamWrapper>) {
        let torrent = instance.torrent.clone();
        torrent.register(Box::new(move |event| {
            let instance = instance.clone();
            tokio::task::block_in_place(move || {
                match event {
                    TorrentEvent::StateChanged(state) => {
                        if state == TorrentState::Completed {
                            instance.update_state(TorrentStreamState::Streaming)
                        } else {
                            instance.verify_ready_to_stream()
                        }
                    }
                    TorrentEvent::PieceFinished(piece) => instance.on_piece_finished(piece),
                }
            })
        }));
    }

    fn start_preparing_pieces(&self) {
        let state = self.torrent.state();
        trace!("Starting stream preparation with torrent state {}", state);
        if state == TorrentState::Completed {
            debug!("Torrent has state {}, starting stream immediately", state);
            self.update_state(TorrentStreamState::Streaming);
        } else {
            let mutex = self.preparing_pieces.blocking_lock();
            debug!("Preparing a total of {} pieces for the stream", mutex.len());
            self.torrent.prioritize_pieces(&mutex[..]);
        }
    }

    fn on_piece_finished(&self, piece: u32) {
        let mut pieces = self.preparing_pieces.blocking_lock();
        let torrent = self.torrent.clone();

        match pieces.iter().position(|e| e == &piece) {
            Some(position) => { pieces.remove(position); }
            _ => {}
        }

        // check if we need to do an initial check as we might not have received all callbacks
        // a download might have been started before it was requested to be streamed
        for index in 0..pieces.len() {
            match pieces.get(index) {
                None => {}
                Some(piece) => {
                    if torrent.has_piece(piece.clone()) {
                        pieces.remove(index);
                    }
                }
            }
        }

        drop(pieces);
        self.verify_ready_to_stream();
    }

    fn verify_ready_to_stream(&self) {
        let pieces = self.preparing_pieces.blocking_lock();

        if pieces.is_empty() {
            self.torrent.sequential_mode();
            self.update_state(TorrentStreamState::Streaming);
        } else {
            debug!("Awaiting {} remaining pieces to be prepared", pieces.len());
        }
    }

    fn update_state(&self, new_state: TorrentStreamState) {
        let mut state = self.state.blocking_lock();
        if *state == new_state {
            return;
        }

        info!("Torrent stream state changed to {}", &new_state);
        *state = new_state.clone();
        self.callbacks.invoke(TorrentStreamEvent::StateChanged(new_state));
    }

    fn preparation_pieces(torrent: &Box<dyn Torrent>) -> Vec<u32> {
        let total_pieces = torrent.total_pieces();
        trace!("Calculating preparation pieces of {:?} for a total of {} pieces", torrent.file(), total_pieces);
        let number_of_preparation_pieces = max(8, (total_pieces as f32 * 0.08) as i32);
        let number_of_preparation_pieces = min(number_of_preparation_pieces, total_pieces - 1);
        let start_of_end_piece_index = max(0, total_pieces - 3);
        let mut pieces = vec![];

        // prepare the first 8% of pieces if it doesn't exceed the total pieces
        for i in 0..number_of_preparation_pieces {
            pieces.push(i);
        }

        // prepare the last 3 pieces
        // this is done for determining the video length during streaming
        for i in start_of_end_piece_index..total_pieces {
            pieces.push(i);
        }

        if pieces.is_empty() {
            warn!("Unable to prepare stream, pieces to prepare couldn't be determined");
        }

        pieces.into_iter()
            .map(|e| e as u32)
            .unique()
            .collect()
    }
}

impl Torrent for TorrentStreamWrapper {
    fn file(&self) -> PathBuf {
        self.torrent.file()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        self.torrent.has_bytes(bytes)
    }

    fn has_piece(&self, piece: u32) -> bool {
        self.torrent.has_piece(piece)
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {
        self.torrent.prioritize_bytes(bytes)
    }

    fn prioritize_pieces(&self, pieces: &[u32]) {
        self.torrent.prioritize_pieces(pieces)
    }

    fn total_pieces(&self) -> i32 {
        self.torrent.total_pieces()
    }

    fn sequential_mode(&self) {
        self.torrent.sequential_mode()
    }

    fn state(&self) -> TorrentState {
        self.torrent.state()
    }

    fn register(&self, callback: TorrentCallback) {
        self.torrent.register(callback)
    }
}

impl TorrentStream for TorrentStreamWrapper {
    fn url(&self) -> Url {
        self.url.clone()
    }

    fn stream(&self) -> torrent::Result<TorrentStreamingResourceWrapper> {
        tokio::task::block_in_place(|| {
            let mutex = self.state.blocking_lock();
            if *mutex == TorrentStreamState::Streaming {
                DefaultTorrentStreamingResource::new(&self.torrent)
                    .map(|e| TorrentStreamingResourceWrapper::new(e))
            } else {
                Err(TorrentError::InvalidStreamState(mutex.clone()))
            }
        })
    }

    fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrent::Result<TorrentStreamingResourceWrapper> {
        tokio::task::block_in_place(|| {
            let mutex = self.state.blocking_lock();
            if *mutex == TorrentStreamState::Streaming {
                DefaultTorrentStreamingResource::new_offset(&self.torrent, offset, len)
                    .map(|e| TorrentStreamingResourceWrapper::new(e))
            } else {
                Err(TorrentError::InvalidStreamState(mutex.clone()))
            }
        })
    }

    fn stream_state(&self) -> TorrentStreamState {
        self.state.blocking_lock().clone()
    }

    fn register_stream(&self, callback: TorrentStreamCallback) {
        debug!("Adding a new callback to stream {}", self);
        self.callbacks.add(callback)
    }

    fn stop_stream(&self) {
        self.update_state(TorrentStreamState::Stopped);
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
    use std::sync::mpsc::channel;

    use futures::TryStreamExt;
    use tokio::runtime;

    use popcorn_fx_core::core::torrent::{MockTorrent, StreamBytes};
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file};

    use super::*;

    #[test]
    fn test_torrent_stream_stream() {
        init_logger();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let (tx, rx) = channel();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        mock.expect_has_piece()
            .return_const(true);
        mock.expect_total_pieces()
            .returning(|| 10);
        mock.expect_prioritize_pieces()
            .returning(|_: &[u32]| {});
        mock.expect_sequential_mode()
            .returning(|| {});
        mock.expect_register()
            .times(1)
            .returning(move |callback: TorrentCallback| {
                tx.send(callback).unwrap();
            });
        mock.expect_state()
            .return_const(TorrentState::Downloading);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock));

        let callback = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        for i in 0..10 {
            callback(TorrentEvent::PieceFinished(i))
        }
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
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();
        let bytes = read_test_file(filename).as_bytes().len();
        let expected_result = format!("bytes 0-{}/{}", bytes - 1, bytes);

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
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
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
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
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
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file(filename);
        let stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();

        let result = read_stream(stream);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_torrent_stream_prepare_pieces() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("lorem.ipsum");
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let (tx, rx) = channel();
        let (tx_c, rx_c) = channel();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        mock.expect_has_piece()
            .return_const(false);
        mock.expect_total_pieces()
            .returning(|| 100);
        mock.expect_prioritize_pieces()
            .returning(move |pieces: &[u32]| {
                tx.send(pieces.to_vec()).unwrap();
            });
        mock.expect_register()
            .returning(move |callback: TorrentCallback| {
                tx_c.send(callback).unwrap()
            });
        mock.expect_sequential_mode()
            .times(1)
            .returning(|| {});
        mock.expect_state()
            .return_const(TorrentState::Downloading);
        let stream = DefaultTorrentStream::new(url, Box::new(mock));
        let expected_pieces: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 97, 98, 99];

        let pieces = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(expected_pieces.clone(), pieces);

        let callback = rx_c.recv_timeout(Duration::from_millis(200)).unwrap();
        for piece in expected_pieces {
            callback(TorrentEvent::PieceFinished(piece));
        }

        let state_result = stream.stream_state();
        assert_eq!(TorrentStreamState::Streaming, state_result)
    }

    #[test]
    fn test_torrent_start_preparing_pieces_torrent_completed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("lorem.ipsum");
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(false);
        mock.expect_has_piece()
            .return_const(false);
        mock.expect_total_pieces()
            .returning(|| 100);
        mock.expect_prioritize_pieces()
            .times(0)
            .returning(|_: &[u32]| {});
        mock.expect_register()
            .returning(|_: TorrentCallback| {});
        mock.expect_state()
            .return_const(TorrentState::Completed);
        let stream = DefaultTorrentStream::new(url, Box::new(mock));

        // retrieve the initial streaming state as it should be streaming
        let result = stream.stream_state();

        assert_eq!(TorrentStreamState::Streaming, result)
    }

    #[test]
    fn test_stop_stream() {
        init_logger();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_bytes()
            .return_const(true);
        mock.expect_total_pieces()
            .returning(|| 10);
        mock.expect_prioritize_pieces()
            .returning(|_: &[u32]| {});
        mock.expect_register()
            .times(1)
            .returning(|_| {});
        mock.expect_state()
            .return_const(TorrentState::Downloading);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock));

        torrent_stream.stop_stream();
        let result = torrent_stream.stream()
            .err()
            .expect("expected an error to be returned");

        match result {
            TorrentError::InvalidStreamState(state) => assert_eq!(TorrentStreamState::Stopped, state),
            _ => assert!(false, "expected TorrentError::InvalidStreamState")
        }
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