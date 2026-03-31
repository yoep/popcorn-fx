use crate::core::channel::{ChannelReceiver, ChannelSender, Reply};
use crate::core::event::{Event, EventCallback, EventHandler, EventPublisher, HIGHEST_ORDER};
use crate::core::loader::{LoadingHandle, MediaLoader};
use crate::core::players::{PlayerManager, PlayerManagerEvent};
use crate::core::playlist::{Playlist, PlaylistItem};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscription};
use fx_handle::Handle;
use log::{debug, error, trace};
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

const PLAYING_NEXT_IN_THRESHOLD_SECONDS: u32 = 60;

/// An event representing changes to the playlist manager.
#[derive(Debug)]
pub enum PlaylistManagerEvent {
    /// Invoked when a new playlist has been set.
    PlaylistChanged,
    /// Invoked when the next item in the playlist has been changed.
    PlayNextChanged(PlayNext),
    /// Invoked when the next item in the playlist will start playing in the specified number of seconds.
    PlayingNextIn(u32),
    /// Invoked when the player time has been rewind and the playing next in does no longer count.
    PlayingNextInAborted,
    /// Invoked when the state of the playlist has changed.
    StateChanged(PlaylistState),
}

#[derive(Debug)]
pub enum PlayNext {
    /// The next item in the playlist will start playing.
    Next(PlaylistItem),
    /// The playlist has reached the end and no more items can be played.
    End,
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
    sender: ChannelSender<PlaylistManagerCommand>,
    callbacks: MultiThreadedCallback<PlaylistManagerEvent>,
}

impl PlaylistManager {
    /// Create a new playlist manager instance for processing playlist.
    pub fn new(
        player_manager: Arc<Box<dyn PlayerManager>>,
        event_publisher: EventPublisher,
        loader: Arc<dyn MediaLoader>,
    ) -> Self {
        let (sender, receiver) = channel!(64);
        let mut inner = InnerPlaylistManager::new(player_manager, event_publisher, loader);
        let callbacks = inner.callbacks.clone();

        // move the main loop to a separate task
        tokio::spawn(async move {
            let callback = inner
                .event_publisher
                .subscribe(HIGHEST_ORDER + 10)
                .expect("expected to be able to subscribe");
            let player_event_receiver = inner.player_manager.subscribe();

            inner.run(receiver, callback, player_event_receiver).await;
        });

        Self { sender, callbacks }
    }

    /// Returns the active playlist.
    pub async fn playlist(&self) -> Playlist {
        self.sender
            .send(|tx| PlaylistManagerCommand::GetPlaylist { response: tx })
            .await
            .await
            .unwrap_or_default()
    }

    /// Start playing the specified playlist.
    ///
    /// It returns the current loading handle of the next playlist item that is being loaded.
    pub async fn play(&self, playlist: Playlist) -> Option<Handle> {
        self.sender
            .send(|tx| PlaylistManagerCommand::Play {
                playlist,
                response: tx,
            })
            .await
            .await
            .map_err(|e| {
                error!("Playlist failed to start, {}", e);
                e
            })
            .ok()
            .flatten()
    }

    /// Play the next item in the playlist.
    /// Attempts to start playback of the next item in the playlist managed by the `PlaylistManager`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `Handle` representing the playlist item loader;
    /// otherwise, `None` if there are no more items to play or if an error occurred during playback initiation.
    pub async fn play_next(&self) -> Option<Handle> {
        self.sender
            .send(|tx| PlaylistManagerCommand::PlayNext { response: tx })
            .await
            .await
            .map_err(|e| {
                error!("Playlist failed to play next item, {}", e);
                e
            })
            .ok()
            .flatten()
    }

    /// Check if there is a next item in the playlist.
    ///
    /// # Returns
    ///
    /// `true` if there is a next item, otherwise `false`.
    pub async fn has_next(&self) -> bool {
        self.sender
            .send(|tx| PlaylistManagerCommand::HasNext { response: tx })
            .await
            .await
            .unwrap_or(false)
    }

