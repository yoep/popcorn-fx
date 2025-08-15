use crate::core::torrents;
use crate::core::torrents::{
    Error, StreamBytesResult, Torrent, TorrentEvent, TorrentHandle, TorrentState, TorrentStream,
    TorrentStreamEvent, TorrentStreamState, TorrentStreamingResource,
    TorrentStreamingResourceWrapper,
};
use async_trait::async_trait;
use derive_more::Display;
use futures::Future;
use futures::Stream;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use popcorn_fx_torrent::torrent;
use popcorn_fx_torrent::torrent::{FilePriority, PieceIndex, TorrentStats};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::pin::{pin, Pin};
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use url::Url;

/// The default buffer size, in bytes, used while streaming the file contents.
const DEFAULT_BUFFER_SIZE: usize = 131_072; // 128KB
const PREPARE_FILE_PERCENTAGE: f32 = 0.08; // 8%

/// The buffer byte range type.
type Buffer = std::ops::Range<usize>;

/// The default implementation of [TorrentStream] which provides a [Stream]
/// over a [File] resource.
///
/// It uses a buffer of [DEFAULT_BUFFER_SIZE] which is checked for availability through the
/// [Torrent] before it's returned.
#[derive(Debug)]
pub struct DefaultTorrentStream {
    /// The underlying handle of the torrent, also used to identify a torrent stream.
    handle: TorrentHandle,
    /// The reference type of the stream.
    ref_type: StreamRefType,
    /// The inner torrent stream instance.
    instance: Weak<TorrentStreamContext>,
}

impl DefaultTorrentStream {
    pub async fn new(url: Url, torrent: Box<dyn Torrent>, filename: &str) -> Self {
        let handle = torrent.handle();
        let inner = Arc::new(TorrentStreamContext::new(url, torrent, filename).await);
        let instance = Self {
            handle,
            ref_type: StreamRefType::Owner,
            instance: Arc::downgrade(&inner),
        };

        let torrent_event_receiver = inner.torrent.subscribe();
        tokio::spawn(async move {
            inner.start(torrent_event_receiver).await;
        });

        instance
    }

    /// Get an underlying instance reference of the torrent stream.
    fn instance(&self) -> Option<Arc<TorrentStreamContext>> {
        self.instance.upgrade()
    }
}

#[async_trait]
impl Torrent for DefaultTorrentStream {
    fn handle(&self) -> TorrentHandle {
        self.handle
    }

    fn absolute_file_path(&self, file: &torrent::File) -> PathBuf {
        if let Some(context) = self.instance() {
            return context.absolute_file_path(file);
        }

        PathBuf::new()
    }

    async fn files(&self) -> Vec<torrent::File> {
        if let Some(context) = self.instance() {
            return context.files().await;
        }

        Vec::with_capacity(0)
    }

    async fn file_by_name(&self, name: &str) -> Option<torrent::File> {
        if let Some(context) = self.instance() {
            return context.torrent.file_by_name(name).await;
        }

        None
    }

    async fn largest_file(&self) -> Option<torrent::File> {
        if let Some(context) = self.instance() {
            return context.largest_file().await;
        }

        None
    }

    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool {
        if let Some(context) = self.instance() {
            return context.has_bytes(bytes).await;
        }

        false
    }

    async fn has_piece(&self, piece: usize) -> bool {
        if let Some(context) = self.instance() {
            return context.has_piece(piece).await;
        }

        false
    }

    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>) {
        if let Some(context) = self.instance() {
            context.prioritize_bytes(bytes).await
        }
    }

    async fn prioritize_pieces(&self, pieces: &[PieceIndex]) {
        if let Some(context) = self.instance() {
            context.prioritize_pieces(pieces).await
        }
    }

    async fn total_pieces(&self) -> usize {
        if let Some(context) = self.instance() {
            return context.total_pieces().await;
        }

        0
    }

    async fn sequential_mode(&self) {
        if let Some(context) = self.instance() {
            context.sequential_mode().await
        }
    }

    async fn state(&self) -> TorrentState {
        if let Some(context) = self.instance() {
            return context.state().await;
        }

        TorrentState::Error
    }

    async fn stats(&self) -> TorrentStats {
        if let Some(context) = self.instance() {
            return context.stats().await;
        }

        TorrentStats::default()
    }
}

