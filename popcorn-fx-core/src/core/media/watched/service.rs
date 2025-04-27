use crate::core::event::{Event, EventCallback, EventHandler, EventPublisher, PlayerStoppedEvent};
use crate::core::media::watched::Watched;
use crate::core::media::{MediaError, MediaIdentifier, MediaType};
use crate::core::storage::{Storage, StorageError};
use crate::core::{event, media};
use async_trait::async_trait;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const FILENAME: &str = "watched.json";
const WATCHED_PERCENTAGE_THRESHOLD: f64 = 85f64;

/// The callback to listen on events of the watched service.
pub type WatchedCallback = Box<dyn Fn(WatchedEvent) + Send>;

#[derive(Debug, Clone)]
pub enum WatchedEvent {
    /// Invoked when a media item's watched state has changed.
    ///
    /// - The IMDB ID of the media item for which the state changed.
    /// - The new state.
    WatchedStateChanged(String, bool),
}

impl Display for WatchedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchedEvent::WatchedStateChanged(id, state) => {
                write!(f, "Watched state changed of {} to {}", id, state)
            }
        }
    }
}

/// The watched service is responsible for tracking seen/unseen media items.
#[async_trait]
pub trait WatchedService: Debug + Callback<WatchedEvent> + Send + Sync {
    /// Verify if the given ID has been seen.
    ///
    /// * `id`  - The ID of the watchable to verify.
    ///
    /// It returns `true` when the ID has been seen, else `false`.
    async fn is_watched(&self, id: &str) -> bool;

    /// Verify if the given identifier item has been seen.
    ///
    /// It returns `true` when the media item has been seen, else `false`.
    async fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool;

    /// Retrieve an array of owned watched media item ids.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn all(&self) -> media::Result<Vec<String>>;

    /// Retrieve an array of watched movie id's.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn watched_movies(&self) -> media::Result<Vec<String>>;

    /// Retrieve an array of watched show id's.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn watched_shows(&self) -> media::Result<Vec<String>>;

    /// Add the given media item to the watched list.
    /// Duplicate media items will be ignored and not result in a [MediaError].
    ///
    /// * `watchable`   - The media item to add to the watched list.
    fn add(&self, watchable: Box<dyn MediaIdentifier>) -> media::Result<()>;

    /// Remove the given media item from the watched list.
    /// Unseen media items will be ignored and not result in an error.
    ///
    /// * `watchable`   - The media item to remove from the watched list.
    fn remove(&self, watchable: Box<dyn MediaIdentifier>);
}

#[derive(Debug)]
pub struct DefaultWatchedService {
    inner: Arc<InnerWatchedService>,
}

impl DefaultWatchedService {
    pub fn new(storage_directory: &str, event_publisher: EventPublisher) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let instance = Self {
            inner: Arc::new(InnerWatchedService {
                storage: Storage::from(storage_directory),
                cache: Arc::new(Mutex::new(None)),
                event_publisher,
                command_sender,
                callbacks: MultiThreadedCallback::new(),
                cancellation_token: Default::default(),
            }),
        };

        let inner_main = instance.inner.clone();
        let event_receiver = instance
            .inner
            .event_publisher
            .subscribe(event::DEFAULT_ORDER)
            .expect("expected to receive a callback");
        tokio::spawn(async move {
            inner_main.start(event_receiver, command_receiver).await;
        });

        instance
    }
}

#[async_trait]
impl WatchedService for DefaultWatchedService {
    async fn is_watched(&self, id: &str) -> bool {
        self.inner.is_watched(id).await
    }

    async fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool {
        self.inner.is_watched_dyn(watchable).await
    }

    fn all(&self) -> media::Result<Vec<String>> {
        self.inner.all()
    }

    fn watched_movies(&self) -> media::Result<Vec<String>> {
        self.inner.watched_movies()
    }

    fn watched_shows(&self) -> media::Result<Vec<String>> {
        self.inner.watched_shows()
    }

    fn add(&self, watchable: Box<dyn MediaIdentifier>) -> media::Result<()> {
        self.inner.add(watchable)
    }

    fn remove(&self, watchable: Box<dyn MediaIdentifier>) {
        self.inner.remove(watchable)
    }
}

impl Callback<WatchedEvent> for DefaultWatchedService {
    fn subscribe(&self) -> Subscription<WatchedEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<WatchedEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for DefaultWatchedService {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, PartialEq)]
enum WatchedServiceCommand {
    Save,
}

