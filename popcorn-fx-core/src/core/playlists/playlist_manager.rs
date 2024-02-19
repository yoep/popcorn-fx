use std::sync::Arc;

use derive_more::Display;
use log::{debug, trace};
use tokio::sync::Mutex;

use crate::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};
use crate::core::events::{Event, EventPublisher, HIGHEST_ORDER};
use crate::core::loader::{LoadingHandle, MediaLoader};
use crate::core::players::{PlayerManager, PlayerManagerEvent, PlayerState};
use crate::core::playlists::{Playlist, PlaylistItem};

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
#[repr(i32)]
#[derive(Debug, Display, Clone, PartialOrd, PartialEq)]
pub enum PlaylistState {
    Idle,
    Playing,
    Stopped,
    Completed,
    Error,
}

/// The manager responsible for handling playlists and player events.
pub struct PlaylistManager {
    inner: Arc<InnerPlaylistManager>,
}

impl PlaylistManager {
    /// Create a new `PlaylistManager` instance.
    ///
    /// # Arguments
    ///
    /// * `player_manager` - A reference to the player manager.
    /// * `event_publisher` - A reference to the event publisher.
    ///
    /// # Returns
    ///
    /// A new `PlaylistManager` instance.
    pub fn new(player_manager: Arc<Box<dyn PlayerManager>>, event_publisher: Arc<EventPublisher>, loader: Arc<Box<dyn MediaLoader>>) -> Self {
        let manager = Self {
            inner: Arc::new(InnerPlaylistManager::new(player_manager, event_publisher, loader)),
        };

        let event_manager = manager.inner.clone();
        manager.inner.event_publisher.register(Box::new(move |event| {
            if let Event::ClosePlayer = event {
                if event_manager.has_next() {
                    debug!("Consuming Event::ClosePlayer, next item will be loaded");
                    return None;
                }
            }

            Some(event)
        }), HIGHEST_ORDER + 10);

        let listener_manager = manager.inner.clone();
        manager.inner.player_manager.subscribe(Box::new(move |e| {
            listener_manager.handle_player_event(e);
        }));

        manager
    }

    /// Get the current playlist.
    ///
    /// # Returns
    ///
    /// The current playlist.
    pub fn playlist(&self) -> Playlist {
        let playlist = block_in_place(self.inner.playlist.lock());
        playlist.iter()
            .cloned()
            .collect()
    }

    /// Start playing the specified playlist.
    ///
    /// # Arguments
    ///
    /// * `playlist` - The playlist to start playing.
    pub fn play(&self, playlist: Playlist) -> Option<Handle> {
        self.inner.play(playlist)
    }

    /// Play the next item in the playlist.
    ///
    /// Attempts to start playback of the next item in the playlist managed by the `PlaylistManager`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `Handle` representing the playlist item loader;
    /// otherwise, `None` if there are no more items to play or if an error occurred during playback initiation.
    pub fn play_next(&self) -> Option<Handle> {
        self.inner.play_next()
    }

    /// Check if there is a next item in the playlist.
    ///
    /// # Returns
    ///
    /// `true` if there is a next item, otherwise `false`.
    pub fn has_next(&self) -> bool {
        self.inner.has_next()
    }

    /// Retrieve the state of the current playlist.
    ///
    /// # Returns
    ///
    /// Returns the [PlaylistState] of the current playlist.
    pub fn state(&self) -> PlaylistState {
        self.inner.state()
    }

    /// Subscribe to playlist manager events.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to be invoked when playlist manager events occur.
    ///
    /// # Returns
    ///
    /// An identifier for the subscription, which can be used to unsubscribe later.
    pub fn subscribe(&self, callback: CoreCallback<PlaylistManagerEvent>) -> CallbackHandle {
        self.inner.callbacks.add(callback)
    }

    /// Unsubscribe from playlist manager events.
    ///
    /// # Arguments
    ///
    /// * `callback_id` - The identifier of the subscription to be removed.
    pub fn unsubscribe(&self, handle: CallbackHandle) {
        self.inner.callbacks.remove(handle)
    }

