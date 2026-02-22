use crate::core::stream;
use crate::core::stream::{
    Stream, StreamBytesResult, StreamEvent, StreamRange, StreamState, StreamStats,
    StreamingResource,
};
use crate::core::torrents::{Torrent, TorrentManager};
use async_trait::async_trait;
use derive_more::Display;
use futures::future::BoxFuture;
use futures::task::AtomicWaker;
use futures::{ready, FutureExt};
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_torrent::{PieceIndex, PiecePriority, TorrentEvent, TorrentState};
use itertools::Itertools;
use log::{debug, error, trace, warn};
use std::cmp::{max, min};
use std::fmt::{Debug, Formatter};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::SeekFrom;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::select;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

/// The default buffer size, in bytes, used while streaming the file contents.
const DEFAULT_BUFFER_SIZE: usize = 256 * 1000; // 256KB
const PREPARE_FILE_PERCENTAGE: f32 = 0.08; // 8%

/// The buffer byte range type.
type Buffer = std::ops::Range<usize>;

#[derive(Debug)]
pub struct TorrentStreamingResource {
    inner: Arc<InnerTorrentStreamingResource>,
}

impl TorrentStreamingResource {
    /// Create a new torrent stream resource from the given torrent.
    pub async fn new(
        filename: impl AsRef<str>,
        torrent: Box<dyn Torrent>,
        manager: Arc<dyn TorrentManager>,
    ) -> Self {
        let preparation_pieces =
            InnerTorrentStreamingResource::preparation_pieces(&torrent, filename.as_ref()).await;
        let inner = Arc::new(InnerTorrentStreamingResource {
            filename: filename.as_ref().to_string(),
            torrent: Arc::from(torrent),
            state: RwLock::new(StreamState::Preparing),
            preparing_pieces: Mutex::new(preparation_pieces),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        let receiver = inner.torrent.subscribe();
        tokio::spawn(async move {
            inner_main.run(receiver, manager).await;
        });

        Self { inner }
    }
}

impl Callback<StreamEvent> for TorrentStreamingResource {
    fn subscribe(&self) -> Subscription<StreamEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<StreamEvent>) {
        self.inner.callbacks.subscribe_with(subscriber);
    }
}

#[async_trait]
impl StreamingResource for TorrentStreamingResource {
    fn filename(&self) -> &str {
        self.inner.filename.as_str()
    }

    async fn stream(&self) -> stream::Result<Box<dyn Stream>> {
        self.stream_range(0, None).await
    }

    async fn stream_range(&self, start: u64, end: Option<u64>) -> stream::Result<Box<dyn Stream>> {
        self.inner.assert_stream_state().await?;
        Ok(Box::new(
            TorrentStream::new(&self.inner.filename, self.inner.torrent.clone(), start, end)
                .await?,
        ))
    }

    async fn state(&self) -> StreamState {
        *self.inner.state.read().await
    }

    async fn stop(&self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display("{}", torrent.handle())]
struct InnerTorrentStreamingResource {
    filename: String,
    torrent: Arc<dyn Torrent>,
    state: RwLock<StreamState>,
    preparing_pieces: Mutex<Vec<PieceIndex>>,
    callbacks: MultiThreadedCallback<StreamEvent>,
    cancellation_token: CancellationToken,
}

impl InnerTorrentStreamingResource {
    async fn run(
        &self,
        mut receiver: Subscription<TorrentEvent>,
        manager: Arc<dyn TorrentManager>,
    ) {
        // initialize the pieces required for the stream to be able to start
        select! {
            _ = self.cancellation_token.cancelled() => {
                self.close(manager).await;
                return;
            },
            _ = self.start_preparing_pieces() => {},
        }

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(event) = receiver.recv() => self.on_event(&event).await,
            }
        }

        self.close(manager).await;
        debug!("Torrent stream {} main loop ended", self);
    }

