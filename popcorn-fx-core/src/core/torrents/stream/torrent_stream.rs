use async_trait::async_trait;
use derive_more::Display;
use futures::Stream;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use std::cmp::{max, min};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::core::callback::{Callback, MultiCallback, Subscriber, Subscription};
use crate::core::torrents::{
    DownloadStatus, Error, StreamBytesResult, Torrent, TorrentEvent, TorrentHandle, TorrentState,
    TorrentStream, TorrentStreamEvent, TorrentStreamState, TorrentStreamingResource,
    TorrentStreamingResourceWrapper,
};
use crate::core::{block_in_place_runtime, torrents, Handle};

/// The default buffer size used while streaming in bytes
const BUFFER_SIZE: usize = 10000;

/// The buffer byte range type.
type Buffer = std::ops::Range<usize>;

/// The default implementation of [TorrentStream] which provides a [Stream]
/// over the [File] resource.
///
/// It uses a buffer of [BUFFER_SIZE] which is checked for availability through the
/// [Torrent] before it's returned.
#[derive(Debug, Clone)]
pub struct DefaultTorrentStream {
    inner: Arc<InnerTorrentStream>,
}

impl DefaultTorrentStream {
    pub fn new(url: Url, torrent: Box<dyn Torrent>, runtime: Arc<Runtime>) -> Self {
        let inner = InnerTorrentStream::new(url, torrent, runtime.clone());
        let instance = Self {
            inner: Arc::new(inner),
        };

        let main_inner = instance.inner.clone();
        let torrent_event_receiver = main_inner.torrent.subscribe();
        runtime.spawn(async move {
            main_inner.start(torrent_event_receiver).await;
        });

        instance
    }
}

#[async_trait]
impl Torrent for DefaultTorrentStream {
    fn handle(&self) -> TorrentHandle {
        self.inner.handle()
    }

    async fn file(&self) -> PathBuf {
        self.inner.file().await
    }

    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool {
        self.inner.has_bytes(bytes).await
    }

    async fn has_piece(&self, piece: usize) -> bool {
        self.inner.has_piece(piece).await
    }

    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>) {
        self.inner.prioritize_bytes(bytes).await
    }

    async fn prioritize_pieces(&self, pieces: &[u32]) {
        self.inner.prioritize_pieces(pieces).await
    }

    async fn total_pieces(&self) -> usize {
        self.inner.total_pieces().await
    }

    async fn sequential_mode(&self) {
        self.inner.sequential_mode().await
    }

    async fn state(&self) -> TorrentState {
        self.inner.state().await
    }
}

#[async_trait]
impl TorrentStream for DefaultTorrentStream {
    fn stream_handle(&self) -> Handle {
        self.inner.stream_handle()
    }

    fn url(&self) -> Url {
        self.inner.url()
    }

    fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper> {
        self.inner.stream()
    }

    fn stream_offset(
        &self,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<TorrentStreamingResourceWrapper> {
        self.inner.stream_offset(offset, len)
    }

    async fn stream_state(&self) -> TorrentStreamState {
        self.inner.stream_state().await
    }

    fn stop_stream(&self) {
        self.inner.stop_stream()
    }
}

impl Callback<TorrentEvent> for DefaultTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        self.inner.torrent.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        self.inner.torrent.subscribe_with(subscriber)
    }
}

impl Callback<TorrentStreamEvent> for DefaultTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentStreamEvent> {
        self.inner.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentStreamEvent>) {
        self.inner.subscribe_with(subscriber)
    }
}

impl Display for DefaultTorrentStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerTorrentStream {
    handle: Handle,
    /// The backing torrent of this stream
    torrent: Arc<Box<dyn Torrent>>,
    /// The url on which this stream is being hosted
    url: Url,
    /// The pieces which should be prepared for the stream
    preparing_pieces: Arc<Mutex<Vec<u32>>>,
    /// The state of this stream
    state: Arc<Mutex<TorrentStreamState>>,
    /// The callbacks for this stream
    callbacks: MultiCallback<TorrentStreamEvent>,
    /// The cancellation token of the torrent stream
    cancellation_token: CancellationToken,
    /// The shared runtime of the torrent stream
    runtime: Arc<Runtime>,
}