    /// Stop the playback of the playlist.
    ///
    /// This method stops the playback of the current playlist.
    /// If there is no playlist currently playing, it has no effect.
    pub fn stop(&self) {
        self.inner.stop();
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
    callbacks: CoreCallbacks<PlaylistManagerEvent>,
    event_publisher: Arc<EventPublisher>,
}

impl InnerPlaylistManager {
    fn new(player_manager: Arc<Box<dyn PlayerManager>>, event_publisher: Arc<EventPublisher>, loader: Arc<Box<dyn MediaLoader>>) -> Self {
        let instance = Self {
            playlist: Default::default(),
            player_manager,
            player_duration: Default::default(),
            player_playing_in: Default::default(),
            loader,
            loading_handle: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(PlaylistState::Idle)),
            callbacks: Default::default(),
            event_publisher,
        };

        instance
    }

    fn play(&self, playlist: Playlist) -> Option<Handle> {
        trace!("Starting new playlist with {:?}", playlist);
        {
            let mut mutex = block_in_place(self.playlist.lock());
            debug!("Replacing playlist with {:?}", playlist);
            *mutex = playlist
        }

        self.callbacks.invoke(PlaylistManagerEvent::PlaylistChanged);
        self.update_state(PlaylistState::Playing);
        self.play_next()
    }

    fn play_next(&self) -> Option<Handle> {
        let mut mutex = block_in_place(self.playlist.lock());

        if let Some(item) = mutex.next() {
            drop(mutex);

            trace!("Processing next item in playlist {}", item);
            Some(self.play_item(item))
        } else {
            self.update_state(PlaylistState::Completed);
            debug!("End of playlist has been reached");
            None
        }
    }

    fn play_item(&self, item: PlaylistItem) -> Handle {
        debug!("Starting playback of next playlist item {}", item);
        self.update_state(PlaylistState::Playing);
        let handle = self.loader.load_playlist_item(item);

        trace!("Updating current playlist item loading handle to {}", handle);
        let store_handle = handle.clone();
        let mut mutex = block_in_place(self.loading_handle.lock());
        *mutex = Some(store_handle);

        handle
    }

    fn has_next(&self) -> bool {
        let playlist = block_in_place(self.playlist.lock());
        playlist.has_next()
    }

    /// Retrieve a cloned version of the next item without removing it from the playlist.
    fn next_cloned(&self) -> Option<PlaylistItem> {
        let mutex = block_in_place(self.playlist.lock());
        mutex.next_as_ref()
            .map(|e| e.clone())
    }

    fn state(&self) -> PlaylistState {
        let state = block_in_place(self.state.lock());
        state.clone()
    }

    fn update_state(&self, state: PlaylistState) {
        Self::update_state_stat(state, self.state.clone(), self.callbacks.clone())
    }

    fn handle_player_event(&self, event: PlayerManagerEvent) {
        trace!("Processing player manager event {:?}", event);
        match event {
            PlayerManagerEvent::PlayerDurationChanged(e) => {
                let mut player_duration = block_in_place(self.player_duration.lock());
                debug!("Updating the last known player duration to {}", e);
                *player_duration = e;
            }
            PlayerManagerEvent::PlayerTimeChanged(time) => {
                let duration = block_in_place(self.player_duration.lock()).clone();

                if duration > 0 && time <= duration {
                    let remaining_time = (duration - time) / 1000;

                    trace!("Player has {} seconds remaining within the playback", remaining_time);
                    if let Some(next_item) = self.next_cloned() {
                        let playing_in: Option<u64>;

                        if remaining_time <= PLAYING_NEXT_IN_THRESHOLD_SECONDS {
                            playing_in = Some(remaining_time);
                        } else {
                            playing_in = None;
                        }

                        {
                            let mut mutex = block_in_place(self.player_playing_in.lock());
                            let invocation_allowed: bool;

                            if let Some((last_playing_in, item)) = mutex.as_ref() {
                                invocation_allowed = last_playing_in != &playing_in || item != &next_item;
                            } else {
                                invocation_allowed = true;
                            }

                            if invocation_allowed {
                                *mutex = Some((playing_in.clone(), next_item.clone()));
                                trace!("Playing next item in {:?} seconds", remaining_time);
                                self.callbacks.invoke(PlaylistManagerEvent::PlayingNext(PlayingNextInfo {
                                    playing_in,
                                    item: next_item,
                                }));
                            }
                        }
                    } else {
                        trace!("Reached end of playlist, PlaylistManagerEvent::PlayingNext won't be invoked");
                    }
                }
            }
            PlayerManagerEvent::PlayerStateChanged(state) => self.handle_player_state_event(state),
            _ => {}
        }
    }