/// The standard Popcorn FX watched service.
#[derive(Debug)]
struct InnerWatchedService {
    storage: Storage,
    cache: Arc<Mutex<Option<Watched>>>,
    event_publisher: EventPublisher,
    command_sender: UnboundedSender<WatchedServiceCommand>,
    callbacks: MultiThreadedCallback<WatchedEvent>,
    cancellation_token: CancellationToken,
}

impl InnerWatchedService {
    async fn start(
        &self,
        mut event_receiver: EventCallback,
        mut command_receiver: UnboundedReceiver<WatchedServiceCommand>,
    ) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(handler) = event_receiver.recv() => self.handle_event(handler).await,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
            }
        }
        self.save().await;
        debug!("Watched service main loop ended");
    }

    async fn handle_event(&self, mut handler: EventHandler) {
        if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
            self.on_player_stopped_event(event.clone());
        }

        handler.next();
    }

    async fn handle_command(&self, command: WatchedServiceCommand) {
        match command {
            WatchedServiceCommand::Save => self.save().await,
        }
    }

    async fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = watchable.imdb_id();
        self.is_watched(imdb_id).await
    }

    async fn is_watched(&self, imdb_id: &str) -> bool {
        trace!("Verifying if {} is watched", imdb_id);
        match self.load_watched_cache().await {
            Ok(_) => self
                .cache
                .lock()
                .await
                .as_ref()
                .map(|e| e.contains(imdb_id))
                .unwrap_or(false),
            Err(e) => {
                warn!("Unable to load {}, {}", FILENAME, e);
                false
            }
        }
    }

    fn all(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");
                let mut movies = watched.movies().clone();
                let mut shows = watched.shows().clone();
                let mut all: Vec<String> = vec![];

                all.append(&mut movies);
                all.append(&mut shows);

                Ok(all)
            }
            Err(e) => Err(e),
        }
    }

    fn watched_movies(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");

                Ok(watched.movies().clone())
            }
            Err(e) => Err(e),
        }
    }

    fn watched_shows(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");

                Ok(watched.shows().clone())
            }
            Err(e) => Err(e),
        }
    }

    fn add(&self, watchable: Box<dyn MediaIdentifier>) -> media::Result<()> {
        futures::executor::block_on(self.load_watched_cache())?;
        let id: &str;
        {
            let mutex = self.cache.clone();
            let mut cache = futures::executor::block_on(mutex.lock());
            let watched = cache
                .as_mut()
                .expect("expected the cache to have been loaded");
            id = watchable.imdb_id();

            match watchable.media_type() {
                MediaType::Movie => watched.add_movie(id),
                MediaType::Show => watched.add_show(id),
                MediaType::Episode => watched.add_show(id),
                _ => {
                    error!("Media type {} is not supported", watchable.media_type());
                }
            }
        }

        self.send_command(WatchedServiceCommand::Save);
        self.callbacks.invoke(WatchedEvent::WatchedStateChanged(
            watchable.imdb_id().to_string(),
            true,
        ));
        Ok(())
    }

    fn remove(&self, watchable: Box<dyn MediaIdentifier>) {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let id: &str;
                {
                    let mutex = self.cache.clone();
                    let mut cache = futures::executor::block_on(mutex.lock());
                    let watched = cache
                        .as_mut()
                        .expect("expected the cache to have been loaded");

                    id = watchable.imdb_id();
                    watched.remove(id);
                }

                self.send_command(WatchedServiceCommand::Save);
                self.callbacks
                    .invoke(WatchedEvent::WatchedStateChanged(id.to_string(), false));
            }
            Err(e) => {
                error!("Failed to remove watched item, {}", e)
            }
        }
    }

    async fn load_watched_cache(&self) -> media::Result<()> {
        let mutex = self.cache.clone();
        let mut cache = mutex.lock().await;

        if cache.is_none() {
            trace!("Loading watched cache");
            return match self.load_watched_from_storage() {
                Ok(e) => {
                    debug!("Loaded watched items {:?}", &e);
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e),
            };
        }

        trace!("Watched cache has already been loaded, nothing to do");
        Ok(())
    }

    fn load_watched_from_storage(&self) -> media::Result<Watched> {
        match self.storage.options().serializer(FILENAME).read() {
            Ok(e) => Ok(e),
            Err(e) => match e {
                StorageError::NotFound(file) => {
                    debug!("Creating new watched file {}", file);
                    Ok(Watched::empty())
                }
                StorageError::ReadingFailed(_, error) => {
                    error!("Failed to load watched items, {}", error);
                    Err(MediaError::WatchedLoadingFailed(error))
                }
                _ => {
                    warn!("Unexpected error returned from storage, {}", e);
                    Ok(Watched::empty())
                }
            },
        }
    }

    async fn save(&self) {
        if let Some(watched) = self.cache.lock().await.as_ref() {
            trace!("Watched service is saving watched items");
            self.save_watched(watched).await
        }
    }

    async fn save_watched(&self, watchable: &Watched) {
        match self
            .storage
            .options()
            .serializer(FILENAME)
            .write_async(watchable)
            .await
        {
            Ok(_) => info!("Watched items have been saved"),
            Err(e) => error!("Failed to save watched items, {}", e),
        }
    }

    fn on_player_stopped_event(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        if let Some(media) = event.media {
            if let (Some(time), Some(duration)) = (&event.time, &event.duration) {
                let imdb_id = media.imdb_id().to_string();
                let title = media.title();
                let percentage_watched = (*time as f64 / *duration as f64) * 100 as f64;

                trace!(
                    "Media item {} has been watched for {:.2}%",
                    media.imdb_id(),
                    percentage_watched
                );
                if percentage_watched >= WATCHED_PERCENTAGE_THRESHOLD {
                    if let Err(e) = self.add(media) {
                        error!(
                            "Failed to add media item {} to the watch list, {}",
                            imdb_id, e
                        );
                    } else {
                        info!(
                            "Added media \"{}\" ({}) to the watched list",
                            title, imdb_id
                        );
                    }
                }
            } else {
                debug!("Player stopped event has an unknown time and/or duration, skipping watched check")
            }
        } else {
            debug!("Player stopped event doesn't have contain media information, skipping watched check")
        }
    }

    fn send_command(&self, command: WatchedServiceCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Watched service failed to send command, {}", e);
        }
    }
}

