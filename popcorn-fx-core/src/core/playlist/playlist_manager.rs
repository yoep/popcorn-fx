use crate::core::event::{Event, EventCallback, EventHandler, EventPublisher, HIGHEST_ORDER};
use crate::core::loader::{LoadingHandle, MediaLoader};
use crate::core::players::{PlayerManager, PlayerManagerEvent, PlayerState};
use crate::core::playlist::{Playlist, PlaylistItem};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, info, trace};
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const PLAYING_NEXT_IN_THRESHOLD_SECONDS: u64 = 60;

/// An event representing changes to the playlist manager.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum PlaylistManagerEvent {
    /// Event indicating that the playlist has been changed.
    #[display(fmt = "Playlist has been changed")]
    PlaylistChanged,
    /// Event indicating that the next item will start playing after a specified delay.
    #[display(fmt = "Playing next item in {:?} seconds", "_0.playing_in")]
    PlayingNext(PlayingNextInfo),
    /// Event indicating a change in the playlist state.
    #[display(fmt = "Playlist state changed to {}", _0)]
    StateChanged(PlaylistState),
}

/// Information about the next item to be played in the playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayingNextInfo {
    /// The time (in seconds) when the next item will start playing, if available.
    pub playing_in: Option<u64>,
    /// The item that will be played next in the playlist.
    pub item: PlaylistItem,
}

/// An enumeration representing the state of a playlist.
///
/// The `PlaylistState` enum is used to indicate the current state of a playlist, such as whether it's idle, playing, stopped, completed, or in an error state.
#[derive(Debug, Display, Copy, Clone, PartialOrd, PartialEq)]
pub enum PlaylistState {
    Idle,
    Playing,
    Stopped,
    Completed,
    Error,
}

/// The manager responsible for handling playlists and player events.
#[derive(Debug, Clone)]
pub struct PlaylistManager {
    inner: Arc<InnerPlaylistManager>,
}

impl PlaylistManager {
    /// Create a new playlist manager instance for processing playlist.
    pub fn new(
        player_manager: Arc<Box<dyn PlayerManager>>,
        event_publisher: EventPublisher,
        loader: Arc<Box<dyn MediaLoader>>,
    ) -> Self {
        let manager = Self {
            inner: Arc::new(InnerPlaylistManager::new(
                player_manager,
                event_publisher,
                loader,
            )),
        };

        let inner_main = manager.inner.clone();
        let callback = manager
            .inner
            .event_publisher
            .subscribe(HIGHEST_ORDER + 10)
            .expect("expected to be able to subscribe");
        let player_event_receiver = manager.inner.player_manager.subscribe();
        tokio::spawn(async move {
            inner_main.start(callback, player_event_receiver).await;
        });

        manager
    }

    /// Get the current playlist.
    ///
    /// # Returns
    ///
    /// The current playlist.
    pub async fn playlist(&self) -> Playlist {
        let playlist = self.inner.playlist.lock().await;
        playlist.iter().cloned().collect()
    }

    /// Start playing the specified playlist.
    ///
    /// # Arguments
    ///
    /// * `playlist` - The playlist to start playing.
    ///
    /// # Returns
    ///
    /// It returns the current loading handle of the next playlist item that is being loaded.
    pub async fn play(&self, playlist: Playlist) -> Option<Handle> {
        self.inner.play(playlist).await
    }

    /// Play the next item in the playlist.
    ///
    /// Attempts to start playback of the next item in the playlist managed by the `PlaylistManager`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `Handle` representing the playlist item loader;
    /// otherwise, `None` if there are no more items to play or if an error occurred during playback initiation.
    pub async fn play_next(&self) -> Option<Handle> {
        self.inner.play_next().await
    }

    /// Check if there is a next item in the playlist.
    ///
    /// # Returns
    ///
    /// `true` if there is a next item, otherwise `false`.
    pub async fn has_next(&self) -> bool {
        self.inner.has_next().await
    }