    fn handle_player_state_event(&self, new_state: PlayerState) {
        let duration = block_in_place(self.player_duration.lock()).clone();

        match (duration, new_state) {
            (0, _) => trace!("Skipping player stopped, last known duration is {}", duration),
            (_, PlayerState::Stopped) => {
                if self.has_next() {
                    debug!("Starting automatic playback of the next playlist item");
                    self.play_next();
                }
            }
            _ => {}
        }
    }

    pub fn stop(&self) {
        trace!("Stopping the current playlist");
        {
            let mut mutex = block_in_place(self.playlist.lock());
            mutex.clear();
        }
        self.event_publisher.publish(Event::ClosePlayer);
    }

    fn update_state_stat(new_state: PlaylistState, state: Arc<Mutex<PlaylistState>>, callbacks: CoreCallbacks<PlaylistManagerEvent>) {
        trace!("Updating playlist state to {}", new_state);
        let event_state = new_state.clone();
        {
            let mut guard = block_in_place(state.lock());
            *guard = new_state;
        }

        debug!("Updated playlist state to {}", event_state);
        callbacks.invoke(PlaylistManagerEvent::StateChanged(event_state));
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::events::{DEFAULT_ORDER, LOWEST_ORDER};
    use crate::core::Handle;
    use crate::core::loader::MockMediaLoader;
    use crate::core::players::MockPlayerManager;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_play() {
        init_logger();
        let mut playlist = Playlist::default();
        let playlist_item = PlaylistItem {
            url: Some("http://localhost/myvideo.mp4".to_string()),
            title: "FooBar".to_string(),
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
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .return_const(Handle::new());
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let (tx, rx) = channel();
        let (tx_event, rx_event) = channel();
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(playlist_item.clone());

        manager.subscribe(Box::new(move |e| {
            if let PlaylistManagerEvent::PlaylistChanged = e {
                tx_event.send(e).unwrap();
            }
        }));
        manager.play(playlist);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(playlist_item, result, "expected the load_playlist_item to have been called");

        let result = rx_event.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result, "expected the PlaylistManagerEvent::PlaylistChanged event to have been published");
    }

    #[test]
    fn test_has_next() {
        init_logger();
        let mut playlist = Playlist::default();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .return_const(Handle::new());
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .returning(move |_| {
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo1.mp4".to_string()),
            title: "FooBar1".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo2.mp4".to_string()),
            title: "FooBar2".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        manager.play(playlist);

        assert!(manager.has_next(), "expected a next playlist item to have been available");
    }

    #[test]
    fn test_player_stopped_event() {
        init_logger();
        let url = "https://www.youtube.com";
        let item1 = "MyFirstItem";
        let item2 = "MySecondItem";
        let mut playlist = Playlist::default();
        let (tx, rx) = channel();
        let (tx_manager, rx_manager) = channel();
        let (tx_player_manager, rx_player_manager) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .times(1)
            .returning(move |e| {
                tx_player_manager.send(e).unwrap();
                Handle::new()
            });
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .times(2)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: item1.to_string(),
            caption: None,
            thumb: None,
            media: None,
            parent_media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
            url: None,
            title: item2.to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });

        manager.subscribe(Box::new(move |e| {
            tx_manager.send(e).unwrap();
        }));

        // start the playlist
        manager.play(playlist);
        let result = rx_manager.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        // verify the playlist item that has been loaded
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(item1.to_string(), result.title);

        let callback = rx_player_manager.recv_timeout(Duration::from_millis(200)).expect("Expected the playlist manager to subscribe to the player manager");
        callback(PlayerManagerEvent::PlayerDurationChanged(50000));
        callback(PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped));

        // verify the playlist item that has been loaded
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(item2.to_string(), result.title);
    }

    #[test]
    fn test_close_player_event_next_item() {
        init_logger();
        let url = "https://www.youtube.com";
        let mut playlist = Playlist::default();
        let (tx_manager, rx_manager) = channel();
        let (tx_event, rx_event) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .times(1)
            .returning(move |_| {
                Handle::new()
            });
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .returning(move |_| {
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: None,
            parent_media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            media: None,
            parent_media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });

        manager.subscribe(Box::new(move |e| {
            tx_manager.send(e).unwrap();
        }));
        event_publisher.register(Box::new(move |event| {
            if let Event::ClosePlayer = event {
                tx_event.send(event.clone()).unwrap();
            }
            Some(event)
        }), LOWEST_ORDER);

        // start the playlist
        manager.play(playlist);
        let result = rx_manager.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        event_publisher.publish(Event::ClosePlayer);
        let result = rx_event.recv_timeout(Duration::from_millis(100));
        assert!(result.is_err(), "expected the close player event to have been consumed");
    }

