use crate::core::loader::loading_chain::{LoadingChain, Order};
use crate::core::loader::task::LoadingTask;
use crate::core::loader::{LoadingData, LoadingEvent, LoadingStrategy};
use crate::core::media::{
    Episode, Images, MediaIdentifier, MediaOverview, MovieDetails, ShowDetails,
};
use crate::core::playlist::PlaylistItem;
use crate::core::torrents;
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, error, trace};
#[cfg(any(test, feature = "testing"))]
pub use mock::*;
use popcorn_fx_torrent::torrent::TorrentStats;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Represents the result of a loading operation.
///
/// It is a specialized `Result` type where the success variant contains a value of type `T`, and the error variant
/// contains a `LoadingError` indicating the reason for the loading failure.
pub type Result<T> = std::result::Result<T, LoadingError>;

/// An enum representing events related to media loading.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum MediaLoaderEvent {
    /// Indicates that loading has started for a media item with the associated event details.
    #[display(fmt = "Loading started for {}", _1)]
    LoadingStarted(LoadingHandle, LoadingStartedEvent),
    /// Indicates a change in the loading state with the associated event details.
    #[display(fmt = "Loading state changed to {}", _1)]
    StateChanged(LoadingHandle, LoadingState),
    /// Indicates a change in loading progress with the associated event details.
    #[display(fmt = "Loading progress changed to {}", _1)]
    ProgressChanged(LoadingHandle, LoadingProgress),
    /// Indicates that an error has occurred during loading with the associated error details.
    #[display(fmt = "Loading {} encountered an error, {}", _0, _1)]
    LoadingError(LoadingHandle, LoadingError),
}

/// Represents the result of a loading strategy's processing.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingResult {
    /// Indicates that processing was successful and provides the resulting `PlaylistItem`.
    Ok,
    /// Indicates that processing has completed.
    Completed,
    /// Indicates an error during processing and includes an associated `LoadingError`.
    Err(LoadingError),
}

/// An enum representing the result of a cancellation operation on loading data.
pub type CancellationResult = std::result::Result<LoadingData, LoadingError>;

#[derive(Debug, Copy, Clone, Display, PartialOrd, PartialEq)]
pub enum LoadingState {
    #[display(fmt = "Loader is initializing")]
    Initializing,
    #[display(fmt = "Loader is starting")]
    Starting,
    #[display(fmt = "Loader is retrieving subtitles")]
    RetrievingSubtitles,
    #[display(fmt = "Loader is downloading a subtitle")]
    DownloadingSubtitle,
    #[display(fmt = "Loader is retrieving the metadata")]
    RetrievingMetadata,
    #[display(fmt = "Loader is verifying the files")]
    VerifyingFiles,
    #[display(fmt = "Loader is connecting")]
    Connecting,
    #[display(fmt = "Loader is downloading the media")]
    Downloading,
    #[display(fmt = "Loader has finished downloading the media")]
    DownloadFinished,
    #[display(fmt = "Loader is ready to start the playback")]
    Ready,
    #[display(fmt = "Loader is playing media")]
    Playing,
    #[display(fmt = "Loader is cancelled")]
    Cancelled,
}

#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "url: {}, title: {}, thumbnail: {:?}", url, title, thumbnail)]
pub struct LoadingStartedEvent {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub background: Option<String>,
    pub quality: Option<String>,
}

impl LoadingStartedEvent {
    fn background(
        parent: Option<&Box<dyn MediaIdentifier>>,
        media: &Box<dyn MediaIdentifier>,
    ) -> Option<String> {
        if let Some(parent) = parent {
            let images: &Images;

            if let Some(e) = parent.downcast_ref::<ShowDetails>() {
                images = e.images();
            } else {
                return None;
            }

            Some(images.fanart().to_string())
        } else {
            if let Some(e) = media.downcast_ref::<MovieDetails>() {
                Some(e.images().fanart().to_string())
            } else {
                None
            }
        }
    }