#[async_trait]
impl TorrentStream for DefaultTorrentStream {
    fn url(&self) -> Url {
        if let Some(context) = self.instance() {
            return context.url();
        }

        Url::parse("/").unwrap()
    }

    async fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper> {
        if let Some(context) = self.instance() {
            return context.stream().await;
        }

        Err(Error::InvalidHandle(self.handle.to_string()))
    }

    async fn stream_offset(
        &self,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<TorrentStreamingResourceWrapper> {
        if let Some(context) = self.instance() {
            return context.stream_offset(offset, len).await;
        }

        Err(Error::InvalidHandle(self.handle.to_string()))
    }

    async fn stream_state(&self) -> TorrentStreamState {
        if let Some(context) = self.instance() {
            return context.stream_state().await;
        }

        TorrentStreamState::Stopped
    }

    fn stop_stream(&self) {
        if let Some(context) = self.instance() {
            context.stop_stream()
        }
    }
}

impl Callback<TorrentEvent> for DefaultTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        if let Some(context) = self.instance() {
            return context.subscribe();
        }

        panic!("Unable to subscribe to a dropped torrent stream")
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        if let Some(context) = self.instance() {
            context.subscribe_with(subscriber)
        }
    }
}

impl Callback<TorrentStreamEvent> for DefaultTorrentStream {
    fn subscribe(&self) -> Subscription<TorrentStreamEvent> {
        if let Some(context) = self.instance() {
            return context.subscribe();
        }

        panic!("Unable to subscribe to a dropped torrent stream")
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentStreamEvent>) {
        if let Some(context) = self.instance() {
            context.subscribe_with(subscriber)
        }
    }
}

impl Clone for DefaultTorrentStream {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            ref_type: StreamRefType::Borrowed,
            instance: self.instance.clone(),
        }
    }
}

impl Drop for DefaultTorrentStream {
    fn drop(&mut self) {
        if self.ref_type == StreamRefType::Owner {
            if let Some(context) = self.instance.upgrade() {
                context.cancellation_token.cancel();
            }
        }
    }
}

/// The type of the stream reference.
#[derive(Debug, Clone, Copy, PartialEq)]
enum StreamRefType {
    Owner,
    Borrowed,
}

#[derive(Debug, Display)]
#[display(fmt = "{}", "torrent.handle()")]
struct TorrentStreamContext {
    /// The backing torrent of this stream
    torrent: Arc<Box<dyn Torrent>>,
    /// The underlying used filename within the torrent that is being streamed
    torrent_filename: String,
    /// The url on which this stream is being hosted
    url: Url,
    /// The pieces which should be prepared for the stream
    preparing_pieces: Arc<Mutex<Vec<PieceIndex>>>,
    /// The state of this stream
    state: Arc<Mutex<TorrentStreamState>>,
    /// The callbacks for this stream
    callbacks: MultiThreadedCallback<TorrentStreamEvent>,
    /// The cancellation token of the torrent stream
    cancellation_token: CancellationToken,
}