impl InnerTorrentStream {
    fn new(url: Url, torrent: Box<dyn Torrent>, runtime: Arc<Runtime>) -> Self {
        let prepare_pieces = block_in_place_runtime(Self::preparation_pieces(&torrent), &runtime);

        Self {
            handle: Handle::new(),
            torrent: Arc::new(torrent),
            url,
            preparing_pieces: Arc::new(Mutex::new(prepare_pieces)),
            state: Arc::new(Mutex::new(TorrentStreamState::Preparing)),
            callbacks: MultiCallback::new(runtime.clone()),
            cancellation_token: Default::default(),
            runtime,
        }
    }

    /// Start the main loop of the torrent stream.
    async fn start(&self, mut torrent_event: Subscription<TorrentEvent>) {
        // initialize the pieces required for the stream to be able to start
        select! {
            _ = self.cancellation_token.cancelled() => return,
            _ = self.start_preparing_pieces() => {},
        }

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(event) = torrent_event.recv() => self.handle_torrent_event(event).await,
            }
        }

        self.update_state(TorrentStreamState::Stopped).await;
        debug!("Torrent stream {} main loop ended", self);
    }

    /// Handle the given torrent event.
    async fn handle_torrent_event(&self, event: Arc<TorrentEvent>) {
        debug!("Torrent stream {} handling torrent event {:?}", self, event);
        match &*event {
            TorrentEvent::StateChanged(state) => {
                if state == &TorrentState::Completed {
                    self.update_state(TorrentStreamState::Streaming).await;
                } else {
                    self.verify_ready_to_stream().await;
                }
            }
            TorrentEvent::PieceFinished(piece) => self.on_piece_finished(*piece).await,
            TorrentEvent::DownloadStatus(status) => self.on_download_status(status.clone()),
        }
    }

    /// Prepare the initial pieces required for the torrent stream to be able to start.
    async fn start_preparing_pieces(&self) {
        let state = self.torrent.state().await;
        trace!(
            "Torrent stream {} preparation with torrent state {}",
            self,
            state
        );
        if state == TorrentState::Completed {
            debug!("Torrent has state {}, starting stream immediately", state);
            self.update_state(TorrentStreamState::Streaming).await;
        } else {
            let mut pieces = self.preparing_pieces.lock().await;
            debug!(
                "Preparing a total of {} pieces for the stream",
                pieces.len()
            );
            self.torrent.prioritize_pieces(&pieces[..]).await;

            // check if some pieces have already been completed by the torrent
            for index in 0..pieces.len() {
                match pieces.get(index) {
                    None => {}
                    Some(piece) => {
                        if self.torrent.has_piece(piece.clone() as usize).await {
                            pieces.remove(index);
                        }
                    }
                }
            }
        }
    }

    async fn on_piece_finished(&self, piece: u32) {
        trace!(
            "Torrent stream {} received piece {} completion",
            self,
            piece
        );
        let mut pieces = self.preparing_pieces.lock().await;

        match pieces.iter().position(|e| e == &piece) {
            Some(position) => {
                pieces.remove(position);
                debug!(
                    "Torrent stream {} prepare piece {} completed, {} remaining",
                    self,
                    piece,
                    pieces.len()
                );
            }
            _ => {}
        }

        drop(pieces);
        self.verify_ready_to_stream().await;
    }

    fn on_download_status(&self, download_status: DownloadStatus) {
        self.callbacks
            .invoke(TorrentStreamEvent::DownloadStatus(download_status))
    }

    async fn verify_ready_to_stream(&self) {
        let pieces = self.preparing_pieces.lock().await;

        if pieces.is_empty() {
            self.torrent.sequential_mode().await;
            self.update_state(TorrentStreamState::Streaming).await;
        } else {
            debug!("Awaiting {} remaining pieces to be prepared", pieces.len());
        }
    }

    async fn update_state(&self, new_state: TorrentStreamState) {
        let mut state = self.state.lock().await;
        if *state == new_state {
            return;
        }

        info!("Torrent stream state changed to {}", &new_state);
        *state = new_state.clone();
        self.callbacks
            .invoke(TorrentStreamEvent::StateChanged(new_state));
    }

    async fn preparation_pieces(torrent: &Box<dyn Torrent>) -> Vec<u32> {
        let total_pieces = torrent.total_pieces().await;
        trace!(
            "Calculating preparation pieces of {:?} for a total of {} pieces",
            torrent.file().await,
            total_pieces
        );
        let number_of_preparation_pieces = max(8, (total_pieces as f32 * 0.08) as u32);
        let number_of_preparation_pieces =
            min(number_of_preparation_pieces, (total_pieces - 1) as u32);
        let start_of_end_piece_index = max(0, total_pieces - 3);
        let mut pieces = vec![];

        // prepare the first 8% of pieces if it doesn't exceed the total pieces
        for i in 0..number_of_preparation_pieces {
            pieces.push(i);
        }

        // prepare the last 3 pieces
        // this is done for determining the video length during streaming
        for i in start_of_end_piece_index..total_pieces {
            pieces.push(i as u32);
        }

        if pieces.is_empty() {
            warn!("Unable to prepare stream, pieces to prepare couldn't be determined");
        }

        pieces.into_iter().map(|e| e).unique().collect()
    }
}