    async fn on_event(&self, event: &TorrentEvent) {
        match event {
            TorrentEvent::StateChanged(state) => {
                if state == &TorrentState::Finished {
                    self.update_state(StreamState::Streaming).await;
                } else {
                    self.verify_ready_to_stream().await;
                }
            }
            TorrentEvent::Stats(stats) => {
                self.callbacks
                    .invoke(StreamEvent::StatsChanged(StreamStats {
                        progress: stats.progress(),
                        connections: stats.peers.get() as usize,
                        download_speed: stats.download_useful.rate(),
                        upload_speed: stats.upload_useful.rate(),
                        downloaded: stats.wanted_completed_size.get() as usize,
                        total_size: stats.wanted_size.get() as usize,
                    }));
            }
            TorrentEvent::PieceCompleted(piece) => self.on_piece_finished(*piece).await,
            _ => {}
        }
    }

    async fn preparation_pieces(torrent: &Box<dyn Torrent>, filename: &str) -> Vec<PieceIndex> {
        let file = match torrent.file_by_name(filename).await {
            None => {
                warn!("Unable to find file {} within torrent", filename);
                return Vec::new();
            }
            Some(file) => file,
        };

        let total_file_pieces = file.pieces.len();
        trace!(
            "Calculating preparation pieces of {:?} for a total of {} pieces",
            file,
            total_file_pieces
        );
        // prepare at least 8 (if available), or the ceil of the PREPARE_FILE_PERCENTAGE
        let prepare_lower_bound = min(8, total_file_pieces);
        let percentage_count =
            ((total_file_pieces as f32) * PREPARE_FILE_PERCENTAGE).ceil() as usize;
        let number_of_preparation_pieces = max(prepare_lower_bound, percentage_count);
        let mut pieces = vec![];

        // prepare the first `PREPARE_FILE_PERCENTAGE` of pieces if it doesn't exceed the total file pieces
        let start = file.pieces.start;
        let end = file
            .pieces
            .start
            .saturating_add(number_of_preparation_pieces)
            .min(file.pieces.end);
        pieces.extend(start..end);

        // prepare the last 3 pieces
        // this is done for determining the video length during streaming
        let tail_start = file.pieces.end.saturating_sub(3);
        pieces.extend(tail_start..file.pieces.end);

        if pieces.is_empty() {
            warn!("Unable to prepare stream, pieces to prepare couldn't be determined");
        }

        pieces.into_iter().unique().collect()
    }

    /// Prepare the initial pieces required for the torrent stream to be able to start.
    async fn start_preparing_pieces(&self) {
        let state = self.torrent.state().await;
        let stats = self.torrent.stats();
        let priorities = self.torrent.piece_priorities().await;
        trace!(
            "Torrent stream {} preparation with torrent state {}",
            self,
            state
        );
        let is_finished = priorities
            .iter()
            .any(|(_, priority)| *priority != PiecePriority::None)
            && (state == TorrentState::Finished || stats.progress() == 1.0);

        if is_finished {
            debug!(
                "Torrent stream {} is already ready, starting stream immediately",
                self
            );
            self.update_state(StreamState::Streaming).await;
        } else {
            let pieces = self.preparing_pieces.lock().await.clone();
            debug!(
                "Torrent stream {} is preparing a total of {} pieces",
                self,
                pieces.len()
            );
            self.torrent.prioritize_pieces(&pieces[..]).await;

            // check if some pieces have already been completed by the torrent
            for index in 0..pieces.len() {
                match pieces.get(index) {
                    None => {}
                    Some(piece) => {
                        if self.torrent.has_piece(*piece).await {
                            self.on_piece_finished(*piece).await;
                        }
                    }
                }
            }
        }
    }