impl TorrentStreamContext {
    /// Create a new torrent streaming context for the given torrent and file.
    async fn new(url: Url, torrent: Box<dyn Torrent>, filename: &str) -> Self {
        let prepare_pieces = Self::preparation_pieces(&torrent).await;

        Self {
            torrent: Arc::new(torrent),
            torrent_filename: filename.to_string(),
            url,
            preparing_pieces: Arc::new(Mutex::new(prepare_pieces)),
            state: Arc::new(Mutex::new(TorrentStreamState::Preparing)),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
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
                if state == &TorrentState::Finished {
                    self.update_state(TorrentStreamState::Streaming).await;
                } else {
                    self.verify_ready_to_stream().await;
                }
            }
            TorrentEvent::PieceCompleted(piece) => self.on_piece_finished(*piece).await,
            TorrentEvent::Stats(status) => self.on_download_status(status.clone()),
            _ => {}
        }
    }

    /// Prepare the initial pieces required for the torrent stream to be able to start.
    async fn start_preparing_pieces(&self) {
        let state = self.torrent.state().await;
        let stats = self.torrent.stats().await;
        trace!(
            "Torrent stream {} preparation with torrent state {}",
            self,
            state
        );
        let is_finished = state == TorrentState::Finished || stats.progress() == 1.0;

        if is_finished {
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

    fn on_download_status(&self, download_status: TorrentStats) {
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

    async fn preparation_pieces(torrent: &Box<dyn Torrent>) -> Vec<PieceIndex> {
        if let Some(file) = torrent
            .files()
            .await
            .iter()
            .find(|e| e.priority != FilePriority::None)
        {
            let total_file_pieces = file.pieces.len();
            trace!(
                "Calculating preparation pieces of {:?} for a total of {} pieces",
                file,
                total_file_pieces
            );
            // prepare at least 8 (if available), or the ceil of the PREPARE_FILE_PERCENTAGE
            let prepare_at_least = min(8, total_file_pieces);
            let percentage_count =
                ((total_file_pieces as f32) * PREPARE_FILE_PERCENTAGE).ceil() as usize;
            let number_of_preparation_pieces =
                max(prepare_at_least, percentage_count).min(total_file_pieces);
            let mut pieces = vec![];

            // prepare the first `PREPARE_FILE_PERCENTAGE` of pieces if it doesn't exceed the total file pieces
            let starting_section_start = file.pieces.start;
            let starting_section_end = file
                .pieces
                .start
                .saturating_add(number_of_preparation_pieces)
                .min(file.pieces.end);
            for i in starting_section_start..starting_section_end {
                pieces.push(i);
            }

            // prepare the last 3 pieces
            // this is done for determining the video length during streaming
            for i in file.pieces.end.saturating_sub(3)..file.pieces.end {
                pieces.push(i);
            }

            if pieces.is_empty() {
                warn!("Unable to prepare stream, pieces to prepare couldn't be determined");
            }

            pieces.into_iter().map(|e| e).unique().collect()
        } else {
            Vec::with_capacity(0)
        }
    }
}

impl Callback<TorrentEvent> for TorrentStreamContext {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        self.torrent.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        self.torrent.subscribe_with(subscriber)
    }
}

impl Callback<TorrentStreamEvent> for TorrentStreamContext {
    fn subscribe(&self) -> Subscription<TorrentStreamEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentStreamEvent>) {
        self.callbacks.subscribe_with(subscriber)
    }
}

#[async_trait]
impl Torrent for TorrentStreamContext {
    fn handle(&self) -> TorrentHandle {
        self.torrent.handle()
    }

    fn absolute_file_path(&self, file: &torrent::File) -> PathBuf {
        self.torrent.absolute_file_path(file)
    }

    async fn files(&self) -> Vec<torrent::File> {
        self.torrent.files().await
    }

    async fn file_by_name(&self, name: &str) -> Option<torrent::File> {
        self.torrent.file_by_name(name).await
    }

