use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};
use crate::core::loader::{LoadingData, LoadingEvent, LoadingStrategy};
use crate::core::loader::loading_chain::{LoadingChain, Order};
use crate::core::loader::task::LoadingTask;
use crate::core::media::{
    Episode, Images, MediaIdentifier, MediaOverview, MovieDetails, ShowDetails,
};
use crate::core::playlists::PlaylistItem;
use crate::core::torrents::{DownloadStatus, Magnet, TorrentError};

/// Represents the result of a loading operation.
///
/// It is a specialized `Result` type where the success variant contains a value of type `T`, and the error variant
/// contains a `LoadingError` indicating the reason for the loading failure.
pub type LoaderResult<T> = Result<T, LoadingError>;

/// A type alias for a callback function that handles loader events.
///
/// `LoaderCallback` functions can be registered with the media loader to receive notifications about loader events,
/// such as loading state changes, progress updates, and errors.
pub type LoaderCallback = CoreCallback<LoaderEvent>;

/// A type alias for a callback function that handles loading events.
///
/// `LoadingCallback` functions can be registered with the loading task to receive notifications about loading events,
/// such as loading state changes, progress updates, and errors.
pub type LoadingCallback = CoreCallback<LoadingEvent>;

/// An enum representing events related to media loading.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum LoaderEvent {
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
    Ok(LoadingData),
    /// Indicates that processing has completed.
    Completed,
    /// Indicates an error during processing and includes an associated `LoadingError`.
    Err(LoadingError),
}

/// An enum representing the result of a cancellation operation on loading data.
pub type CancellationResult = Result<LoadingData, LoadingError>;

#[repr(i32)]
#[derive(Debug, Clone, Display, PartialOrd, PartialEq)]
pub enum LoadingState {
    #[display(fmt = "Loader is initializing")]
    Initializing,
    #[display(fmt = "Loader is starting")]
    Starting,
    #[display(fmt = "Loader is retrieving subtitles")]
    RetrievingSubtitles,
    #[display(fmt = "Loader is downloading a subtitle")]
    DownloadingSubtitle,
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

            return Some(images.fanart().to_string());
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
        let url = value.url.clone();
        let title = value.title.clone().unwrap_or_else(move || {
            url.and_then(|e| {
                Magnet::from_str(e.as_str())
                    .map(|e| Some(e))
                    .unwrap_or(None)
            })
            .and_then(|e| e.dn().map(|e| e.to_string()))
            .unwrap_or(String::new())
        });

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
    pub seeds: u32,
    /// The number of peers connected to the torrent.
    pub peers: u32,
    /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
    pub download_speed: u32,
    /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
    pub upload_speed: u32,
    /// The total amount of data downloaded in bytes.
    pub downloaded: u64,
    /// The total size of the torrent in bytes.
    pub total_size: u64,
}

impl From<DownloadStatus> for LoadingProgress {
    fn from(value: DownloadStatus) -> Self {
        Self {
            progress: value.progress,
            seeds: value.seeds,
            peers: value.peers,
            download_speed: value.download_speed,
            upload_speed: value.upload_speed,
            downloaded: value.downloaded,
            total_size: value.total_size,
        }
    }
}

/// Represents an error that may occur during media item loading.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum LoadingError {
    #[error("Failed to parse URL: {0}")]
    ParseError(String),
    #[error("Failed to load torrent, {0}")]
    TorrentError(TorrentError),
    #[error("Failed to process media information, {0}")]
    MediaError(String),
    #[error("Loading timed-out, {0}")]
    TimeoutError(String),
    #[error("Loading data is invalid, {0}")]
    InvalidData(String),
    #[error("Loading task has been cancelled")]
    Cancelled,
}

/// A handle representing a loading process for media items in a playlist.
///
/// This handle is used to identify and manage individual loading processes.
pub type LoadingHandle = Handle;