    fn thumbnail(media: &Box<dyn MediaIdentifier>) -> Option<String> {
        if let Some(e) = media.downcast_ref::<Episode>() {
            e.thumb().cloned()
        } else if let Some(e) = media.downcast_ref::<MovieDetails>() {
            Some(e.images().poster().to_string())
        } else {
            None
        }
    }
}

impl From<&LoadingData> for LoadingStartedEvent {
    fn from(value: &LoadingData) -> Self {
        let title = value.title.clone().unwrap_or(String::new());

        Self {
            url: value
                .url
                .as_ref()
                .map(|e| e.clone())
                .unwrap_or(String::new()),
            title,
            thumbnail: value.media.as_ref().and_then(Self::thumbnail),
            background: value
                .media
                .as_ref()
                .and_then(|e| Self::background(value.parent_media.as_ref(), e)),
            quality: value.quality.clone(),
        }
    }
}

#[derive(Debug, Clone, Display, PartialEq)]
#[display(
    fmt = "progress: {}, seeds: {}, peers: {}, download_speed: {}",
    progress,
    seeds,
    peers,
    download_speed
)]
pub struct LoadingProgress {
    /// Progress indication between 0 and 1 that represents the progress of the download.
    pub progress: f32,
    /// The number of seeds available for the torrent.
    pub seeds: usize,
    /// The number of peers connected to the torrent.
    pub peers: usize,
    /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
    pub download_speed: u64,
    /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
    pub upload_speed: u64,
    /// The total amount of data downloaded in bytes.
    pub downloaded: u64,
    /// The total size of the torrent in bytes.
    pub total_size: usize,
}

impl From<TorrentStats> for LoadingProgress {
    fn from(value: TorrentStats) -> Self {
        Self {
            progress: value.progress(),
            seeds: value.total_peers,
            peers: value.total_peers,
            download_speed: value.download_useful_rate,
            upload_speed: value.upload_useful_rate,
            downloaded: value.total_completed_size as u64,
            total_size: value.total_size,
        }
    }
}

/// Represents an error that may occur while a media item is being loaded.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum LoadingError {
    #[error("failed to parse URL: {0}")]
    ParseError(String),
    #[error("failed to load torrent, {0}")]
    TorrentError(torrents::Error),
    #[error("failed to process media information, {0}")]
    MediaError(String),
    #[error("loading timed-out, {0}")]
    TimeoutError(String),
    #[error("loading data is invalid, {0}")]
    InvalidData(String),
    #[error("loading task has been cancelled")]
    Cancelled,
}

/// A handle representing a loading process for media items in a playlist.
///
/// This handle is used to identify and manage individual loading processes.
pub type LoadingHandle = Handle;

/// A trait for managing media loading in a playlist.
///
/// Media loaders are responsible for coordinating the loading of media items in a playlist, utilizing loading strategies to process and prepare these items before playback.
#[async_trait]
pub trait MediaLoader: Debug + Callback<MediaLoaderEvent> + Send + Sync {
    /// Add a new loading strategy to the loading chain at the specified order.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed loading strategy.
    /// * `order` - The order at which the strategy should be added.
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order);

    /// Load a torrent magnet url.
    async fn load_url(&self, url: &str) -> LoadingHandle;

    /// Load a media item in the playlist using the media loader.
    ///
    /// # Arguments
    ///
    /// * `item` - The playlist item to be loaded.
    ///
    /// Returns a `LoadingHandle` representing the loading process associated with the loaded item.
    async fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle;

    /// Get the current loading state for a specific loading process represented by the provided `LoadingHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The `LoadingHandle` associated with the loading process.
    ///
    /// Returns an `Option` containing the loading state if the handle is valid; otherwise, `None`.
    async fn state(&self, handle: LoadingHandle) -> Option<LoadingState>;

    /// Subscribe to the loading events of the given loading task handle.
    /// It returns a subscription if the task is still running, else [None].
    async fn subscribe_loading(&self, handle: LoadingHandle) -> Option<Subscription<LoadingEvent>>;

    /// Cancel the loading process associated with the provided `LoadingHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The `LoadingHandle` representing the loading process to be canceled.
    fn cancel(&self, handle: LoadingHandle);
}

