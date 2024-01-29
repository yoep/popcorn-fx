use std::fmt::Debug;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::core::{block_in_place, Callbacks, CoreCallback, CoreCallbacks};
use crate::core::events::{Event, EventPublisher, LoadingStartedEvent};
use crate::core::loader::loading_chain::{LoadingChain, Order};
use crate::core::loader::LoadingStrategy;
use crate::core::media::TorrentInfo;
use crate::core::playlists::PlaylistItem;
use crate::core::torrents::{Torrent, TorrentError, TorrentStream};

pub type LoaderResult<T> = Result<T, LoadingError>;

pub type LoaderCallback = CoreCallback<LoaderEvent>;

#[derive(Debug, Display, Clone)]
pub enum LoaderEvent {
    #[display(fmt = "Loading state changed to {}", _0)]
    StateChanged(LoadingState),
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

#[repr(i32)]
#[derive(Debug, Clone, Display)]
pub enum LoadingState {
    #[display(fmt = "Loader is currently idle")]
    Idle,
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
    #[display(fmt = "Loader is playing media")]
    Playing,
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
}

#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
/// A trait for managing media loading.
pub trait MediaLoader: Debug + Send + Sync {
    /// Get the current loading state.
    fn state(&self) -> LoadingState;

    /// Add a new loading strategy to the loading chain at the specified order.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed loading strategy.
    /// * `order` - The order at which the strategy should be added.
    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order);

    /// Subscribe to loader events.
    ///
    /// # Arguments
    ///
    /// * `callback` - A callback function to receive loader events.
    ///
    /// Returns an ID representing the subscription.
    fn subscribe(&self, callback: LoaderCallback) -> i64;

    /// Load a playlist item.
    ///
    /// # Arguments
    ///
    /// * `item` - The playlist item to be loaded.
    ///
    /// Returns a result indicating the success or failure of loading.
    async fn load_playlist_item(&self, item: PlaylistItem) -> LoaderResult<()>;
}

/// A structure representing loading data for a media item.
#[derive(Debug, Clone)]
pub struct LoadingData {
    pub item: PlaylistItem,
    pub media_torrent_info: Option<TorrentInfo>,
    pub torrent: Option<Weak<Box<dyn Torrent>>>,
    pub torrent_stream: Option<Weak<dyn TorrentStream>>,
}

impl PartialEq for LoadingData {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item &&
            self.torrent.is_some() == other.torrent.is_some() &&
            self.torrent_stream.is_some() == other.torrent_stream.is_some()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl From<PlaylistItem> for LoadingData {
    fn from(value: PlaylistItem) -> Self {
        Self {
            item: value,
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        }
    }
}

#[derive(Debug)]
pub struct DefaultMediaLoader {
    inner: Arc<InnerMediaLoader>,
}

impl DefaultMediaLoader {
    pub fn new(loading_chain: Vec<Box<dyn LoadingStrategy>>, event_publisher: Arc<EventPublisher>) -> Self {
        let instance = Self {
            inner: Arc::new(InnerMediaLoader::new(loading_chain, event_publisher)),
        };
        instance.register_state_updates();

        instance
    }

    fn register_state_updates(&self) {
        for strategy in self.inner.loading_chain.strategies().iter() {
            if let Some(strategy) = strategy.upgrade() {
                let inner_ref = Arc::downgrade(&self.inner);
                Self::register_strategy_state_update(&strategy, inner_ref)
            }
        }
    }

    fn register_strategy_state_update(strategy: &Arc<Box<dyn LoadingStrategy>>, loader: Weak<InnerMediaLoader>) {
        strategy.on_state_update(Box::new(move |state| {
            if let Some(loader) = loader.upgrade() {
                loader.update_state(state);
            } else {
                warn!("Unable to invoke state update, media loader is disposed");
            }
        }))
    }
}

#[async_trait]
impl MediaLoader for DefaultMediaLoader {
    fn state(&self) -> LoadingState {
        self.inner.state()
    }

    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.inner.add(strategy, order);
        self.register_state_updates();
    }

    fn subscribe(&self, callback: LoaderCallback) -> i64 {
        self.inner.subscribe(callback)
    }

    async fn load_playlist_item(&self, item: PlaylistItem) -> LoaderResult<()> {
        self.inner.load_playlist_item(item).await
    }
}

#[derive(Debug)]
struct InnerMediaLoader {
    state: Arc<Mutex<LoadingState>>,
    loading_chain: LoadingChain,
    callbacks: CoreCallbacks<LoaderEvent>,
    event_publisher: Arc<EventPublisher>,
}

impl InnerMediaLoader {
    fn new(loading_chain: Vec<Box<dyn LoadingStrategy>>, event_publisher: Arc<EventPublisher>) -> Self {
        Self {
            state: Arc::new(Mutex::new(LoadingState::Idle)),
            loading_chain: LoadingChain::from(loading_chain),
            callbacks: Default::default(),
            event_publisher,
        }
    }

    fn update_state(&self, new_state: LoadingState) {
        let event_state = new_state.clone();

        {
            let mut state = block_in_place(self.state.lock());
            *state = new_state;
        }

        self.callbacks.invoke(LoaderEvent::StateChanged(event_state))
    }
}

#[async_trait]
impl MediaLoader for InnerMediaLoader {
    fn state(&self) -> LoadingState {
        let state = block_in_place(self.state.lock());
        state.clone()
    }

    fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        self.loading_chain.add(strategy, order)
    }

    fn subscribe(&self, callback: LoaderCallback) -> i64 {
        self.callbacks.add(callback)
    }

    async fn load_playlist_item(&self, item: PlaylistItem) -> LoaderResult<()> {
        trace!("Starting loading procedure for {}", item);
        self.event_publisher.publish(Event::LoadingStarted(LoadingStartedEvent {
            url: item.url.clone()
                .or(Some(String::new()))
                .unwrap(),
            title: item.title.clone(),
        }));
        let strategies = self.loading_chain.strategies();
        let mut data = LoadingData::from(item);

        trace!("Processing a total of {} loading strategies", strategies.len());
        for strategy in strategies.iter() {
            if let Some(strategy) = strategy.upgrade() {
                trace!("Executing {}", strategy);

                match strategy.process(data).await {
                    LoadingResult::Ok(updated_data) => data = updated_data,
                    LoadingResult::Completed => {
                        debug!("{} has ended the loading chain", strategy);
                        break;
                    }
                    LoadingResult::Err(err) => {
                        error!("An unexpected error occurred while loading playlist item, {}", err);
                        return Err(err);
                    }
                }
            } else {
                warn!("Loading strategy is no longer in scope");
            }
        }

        self.event_publisher.publish(Event::LoadingCompleted);
        debug!("Loading strategies have been completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::loader::MockLoadingStrategy;

    use super::*;

    #[test]
    fn test_load_playlist_item() {
        let item = PlaylistItem {
            url: None,
            title: "LoremIpsum".to_string(),
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
        strategy.expect_on_state_update()
            .return_const(());
        strategy.expect_process()
            .returning(move |e| {
                tx.send(e).unwrap();
                LoadingResult::Completed
            });
        let chain: Vec<Box<dyn LoadingStrategy>> = vec![Box::new(strategy)];
        let loader = DefaultMediaLoader::new(chain, Arc::new(EventPublisher::default()));

        let result = block_in_place(loader.load_playlist_item(item));
        assert_eq!(Ok(()), result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(expected_result, result);
    }
}