#[cfg_attr(any(test, feature = "testing"), automock)]
/// A trait for managing media loading in a playlist.
///
/// Media loaders are responsible for coordinating the loading of media items in a playlist, utilizing loading strategies to process and prepare these items before playback.
pub trait MediaLoader: Debug + Send + Sync {
    /// Add a new loading strategy to the loading chain at the specified order.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed loading strategy.
    /// * `order` - The order at which the strategy should be added.
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order);

    /// Subscribe to loader events and receive notifications when loading events occur.
    ///
    /// # Arguments
    ///
    /// * `callback` - A callback function to receive loader events.
    ///
    /// Returns a `CallbackHandle` representing the subscription to loader events.
    fn subscribe(&self, callback: LoaderCallback) -> CallbackHandle;

    fn load_url(&self, url: &str) -> LoadingHandle;

    /// Load a media item in the playlist using the media loader.
    ///
    /// # Arguments
    ///
    /// * `item` - The playlist item to be loaded.
    ///
    /// Returns a `LoadingHandle` representing the loading process associated with the loaded item.
    fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle;

    /// Get the current loading state for a specific loading process represented by the provided `LoadingHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The `LoadingHandle` associated with the loading process.
    ///
    /// Returns an `Option` containing the loading state if the handle is valid; otherwise, `None`.
    fn state(&self, handle: LoadingHandle) -> Option<LoadingState>;

    /// Subscribe to loading events for a specific loading process represented by the provided `LoadingHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The `LoadingHandle` associated with the loading process.
    /// * `callback` - A callback function to receive loading events for the specified process.
    ///
    /// Returns an `Option` containing a `CallbackHandle` representing the subscription if the handle is valid; otherwise, `None`.
    fn subscribe_loading(
        &self,
        handle: LoadingHandle,
        callback: LoadingCallback,
    ) -> Option<CallbackHandle>;

    /// Unsubscribe from loading events for a specific loading process represented by the provided `LoadingHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The `LoadingHandle` associated with the loading process.
    /// * `callback_handle` - The `CallbackHandle` representing the subscription to be canceled.
    fn unsubscribe_loading(&self, handle: LoadingHandle, callback_handle: CallbackHandle);

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
        Self {
            inner: Arc::new(InnerMediaLoader::new(loading_chain)),
        }
    }
}

#[async_trait]
impl MediaLoader for DefaultMediaLoader {
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.inner.add(strategy, order);
    }

    fn subscribe(&self, callback: LoaderCallback) -> CallbackHandle {
        self.inner.subscribe(callback)
    }

    fn load_url(&self, url: &str) -> LoadingHandle {
        self.inner.load_url(url)
    }

    fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle {
        self.inner.load_playlist_item(item)
    }

    fn state(&self, handle: LoadingHandle) -> Option<LoadingState> {
        self.inner.state(handle)
    }

    fn subscribe_loading(
        &self,
        handle: LoadingHandle,
        callback: LoadingCallback,
    ) -> Option<CallbackHandle> {
        self.inner.subscribe_loading(handle, callback)
    }

    fn unsubscribe_loading(&self, handle: LoadingHandle, callback_handle: CallbackHandle) {
        self.inner.unsubscribe_loading(handle, callback_handle)
    }

    fn cancel(&self, handle: LoadingHandle) {
        self.inner.cancel(handle)
    }
}

#[derive(Debug)]
struct InnerMediaLoader {
    loading_chain: Arc<LoadingChain>,
    tasks: Arc<Mutex<Vec<Arc<LoadingTask>>>>,
    callbacks: CoreCallbacks<LoaderEvent>,
    runtime: Arc<Runtime>,
}

impl InnerMediaLoader {
    fn new(loading_chain: Vec<Box<dyn LoadingStrategy>>) -> Self {
        Self {
            loading_chain: Arc::new(LoadingChain::from(loading_chain)),
            tasks: Arc::new(Mutex::new(Vec::default())),
            callbacks: Default::default(),
            runtime: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(5)
                    .thread_name("media_loader")
                    .build()
                    .expect("expected a new runtime"),
            ),
        }
    }

    fn do_internal_load(&self, data: LoadingData) -> LoadingHandle {
        let task = Arc::new(LoadingTask::new(
            self.loading_chain.clone(),
            self.runtime.clone(),
        ));
        let loading_handle = task.handle();
        let started_event = LoadingStartedEvent::from(&data);

        let task_to_store = task.clone();
        {
            let mut mutex = block_in_place(self.tasks.lock());
            mutex.push(task_to_store);
        }

        let task_callback_handle = loading_handle.clone();
        let task_callbacks = self.callbacks.clone();
        task.subscribe(Box::new(move |event| {
            let loader_event: LoaderEvent;

            match event {
                LoadingEvent::StateChanged(e) => {
                    loader_event = LoaderEvent::StateChanged(task_callback_handle, e)
                }
                LoadingEvent::ProgressChanged(e) => {
                    loader_event = LoaderEvent::ProgressChanged(task_callback_handle, e)
                }
                LoadingEvent::LoadingError(e) => {
                    loader_event = LoaderEvent::LoadingError(task_callback_handle, e)
                }
            }

            task_callbacks.invoke(loader_event);
        }));

        let tasks = self.tasks.clone();
        let callbacks = self.callbacks.clone();
        self.runtime.spawn(async move {
            let task_handle = task.handle();
            match task.load(data).await {
                Ok(_) => {
                    info!("Loading task {} has completed", task_handle);
                }
                Err(e) => {
                    error!("Loading task {} failed, {}", task_handle, e);
                    callbacks.invoke(LoaderEvent::LoadingError(task_handle, e));
                }
            }

            trace!("Removing task handle of {}", task_handle);
            Self::remove_task(task_handle, tasks);
        });

        self.callbacks.invoke(LoaderEvent::LoadingStarted(
            loading_handle.clone(),
            started_event,
        ));
        loading_handle
    }

    fn remove_task(handle: LoadingHandle, tasks: Arc<Mutex<Vec<Arc<LoadingTask>>>>) {
        let mut tasks = block_in_place(tasks.lock());
        let position = tasks.iter().position(|e| e.handle() == handle);

        if let Some(position) = position {
            let task = tasks.remove(position);
            debug!("Loading task {} has been removed", task.handle());
        }
    }
}