#[cfg(test)]
pub mod test {
    use mockall::mock;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::testing::copy_test_file;
    use crate::{assert_timeout, init_logger, recv_timeout};

    use super::*;

    mock! {
        #[derive(Debug)]
        pub WatchedService {}

        #[async_trait]
        impl WatchedService for WatchedService {
            async fn is_watched(&self, id: &str) -> bool;
            async fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool;
            fn all(&self) -> media::Result<Vec<String>>;
            fn watched_movies(&self) -> media::Result<Vec<String>>;
            fn watched_shows(&self) -> media::Result<Vec<String>>;
            fn add(&self, watchable: Box<dyn MediaIdentifier>) -> media::Result<()>;
            fn remove(&self, watchable: Box<dyn MediaIdentifier>);
        }

        impl Callback<WatchedEvent> for WatchedService {
            fn subscribe(&self) -> Subscription<WatchedEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<WatchedEvent>);
        }
    }

    #[tokio::test]
    async fn test_is_watched_when_item_is_watched_should_return_true() {
        init_logger!();
        let imdb_id = "tt548723".to_string();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(temp_path, EventPublisher::default());
        let movie = MovieOverview::new(String::new(), imdb_id, String::new());
        copy_test_file(temp_dir.path().to_str().unwrap(), "watched.json", None);

        let result = service
            .is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>))
            .await;

        assert!(result, "expected the media to have been watched")
    }

    #[tokio::test]
    async fn test_is_watched_when_item_is_not_watched_should_return_false() {
        init_logger!();
        let imdb_id = "tt548766".to_string();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(temp_path, EventPublisher::default());
        let movie = MovieOverview::new(String::new(), imdb_id, String::new());
        copy_test_file(temp_dir.path().to_str().unwrap(), "watched.json", None);

        let result = service
            .is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>))
            .await;

        assert!(!result, "expected the media to not have been watched")
    }

    #[tokio::test]
    async fn test_is_watched_boxed() {
        init_logger!();
        let imdb_id = "tt541345".to_string();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "watched.json", None);
        let service = DefaultWatchedService::new(temp_path, EventPublisher::default());
        let movie = MovieOverview::new(String::new(), imdb_id, String::new());

        let result = service
            .is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>))
            .await;

        assert!(result, "expected the media to have been watched")
    }

    #[tokio::test]
    async fn test_all() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(temp_path, EventPublisher::default());
        let expected_result = vec!["tt548723", "tt541345", "tt3915174"];
        copy_test_file(temp_dir.path().to_str().unwrap(), "watched.json", None);

        let result = service
            .all()
            .expect("expected the watched ids to have been returned");

        assert_eq!(expected_result, result)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_add_movie() {
        init_logger!();
        let imdb_id = "tt548795".to_string();
        let resource_directory = tempdir().unwrap();
        let resource_path = resource_directory.path().to_str().unwrap();
        let service = DefaultWatchedService::new(resource_path, EventPublisher::default());
        let movie = MovieOverview::new(String::new(), imdb_id.clone(), String::new());

        service
            .add(Box::new(movie.clone()) as Box<dyn MediaIdentifier>)
            .expect("add should have succeeded");
        let result = service
            .is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>))
            .await;

        assert!(result, "expected the media item to have been watched")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_add_show() {
        init_logger!();
        let imdb_id = "tt88877554".to_string();
        let temp_dir = tempdir().unwrap();
        let resource_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(resource_path, EventPublisher::default());
        let show = ShowOverview::new(
            imdb_id.clone(),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        );

        service
            .add(Box::new(show.clone()) as Box<dyn MediaIdentifier>)
            .expect("add should have succeeded");
        let result = service
            .is_watched_dyn(&(Box::new(show) as Box<dyn MediaIdentifier>))
            .await;

        assert!(result, "expected the media item to have been watched")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_register_when_add_is_called_should_invoke_callbacks() {
        init_logger!();
        let id = "tt8744557";
        let temp_dir = tempdir().unwrap();
        let resource_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(resource_path, EventPublisher::default());
        let (tx, rx) = channel();
        let movie: Box<dyn MediaIdentifier> = Box::new(MovieOverview::new(
            String::new(),
            id.to_string(),
            String::new(),
        ));

        let mut receiver = service.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        service
            .add(movie)
            .expect("expected the movie to be added to watched");

        let result = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        match result {
            WatchedEvent::WatchedStateChanged(imdb_id, state) => {
                assert_eq!(id.to_string(), imdb_id);
                assert_eq!(true, state)
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_register_when_remove_is_called_should_invoke_callbacks() {
        init_logger!();
        let id = "tt8744557";
        let temp_dir = tempdir().unwrap();
        let resource_path = temp_dir.path().to_str().unwrap();
        let service = DefaultWatchedService::new(resource_path, EventPublisher::default());
        let (tx, rx) = channel();
        let movie: Box<dyn MediaIdentifier> = Box::new(MovieOverview::new(
            String::new(),
            id.to_string(),
            String::new(),
        ));

        let mut receiver = service.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });
        service.remove(movie);

        let result = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        match result {
            WatchedEvent::WatchedStateChanged(imdb_id, state) => {
                assert_eq!(id.to_string(), imdb_id);
                assert_eq!(false, state)
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_on_player_stopped_watched() {
        init_logger!();
        let imdb_id = "tt12455512";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let event_publisher = EventPublisher::default();
        let service = DefaultWatchedService::new(temp_path, event_publisher.clone());

        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: "http://localhost:8052/example.mp4".to_string(),
            media: Some(Box::new(MovieOverview {
                imdb_id: imdb_id.to_string(),
                title: "Lorem ipsum dolor".to_string(),
                year: "2013".to_string(),
                rating: None,
                images: Default::default(),
            })),
            time: Some(55000),
            duration: Some(60000),
        }));

        assert_timeout!(
            Duration::from_millis(100),
            service.is_watched(imdb_id).await,
            "expected the media item to have been watched"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_on_player_stopped_not_fully_watched() {
        init_logger!();
        let imdb_id = "tt0001212";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let event_publisher = EventPublisher::default();
        let mut callback = event_publisher.subscribe(event::LOWEST_ORDER).unwrap();
        let service = DefaultWatchedService::new(temp_path, event_publisher.clone());
        let (tx, mut rx) = unbounded_channel();

        tokio::spawn(async move {
            if let Some(mut handler) = callback.recv().await {
                tx.send(true).unwrap();
                handler.next();
            }
        });
        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: "http://localhost:8052/example.mp4".to_string(),
            media: Some(Box::new(MovieOverview {
                imdb_id: imdb_id.to_string(),
                title: "Lorem dolor esta amit".to_string(),
                year: "2009".to_string(),
                rating: None,
                images: Default::default(),
            })),
            time: Some(90000),
            duration: Some(120000),
        }));

        let _ = recv_timeout!(&mut rx, Duration::from_millis(100));
        let result = service.is_watched(imdb_id).await;
        assert_eq!(
            false, result,
            "expected the media item to not have been watched"
        );
    }
}