    /// Returns the [PlaylistState] of the current playlist.
    pub async fn state(&self) -> PlaylistState {
        self.sender
            .send(|tx| PlaylistManagerCommand::GetPlaylistState { response: tx })
            .await
            .await
            .unwrap_or(PlaylistState::Error)
    }

    /// Stop the playback of the playlist.
    ///
    /// This method stops the playback of the current playlist.
    /// If there is no playlist currently playing, it has no effect.
    pub async fn stop(&self) {
        let _ = self
            .sender
            .send(|tx| PlaylistManagerCommand::Stop { response: tx })
            .await
            .await;
    }
}

impl Callback<PlaylistManagerEvent> for PlaylistManager {
    fn subscribe(&self) -> Subscription<PlaylistManagerEvent> {
        self.callbacks.subscribe()
    }
}

#[derive(Debug, PartialEq)]
enum PlayNextState {
    Shown,
    Aborted,
}

#[derive(Debug)]
enum PlaylistManagerCommand {
    /// Returns the current state of the playlist manager.
    GetPlaylistState { response: Reply<PlaylistState> },
    /// Returns the current active playlist
    GetPlaylist { response: Reply<Playlist> },
    /// Start the playback of the specified playlist
    Play {
        playlist: Playlist,
        response: Reply<Option<Handle>>,
    },
    /// Play the next item in the playlist
    PlayNext { response: Reply<Option<Handle>> },
    /// Returns true if there is a next item in the playlist, otherwise false.
    HasNext { response: Reply<bool> },
    /// Stop the playback of the current playlist.
    Stop { response: Reply<()> },
}

#[derive(Debug)]
struct InnerPlaylistManager {
    playlist: Playlist,
    player_manager: Arc<Box<dyn PlayerManager>>,
    /// The player duration in millis.
    player_duration: u64,
    loader: Arc<dyn MediaLoader>,
    loading_handle: Option<LoadingHandle>,
    state: PlaylistState,
    next_state: PlayNextState,
    callbacks: MultiThreadedCallback<PlaylistManagerEvent>,
    event_publisher: EventPublisher,
    cancellation_token: CancellationToken,
}

impl InnerPlaylistManager {
    fn new(
        player_manager: Arc<Box<dyn PlayerManager>>,
        event_publisher: EventPublisher,
        loader: Arc<dyn MediaLoader>,
    ) -> Self {
        let instance = Self {
            playlist: Default::default(),
            player_manager,
            player_duration: Default::default(),
            loader,
            loading_handle: None,
            state: PlaylistState::Idle,
            next_state: PlayNextState::Aborted,
            callbacks: MultiThreadedCallback::new(),
            event_publisher,
            cancellation_token: Default::default(),
        };

        instance
    }