impl Callback<TorrentEvent> for InnerTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        self.torrent.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        self.torrent.subscribe_with(subscriber)
    }
}

impl Callback<TorrentStreamEvent> for InnerTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentStreamEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentStreamEvent>) {
        self.callbacks.subscribe_with(subscriber)
    }
}

#[async_trait]
impl Torrent for InnerTorrentStream {
    fn handle(&self) -> TorrentHandle {
        self.torrent.handle()
    }

    async fn file(&self) -> PathBuf {
        self.torrent.file().await
    }

    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool {
        self.torrent.has_bytes(bytes).await
    }

    async fn has_piece(&self, piece: usize) -> bool {
        self.torrent.has_piece(piece).await
    }

    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>) {
        self.torrent.prioritize_bytes(bytes).await
    }

    async fn prioritize_pieces(&self, pieces: &[u32]) {
        self.torrent.prioritize_pieces(pieces).await
    }

    async fn total_pieces(&self) -> usize {
        self.torrent.total_pieces().await
    }

    async fn sequential_mode(&self) {
        self.torrent.sequential_mode().await
    }

    async fn state(&self) -> TorrentState {
        self.torrent.state().await
    }
}

#[async_trait]
impl TorrentStream for InnerTorrentStream {
    fn stream_handle(&self) -> Handle {
        self.handle.clone()
    }

    fn url(&self) -> Url {
        self.url.clone()
    }

    fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper> {
        tokio::task::block_in_place(|| {
            let mutex = block_in_place_runtime(self.state.lock(), &self.runtime);
            if *mutex == TorrentStreamState::Streaming {
                DefaultTorrentStreamingResource::new(self.torrent.clone(), self.runtime.clone())
                    .map(|e| TorrentStreamingResourceWrapper::new(e))
            } else {
                Err(Error::InvalidStreamState(mutex.clone()))
            }
        })
    }

    fn stream_offset(
        &self,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<TorrentStreamingResourceWrapper> {
        tokio::task::block_in_place(|| {
            let mutex = block_in_place_runtime(self.state.lock(), &self.runtime);
            if *mutex == TorrentStreamState::Streaming {
                DefaultTorrentStreamingResource::new_offset(
                    self.torrent.clone(),
                    offset,
                    len,
                    self.runtime.clone(),
                )
                .map(|e| TorrentStreamingResourceWrapper::new(e))
            } else {
                Err(Error::InvalidStreamState(mutex.clone()))
            }
        })
    }

    async fn stream_state(&self) -> TorrentStreamState {
        *self.state.lock().await
    }

    fn stop_stream(&self) {
        self.cancellation_token.cancel();
    }
}

/// The default implementation of a [Stream] for torrents.
#[derive(Debug, Display)]
#[display(
    fmt = "torrent: {:?}, file: {:?}, cursor: {}",
    torrent,
    filepath,
    cursor
)]
pub struct DefaultTorrentStreamingResource {
    torrent: Arc<Box<dyn Torrent>>,
    /// The open reader handle to the torrent file
    file: File,
    filepath: PathBuf,
    /// The total length of the file resource.
    resource_length: u64,
    /// The current reading cursor for the stream
    cursor: usize,
    /// The starting offset of the stream
    offset: u64,
    /// The total len of the stream
    len: u64,
    /// The shared runtime
    runtime: Arc<Runtime>,
}