    #[test]
    fn test_player_stopped_event_without_known_duration() {
        init_logger();
        let url = "https://www.youtube.com";
        let item1 = "MyFirstItem";
        let item2 = "MySecondItem";
        let mut playlist = Playlist::default();
        let (tx, rx) = channel();
        let (tx_manager, rx_manager) = channel();
        let (tx_player_manager, rx_player_manager) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .times(1)
            .returning(move |e| {
                tx_player_manager.send(e).unwrap();
                Handle::new()
            });
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: item1.to_string(),
            caption: None,
            thumb: None,
            media: None,
            parent_media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
            url: None,
            title: item2.to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });

        manager.subscribe(Box::new(move |e| {
            tx_manager.send(e).unwrap();
        }));

        // start the playlist
        manager.play(playlist);
        let result = rx_manager.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result);

        // verify the playlist item that has been loaded
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(item1.to_string(), result.title);

        let callback = rx_player_manager.recv_timeout(Duration::from_millis(200)).expect("Expected the playlist manager to subscribe to the player manager");
        callback(PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped));
        // should not invoke any events, otherwise, the MockPlayerManager will fail at this point due to too many calls
    }

    #[test]
    fn test_player_time_changed() {
        init_logger();
        let mut playlist = Playlist::default();
        let playing_next_item = PlaylistItem {
            url: Some("http://localhost/my-video.mp4".to_string()),
            title: "FooBar".to_string(),
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
        let callback = Arc::new(CoreCallbacks::<PlayerManagerEvent>::default());
        let subscribe_callback = callback.clone();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .times(1)
            .returning(move |e| {
                subscribe_callback.add(e);
                Handle::new()
            });
        let (tx, rx) = channel();
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .returning(move |_| {
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: None,
            title: "MyFirstItem".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(playing_next_item.clone());
        manager.subscribe(Box::new(move |e| {
            if let PlaylistManagerEvent::PlayingNext(_) = &e {
                tx.send(e).unwrap();
            }
        }));
        manager.play(playlist);

        callback.invoke(PlayerManagerEvent::PlayerDurationChanged(100000));
        callback.invoke(PlayerManagerEvent::PlayerTimeChanged(40000));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlaylistManagerEvent::PlayingNext(e) = result {
            assert_eq!(playing_next_item, e.item);
            assert_eq!(Some(60u64), e.playing_in);
        } else {
            assert!(false, "expected PlaylistManagerEvent::PlayingNext, but got {} instead", result)
        }

        callback.invoke(PlayerManagerEvent::PlayerTimeChanged(35000));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlaylistManagerEvent::PlayingNext(e) = result {
            assert_eq!(playing_next_item, e.item);
            assert_eq!(None, e.playing_in);
        } else {
            assert!(false, "expected PlaylistManagerEvent::PlayingNext, but got {} instead", result)
        }
    }

    #[test]
    fn test_stop() {
        init_logger();
        let mut playlist = Playlist::default();
        let callback = Arc::new(CoreCallbacks::<PlayerManagerEvent>::default());
        let subscribe_callback = callback.clone();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .times(1)
            .returning(move |e| {
                subscribe_callback.add(e);
                Handle::new()
            });
        let (tx, rx) = channel();
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader.expect_load_playlist_item()
            .returning(move |_| {
                Handle::new()
            });
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), Arc::new(Box::new(loader)));

        playlist.add(PlaylistItem {
            url: None,
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
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
        });

        event_publisher.register(Box::new(move |event| {
            if let Event::ClosePlayer = event {
                tx.send(event.clone()).unwrap();
            }
            Some(event)
        }), DEFAULT_ORDER);

        let result = manager.play(playlist);
        assert!(result.is_some(), "expected a loader handle to have been returned");
        assert_eq!(true, manager.has_next(), "expected a next item to have been available");

        manager.stop();
        assert_eq!(false, manager.has_next(), "expected all playlist items to have been cleared");

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(Event::ClosePlayer, result);
    }
}