    async fn on_piece_finished(&self, piece: PieceIndex) {
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

    async fn verify_ready_to_stream(&self) {
        let (is_empty, remaining) = {
            let pieces = self.preparing_pieces.lock().await;
            (pieces.is_empty(), pieces.len())
        };

        if is_empty {
            self.torrent.sequential_mode().await;
            self.update_state(StreamState::Streaming).await;
        } else {
            debug!(
                "Torrent stream {} is awaiting {} remaining pieces to be prepared",
                self, remaining
            );
        }
    }

    async fn assert_stream_state(&self) -> stream::Result<()> {
        if self.cancellation_token.is_cancelled()
            || *self.state.read().await != StreamState::Streaming
        {
            Err(stream::Error::InvalidState)
        } else {
            Ok(())
        }
    }

    async fn update_state(&self, new_state: StreamState) {
        {
            let mut state = self.state.write().await;
            if *state == new_state {
                return;
            }

            *state = new_state;
        }

        debug!("Torrent stream {} state changed to {}", self, new_state);
        self.callbacks.invoke(StreamEvent::StateChanged(new_state));
    }

    async fn close(&self, manager: Arc<dyn TorrentManager>) {
        self.update_state(StreamState::Stopped).await;
        manager.remove(&self.torrent.handle()).await;
    }
}

#[derive(Display)]
#[display("{}", torrent.handle())]
pub struct TorrentStream {
    torrent: Arc<dyn Torrent>,
    torrent_filename: String,
    /// The offset of the torrent file within the torrent
    torrent_offset: usize,
    /// The open reader handle to the torrent file
    file: File,
    /// The absolute path to the torrent file
    filepath: PathBuf,
    /// The total length of the file resource
    resource_length: u64,
    /// The cursor position within the stream range
    cursor: usize,
    /// The range of bytes that will be streamed
    stream_range: StreamRange,
    waker: Arc<AtomicWaker>,
    pending_has_bytes: Option<BoxFuture<'static, bool>>,
    cancellation_token: CancellationToken,
}

impl TorrentStream {
    async fn new(
        filename: &str,
        torrent: Arc<dyn Torrent>,
        start: u64,
        end: Option<u64>,
    ) -> stream::Result<Self> {
        let torrent = torrent.clone();
        let handle = torrent.handle();

        trace!(
            "Torrent streaming resource {} is being created for {:?}",
            handle,
            torrent
        );
        let torrent_file = match torrent.file_by_name(filename).await {
            None => return Err(stream::Error::NotFound(filename.to_string())),
            Some(file) => file,
        };
        let absolute_filepath = torrent.absolute_file_path(&torrent_file).await;
        trace!(
            "Torrent streaming resource {} is opening file {:?}",
            handle,
            absolute_filepath
        );
        OpenOptions::new()
            .read(true)
            .open(&absolute_filepath)
            .map_err(|e| {
                warn!(
                    "Torrent streaming resource {} failed to open torrent file {:?}, {}",
                    handle, absolute_filepath, e
                );
                stream::Error::NotFound(filename.to_string())
            })
            .and_then(|file| {
                // always use the torrent file length rather than the underlying fs length
                // as the disk file might not have the full file length written to disk
                let resource_length = torrent_file.len() as u64;
                let stream_start = start as usize;
                let stream_end = min(
                    end.unwrap_or(resource_length).saturating_add(start),
                    resource_length,
                ) as usize;
                let waker = Arc::new(AtomicWaker::new());
                let cancellation_token = CancellationToken::new();

                // create a generic torrent event listener for waking streaming tasks when needed
                let event_waker = waker.clone();
                let event_cancellation_token = cancellation_token.clone();
                let receiver = torrent.subscribe();
                tokio::spawn(async move {
                    Self::start_torrent_event_handler(
                        receiver,
                        event_waker,
                        event_cancellation_token,
                    )
                    .await;
                });

                Ok(Self {
                    torrent,
                    torrent_filename: filename.to_string(),
                    torrent_offset: torrent_file.torrent_offset,
                    file,
                    filepath: absolute_filepath,
                    resource_length,
                    cursor: start as usize,
                    stream_range: stream_start..stream_end,
                    waker,
                    pending_has_bytes: None,
                    cancellation_token,
                })
            })
    }

    /// Wait for the current cursor to become available.
    fn wait_for(&mut self, buffer: &Buffer, cx: &mut Context) -> Poll<Option<StreamBytesResult>> {
        self.waker.register(cx.waker());

        // prioritize the given buffer range within the torrent
        let torrent = self.torrent.clone();
        let torrent_range = self.as_torrent_range(buffer);
        tokio::spawn(async move {
            torrent.prioritize_bytes(&torrent_range).await;
        });

        debug!(
            "Torrent streaming resource {} is waiting for buffer {{{}-{}}} to be available",
            self, &buffer.start, &buffer.end
        );
        Poll::Pending
    }