impl DefaultTorrentStreamingResource {
    /// Create a new streaming resource which will read the full [Torrent].
    pub fn new(torrent: Arc<Box<dyn Torrent>>, runtime: Arc<Runtime>) -> torrents::Result<Self> {
        Self::new_offset(torrent, 0, None, runtime)
    }

    /// Create a new streaming resource for the given offset.
    /// If no `len` is given, the streaming resource will be read till it's end.
    pub fn new_offset(
        torrent: Arc<Box<dyn Torrent>>,
        offset: u64,
        len: Option<u64>,
        runtime: Arc<Runtime>,
    ) -> torrents::Result<Self> {
        let torrent = torrent.clone();

        debug!(
            "Creating a new streaming resource for torrent {:?}",
            torrent
        );
        futures::executor::block_on(async {
            let filepath = torrent.file().await;

            trace!("Opening torrent file {:?}", &filepath);
            fs::OpenOptions::new()
                .read(true)
                .open(&filepath)
                .map(|mut file| {
                    let resource_length =
                        Self::file_bytes(&mut file).expect("expected a file length");
                    let mut stream_length = match len {
                        None => resource_length,
                        Some(e) => e,
                    };
                    let stream_end = offset + stream_length;

                    if stream_end > resource_length {
                        warn!(
                            "Requested stream range ({}-{}) is larger than {} resource length",
                            &offset, &stream_end, &resource_length
                        );
                        stream_length = resource_length - offset;
                    }

                    Self {
                        torrent,
                        file,
                        filepath: filepath.clone(),
                        resource_length,
                        cursor: offset as usize,
                        offset,
                        len: stream_length,
                        runtime,
                    }
                })
                .map_err(|e| {
                    warn!("Failed to open torrent file {:?}, {}", &filepath, e);
                    let file = filepath;
                    let filepath = file.as_path().to_str().expect("expected a valid path");
                    Error::FileNotFound(filepath.to_string())
                })
        })
    }

