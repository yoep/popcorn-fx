use std::sync::Arc;

use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, Callbacks, CoreCallback, CoreCallbacks};
use crate::core::events::{DEFAULT_ORDER, Event, EventPublisher};
use crate::core::players::{PlayerManager, PlayerManagerEvent};
use crate::core::playlists::{LoadingChain, Playlist, PlaylistItem};

/// An event representing changes to the playlist manager.
#[derive(Debug, Display, Clone)]
pub enum PlaylistManagerEvent {
    /// Event indicating that the playlist has been changed.
    #[display(fmt = "Playlist has been changed")]
    PlaylistChanged,
    /// Event indicating that the next item will start playing after a specified delay.
    #[display(fmt = "Playing next item in {:?} seconds", "_0.playing_in")]
    PlayingNext(PlayingNextInfo),
}

/// Information about the next item to be played in the playlist.
#[derive(Debug, Clone)]
pub struct PlayingNextInfo {
    /// The time (in seconds) when the next item will start playing, if available.
    pub playing_in: Option<u64>,
    /// The item that will be played next in the playlist.
    pub item: PlaylistItem,
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
    pub fn new(player_manager: Arc<Box<dyn PlayerManager>>, event_publisher: Arc<EventPublisher>, loading_chain: LoadingChain) -> Self {
        let manager = Self {
            inner: Arc::new(InnerPlaylistManager::new(player_manager, event_publisher, loading_chain)),
        };

        let event_manager = manager.inner.clone();
        manager.inner.event_publisher.register(Box::new(move |event| {
            if let Event::PlayerStopped(_) = &event {
                event_manager.play_next();
            }

            Some(event)
        }), DEFAULT_ORDER - 10);

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
        let playlist = self.inner.playlist.blocking_lock();
        playlist.iter()
            .cloned()
            .collect()
    }

    /// Start playing the specified playlist.
    ///
    /// # Arguments
    ///
    /// * `playlist` - The playlist to start playing.
    pub fn play(&self, playlist: Playlist) {
        self.inner.play(playlist)
    }

    /// Check if there is a next item in the playlist.
    ///
    /// # Returns
    ///
    /// `true` if there is a next item, otherwise `false`.
    pub fn has_next(&self) -> bool {
        self.inner.has_next()
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
    pub fn subscribe(&self, callback: CoreCallback<PlaylistManagerEvent>) -> i64 {
        self.inner.callbacks.add(callback)
    }

    /// Unsubscribe from playlist manager events.
    ///
    /// # Arguments
    ///
    /// * `callback_id` - The identifier of the subscription to be removed.
    pub fn unsubscribe(&self, callback_id: i64) {
        self.inner.callbacks.remove(callback_id)
    }
}

#[derive(Debug)]
struct InnerPlaylistManager {
    playlist: Mutex<Playlist>,
    player_manager: Arc<Box<dyn PlayerManager>>,
    player_duration: Mutex<u64>,
    loading_chain: LoadingChain,
    callbacks: CoreCallbacks<PlaylistManagerEvent>,
    event_publisher: Arc<EventPublisher>,
}

impl InnerPlaylistManager {
    fn new(player_manager: Arc<Box<dyn PlayerManager>>, event_publisher: Arc<EventPublisher>, loading_chain: LoadingChain) -> Self {
        let instance = Self {
            playlist: Default::default(),
            player_manager,
            player_duration: Default::default(),
            loading_chain,
            callbacks: Default::default(),
            event_publisher,
        };

        instance
    }

    fn play(&self, playlist: Playlist) {
        trace!("Starting new playlist with {:?}", playlist);
        {
            let mut mutex = futures::executor::block_on(self.playlist.lock());
            debug!("Replacing playlist with {:?}", playlist);
            *mutex = playlist
        }

        self.play_next()
    }

    fn play_next(&self) {
        let mut mutex = futures::executor::block_on(self.playlist.lock());

        if let Some(item) = mutex.next() {
            drop(mutex);

            trace!("Processing next item in playlist {}", item);
            if item.url.is_some() {
                self.play_item(item);
            }
        } else {
            debug!("End of playlist has been reached")
        }
    }

    fn play_item(&self, item: PlaylistItem) {
        debug!("Starting playback of next playlist item {}", item);
        let strategies = self.loading_chain.strategies();
        let mut item = item;

        block_in_place(async {
            trace!("Processing a total of {} loading strategies", strategies.len());
            for strategy in strategies.iter() {
                if let Some(strategy) = strategy.upgrade() {
                    if let Some(new_item) = strategy.process(item).await {
                        item = new_item;
                    } else {
                        debug!("Reached end of playlist item loading chain");
                        break;
                    }
                } else {
                    warn!("Loading strategy is no longer in scope");
                }
            }
        });
    }