    /// Read the data of the stream at the current cursor.
    fn read_data(&mut self) -> StreamBytesResult {
        let buffer_size = self.calculate_buffer_size();
        let mut buffer = vec![0u8; buffer_size];
        let reader = &mut self.file;
        let cursor = self.cursor.clone();

        if let Err(e) = reader.seek(SeekFrom::Start(cursor as u64)) {
            error!(
                "Torrent streaming resource {} failed to modify the file cursor to {}, {}",
                self, &self.cursor, e
            );
            return Err(stream::Error::Io(e));
        }

        let size = reader.read(&mut buffer)?;
        if size == 0 {
            trace!(
                "Torrent streaming resource {} reached EOF for {:?}",
                self,
                &self.filepath
            );
            return Err(stream::Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "stream reached unexpected EOF",
            )));
        }

        self.cursor += size;

        if buffer_size != DEFAULT_BUFFER_SIZE {
            trace!(
                "Torrent streaming resource {} reached EOF for {:?} with {} remaining bytes (cursor {})",
                self,
                &self.filepath,
                size,
                &self.cursor
            )
        }
        trace!(
            "Torrent streaming resource {} read {} bytes from {:?}",
            self,
            size,
            &self.filepath
        );
        buffer.truncate(size);
        Ok(buffer)
    }

    fn calculate_buffer_size(&self) -> usize {
        let cursor = self.cursor.clone();
        self.stream_range
            .end
            .saturating_sub(cursor)
            .min(DEFAULT_BUFFER_SIZE)
    }

    /// Returns the next [Buffer] range for the stream.
    fn next_buffer(&self) -> Buffer {
        let buffer_end_byte = min(self.cursor + DEFAULT_BUFFER_SIZE, self.stream_range.end);
        self.cursor..buffer_end_byte
    }

    /// Returns the given buffer range relative to the torrent file.
    fn as_torrent_range(&self, buffer: &Buffer) -> Buffer {
        buffer.start + self.torrent_offset..buffer.end + self.torrent_offset
    }

    async fn start_torrent_event_handler(
        mut receiver: Subscription<TorrentEvent>,
        event_waker: Arc<AtomicWaker>,
        cancellation_token: CancellationToken,
    ) {
        loop {
            select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                event = receiver.recv() => {
                    if let Some(event) = event {
                        match &*event {
                            TorrentEvent::StateChanged(_) | TorrentEvent::PieceCompleted(_) => {
                                event_waker.wake();
                            }
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

impl futures::Stream for TorrentStream {
    type Item = StreamBytesResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // check if the current cursor position is out-of-bounds,
        // if so, return additional bytes
        if self.cursor >= self.stream_range.end {
            return Poll::Ready(None);
        }

        // get the next buffer to read from
        let buffer = self.next_buffer();
        if self.pending_has_bytes.is_none() {
            let torrent = self.torrent.clone();
            let torrent_range = self.as_torrent_range(&buffer);

            self.pending_has_bytes =
                Some(async move { torrent.has_bytes(&torrent_range).await }.boxed());
        }

        let is_available = match self.pending_has_bytes.as_mut() {
            None => return Poll::Ready(None),
            Some(future) => ready!(future.as_mut().poll(cx)),
        };
        self.pending_has_bytes = None;

        if !is_available {
            return self.as_mut().wait_for(&buffer, cx);
        }

        Poll::Ready(Some(self.read_data()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.stream_range.len() as f64;
        let total_buffers = length / DEFAULT_BUFFER_SIZE as f64;

        (0, Some(total_buffers.ceil() as usize))
    }
}

impl Stream for TorrentStream {
    fn range(&self) -> StreamRange {
        self.stream_range.clone()
    }

    fn resource_len(&self) -> u64 {
        self.resource_length.clone()
    }

    fn content_range(&self) -> String {
        let range = format!(
            "bytes {}-{}/{}",
            self.stream_range.start,
            self.stream_range.end.saturating_sub(1),
            self.resource_len()
        );

        trace!("Torrent stream {} has content range {{{}}}", self, &range);
        range
    }
}

impl Debug for TorrentStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FXTorrentStreamingResource")
            .field("torrent", &self.torrent)
            .field("torrent_filename", &self.torrent_filename)
            .field("torrent_offset", &self.torrent_offset)
            .field("file", &self.filepath)
            .field("resource_length", &self.resource_length)
            .field("cursor", &self.cursor)
            .field("stream_range", &self.stream_range)
            .field("waker", &self.waker)
            .field("cancellation_token", &self.cancellation_token)
            .finish()
    }
}

impl Drop for TorrentStream {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::torrents::MockTorrent;
    use crate::core::torrents::MockTorrentManager;
    use crate::core::torrents::TorrentHandle;
    use crate::create_torrent_file;
    use crate::init_logger;
    use crate::recv_timeout;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    mod filename {
        use super::*;

        #[tokio::test]
        async fn test_filename() {
            init_logger!();
            let filename = "test_filename.mp4";
            let pieces_len = 20;
            let mut torrent = MockTorrent::new();
            torrent.expect_handle().return_const(TorrentHandle::new());
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_total_pieces().return_const(pieces_len);
            torrent.expect_subscribe().returning(|| {
                let (_, rx) = unbounded_channel();
                rx
            });
            let torrent_manager = MockTorrentManager::new();
            let stream = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            let result = stream.filename();

            assert_eq!(filename, result);
        }
    }

    mod torrent_events {
        use super::*;
        use fx_torrent::Metrics;
        use tokio::sync::oneshot;
        use tokio::time::timeout;

        #[tokio::test]
        async fn test_on_state_event() {
            init_logger!();
            let filename = "test_filename.mp4";
            let pieces_len = 20;
            let callbacks = MultiThreadedCallback::new();
            let mut torrent = create_torrent(TorrentHandle::new(), pieces_len, false);
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_stats().return_const(Metrics::default());
            let subscribe_callbacks = callbacks.clone();
            torrent
                .expect_subscribe()
                .returning(move || subscribe_callbacks.subscribe());
            let torrent_manager = MockTorrentManager::new();
            let stream = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            // subscribe to the stream events
            let mut receiver = stream.subscribe();

            // invoke the state change event
            callbacks.invoke(TorrentEvent::StateChanged(TorrentState::Finished));

            let event = recv_timeout!(&mut receiver, Duration::from_millis(250));
            match &*event {
                StreamEvent::StateChanged(result) => {
                    assert_eq!(StreamState::Streaming, *result);
                }
                _ => assert!(
                    false,
                    "expected StreamEvent::StateChanged, but got {:?}",
                    event
                ),
            }
        }

        #[tokio::test]
        async fn test_on_stats_event() {
            init_logger!();
            let filename = "test_filename.mp4";
            let pieces_len = 20;
            let callbacks = MultiThreadedCallback::new();
            let mut torrent = create_torrent(TorrentHandle::new(), pieces_len, false);
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_stats().return_const(Metrics::default());
            let subscribe_callbacks = callbacks.clone();
            torrent
                .expect_subscribe()
                .returning(move || subscribe_callbacks.subscribe());
            let torrent_manager = MockTorrentManager::new();
            let stream = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            // subscribe to the stream events
            let mut receiver = stream.subscribe();
            let (tx, mut rx) = oneshot::channel();
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    if let StreamEvent::StatsChanged(stats) = &*event {
                        let _ = tx.send(stats.clone());
                        break;
                    }
                }
            });

            // invoke the state change event
            let stats = Metrics::default();
            stats.peers.set(16);
            callbacks.invoke(TorrentEvent::Stats(stats.clone()));

            let result = timeout(Duration::from_millis(250), &mut rx)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(16, result.connections);
        }
    }

    mod stop {
        use super::*;
        use crate::recv_timeout;
        use fx_torrent::Metrics;
        use std::time::Duration;

        #[tokio::test]
        async fn test_preparing() {
            init_logger!();
            let handle = TorrentHandle::new();
            let filename = "TorrentVideoFile.mp4";
            let pieces_len = 100;
            let (tx, mut rx) = unbounded_channel();
            let (_sender, receiver) = unbounded_channel();
            let mut torrent = create_torrent(handle, pieces_len, false);
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_subscribe().return_once(move || receiver);
            torrent.expect_stats().return_const({
                let metrics = Metrics::default();
                metrics.wanted_pieces.set(pieces_len as u64);
                metrics.wanted_completed_pieces.set(0);
                metrics
            });
            let mut torrent_manager = MockTorrentManager::new();
            torrent_manager
                .expect_remove()
                .times(1)
                .returning(move |handle| {
                    let _ = tx.send(*handle);
                });
            let stream = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            // subscribe to stream events
            let mut receiver = stream.subscribe();

            // stop the stream
            stream.stop().await;

            // wait for the state change event
            wait_for_state_event(&mut receiver, StreamState::Stopped).await;

            // verify the state on the stream
            let result = stream.state().await;
            assert_eq!(
                StreamState::Stopped,
                result,
                "expected the stream state to have been stopped"
            );

            // verify that the torrent was removed from the manager
            let result = recv_timeout!(
                &mut rx,
                Duration::from_millis(250),
                "expected the torrent to be removed from the manager"
            );
            assert_eq!(handle, result);
        }

        #[tokio::test]
        async fn test_streaming() {
            init_logger!();
            let handle = TorrentHandle::new();
            let filename = "TorrentVideoFile.mp4";
            let pieces_len = 100;
            let (tx, mut rx) = unbounded_channel();
            let (_sender, receiver) = unbounded_channel();
            let mut torrent = create_torrent(handle, pieces_len, true);
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_subscribe().return_once(move || receiver);
            torrent.expect_stats().return_const({
                let metrics = Metrics::default();
                metrics.wanted_pieces.set(pieces_len as u64);
                metrics.wanted_completed_pieces.set(0);
                metrics
            });
            let mut torrent_manager = MockTorrentManager::new();
            torrent_manager
                .expect_remove()
                .times(1)
                .returning(move |handle| {
                    let _ = tx.send(*handle);
                });
            let stream = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            // subscribe to stream events
            let mut receiver = stream.subscribe();

            // wait for the state change event
            wait_for_state_event(&mut receiver, StreamState::Streaming).await;

            // stop the stream
            stream.stop().await;

            // wait for the state change event
            wait_for_state_event(&mut receiver, StreamState::Stopped).await;

            // verify the state on the stream
            let result = stream.state().await;
            assert_eq!(
                StreamState::Stopped,
                result,
                "expected the stream state to have been stopped"
            );

            // verify that the torrent was removed from the manager
            let result = recv_timeout!(
                &mut rx,
                Duration::from_millis(250),
                "expected the torrent to be removed from the manager"
            );
            assert_eq!(handle, result);
        }

        async fn wait_for_state_event(
            receiver: &mut Subscription<StreamEvent>,
            expected_state: StreamState,
        ) {
            let event = recv_timeout!(
                receiver,
                Duration::from_millis(250),
                "expected a stream event"
            );
            match &*event {
                StreamEvent::StateChanged(result) => {
                    assert_eq!(expected_state, *result);
                }
                _ => assert!(
                    false,
                    "expected StreamEvent::StateChanged, but got {:?}",
                    event
                ),
            }
        }
    }

    mod prepare_pieces {
        use super::*;

        #[tokio::test]
        async fn test_calculate_preparation_pieces() {
            init_logger!();
            let filename = "simple.txt";
            let pieces_len = 150;
            let mut torrent = MockTorrent::new();
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            let torrent: Box<dyn Torrent> = Box::new(torrent);
            let mut expected_result = (0..12).into_iter().collect::<Vec<_>>();
            expected_result.append(&mut (147..150).into_iter().collect::<Vec<_>>());

            let result =
                InnerTorrentStreamingResource::preparation_pieces(&torrent, filename).await;

            assert_eq!(expected_result, result);
        }
    }

    mod poll_next {
        use super::*;
        use crate::core::stream::tests::read_stream;
        use crate::recv_timeout;
        use crate::testing::{copy_test_file, read_test_file_to_string};
        use fx_torrent::Metrics;
        use std::time::Duration;

        #[tokio::test]
        async fn test_bytes_unavailable() {
            init_logger!();
            let filename = "simple.txt";
            let pieces_len = 20;
            let handle = TorrentHandle::new();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().join(filename);
            let absolute_path = copy_test_file(temp_path.to_str().unwrap(), filename, None);
            let expected_result = read_test_file_to_string(filename);
            let mut has_bytes = Some(false);
            let callbacks = MultiThreadedCallback::new();
            let has_bytes_callback = callbacks.clone();
            let mut torrent = MockTorrent::new();
            torrent.expect_handle().return_const(handle);
            torrent
                .expect_state()
                .return_const(TorrentState::Downloading);
            torrent
                .expect_file_by_name()
                .returning(move |file| Some(create_torrent_file!(file, pieces_len)));
            torrent.expect_total_pieces().return_const(pieces_len);
            let subscribe_callback = callbacks.clone();
            torrent
                .expect_subscribe()
                .times(2)
                .returning(move || subscribe_callback.subscribe());
            torrent.expect_has_bytes().returning(move |_| {
                if let Some(has_bytes) = has_bytes.take() {
                    has_bytes_callback.invoke(TorrentEvent::PieceCompleted(2));
                    return has_bytes;
                }

                true
            });
            torrent.expect_stats().return_const(Metrics::default());
            torrent.expect_prioritize_pieces().returning(|_| {});
            torrent.expect_piece_priorities().returning(move || {
                (0..pieces_len)
                    .into_iter()
                    .map(|piece| (piece, PiecePriority::Normal))
                    .collect()
            });
            torrent
                .expect_absolute_file_path()
                .returning(move |_| PathBuf::from(absolute_path.as_str()));
            let torrent_manager = MockTorrentManager::new();
            let resource = TorrentStreamingResource::new(
                filename,
                Box::new(torrent),
                Arc::new(torrent_manager),
            )
            .await;

            // subscribe to the resource events
            let (tx, mut rx) = unbounded_channel();
            let mut receiver = resource.subscribe();
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    match &*event {
                        StreamEvent::StateChanged(state) => {
                            let _ = tx.send(*state);
                        }
                        _ => {}
                    }
                }
            });

            // invoke `PieceCompleted` for the preparation pieces
            let mut preparation_pieces = (0..2).into_iter().collect::<Vec<_>>();
            preparation_pieces.append(&mut (17..20).into_iter().collect::<Vec<_>>());
            for piece in preparation_pieces {
                callbacks.invoke(TorrentEvent::PieceCompleted(piece));
            }

            // wait for the state to change
            let result = recv_timeout!(&mut rx, Duration::from_millis(200));
            assert_eq!(StreamState::Streaming, result);

            // create a new stream from the resource and read it
            let stream = resource.stream().await.unwrap();
            let result = read_stream(stream).await;
            assert_eq!(expected_result, result);
        }
    }

    fn create_torrent(handle: TorrentHandle, pieces_len: usize, has_pieces: bool) -> MockTorrent {
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(handle);
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent.expect_has_bytes().returning(move |_| has_pieces);
        torrent.expect_has_piece().returning(move |_| has_pieces);
        torrent.expect_total_pieces().return_const(pieces_len);
        torrent.expect_prioritize_pieces().returning(|_| {});
        torrent.expect_piece_priorities().returning(move || {
            (0..pieces_len)
                .into_iter()
                .map(|piece| (piece, PiecePriority::Normal))
                .collect()
        });
        torrent.expect_sequential_mode().return_const(());
        torrent
    }

    #[macro_export]
    macro_rules! create_torrent_file {
        ($temp_path:expr) => {{
            use crate::create_torrent_file;
            create_torrent_file!($temp_path, 0)
        }};
        ($temp_path:expr, $pieces_len:expr) => {{
            use crate::create_torrent_file;
            create_torrent_file!($temp_path, $pieces_len, 1024)
        }};
        ($temp_path:expr, $pieces_len:expr, $file_len:expr) => {{
            use fx_torrent::{File, TorrentFileInfo};

            let torrent_path: PathBuf = Into::<PathBuf>::into($temp_path);
            let pieces_len: usize = $pieces_len;
            let length: u64 = $file_len;

            File {
                index: 0,
                torrent_path,
                torrent_offset: 0,
                info: TorrentFileInfo {
                    length,
                    path: None,
                    path_utf8: None,
                    md5sum: None,
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                },
                priority: Default::default(),
                pieces: 0..pieces_len,
            }
        }};
    }
}
