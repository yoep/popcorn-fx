use crate::core::stream;
use crate::core::stream::{
    Error, FxStream, StreamEvent, StreamState, StreamStats, StreamingResource,
};
use crate::core::torrents::{Torrent, TorrentManager};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscription};
use fx_torrent::{FileStream, PieceIndex, PiecePriority, TorrentEvent, TorrentState};
use itertools::Itertools;
use log::{debug, trace, warn};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::io;
use std::sync::Arc;
use tokio::select;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

/// The default buffer size, in bytes, used while streaming the file contents.
const PREPARE_FILE_PERCENTAGE: f32 = 0.08; // 8%

/// Streams the contents of a single torrent file.
pub type TorrentStream = FileStream;

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
}

#[async_trait]
impl StreamingResource for TorrentStreamingResource {
    fn filename(&self) -> &str {
        self.inner.filename.as_str()
    }

    async fn stream(&self) -> stream::Result<FxStream> {
        self.stream_range(0, None).await
    }

    async fn stream_range(&self, start: u64, _end: Option<u64>) -> stream::Result<FxStream> {
        self.inner.assert_stream_state().await?;
        let file = match self.inner.torrent.file_by_name(&self.inner.filename).await {
            None => return Err(Error::NotFound(self.inner.filename.clone())),
            Some(file) => file,
        };
        let mut stream = match self.inner.torrent.stream(&file).await {
            Err(e) => return Err(Error::Io(io::Error::new(io::ErrorKind::Other, e))),
            Ok(stream) => stream,
        };

        if let Err(e) = stream.seek(start as usize) {
            return Err(Error::Io(io::Error::new(io::ErrorKind::Other, e)));
        };

        Ok(stream.into())
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
                Ok(event) = receiver.recv() => self.on_event(&event).await,
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
        let is_finished = state == TorrentState::Finished || stats.progress() == 1.0;
        trace!(
            "Torrent stream {} preparation with torrent state {}",
            self,
            state
        );
        let is_finished = priorities
            .iter()
            .any(|(_, priority)| *priority != PiecePriority::None)
            && (is_finished);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::torrents::MockTorrent;
    use crate::core::torrents::MockTorrentManager;
    use crate::core::torrents::TorrentHandle;
    use crate::create_torrent_file;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    mod filename {
        use super::*;
        use tokio::sync::broadcast;

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
            torrent.expect_subscribe().returning(|| {
                let (_, rx) = broadcast::channel(64);
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

            let event = timeout!(receiver.recv(), Duration::from_millis(250)).unwrap();
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
                while let Ok(event) = receiver.recv().await {
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
        use tokio::sync::broadcast;

        #[tokio::test]
        async fn test_preparing() {
            init_logger!();
            let handle = TorrentHandle::new();
            let filename = "TorrentVideoFile.mp4";
            let pieces_len = 100;
            let (tx, mut rx) = unbounded_channel();
            let (_sender, receiver) = broadcast::channel(64);
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
            let (_sender, receiver) = broadcast::channel(64);
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
            let event = timeout!(
                receiver.recv(),
                Duration::from_millis(250),
                "expected a stream event"
            )
            .unwrap();
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

    fn create_torrent(handle: TorrentHandle, pieces_len: usize, has_pieces: bool) -> MockTorrent {
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(handle);
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent.expect_has_piece().returning(move |_| has_pieces);
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
            use std::path::PathBuf;

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