    /// Run the main loop of the playlist manager.
    /// This loop will handle any command that needs to be processed for the playlist manager.
    async fn run(
        &mut self,
        mut command_receiver: ChannelReceiver<PlaylistManagerCommand>,
        mut event_receiver: EventCallback,
        mut player_event_receiver: Subscription<PlayerManagerEvent>,
    ) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Ok(event) = player_event_receiver.recv() => self.on_player_event((*event).clone()),
                Some(handler) = event_receiver.recv() => self.on_event(handler).await,
                command = command_receiver.recv() => match command {
                    Some(command) => self.on_command(command).await,
                    None => break,
                },
            }
        }

        debug!("Playlist manager main loop ended");
    }

    async fn on_event(&mut self, mut handler: EventHandler) {
        if let Some(event) = handler.event_ref() {
            if let Event::PlayerStopped(event) = event {
                match self.should_play_next_on_player_stopped(
                    event.time.as_ref(),
                    event.duration.as_ref(),
                ) {
                    true => {
                        if self.play_next().await.is_some() {
                            self.event_publisher.publish(Event::LoadingStarted);
                        }
                    }
                    false => {
                        let is_playback_known = !event.url.is_empty() || event.media.is_some();
                        if event.duration.is_some() && is_playback_known {
                            debug!("Playlist manager is closing player, end of playlist playback has been reached");
                            self.event_publisher.publish(Event::ClosePlayer);
                        }
                    }
                }
            }
            handler.next();
        }
    }

    async fn play(&mut self, playlist: Playlist) -> Option<Handle> {
        trace!("Starting new playlist with {:?}", playlist);
        self.playlist = playlist;
        self.callbacks.invoke(PlaylistManagerEvent::PlaylistChanged);
        self.update_state(PlaylistState::Playing);
        self.play_next().await
    }

    async fn play_next(&mut self) -> Option<Handle> {
        // get the next item in the playlist
        let item = match self.playlist.next() {
            Some(item) => item,
            None => {
                debug!("End of playlist has been reached");
                self.update_state(PlaylistState::Completed);
                return None;
            }
        };

        // notify the subscribers about the next items after this item
        match self.playlist.next_as_ref() {
            None => self
                .callbacks
                .invoke(PlaylistManagerEvent::PlayNextChanged(PlayNext::End)),
            Some(next) => {
                self.callbacks
                    .invoke(PlaylistManagerEvent::PlayNextChanged(PlayNext::Next(
                        next.clone(),
                    )));
            }
        }

        trace!("Processing next item in playlist {}", item);
        Some(self.play_item(item).await)
    }

    async fn play_item(&mut self, item: PlaylistItem) -> Handle {
        debug!("Starting playback of next playlist item {}", item);
        self.update_state(PlaylistState::Playing);
        let handle = self.loader.load_playlist_item(item).await;

        trace!(
            "Updating current playlist item loading handle to {}",
            handle
        );
        let store_handle = handle.clone();
        self.loading_handle = Some(store_handle);
        self.next_state = PlayNextState::Aborted;

        handle
    }

    /// Returns `true` if there is an item in the playlist to play.
    fn has_next(&self) -> bool {
        self.playlist.has_next()
    }

    /// Get the current state of the playlist.
    fn state(&self) -> PlaylistState {
        self.state
    }

    fn update_state(&mut self, new_state: PlaylistState) {
        if self.state == new_state {
            return;
        }

        self.state = new_state;
        self.callbacks
            .invoke(PlaylistManagerEvent::StateChanged(new_state));
        trace!("Playlist state changed to {}", new_state);
    }

    fn on_player_event(&mut self, event: PlayerManagerEvent) {
        trace!("Processing player manager event {:?}", event);
        match event {
            PlayerManagerEvent::PlayerDurationChanged(duration) => {
                debug!("Updating the last known player duration to {}", duration);
                self.player_duration = duration;
            }
            PlayerManagerEvent::PlayerTimeChanged(time) => {
                self.on_player_time_changed(time);
            }
            _ => {}
        }
    }

    fn on_player_time_changed(&mut self, time: u64) {
        // early exit if we're unable to determine remaining time of the playback
        if self.player_duration == 0 || time > self.player_duration {
            return;
        }
        // early exit if the playlist has reached the end
        if !self.has_next() {
            return;
        }

        // calculate the remaining time of the playback
        let remaining_time = ((self.player_duration - time) / 1000) as u32;
        if remaining_time > PLAYING_NEXT_IN_THRESHOLD_SECONDS {
            if self.next_state == PlayNextState::Shown {
                self.next_state = PlayNextState::Aborted;
                self.callbacks
                    .invoke(PlaylistManagerEvent::PlayingNextInAborted);
            }
            return;
        }

        self.next_state = PlayNextState::Shown;
        self.callbacks
            .invoke(PlaylistManagerEvent::PlayingNextIn(remaining_time));
    }

    async fn on_command(&mut self, command: PlaylistManagerCommand) {
        match command {
            PlaylistManagerCommand::GetPlaylistState { response } => {
                response.send(self.state());
            }
            PlaylistManagerCommand::GetPlaylist { response } => {
                response.send(self.playlist.iter().cloned().collect())
            }
            PlaylistManagerCommand::Play { playlist, response } => {
                response.send(self.play(playlist).await);
            }
            PlaylistManagerCommand::PlayNext { response } => {
                response.send(self.play_next().await);
            }
            PlaylistManagerCommand::HasNext { response } => {
                response.send(self.has_next());
            }
            PlaylistManagerCommand::Stop { response } => {
                response.send(self.stop().await);
            }
        }
    }

    async fn stop(&mut self) {
        if let Some(player) = self
            .player_manager
            .active_player()
            .await
            .and_then(|e| e.upgrade())
        {
            trace!("Playlist is stopping active player");
            player.stop().await;
        }

        self.playlist.clear();
        self.next_state = PlayNextState::Aborted;
        self.event_publisher.publish(Event::ClosePlayer);
        self.update_state(PlaylistState::Stopped);
        debug!("Playlist has been stopped");
    }

    /// Returns `true` when the next playlist item should be played when the playback stops.
    fn should_play_next_on_player_stopped(
        &self,
        time: Option<&u64>,
        duration: Option<&u64>,
    ) -> bool {
        let (time, duration) = match (time, duration) {
            (Some(time), Some(duration)) => (time, duration),
            _ => return false,
        };

        let remaining = duration.saturating_sub(*time);
        self.has_next() && *duration > 0 && remaining <= 1200
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::event::PlayerStoppedEvent;
    use crate::core::loader::MockMediaLoader;
    use crate::core::players::MockPlayerManager;
    use crate::core::players::Player;
    use crate::testing::MockPlayer;
    use crate::{init_logger, recv_timeout};
    use fx_callback::{Callback, MultiThreadedCallback};
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    macro_rules! playlist_item {
        ($title:expr) => {{
            playlist_item!($title, None)
        }};
        ($title:expr, $url:expr) => {{
            use crate::core::playlist::PlaylistItem;

            let title: String = $title.to_string();
            let url: Option<String> = $url;

            PlaylistItem {
                url,
                title: title.to_string(),
                caption: None,
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            }
        }};
    }

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
            Arc::new(loader),
        );

        playlist.add(playlist_item.clone());

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let PlaylistManagerEvent::PlaylistChanged = &*event {
                        tx_event.send(event.clone()).unwrap();
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
        match &*result {
            PlaylistManagerEvent::PlaylistChanged => {}
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::PlaylistChanged, but got {:?}",
                result
            ),
        }
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
            Arc::new(loader),
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
        let playlist = Playlist::from_iter(vec![
            playlist_item!(item1, Some(url.to_string())),
            playlist_item!(item2),
        ]);
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
            Arc::new(loader),
        );

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    tx_manager.send(event.clone()).unwrap()
                } else {
                    break;
                }
            }
        });

        // start the playlist
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        match &*result {
            PlaylistManagerEvent::PlaylistChanged => {}
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::PlaylistChanged, but got {:?}",
                result
            ),
        }

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item1.to_string(), result.title);

        let duration = 50000;
        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(duration));

        // invoke the player stopped event
        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: "".to_string(),
            media: None,
            time: Some(duration),
            duration: Some(duration),
        }));

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item2.to_string(), result.title);
    }

    #[tokio::test]
    async fn test_should_play_next_on_player_stopped() {
        init_logger!();
        let playlist = Playlist::from_iter(vec![
            playlist_item!("Foo"),
            playlist_item!("Bar"),
            playlist_item!("Lorem"),
        ]);
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .return_once(move || player_manager_subscription);
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let mut manager = InnerPlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(loader),
        );

        // start the playlist
        manager.play(playlist).await;

        // play next is not allowed when the time is not within the last second
        let result = manager.should_play_next_on_player_stopped(Some(&40000), Some(&120000));
        assert_eq!(false, result, "expected play next to not be allowed");

        // play next is allowed when the time is within the last second
        let result = manager.should_play_next_on_player_stopped(Some(&119990), Some(&120000));
        assert_eq!(true, result, "expected play next to be allowed");
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
            Arc::new(loader),
        );

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx_manager.send(event.clone()).unwrap()
            }
        });

        // start the playlist
        let playlist = Playlist::from_iter(vec![
            playlist_item!(item1, Some(url.to_string())),
            playlist_item!(item2),
        ]);
        manager.play(playlist).await;
        let result = recv_timeout!(&mut rx_manager, Duration::from_millis(200));
        match &*result {
            PlaylistManagerEvent::PlaylistChanged => {}
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::PlaylistChanged, but got {:?}",
                result
            ),
        }

        // verify the playlist item that has been loaded
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(item1.to_string(), result.title);

        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(100000));
        // should not invoke any events, otherwise, the MockPlayerManager will fail at this point due to too many calls
    }

    #[tokio::test]
    async fn test_player_time_changed() {
        init_logger!();
        let playlist = Playlist::from_iter(vec![playlist_item!("Item1"), playlist_item!("FooBar")]);
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
            Arc::new(loader),
        );

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    PlaylistManagerEvent::PlayingNextIn(_)
                    | PlaylistManagerEvent::PlayingNextInAborted => {
                        let _ = tx.send(event.clone());
                    }
                    _ => {}
                }
            }
        });

        let result = manager.play(playlist).await;
        assert_ne!(None, result, "expected to receive a playlist handle");

        callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(360_000));
        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(320_000));

        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(500),
            "expected to receive PlaylistManagerEvent::PlayingNextIn"
        );
        match &*result {
            PlaylistManagerEvent::PlayingNextIn(playing_in) => {
                assert_eq!(40u32, *playing_in);
            }
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::PlayingNext, but got {:?}",
                result
            ),
        }

        callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(270_000));
        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(200),
            "expected to receive PlaylistManagerEvent::PlayingNextInAborted"
        );
        match &*result {
            PlaylistManagerEvent::PlayingNextInAborted => {}
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::PlayingNextInAborted, but got {:?}",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_stop() {
        init_logger!();
        let event_publisher = EventPublisher::default();
        let callbacks = MultiThreadedCallback::new();
        let player_manager_subscription = callbacks.subscribe();
        let mut player = MockPlayer::new();
        player.expect_stop().times(1).return_const(());
        let player: Arc<Box<dyn Player>> = Arc::new(Box::new(player));
        let active_player = Arc::downgrade(&player);
        let mut player_manager = Box::new(MockPlayerManager::new());
        player_manager
            .expect_subscribe()
            .times(1)
            .return_once(move || player_manager_subscription);
        player_manager
            .expect_active_player()
            .times(1)
            .return_once(move || Some(active_player));
        let (tx, mut rx) = unbounded_channel();
        let player_manager = Arc::new(player_manager as Box<dyn PlayerManager>);
        let mut loader = MockMediaLoader::new();
        loader
            .expect_load_playlist_item()
            .returning(move |_| Handle::new());
        let manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            Arc::new(loader),
        );

        // start the playlist
        let playlist = Playlist::from_iter(vec![playlist_item!("Item1"), playlist_item!("Item2")]);
        let result = manager.play(playlist).await;
        assert!(
            result.is_some(),
            "expected a loader handle to have been returned"
        );

        // verify a next item is available
        let result = manager.has_next().await;
        assert_eq!(true, result, "expected a next item to have been available");

        // subscribe to the events
        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    PlaylistManagerEvent::StateChanged(_) => {
                        let _ = tx.send(event.clone());
                    }
                    _ => {}
                }
            }
        });

        // stop the playlist playback
        manager.stop().await;

        // wait for the state changed event
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        match &*result {
            PlaylistManagerEvent::StateChanged(state) => {
                assert_eq!(
                    &PlaylistState::Stopped,
                    state,
                    "expected the playlist to be stopped"
                );
            }
            _ => assert!(
                false,
                "expected PlaylistManagerEvent::StateChanged, but got {:?}",
                result
            ),
        }

        // check that the playlist has been cleared
        let result = manager.playlist().await;
        assert_eq!(false, result.has_next(), "expected an empty playlist");
    }
}