    /// Wait for the current cursor to become available.
    fn wait_for(&mut self, cx: &mut Context) -> Poll<Option<StreamBytesResult>> {
        let waker = cx.waker().clone();
        let buffer = self.next_buffer();

        // check if the buffer is already available
        if block_in_place_runtime(
            Self::is_buffer_available_(&self.torrent, &buffer),
            &self.runtime,
        ) {
            return Poll::Ready(self.read_data());
        }

        // request the torrent to prioritize the buffer
        debug!(
            "Waiting for buffer {{{}-{}}} to be available",
            &buffer.start, &buffer.end
        );
        block_in_place_runtime(self.torrent.prioritize_bytes(&buffer), &self.runtime);

        let mut receiver = self.torrent.subscribe();
        self.runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::PieceFinished(_) = *event {
                        waker.wake_by_ref();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Poll::Pending
    }

    /// Read the data of the stream at the current cursor.
    fn read_data(&mut self) -> Option<StreamBytesResult> {
        let buffer_size = self.calculate_buffer_size();
        let reader = &mut self.file;
        let cursor = self.cursor.clone();
        let mut buffer = vec![0; buffer_size];

        match reader.seek(SeekFrom::Start(cursor as u64)) {
            Err(e) => {
                error!(
                    "Failed to modify the file cursor to {}, {}",
                    &self.cursor, e
                );
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

                self.cursor += size;

                if buffer_size != BUFFER_SIZE {
                    trace!(
                        "Reached EOF for {:?} with {} remaining bytes (cursor {})",
                        &self.filepath,
                        size,
                        &self.cursor
                    )
                }
                Some(Ok(buffer))
            }
        }
    }

    fn calculate_buffer_size(&self) -> usize {
        let cursor = self.cursor.clone();
        let range_end = (self.offset + self.len) as usize;

        if cursor + BUFFER_SIZE <= range_end {
            BUFFER_SIZE
        } else {
            range_end - cursor
        }
    }

    /// Verify if the [Torrent] resource has loaded the next buffer to provide to the [Stream::poll_next].
    ///
    /// It returns true when all bytes for the next poll buffer are present, else false.
    fn is_buffer_available(&self) -> bool {
        let buffer = self.next_buffer();

        block_in_place_runtime(
            Self::is_buffer_available_(&self.torrent, &buffer),
            &self.runtime,
        )
    }

    /// Get the next buffer byte range.
    ///
    /// It returns the [Buffer] range.
    fn next_buffer(&self) -> Buffer {
        let mut buffer_end_byte = self.cursor + BUFFER_SIZE;
        let stream_end = (self.offset() + self.content_length()) as usize;

        if buffer_end_byte > stream_end {
            buffer_end_byte = stream_end;
        }

        self.cursor..buffer_end_byte
    }

    /// Retrieve the last byte for the given file.
    fn file_bytes(file: &mut File) -> torrents::Result<u64> {
        match file.seek(SeekFrom::End(0)) {
            Ok(e) => Ok(e),
            Err(e) => {
                error!("Failed determining the file len, {}", e);
                Err(Error::FileError(e.to_string()))
            }
        }
    }

    async fn is_buffer_available_(torrent: &Box<dyn Torrent>, buffer: &Buffer) -> bool {
        torrent.has_bytes(buffer).await
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
        let range = format!(
            "bytes {}-{}/{}",
            self.offset(),
            range_end,
            self.total_length()
        );

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::torrents::{MockTorrent, StreamBytes};
    use crate::init_logger;
    use crate::testing::{copy_test_file, read_test_file_to_string};
    use futures::TryStreamExt;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_torrent_stream_stream() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let callbacks = MultiCallback::new(runtime.clone());
        let subscription_callbacks = callbacks.clone();
        let (tx_ready, rx_ready) = channel();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_has_piece().return_const(true);
        mock.expect_total_pieces().returning(|| 10);
        mock.expect_prioritize_pieces().returning(|_: &[u32]| {});
        mock.expect_sequential_mode().returning(|| {});
        mock.expect_state().return_const(TorrentState::Downloading);
        mock.expect_subscribe()
            .returning(move || subscription_callbacks.subscribe());
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock), runtime.clone());

        // update the ready pieces
        for i in 0..10 {
            callbacks.invoke(TorrentEvent::PieceFinished(i));
        }

        // listen on the streaming state event
        let mut receiver = Callback::<TorrentStreamEvent>::subscribe(&torrent_stream);
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentStreamEvent::StateChanged(state) = &*event {
                        if *state == TorrentStreamState::Streaming {
                            tx_ready.send(()).unwrap();
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        let _ = rx_ready
            .recv_timeout(Duration::from_millis(500))
            .expect("expected the stream to enter the streaming state");
        let result = torrent_stream.stream().expect("expected a stream wrapper");

        assert_eq!(0, result.resource().offset());
        assert_eq!(
            result.resource().total_length(),
            result.resource().content_length()
        );
    }

    #[test]
    fn test_content_range() {
        init_logger!();
        let filename = "range.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(true);
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = DefaultTorrentStreamingResource::new(torrent, runtime).unwrap();
        let bytes = read_test_file_to_string(filename).as_bytes().len();
        let expected_result = format!("bytes 0-{}/{}", bytes - 1, bytes);

        let result = stream.content_range();

        assert_eq!(expected_result, result.as_str())
    }

    #[test]
    fn test_offset() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(true);
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream =
            DefaultTorrentStreamingResource::new_offset(torrent, 1, Some(3), runtime.clone())
                .unwrap();

        let result = read_stream(stream, &runtime);

        assert_eq!("ore".to_string(), result)
    }