#[derive(Debug)]
pub struct DefaultMediaLoader {
    inner: Arc<InnerMediaLoader>,
}

impl DefaultMediaLoader {
    pub fn new(loading_chain: Vec<Box<dyn LoadingStrategy>>) -> Self {
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerMediaLoader::new(
            loading_chain,
            event_sender,
            command_sender,
        ));

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(event_receiver, command_receiver).await;
        });

        Self { inner }
    }
}

#[async_trait]
impl MediaLoader for DefaultMediaLoader {
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.inner.add(strategy, order);
    }

    async fn load_url(&self, url: &str) -> LoadingHandle {
        self.inner.load_url(url).await
    }

    async fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle {
        self.inner.load_playlist_item(item).await
    }

    async fn state(&self, handle: LoadingHandle) -> Option<LoadingState> {
        self.inner.task_state(handle).await
    }

    async fn subscribe_loading(&self, handle: LoadingHandle) -> Option<Subscription<LoadingEvent>> {
        self.inner.subscribe_loading(handle).await
    }

    fn cancel(&self, handle: LoadingHandle) {
        self.inner
            .send_command_event(MediaLoaderCommandEvent::Cancel(handle))
    }
}

impl Callback<MediaLoaderEvent> for DefaultMediaLoader {
    fn subscribe(&self) -> Subscription<MediaLoaderEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<MediaLoaderEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for DefaultMediaLoader {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, PartialEq)]
enum MediaLoaderCommandEvent {
    /// Cancel the given loading task
    Cancel(LoadingHandle),
    /// Cancel all loading tasks
    CancelAll,
    /// Indicates that a task has completed loading.
    TaskCompleted(LoadingHandle),
    /// Indicates that a loading task has been cancelled.
    TaskCancelled(LoadingHandle),
}

#[derive(Debug)]
struct InnerMediaLoader {
    loading_chain: Arc<LoadingChain>,
    tasks: RwLock<HashMap<LoadingHandle, LoadingTask>>,
    event_sender: UnboundedSender<MediaLoaderEvent>,
    command_sender: UnboundedSender<MediaLoaderCommandEvent>,
    callbacks: MultiThreadedCallback<MediaLoaderEvent>,
    cancellation_token: CancellationToken,
}

impl InnerMediaLoader {
    fn new(
        loading_chain: Vec<Box<dyn LoadingStrategy>>,
        event_sender: UnboundedSender<MediaLoaderEvent>,
        command_sender: UnboundedSender<MediaLoaderCommandEvent>,
    ) -> Self {
        Self {
            loading_chain: Arc::new(LoadingChain::from(loading_chain)),
            tasks: RwLock::new(HashMap::new()),
            event_sender,
            command_sender,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        }
    }