    fn has_next(&self) -> bool {
        let playlist = self.playlist.blocking_lock();
        playlist.has_next()
    }

    /// Retrieve a cloned version of the next item without removing it from the playlist.
    fn next_cloned(&self) -> Option<PlaylistItem> {
        let mutex = futures::executor::block_on(self.playlist.lock());
        mutex.next_as_ref()
            .map(|e| e.clone())
    }

    fn handle_player_event(&self, event: PlayerManagerEvent) {
        match event {
            PlayerManagerEvent::PlayerDurationChanged(e) => {
                let mut player_duration = self.player_duration.blocking_lock();
                debug!("Updating the last known player duration to {}", e);
                *player_duration = e;
            }
            PlayerManagerEvent::PlayerTimeChanged(time) => {
                let duration = self.player_duration.blocking_lock().clone();
                let remaining_time = (duration - time) / 1000;

                if duration > 0 && remaining_time <= 60 {
                    if let Some(next_item) = self.next_cloned() {
                        trace!("Playing next item in {} seconds", remaining_time);
                        self.callbacks.invoke(PlaylistManagerEvent::PlayingNext(PlayingNextInfo {
                            playing_in: Some(remaining_time),
                            item: next_item,
                        }));
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::events::PlayerStoppedEvent;
    use crate::core::players::MockPlayerManager;
    use crate::core::playlists::{LoadingStrategy, MockLoadingStrategy};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_play() {
        init_logger();
        let mut playlist = Playlist::default();
        let playlist_item = PlaylistItem {
            url: Some("http://localhost/myvideo.mp4".to_string()),
            title: "FooBar".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .return_const(());
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let (tx, rx) = channel();
        let mut strategy = MockLoadingStrategy::new();
        strategy.expect_process().returning(move |e| {
            tx.send(e).unwrap();
            None
        });
        let loading_chain: Vec<Box<dyn LoadingStrategy>> = vec![Box::new(strategy)];
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), LoadingChain::from(loading_chain));

        playlist.add(playlist_item.clone());
        manager.play(playlist);
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(playlist_item, result);
    }

    #[test]
    fn test_has_next() {
        init_logger();
        let mut playlist = Playlist::default();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .return_const(());
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), LoadingChain::default());

        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo1.mp4".to_string()),
            title: "FooBar1".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        playlist.add(PlaylistItem {
            url: Some("http://localhost/myvideo2.mp4".to_string()),
            title: "FooBar2".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        manager.play(playlist);

        assert!(manager.has_next(), "expected a next playlist item to have been available");
    }

    #[test]
    fn test_player_stopped_event_item_url() {
        init_logger();
        let url = "https://www.youtube.com";
        let mut playlist = Playlist::default();
        let (tx, rc) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager.expect_subscribe()
            .return_const(());
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), LoadingChain::default());

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "Foo Bar".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        manager.play(playlist);
        event_publisher.register(Box::new(move |event| {
            match event {
                Event::PlayerStopped(_) => {}
                _ => tx.send(event.clone()).unwrap(),
            }
            Some(event)
        }), DEFAULT_ORDER);

        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: url.to_string(),
            media: None,
            time: None,
            duration: None,
        }));
        let result = rc.recv_timeout(Duration::from_millis(250)).unwrap();

        match result {
            Event::PlayVideo(e) => assert_eq!(url, e.url.as_str()),
            _ => assert!(false, "Expected Event::PlayVideo, but got {} instead", result)
        }
    }

    #[test]
    fn test_player_time_changed() {
        init_logger();
        let mut playlist = Playlist::default();
        let callback = Arc::new(Mutex::new(Box::new(|_| {}) as CoreCallback<PlayerManagerEvent>));
        let event_publisher = Arc::new(EventPublisher::default());
        let mut player_manager = Box::new(MockPlayerManager::new());
        let subscribe_callback = callback.clone();
        player_manager.expect_subscribe()
            .returning(move |e| {
                let mut callback = subscribe_callback.blocking_lock();
                *callback = e;
                ()
            });
        let (tx, rx) = channel();
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let manager = PlaylistManager::new(player_manager.clone(), event_publisher.clone(), LoadingChain::default());

        playlist.add(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        manager.subscribe(Box::new(move |e| {
            tx.send(e).unwrap();
        }));
        manager.play(playlist);
        let callback = callback.blocking_lock();

        callback(PlayerManagerEvent::PlayerDurationChanged(100000));
        callback(PlayerManagerEvent::PlayerTimeChanged(40000));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlaylistManagerEvent::PlayingNext(e) = result {
            assert_eq!(Some(60u64), e.playing_in);
        } else {
            assert!(false, "expected PlaylistManagerEvent::PlayingNext, but got {} instead", result)
        }
    }
}