    #[test]
    fn test_poll_mismatching_buffer_size() {
        init_logger!();
        let filename = "mismatch.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let mut mock = MockTorrent::new();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().returning(move |_| {
            if a.is_some() {
                a.take();
                return false;
            }

            true
        });
        mock.expect_prioritize_bytes().times(1).return_const(());
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file_to_string(filename);
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = DefaultTorrentStreamingResource::new(torrent, runtime.clone()).unwrap();

        let range = stream.content_range();
        let result = read_stream(stream, &runtime);

        assert_eq!("bytes 0-29/30", range);
        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_poll_next_byte_not_present() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let mut mock = MockTorrent::new();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().returning(move |_| {
            if a.is_some() {
                a.take();
                return false;
            }

            true
        });
        mock.expect_prioritize_bytes().return_const(());
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file_to_string(filename);
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = DefaultTorrentStreamingResource::new(torrent, runtime.clone()).unwrap();

        let result = read_stream(stream, &runtime);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_torrent_stream_prepare_pieces() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("lorem.ipsum");
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let (tx, rx) = channel();
        let runtime = Arc::new(Runtime::new().unwrap());
        let callbacks = MultiCallback::new(runtime.clone());
        let subscribe_callbacks = callbacks.clone();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_has_piece().return_const(false);
        mock.expect_total_pieces().returning(|| 100);
        mock.expect_prioritize_pieces()
            .returning(move |pieces: &[u32]| {
                tx.send(pieces.to_vec()).unwrap();
            });
        mock.expect_sequential_mode().times(1).returning(|| {});
        mock.expect_state().return_const(TorrentState::Downloading);
        mock.expect_subscribe()
            .returning(move || subscribe_callbacks.subscribe());
        let stream = DefaultTorrentStream::new(url, Box::new(mock), runtime.clone());
        let expected_pieces: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 97, 98, 99];

        let pieces = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(expected_pieces.clone(), pieces);

        let (tx, rx) = channel();
        let mut receiver = Callback::<TorrentStreamEvent>::subscribe(&stream);
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentStreamEvent::StateChanged(state) = &*event {
                        tx.send(state.clone()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        for piece in expected_pieces {
            callbacks.invoke(TorrentEvent::PieceFinished(piece));
        }

        let state_result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(TorrentStreamState::Streaming, state_result)
    }

    #[test]
    fn test_torrent_start_preparing_pieces_torrent_completed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("lorem.ipsum");
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(false);
        mock.expect_has_piece().return_const(false);
        mock.expect_total_pieces().returning(|| 100);
        mock.expect_prioritize_pieces()
            .times(0)
            .returning(|_: &[u32]| {});
        mock.expect_state().return_const(TorrentState::Completed);
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = DefaultTorrentStream::new(url, Box::new(mock), runtime.clone());

        // retrieve the initial streaming state as it should be streaming
        let result = runtime.block_on(stream.stream_state());

        assert_eq!(TorrentStreamState::Streaming, result)
    }

    #[test]
    fn test_stop_stream() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        mock.expect_file().returning(move || temp_path.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_total_pieces().returning(|| 10);
        mock.expect_prioritize_pieces().returning(|_: &[u32]| {});
        mock.expect_state().return_const(TorrentState::Downloading);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock), runtime);

        torrent_stream.stop_stream();
        let result = torrent_stream
            .stream()
            .err()
            .expect("expected an error to be returned");

        match result {
            Error::InvalidStreamState(state) => {
                assert_eq!(TorrentStreamState::Stopped, state)
            }
            _ => assert!(false, "expected TorrentError::InvalidStreamState"),
        }
    }

    fn read_stream(mut stream: DefaultTorrentStreamingResource, runtime: &Arc<Runtime>) -> String {
        block_in_place_runtime(
            async {
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
            },
            runtime,
        )
        .expect("expected a valid string")
    }
}