    async fn start(
        &self,
        mut event_receiver: UnboundedReceiver<MediaLoaderEvent>,
        mut command_receiver: UnboundedReceiver<MediaLoaderCommandEvent>,
    ) {
        trace!("Media loader main loop is starting");
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                event = event_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_event(event).await;
                    } else {
                        break;
                    }
                },
                command = command_receiver.recv() => {
                    if let Some(event) = command {
                        self.handle_command_event(event).await;
                    } else {
                        break;
                    }
                }
            }
        }
        debug!("Media loader main loop ended");
    }

    /// Handle the given loading task event from a certain task.
    /// It will delegate the received loading event to the callbacks of the media loader.
    async fn handle_event(&self, event: MediaLoaderEvent) {
        self.callbacks.invoke(event);
    }

    /// Handle the given media loader command event.
    async fn handle_command_event(&self, event: MediaLoaderCommandEvent) {
        debug!("Media loader is handling command event {:?}", event);
        match event {
            MediaLoaderCommandEvent::Cancel(handle) => self.cancel_task(handle).await,
            MediaLoaderCommandEvent::CancelAll => self.cancel_all().await,
            MediaLoaderCommandEvent::TaskCompleted(handle) => self.remove(handle).await,
            MediaLoaderCommandEvent::TaskCancelled(handle) => self.remove(handle).await,
        }
    }

    /// Get the state from the given loading task handle.
    /// It returns the state when the task was found back, else [None].
    async fn task_state(&self, task_handle: LoadingHandle) -> Option<LoadingState> {
        if let Some(task) = self.tasks.read().await.get(&task_handle).as_ref() {
            return Some(task.state().await);
        }

        None
    }

    /// Cancel a specific task if it still exists.
    /// If the loading task no longer exists, this will be a no-op.
    async fn cancel_task(&self, handle: LoadingHandle) {
        if let Some(task) = self.tasks.read().await.get(&handle).as_ref() {
            task.cancel()
        } else {
            debug!("Media loader couldn't find handle {}", handle);
        }
    }

    async fn cancel_all(&self) {
        let tasks = self.tasks.read().await;

        for (handle, task) in tasks.iter() {
            trace!("Media loader is cancelling loading task {}", handle);
            task.cancel();
        }
    }

    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.loading_chain.add(strategy, order);
    }

    /// Remove the given loading task from the media loader.
    async fn remove(&self, handle: LoadingHandle) {
        self.tasks.write().await.remove(&handle);
        debug!("Loading task {} has been removed", handle);
    }

    async fn load_url(&self, url: &str) -> LoadingHandle {
        trace!("Starting loading procedure for {}", url);
        self.do_internal_load(LoadingData::from(url)).await
    }

    async fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle {
        trace!("Starting loading procedure for {}", item);
        self.do_internal_load(LoadingData::from(item)).await
    }

    async fn subscribe_loading(&self, handle: LoadingHandle) -> Option<Subscription<LoadingEvent>> {
        if let Some(task) = self.tasks.read().await.get(&handle).as_ref() {
            return Some(task.subscribe());
        }

        None
    }

    async fn do_internal_load(&self, data: LoadingData) -> LoadingHandle {
        trace!("Media loader is starting loading task for {:?}", data);
        let task = LoadingTask::new(self.loading_chain.clone());
        let task_handle = task.handle();
        let started_event = LoadingStartedEvent::from(&data);

        let mut task_event_receiver = task.subscribe();
        let task_event_cancel = self.cancellation_token.clone();
        let task_event_sender = self.event_sender.clone();
        let task_command_sender = self.command_sender.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    _ = task_event_cancel.cancelled() => break,
                    event = task_event_receiver.recv() => {
                        if let Some(event) = event {
                            Self::handle_task_event(&event, task_handle, &task_event_sender, &task_command_sender);
                        } else {
                            break;
                        }
                    }
                }
            }
        });

        // start loading the task
        trace!("Media loader is starting loading task {}", task_handle);
        task.load(data);
        self.invoke_event(MediaLoaderEvent::LoadingStarted(task_handle, started_event));

        // store the task
        self.tasks.write().await.insert(task_handle, task);
        debug!("Media loader has added loading task {}", task_handle);

        task_handle
    }

    fn send_command_event(&self, command: MediaLoaderCommandEvent) {
        if let Err(_) = self.command_sender.send(command) {
            debug!("Media loader command channel is closed");
            self.cancellation_token.cancel();
        }
    }

    fn invoke_event(&self, event: MediaLoaderEvent) {
        self.callbacks.invoke(event);
    }

    fn handle_task_event(
        event: &Arc<LoadingEvent>,
        handle: LoadingHandle,
        task_event_sender: &UnboundedSender<MediaLoaderEvent>,
        task_command_sender: &UnboundedSender<MediaLoaderCommandEvent>,
    ) {
        let mut media_event: Option<MediaLoaderEvent> = None;

        match &**event {
            LoadingEvent::StateChanged(state) => {
                media_event = Some(MediaLoaderEvent::StateChanged(handle, state.clone()));
            }
            LoadingEvent::ProgressChanged(progress) => {
                media_event = Some(MediaLoaderEvent::ProgressChanged(handle, progress.clone()));
            }
            LoadingEvent::LoadingError(err) => {
                media_event = Some(MediaLoaderEvent::LoadingError(handle, err.clone()));
            }
            LoadingEvent::Completed => {
                let _ = task_command_sender.send(MediaLoaderCommandEvent::TaskCompleted(handle));
            }
            LoadingEvent::Cancelled => {
                let _ = task_command_sender.send(MediaLoaderCommandEvent::TaskCompleted(handle));
            }
        }

        if let Some(event) = media_event {
            if let Err(_) = task_event_sender.send(event) {
                debug!("Media loader event channel is closed");
            }
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub MediaLoader {}

        #[async_trait]
        impl MediaLoader for MediaLoader {
            fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order);
            async fn load_url(&self, url: &str) -> LoadingHandle;
            async fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle;
            async fn state(&self, handle: LoadingHandle) -> Option<LoadingState>;
            async fn subscribe_loading(&self, handle: LoadingHandle) -> Option<Subscription<LoadingEvent>>;
            fn cancel(&self, handle: LoadingHandle);
        }

        impl Callback<MediaLoaderEvent> for MediaLoader {
            fn subscribe(&self) -> Subscription<MediaLoaderEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<MediaLoaderEvent>);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::loader::tests::TestingLoadingStrategy;
    use crate::core::loader::SubtitleData;
    use crate::{init_logger, recv_timeout};

    use std::time::Duration;

    #[test]
    fn test_load_data_from_str() {
        init_logger!();
        let url = "magnet:?MyTestingUrl";
        let expected_result = LoadingData {
            url: Some(url.to_string()),
            title: None,
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            torrent: None,
            torrent_file: None,
            subtitle: SubtitleData::default(),
        };

        let result = LoadingData::from(url);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_loading_data_from_playlist_item() {
        init_logger!();
        let item = PlaylistItem {
            url: None,
            title: "MyItemTitle".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let expected_result = LoadingData {
            url: None,
            title: Some("MyItemTitle".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData {
                enabled: Some(false),
                info: None,
                subtitle: None,
            },
            torrent: None,
            torrent_file: None,
        };

        let result = LoadingData::from(item);

        assert_eq!(expected_result, result);
    }

    #[tokio::test]
    async fn test_load_playlist_item() {
        init_logger!();
        let item = PlaylistItem {
            url: None,
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let (tx, mut rx) = unbounded_channel();
        let expected_result = LoadingData::from(item.clone());
        let strategy = TestingLoadingStrategy::builder().data_sender(tx).build();
        let chain: Vec<Box<dyn LoadingStrategy>> = vec![Box::new(strategy)];
        let loader = DefaultMediaLoader::new(chain);

        let handle = loader.load_playlist_item(item).await;
        let state = loader
            .state(handle.clone())
            .await
            .expect("expected a loading state");
        assert_eq!(LoadingState::Initializing, state);

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(expected_result, result);
    }

    #[tokio::test]
    async fn test_load_playlist_item_bind_task_events() {
        init_logger!();
        let item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let expected_result = LoadingProgress {
            progress: 0.125,
            seeds: 10,
            peers: 2,
            download_speed: 0,
            upload_speed: 0,
            downloaded: 0,
            total_size: 0,
        };
        let (tx_data, mut rx_data) = unbounded_channel();
        let (tx_event, mut rx_event) = unbounded_channel();
        let strategy = TestingLoadingStrategy::builder()
            .data_sender(tx_data)
            .event(LoadingEvent::ProgressChanged(expected_result.clone()))
            .delay(Duration::from_millis(150))
            .build();
        let loader = DefaultMediaLoader::new(vec![Box::new(strategy)]);

        let mut receiver = loader.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let MediaLoaderEvent::ProgressChanged(_, progress) = &*event {
                        tx_event.send(progress.clone()).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        let _ = loader.load_playlist_item(item).await;
        let _ = recv_timeout!(
            &mut rx_data,
            Duration::from_millis(500),
            "expected the loading process to have been started"
        );

        let result = recv_timeout!(&mut rx_event, Duration::from_millis(500));
        assert_eq!(expected_result, result);
    }
}