    async fn largest_file(&self) -> Option<torrent::File> {
        self.torrent.largest_file().await
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

    async fn prioritize_pieces(&self, pieces: &[PieceIndex]) {
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

    async fn stats(&self) -> TorrentStats {
        self.torrent.stats().await
    }
}

#[async_trait]
impl TorrentStream for TorrentStreamContext {
    fn url(&self) -> Url {
        self.url.clone()
    }

    async fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper> {
        if !self.cancellation_token.is_cancelled() {
            let mutex = self.state.lock().await;
            if *mutex == TorrentStreamState::Streaming {
                FXTorrentStreamingResource::new(self.torrent.clone(), &self.torrent_filename)
                    .await
                    .map(|e| TorrentStreamingResourceWrapper::new(e))
            } else {
                Err(Error::InvalidStreamState(*mutex))
            }
        } else {
            Err(Error::InvalidStreamState(TorrentStreamState::Stopped))
        }
    }

    async fn stream_offset(
        &self,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<TorrentStreamingResourceWrapper> {
        let mutex = self.state.lock().await;
        if *mutex == TorrentStreamState::Streaming {
            FXTorrentStreamingResource::new_offset(
                self.torrent.clone(),
                &self.torrent_filename,
                offset,
                len,
            )
            .await
            .map(|e| TorrentStreamingResourceWrapper::new(e))
        } else {
            Err(Error::InvalidStreamState(mutex.clone()))
        }
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
#[display(fmt = "{}", "torrent.handle()")]
pub struct FXTorrentStreamingResource {
    torrent: Arc<Box<dyn Torrent>>,
    torrent_filename: String,
    /// The open reader handle to the torrent file
    file: File,
    /// The absolute path to the torrent file
    filepath: PathBuf,
    /// The total length of the file resource.
    resource_length: u64,
    /// The current reading cursor for the stream
    cursor: usize,
    /// The starting offset of the stream
    offset: u64,
    /// The total len of the stream
    len: u64,
}

impl FXTorrentStreamingResource {
    /// Create a new streaming resource which will read the full [Torrent].
    pub async fn new(torrent: Arc<Box<dyn Torrent>>, filename: &str) -> torrents::Result<Self> {
        Self::new_offset(torrent, filename, 0, None).await
    }

    /// Create a new streaming resource for the given offset.
    /// If no `len` is given, the streaming resource will be read till it's end.
    pub async fn new_offset(
        torrent: Arc<Box<dyn Torrent>>,
        filename: &str,
        offset: u64,
        len: Option<u64>,
    ) -> torrents::Result<Self> {
        let torrent = torrent.clone();
        let handle = torrent.handle();

        trace!(
            "Torrent streaming resource {} is being created for {:?}",
            handle,
            torrent
        );
        let files = torrent.files().await;

        // check if the torrent handle is still valid
        if files.is_empty() {
            debug!(
                "Torrent streaming resource {} failed to create, invalid handle",
                handle
            );
            return Err(Error::InvalidHandle(handle.to_string()));
        }

        // try to find the given filename within the torrent
        trace!(
            "Torrent streaming resource {} is searching for file {} in {:?}",
            handle,
            filename,
            files
        );
        if let Some(torrent_file) = files
            .into_iter()
            .find(|e| Self::normalize(e.filename()) == Self::normalize(filename))
        {
            let absolute_filepath = torrent.absolute_file_path(&torrent_file);
            trace!(
                "Torrent streaming resource {} is opening file {:?}",
                handle,
                absolute_filepath
            );
            fs::OpenOptions::new()
                .read(true)
                .open(&absolute_filepath)
                .map_err(|e| {
                    warn!(
                        "Torrent streaming resource {} failed to open torrent file {:?}, {}",
                        handle, absolute_filepath, e
                    );
                    Error::FileNotFound(filename.to_string())
                })
                .and_then(|mut file| {
                    let resource_length = Self::file_bytes(&mut file)?;
                    let mut stream_length = len.unwrap_or_else(|| resource_length);
                    let stream_end = offset + stream_length;

                    if stream_end > resource_length {
                        warn!(
                            "Requested stream range ({}-{}) is larger than {} resource length",
                            &offset, &stream_end, &resource_length
                        );
                        stream_length = resource_length - offset;
                    }

                    Ok(Self {
                        torrent,
                        torrent_filename: filename.to_string(),
                        file,
                        filepath: absolute_filepath,
                        resource_length,
                        cursor: offset as usize,
                        offset,
                        len: stream_length,
                    })
                })
        } else {
            debug!(
                "Torrent streaming resource {} failed to create, file {} not found",
                handle, filename
            );
            Err(Error::FileNotFound(filename.to_string()))
        }
    }

    /// Wait for the current cursor to become available.
    fn wait_for(&mut self, cx: &mut Context) -> Poll<Option<StreamBytesResult>> {
        let waker = cx.waker().clone();
        let mut buffer = self.next_buffer();

        // request the torrent file info
        let file_info =
            match pin!(self.torrent.file_by_name(self.torrent_filename.as_str())).poll(cx) {
                Poll::Ready(e) => e,
                Poll::Pending => return Poll::Pending,
            };

        // make the buffer relative to the offset within the torrent
        if let Some(file_info) = file_info {
            buffer = buffer.start + file_info.offset..buffer.end + file_info.offset;
        } else {
            warn!("Torrent streaming resource {} is unable to update buffer info, torrent file info not found", self);
        }

        // request the torrent to prioritize the buffer
        debug!(
            "Torrent streaming resource {} is waiting for buffer {{{}-{}}} to be available",
            self, &buffer.start, &buffer.end
        );
        match pin!(self.torrent.prioritize_bytes(&buffer)).poll(cx) {
            Poll::Ready(_) => {}
            Poll::Pending => return Poll::Pending,
        }

        let mut receiver = self.torrent.subscribe();
        // FIXME: this is very insufficient and should be refactored to a more global waker
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    TorrentEvent::StateChanged(_) | TorrentEvent::PieceCompleted(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            waker.wake_by_ref();
        });

        Poll::Pending
    }

    /// Read the data of the stream at the current cursor.
    fn read_data(&mut self) -> Option<StreamBytesResult> {
        let buffer_size = self.calculate_buffer_size();
        let reader = &mut self.file;
        let cursor = self.cursor.clone();
        let mut buffer = vec![0; buffer_size];

        if let Err(e) = reader.seek(SeekFrom::Start(cursor as u64)) {
            error!(
                "Torrent streaming resource {} failed to modify the file cursor to {}, {}",
                self, &self.cursor, e
            );
            return Some(Err(Error::Io(e.to_string())));
        }

        match reader.read(&mut buffer) {
            Err(e) => {
                error!(
                    "Torrent streaming resource {} failed to read the file cursor data, {}",
                    self, e
                );
                Some(Err(Error::Io(e.to_string())))
            }
            Ok(size) => {
                if size == 0 {
                    trace!(
                        "Torrent streaming resource {} reached EOF for {:?}",
                        self,
                        &self.filepath
                    );
                    return None;
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
                Some(Ok(buffer))
            }
        }
    }

    fn calculate_buffer_size(&self) -> usize {
        let cursor = self.cursor.clone();
        let range_end = (self.offset + self.len) as usize;

        range_end.saturating_sub(cursor).min(DEFAULT_BUFFER_SIZE)
    }

    /// Get the next buffer byte range.
    ///
    /// It returns the [Buffer] range.
    fn next_buffer(&self) -> Buffer {
        let mut buffer_end_byte = self.cursor + DEFAULT_BUFFER_SIZE;
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
                error!(
                    "Torrent streaming resource failed to determine the file length, {}",
                    e
                );
                Err(Error::Io(e.to_string()))
            }
        }
    }

    /// Normalize the given string slice value.
    /// It returns a normalized value lowercased and trimmed.
    fn normalize<S: AsRef<str>>(value: S) -> String {
        value.as_ref().trim().to_lowercase()
    }
}

impl TorrentStreamingResource for FXTorrentStreamingResource {
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

        trace!(
            "Torrent streaming resource {} has content range {{{}}}",
            self,
            &range
        );
        range
    }
}

impl Stream for FXTorrentStreamingResource {
    type Item = StreamBytesResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // check if the current cursor position is out-of-bounds,
        // if so, return additional bytes
        if self.cursor as u64 >= self.offset + self.len {
            return Poll::Ready(None);
        }

        let buffer = self.next_buffer();
        let is_available = match pin!(self.torrent.has_bytes(&buffer)).poll(cx) {
            Poll::Ready(e) => e,
            Poll::Pending => return Poll::Pending,
        };
        trace!(
            "Torrent streaming resource {} buffer {:?} {}",
            self,
            buffer,
            if is_available {
                "is available"
            } else {
                "is not available"
            }
        );
        if !is_available {
            return self.as_mut().wait_for(cx);
        }

        Poll::Ready(self.as_mut().read_data())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.content_length() as f64;
        let total_buffers = length / DEFAULT_BUFFER_SIZE as f64;

        (0, Some(total_buffers.ceil() as usize))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::core::torrents::{MockTorrent, StreamBytes};
    use crate::testing::{copy_test_file, read_test_file_to_string};
    use crate::{assert_timeout, init_logger, recv_timeout};

    use futures::TryStreamExt;
    use popcorn_fx_torrent::torrent::{PieceIndex, TorrentFileInfo};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_torrent_stream_stream() {
        init_logger!();
        let filename = "simple.txt";
        let total_pieces = 10usize;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let torrent_handle = TorrentHandle::new();
        let mut mock = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let callbacks = MultiThreadedCallback::new();
        let subscription_callbacks = callbacks.clone();
        let (tx_ready, rx_ready) = channel();
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        mock.expect_handle().return_const(torrent_handle);
        mock.expect_files().returning(move || files.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_has_piece().return_const(true);
        mock.expect_total_pieces().return_const(total_pieces);
        mock.expect_prioritize_pieces()
            .returning(|_: &[PieceIndex]| {});
        mock.expect_sequential_mode().returning(|| {});
        mock.expect_state().return_const(TorrentState::Downloading);
        mock.expect_stats().returning(|| TorrentStats::default());
        mock.expect_subscribe()
            .returning(move || subscription_callbacks.subscribe());
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(mock), filename).await;

        // listen on the streaming state event
        let mut receiver = Callback::<TorrentStreamEvent>::subscribe(&torrent_stream);
        tokio::spawn(async move {
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

        // update the completed pieces
        for i in 0..total_pieces {
            callbacks.invoke(TorrentEvent::PieceCompleted(i));
        }
        assert_timeout!(
            Duration::from_millis(250),
            torrent_stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to be streaming"
        );

        let _ = rx_ready
            .recv_timeout(Duration::from_millis(500))
            .expect("expected the stream to enter the streaming state");
        let result = torrent_stream
            .stream()
            .await
            .expect("expected a stream wrapper");

        assert_eq!(0, result.resource().offset());
        assert_eq!(
            result.resource().total_length(),
            result.resource().content_length()
        );
    }

    #[tokio::test]
    async fn test_content_range() {
        init_logger!();
        let filename = "range.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let file_info = create_file_from_temp_path(temp_path);
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent
            .expect_files()
            .returning(move || vec![file_info.clone()]);
        torrent.expect_has_bytes().return_const(true);
        let torrent = Arc::new(Box::new(torrent) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let stream = FXTorrentStreamingResource::new(torrent, filename)
            .await
            .unwrap();
        let bytes = read_test_file_to_string(filename).as_bytes().len();
        let expected_result = format!("bytes 0-{}/{}", bytes - 1, bytes);

        let result = stream.content_range();

        assert_eq!(expected_result, result.as_str())
    }

    #[tokio::test]
    async fn test_offset() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent.expect_files().return_once(move || files);
        torrent.expect_has_bytes().return_const(true);
        let torrent = Arc::new(Box::new(torrent) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let stream = FXTorrentStreamingResource::new_offset(torrent, filename, 1, Some(3))
            .await
            .unwrap();

        let result = read_stream(stream).await;

        assert_eq!("ore".to_string(), result)
    }

    #[tokio::test]
    async fn test_poll_mismatching_buffer_size() {
        init_logger!();
        let filename = "mismatch.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let file = create_file_from_temp_path(temp_path.clone());
        let files = vec![file.clone()];
        let callbacks = MultiThreadedCallback::new();
        let callback_receiver = callbacks.subscribe();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent.expect_files().return_once(move || files);
        torrent.expect_has_bytes().returning(move |_| {
            if a.is_some() {
                a.take();
                callbacks.invoke(TorrentEvent::PieceCompleted(1));
                return false;
            }

            true
        });
        torrent.expect_prioritize_bytes().times(1).return_const(());
        torrent
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        torrent
            .expect_file_by_name()
            .times(1)
            .returning(move |_| Some(file.clone()));
        let torrent = Arc::new(Box::new(torrent) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file_to_string(filename);

        let stream = FXTorrentStreamingResource::new(torrent, filename)
            .await
            .unwrap();

        let range = stream.content_range();
        let result = read_stream(stream).await;

        assert_eq!("bytes 0-29/30", range);
        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_poll_next_byte_not_present() {
        init_logger!();
        let filename = "simple.txt";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let file = create_file_from_temp_path(temp_path.clone());
        let files = vec![file.clone()];
        let callback = MultiThreadedCallback::new();
        let callback_subscription = callback.subscribe();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent.expect_files().return_once(move || files);
        torrent.expect_has_bytes().returning(move |_| {
            if a.is_some() {
                a.take();
                callback.invoke(TorrentEvent::PieceCompleted(2));
                return false;
            }

            true
        });
        torrent.expect_prioritize_bytes().return_const(());
        torrent
            .expect_file_by_name()
            .times(1)
            .returning(move |_| Some(file.clone()));
        torrent
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_subscription);
        let torrent = Arc::new(Box::new(torrent) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file_to_string(filename);

        let stream = FXTorrentStreamingResource::new(torrent, filename)
            .await
            .unwrap();

        let result = read_stream(stream).await;

        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_torrent_stream_prepare_pieces() {
        init_logger!();
        let filename = "lorem.ipsum";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut torrent = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let (tx, mut rx) = unbounded_channel();
        let callbacks = MultiThreadedCallback::new();
        let subscribe_callbacks = callbacks.clone();
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent.expect_files().return_once(move || files);
        torrent.expect_has_bytes().return_const(true);
        torrent.expect_has_piece().return_const(false);
        torrent
            .expect_prioritize_pieces()
            .returning(move |pieces: &[PieceIndex]| {
                tx.send(pieces.to_vec()).unwrap();
            });
        torrent.expect_sequential_mode().times(1).returning(|| {});
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent
            .expect_stats()
            .returning(|| torrent_stats_not_completed());
        torrent
            .expect_subscribe()
            .returning(move || subscribe_callbacks.subscribe());
        torrent.expect_files().returning(move || {
            vec![torrent::File {
                index: 0,
                torrent_path: Default::default(),
                offset: 0,
                info: TorrentFileInfo {
                    length: 0,
                    path: None,
                    path_utf8: None,
                    md5sum: None,
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                },
                priority: FilePriority::Normal,
                pieces: 0..101,
            }]
        });
        let stream = DefaultTorrentStream::new(url, Box::new(torrent), filename).await;
        let expected_pieces: Vec<PieceIndex> = vec![0, 1, 2, 3, 4, 5, 6, 7, 97, 98, 99];

        let pieces = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(expected_pieces.clone(), pieces);

        let (tx, mut rx) = unbounded_channel();
        let mut receiver = Callback::<TorrentStreamEvent>::subscribe(&stream);
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let TorrentStreamEvent::StateChanged(state) = &*event {
                    tx.send(state.clone()).unwrap();
                    break;
                }
            }
        });

        for piece in expected_pieces {
            callbacks.invoke(TorrentEvent::PieceCompleted(piece as PieceIndex));
        }

        let state_result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(TorrentStreamState::Streaming, state_result)
    }

    #[tokio::test]
    async fn test_torrent_start_preparing_pieces_torrent_state_finished() {
        init_logger!();
        let filename = "lorem.ipsum";
        let total_pieces = 100usize;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let callbacks = MultiThreadedCallback::new();
        let callback_subscription = callbacks.subscribe();
        let url = Url::parse("http://localhost").unwrap();
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        mock.expect_handle().return_const(TorrentHandle::new());
        mock.expect_files().returning(move || files.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_has_piece().return_const(false);
        mock.expect_total_pieces().return_const(total_pieces);
        mock.expect_prioritize_pieces()
            .times(0)
            .returning(|_: &[PieceIndex]| {});
        mock.expect_state().return_const(TorrentState::Finished);
        mock.expect_stats()
            .returning(|| torrent_stats_not_completed());
        mock.expect_subscribe()
            .return_once(move || callback_subscription);
        let stream = DefaultTorrentStream::new(url, Box::new(mock), filename).await;

        // check if the initial state automatically becomes streaming as the torrent is in finished state
        // this can however take some milliseconds as it's checked async after the resource is created
        assert_timeout!(
            Duration::from_millis(200),
            stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to be streaming"
        );
    }

    #[tokio::test]
    async fn test_torrent_start_preparing_pieces_torrent_stats_progress_100() {
        init_logger!();
        let filename = "lorem.ipsum";
        let total_pieces = 100usize;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut mock = MockTorrent::new();
        let callbacks = MultiThreadedCallback::new();
        let callback_subscription = callbacks.subscribe();
        let url = Url::parse("http://localhost").unwrap();
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        mock.expect_handle().return_const(TorrentHandle::new());
        mock.expect_files().returning(move || files.clone());
        mock.expect_has_bytes().return_const(true);
        mock.expect_has_piece().return_const(false);
        mock.expect_total_pieces().return_const(total_pieces);
        mock.expect_prioritize_pieces()
            .times(0)
            .returning(|_: &[PieceIndex]| {});
        mock.expect_state().return_const(TorrentState::Seeding);
        mock.expect_stats().returning(|| TorrentStats {
            upload: 0,
            upload_rate: 0,
            upload_useful: 0,
            upload_useful_rate: 0,
            download: 0,
            download_rate: 0,
            download_useful: 0,
            download_useful_rate: 0,
            total_uploaded: 0,
            total_downloaded: 0,
            total_downloaded_useful: 0,
            wanted_pieces: 30,
            completed_pieces: 30,
            total_size: 15000,
            total_completed_size: 15000,
            total_peers: 15,
        });
        mock.expect_subscribe()
            .return_once(move || callback_subscription);
        let stream = DefaultTorrentStream::new(url, Box::new(mock), filename).await;

        // check if the initial state automatically becomes streaming as the torrent is in finished state
        // this can however take some milliseconds as it's checked async after the resource is created
        assert_timeout!(
            Duration::from_millis(200),
            stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to be streaming"
        );
    }

    #[tokio::test]
    async fn test_stop_stream() {
        init_logger!();
        let filename = "simple.txt";
        let total_pieces = 35usize;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let torrent_handle = TorrentHandle::new();
        let callbacks = MultiThreadedCallback::new();
        let callback_subscription = callbacks.subscribe();
        let mut torrent = MockTorrent::new();
        let url = Url::parse("http://localhost").unwrap();
        let files = vec![create_file_from_temp_path(temp_path.clone())];
        torrent.expect_handle().return_const(torrent_handle);
        torrent.expect_files().returning(move || files.clone());
        torrent.expect_has_bytes().return_const(true);
        torrent.expect_has_piece().return_const(false);
        torrent.expect_sequential_mode().return_const(());
        torrent.expect_total_pieces().return_const(total_pieces);
        torrent
            .expect_prioritize_pieces()
            .returning(|_: &[PieceIndex]| {});
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent.expect_stats().returning(|| TorrentStats::default());
        torrent
            .expect_subscribe()
            .return_once(move || callback_subscription);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let torrent_stream = DefaultTorrentStream::new(url, Box::new(torrent), filename).await;

        // update the completed pieces
        for i in 0..total_pieces {
            callbacks.invoke(TorrentEvent::PieceCompleted(i));
        }
        assert_timeout!(
            Duration::from_millis(250),
            torrent_stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to be streaming"
        );

        torrent_stream.stop_stream();
        let result = torrent_stream
            .stream()
            .await
            .err()
            .expect("expected an error to be returned");

        match result {
            Error::InvalidStreamState(state) => {
                assert_eq!(TorrentStreamState::Stopped, state)
            }
            _ => assert!(false, "expected TorrentError::InvalidStreamState"),
        }
    }

    async fn read_stream(mut stream: FXTorrentStreamingResource) -> String {
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

        String::from_utf8(result).expect("expected a valid string")
    }

    fn create_file_from_temp_path(temp_path: PathBuf) -> torrent::File {
        torrent::File {
            index: 0,
            torrent_path: temp_path.clone(),
            offset: 0,
            info: TorrentFileInfo {
                length: 0,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
            pieces: 0..100,
        }
    }

    fn torrent_stats_not_completed() -> TorrentStats {
        TorrentStats {
            upload: 0,
            upload_rate: 0,
            upload_useful: 0,
            upload_useful_rate: 0,
            download: 0,
            download_rate: 0,
            download_useful: 0,
            download_useful_rate: 0,
            total_uploaded: 0,
            total_downloaded: 0,
            total_downloaded_useful: 0,
            wanted_pieces: 10,
            completed_pieces: 0,
            total_size: 0,
            total_completed_size: 0,
            total_peers: 0,
        }
    }
}