#[async_trait]
impl MediaLoader for InnerMediaLoader {
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.loading_chain.add(strategy, order)
    }

    fn subscribe(&self, callback: LoaderCallback) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn load_url(&self, url: &str) -> LoadingHandle {
        trace!("Starting loading procedure for {}", url);
        self.do_internal_load(LoadingData::from(url))
    }

    fn load_playlist_item(&self, item: PlaylistItem) -> LoadingHandle {
        trace!("Starting loading procedure for {}", item);
        self.do_internal_load(LoadingData::from(item))
    }

    fn state(&self, handle: LoadingHandle) -> Option<LoadingState> {
        block_in_place(self.tasks.lock())
            .iter()
            .find(|e| e.handle() == handle)
            .map(|e| e.state())
    }

    fn subscribe_loading(
        &self,
        handle: LoadingHandle,
        callback: LoadingCallback,
    ) -> Option<CallbackHandle> {
        let tasks = block_in_place(self.tasks.lock());
        tasks
            .iter()
            .find(|e| e.handle() == handle)
            .map(|task| task.subscribe(callback))
    }

    fn unsubscribe_loading(&self, handle: LoadingHandle, callback_handle: CallbackHandle) {
        if let Some(task) = block_in_place(self.tasks.lock())
            .iter()
            .find(|e| e.handle() == handle)
        {
            task.unsubscribe(callback_handle)
        }
    }

    fn cancel(&self, handle: LoadingHandle) {
        if let Some(task) = block_in_place(self.tasks.lock())
            .iter()
            .find(|e| e.handle() == handle)
        {
            info!("Cancelling loading task {}", handle);
            task.cancel()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::loader::loading_chain::DEFAULT_ORDER;
    use crate::core::loader::MockLoadingStrategy;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_load_data_from_str() {
        init_logger();
        let url = "magnet:?MyTestingUrl";
        let expected_result = LoadingData {
            url: Some(url.to_string()),
            title: None,
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
            subtitles_enabled: None,
        };

        let result = LoadingData::from(url);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_loading_data_from_playlist_item() {
        init_logger();
        let item = PlaylistItem {
            url: None,
            title: "MyItemTitle".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let expected_result = LoadingData {
            url: None,
            title: Some("MyItemTitle".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: Some(false),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };

        let result = LoadingData::from(item);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_load_playlist_item() {
        init_logger();
        let item = PlaylistItem {
            url: None,
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let (tx, rx) = channel();
        let expected_result = LoadingData::from(item.clone());
        let mut strategy = MockLoadingStrategy::new();
        strategy.expect_process().returning(move |e, _, _| {
            tx.send(e).unwrap();
            LoadingResult::Completed
        });
        let chain: Vec<Box<dyn LoadingStrategy>> = vec![Box::new(strategy)];
        let loader = DefaultMediaLoader::new(chain);

        let handle = loader.load_playlist_item(item);
        assert_eq!(
            Some(LoadingState::Initializing),
            loader.state(handle.clone())
        );

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_load_playlist_item_bind_task_events() {
        init_logger();
        let (tx, rx) = channel();
        let (tx_event, rx_event) = channel();
        let item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
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
        let mut strategy = MockLoadingStrategy::new();
        strategy
            .expect_process()
            .times(1)
            .returning(Box::new(move |_, event_channel, _| {
                tx.send(event_channel).unwrap();
                LoadingResult::Completed
            }));
        let loader = DefaultMediaLoader::new(vec![]);

        loader.subscribe(Box::new(move |e| {
            if let LoaderEvent::ProgressChanged(_, e) = e {
                tx_event.send(e).unwrap();
            }
        }));
        loader.add(Box::new(strategy), DEFAULT_ORDER);
        let _ = loader.load_playlist_item(item);
        let callback = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        callback
            .send(LoadingEvent::ProgressChanged(expected_result.clone()))
            .unwrap();
        let result = rx_event.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(expected_result, result);
    }
}