    /// Retrieve the state of the current playlist.
    ///
    /// # Returns
    ///
    /// Returns the [PlaylistState] of the current playlist.
    pub async fn state(&self) -> PlaylistState {
        self.inner.state().await
    }

    /// Stop the playback of the playlist.
    ///
    /// This method stops the playback of the current playlist.
    /// If there is no playlist currently playing, it has no effect.
    pub async fn stop(&self) {
        self.inner.stop().await
    }
}

impl Callback<PlaylistManagerEvent> for PlaylistManager {
    fn subscribe(&self) -> Subscription<PlaylistManagerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlaylistManagerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

#[derive(Debug)]
struct InnerPlaylistManager {
    playlist: Mutex<Playlist>,
    player_manager: Arc<Box<dyn PlayerManager>>,
    player_duration: Mutex<u64>,
    player_playing_in: Mutex<Option<(Option<u64>, PlaylistItem)>>,
    loader: Arc<Box<dyn MediaLoader>>,
    loading_handle: Arc<Mutex<Option<LoadingHandle>>>,
    state: Arc<Mutex<PlaylistState>>,
    callbacks: MultiThreadedCallback<PlaylistManagerEvent>,
    event_publisher: EventPublisher,
    cancellation_token: CancellationToken,
}

impl InnerPlaylistManager {
    fn new(
        player_manager: Arc<Box<dyn PlayerManager>>,
        event_publisher: EventPublisher,
        loader: Arc<Box<dyn MediaLoader>>,
    ) -> Self {
        let instance = Self {
            playlist: Default::default(),
            player_manager,
            player_duration: Default::default(),
            player_playing_in: Default::default(),
            loader,
            loading_handle: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(PlaylistState::Idle)),
            callbacks: MultiThreadedCallback::new(),
            event_publisher,
            cancellation_token: Default::default(),
        };

        instance
    }

    /// Start the main loop of the playlist manager.
    /// This loop will handle any command that needs to be processed for the playlist manager.
    async fn start(
        &self,
        mut event_receiver: EventCallback,
        mut player_event_receiver: Subscription<PlayerManagerEvent>,
    ) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(handler) = event_receiver.recv() => self.handle_event(handler).await,
                Some(event) = player_event_receiver.recv() => self.handle_player_event((*event).clone()).await
            }
        }

        debug!("Playlist manager main loop ended");
    }

    async fn handle_event(&self, mut handler: EventHandler) {
        if let Some(event) = handler.event_ref() {
            if let Event::ClosePlayer = event {
                if self.is_next_allowed().await {
                    debug!("Consuming Event::ClosePlayer, next playlist item will be loaded");
                    handler.stop();
                }
            }
            handler.next();
        }
    }

    async fn play(&self, playlist: Playlist) -> Option<Handle> {
        trace!("Starting new playlist with {:?}", playlist);
        {
            let mut mutex = self.playlist.lock().await;
            *mutex = playlist
        }

        self.callbacks.invoke(PlaylistManagerEvent::PlaylistChanged);
        self.update_state(PlaylistState::Playing).await;
        self.play_next().await
    }

    async fn play_next(&self) -> Option<Handle> {
        let mut mutex = self.playlist.lock().await;

        if let Some(item) = mutex.next() {
            drop(mutex);

            trace!("Processing next item in playlist {}", item);
            Some(self.play_item(item).await)
        } else {
            self.update_state(PlaylistState::Completed).await;
            debug!("End of playlist has been reached");
            None
        }
    }

    async fn play_item(&self, item: PlaylistItem) -> Handle {
        debug!("Starting playback of next playlist item {}", item);
        self.update_state(PlaylistState::Playing).await;
        let handle = self.loader.load_playlist_item(item).await;

        trace!(
            "Updating current playlist item loading handle to {}",
            handle
        );
        let store_handle = handle.clone();
        let mut mutex = self.loading_handle.lock().await;
        *mutex = Some(store_handle);

        handle
    }

    async fn has_next(&self) -> bool {
        let playlist = self.playlist.lock().await;
        playlist.has_next()
    }

    /// Retrieve a cloned version of the next item without removing it from the playlist.
    async fn next_cloned(&self) -> Option<PlaylistItem> {
        let mutex = self.playlist.lock().await;
        mutex.next_as_ref().map(|e| e.clone())
    }

    /// Get the current state of the playlist.
    async fn state(&self) -> PlaylistState {
        *self.state.lock().await
    }

    async fn update_state(&self, state: PlaylistState) {
        Self::update_state_stat(state, &self.state, &self.callbacks).await
    }

    async fn handle_player_event(&self, event: PlayerManagerEvent) {
        trace!("Processing player manager event {:?}", event);
        match event {
            PlayerManagerEvent::PlayerDurationChanged(e) => {
                let mut player_duration = self.player_duration.lock().await;
                debug!("Updating the last known player duration to {}", e);
                *player_duration = e;
            }
            PlayerManagerEvent::PlayerTimeChanged(time) => {
                let duration = self.player_duration.lock().await.clone();

                if duration > 0 && time <= duration {
                    let remaining_time = (duration - time) / 1000;

                    trace!(
                        "Player has {} seconds remaining within the playback",
                        remaining_time
                    );
                    if let Some(next_item) = self.next_cloned().await {
                        let playing_in: Option<u64>;

                        if remaining_time <= PLAYING_NEXT_IN_THRESHOLD_SECONDS {
                            playing_in = Some(remaining_time);
                        } else {
                            playing_in = None;
                        }

                        {
                            let mut mutex = self.player_playing_in.lock().await;
                            let invocation_allowed: bool;

                            if let Some((last_playing_in, item)) = mutex.as_ref() {
                                invocation_allowed =
                                    last_playing_in != &playing_in || item != &next_item;
                            } else {
                                invocation_allowed = true;
                            }

                            if invocation_allowed {
                                *mutex = Some((playing_in.clone(), next_item.clone()));
                                if remaining_time <= 3 {
                                    debug!("Playing next item in {:?} seconds", remaining_time);
                                } else {
                                    trace!("Playing next item in {:?} seconds", remaining_time);
                                }

                                self.callbacks.invoke(PlaylistManagerEvent::PlayingNext(
                                    PlayingNextInfo {
                                        playing_in,
                                        item: next_item,
                                    },
                                ));
                            }
                        }
                    } else {
                        trace!("Reached end of playlist, PlaylistManagerEvent::PlayingNext won't be invoked");
                    }
                }
            }
            PlayerManagerEvent::PlayerStateChanged(state) => {
                self.handle_player_state_event(state).await
            }
            _ => {}
        }
    }

    async fn handle_player_state_event(&self, new_state: PlayerState) {
        let duration = self.player_duration.lock().await.clone();

        match (duration, new_state) {
            (0, _) => trace!(
                "Skipping player stopped, last known duration is {}",
                duration
            ),
            (_, PlayerState::Stopped) => {
                if self.is_next_allowed().await {
                    let next_item: String;

                    {
                        let mutex = self.playlist.lock().await;
                        next_item = mutex
                            .next_as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| String::new());
                    }

                    info!("Starting next playlist item {}", next_item);
                    self.play_next().await;
                } else {
                    debug!("Automatic playback is not allowed to start next playlist item");
                }
            }
            _ => {}
        }
    }

    async fn stop(&self) {
        trace!("Stopping the current playlist");
        {
            let mut mutex = self.playlist.lock().await;
            mutex.clear();
            debug!("Active playlist has been cleared");
        }
        self.event_publisher.publish(Event::ClosePlayer);
    }

    /// Determine with-either the next item is allowed to be played.
    async fn is_next_allowed(&self) -> bool {
        let duration = self.player_duration.lock().await.clone();
        let playing_in = self
            .player_playing_in
            .lock()
            .await
            .clone()
            .and_then(|(time, _)| time)
            .filter(|e| e <= &PLAYING_NEXT_IN_THRESHOLD_SECONDS);

        self.has_next().await && duration > 0 && playing_in.is_some()
    }

    async fn update_state_stat(
        new_state: PlaylistState,
        state: &Arc<Mutex<PlaylistState>>,
        callbacks: &MultiThreadedCallback<PlaylistManagerEvent>,
    ) {
        trace!("Updating playlist state to {}", new_state);
        let event_state = new_state.clone();
        {
            let mut guard = state.lock().await;
            *guard = new_state;
        }

        debug!("Updated playlist state to {}", event_state);
        callbacks.invoke(PlaylistManagerEvent::StateChanged(event_state));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::core::event::{DEFAULT_ORDER, LOWEST_ORDER};
    use crate::core::loader::MockMediaLoader;
    use crate::core::players::MockPlayerManager;
    use crate::{init_logger, recv_timeout};

    use fx_callback::{Callback, MultiThreadedCallback};
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::time;

    #[tokio::test]
    async fn test_play() {
        init_logger!();
        let mut playlist = Playlist::default();
        let playlist_item = PlaylistItem {
            url: Some("http://localhost/myvideo.mp4".to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::<PlayerManagerEvent>::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let (tx, mut rx) = unbounded_channel();
        let (tx_event, mut rx_event) = unbounded_channel();
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(playlist_item.clone());

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let PlaylistManagerEvent::PlaylistChanged = &*event {
                        tx_event.send((*event).clone()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });
        manager.play(playlist).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(
            playlist_item, result,
            "expected the load_playlist_item to have been called"
        );

        let result = recv_timeout!(&mut rx_event, Duration::from_millis(200));
        assert_eq!(
            PlaylistManagerEvent::PlaylistChanged,
            result,
            "expected the PlaylistManagerEvent::PlaylistChanged event to have been published"
        );
    }

    #[tokio::test]
    async fn test_has_next() {
        init_logger!();
        let mut playlist = Playlist::default();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo1.mp4".to_string()),
            title: "FooBar1".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo2.mp4".to_string()),
            title: "FooBar2".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        manager.play(playlist).await;

        let result = manager.has_next().await;
        assert!(
            result,
            "expected a next playlist item to have been available"
        );
    }

    #[tokio::test]
    async fn test_player_stopped_event() {
        init_logger!();
        let url = "https://www.youtube.com";
        let item1 = "MyFirstItem";
        let item2 = "MySecondItem";
        let mut playlist = Playlist::default();
        let (tx, mut rx) = unbounded_channel();
        let (tx_manager, mut rx_manager) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .times(2)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: item1.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: None,
            title: item2.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    tx_manager.send((*event).clone()).unwrap()
                } else {
                    break;
                }
            }
        });

        // start the playlist
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item1.to_string(), result.title);

        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(50000));
        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(40000));
        callbacks.invoke(PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped));

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item2.to_string(), result.title);
    }

    #[tokio::test]
    async fn test_player_stopped_event_by_player_during_playback() {
        init_logger!();
        let url = "https://www.youtube.com";
        let item1 = "MyFirstItem";
        let item2 = "MySecondItem";
        let mut playlist = Playlist::default();
        let (tx_manager, mut rx_manager) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .times(2)
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: item1.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: None,
            title: item2.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    tx_manager.send((*event).clone()).unwrap()
                } else {
                    break;
                }
            }
        });

        // start the playlist
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(120000));
        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(40000));
        callbacks.invoke(PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped));

        // verify the playlist item that has been loaded
        let result = manager.inner.is_next_allowed().await;
        assert_eq!(false, result, "expected the next item to not be loaded");
    }

    #[tokio::test]
    async fn test_close_player_event_next_item() {
        init_logger!();
        let url = "https://www.youtube.com";
        let mut playlist = Playlist::default();
        let (tx_manager, mut rx_manager) = unbounded_channel();
        let (tx_event, mut rx_event) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    tx_manager.send((*event).clone()).unwrap()
                } else {
                    break;
                }
            }
        });
        let mut callback = event_publisher.subscribe(LOWEST_ORDER).unwrap();
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback.recv().await {
                    if let Some(Event::ClosePlayer) = handler.event_ref() {
                        tx_event.send(handler.take()).unwrap();
                        break;
                    }
                    handler.next();
                } else {
                    break;
                }
            }
        });

        // start the playlist
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(120000));
        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(100000));
        time::sleep(Duration::from_millis(50)).await;
        event_publisher.publish(Event::ClosePlayer);

        let result = select! {
            _ = time::sleep(Duration::from_millis(100)) => false,
            Some(_) = rx_event.recv() => true,
        };
        assert_eq!(
            false, result,
            "expected the close player event to have been consumed"
        );
    }

    #[tokio::test]
    async fn test_player_stopped_event_without_known_duration() {
        init_logger!();
        let url = "https://www.youtube.com";
        let item1 = "MyFirstItem";
        let item2 = "MySecondItem";
        let (tx, mut rx) = unbounded_channel();
        let (tx_manager, mut rx_manager) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        let mut playlist = Playlist::default();
        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: item1.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: None,
            title: item2.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx_manager.send((*event).clone()).unwrap()
            }
        });

        // start the playlist
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item1.to_string(), result.title);

        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(100000));
        callbacks.invoke(PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped));
        // should not invoke any events, otherwise, the MockPlayerManager will fail at this point due to too many calls
    }

    #[tokio::test]
    async fn test_player_time_changed() {
        init_logger!();
        let mut playlist = Playlist::default();
        let playing_next_item = PlaylistItem {
            url: Some("http://localhost/my-video.mp4".to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = MockPlayerManager::new();
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let (tx, mut rx) = unbounded_channel();
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            Arc::new(Box::new(player_manager)),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: None,
            title: "MyFirstItem".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(playing_next_item.clone());

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let PlaylistManagerEvent::PlayingNext(_) = &*event {
                    tx.send((*event).clone()).unwrap();
                }
            }
        });

        let result = manager.play(playlist).await;
        assert_ne!(None, result, "expected to receive a playlist handle");

        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(100000));
        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(40000));
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        if let PlaylistManagerEvent::PlayingNext(e) = result {
            assert_eq!(playing_next_item, e.item);
            assert_eq!(Some(60u64), e.playing_in);
        } else {
            assert!(
                false,
                "expected PlaylistManagerEvent::PlayingNext, but got {} instead",
                result
            )
        }

        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(35000));
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        if let PlaylistManagerEvent::PlayingNext(e) = result {
            assert_eq!(playing_next_item, e.item);
            assert_eq!(None, e.playing_in);
        } else {
            assert!(
                false,
                "expected PlaylistManagerEvent::PlayingNext, but got {} instead",
                result
            )
        }
    }

    #[tokio::test]
    async fn test_stop() {
        init_logger!();
        let mut playlist = Playlist::default();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        let (tx, mut rx) = unbounded_channel();
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(Box::new(loader)),
        );

        playlist.add(PlaylistItem {
            url: None,
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        playlist.add(PlaylistItem {
            url: None,
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });

        let mut callback = event_publisher.subscribe(DEFAULT_ORDER).unwrap();
        tokio::spawn(async move {
            loop {
                if let Some(mut handler) = callback.recv().await {
                    if let Some(Event::ClosePlayer) = handler.event_ref() {
                        tx.send(handler.take()).unwrap();
                    }
                    handler.next();
                }
            }
        });

        let result = manager.play(playlist).await;
        assert!(
            result.is_some(),
            "expected a loader handle to have been returned"
        );
        let result = manager.has_next().await;
        assert_eq!(true, result, "expected a next item to have been available");

        manager.stop().await;
        let result = manager.has_next().await;
        assert_eq!(
            false, result,
            "expected all playlist items to have been cleared"
        );

        let result = recv_timeout!(&mut rx, Duration::from_millis(200)).unwrap();
        assert_eq!(Event::ClosePlayer, result);
    }
}
