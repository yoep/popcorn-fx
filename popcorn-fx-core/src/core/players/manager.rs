use std::fmt::Debug;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock, Weak};

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::config::ApplicationConfig;
use crate::core::event::{
    Event, EventPublisher, PlayerChangedEvent, PlayerStartedEvent, PlayerStoppedEvent,
};
use crate::core::media::MediaIdentifier;
use crate::core::players::{PlayMediaRequest, PlayRequest, Player, PlayerEvent, PlayerState};
use crate::core::screen::ScreenService;
use crate::core::torrents::{TorrentManager, TorrentStreamServer};
use crate::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};

/// An event representing changes to the player manager.
#[derive(Debug, Clone, Display)]
pub enum PlayerManagerEvent {
    /// Event indicating that the active player has changed.
    #[display(fmt = "Active player changed")]
    ActivePlayerChanged(PlayerChange),
    /// Indicates that the list of players has changed.
    #[display(fmt = "Available players have been changed")]
    PlayersChanged,
    /// Indicates that the active player playback has been changed with a new [PlayRequest].
    #[display(fmt = "Player playback changed to {:?}", _0)]
    PlayerPlaybackChanged(Weak<Box<dyn PlayRequest>>),
    /// Indicates that the duration of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display(fmt = "Active player duration changed to {}", _0)]
    PlayerDurationChanged(u64),
    /// Indicates that the time of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display(fmt = "Active player time changed to {}", _0)]
    PlayerTimeChanged(u64),
    /// Indicates that the state of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display(fmt = "Active player state changed to {}", _0)]
    PlayerStateChanged(PlayerState),
}

/// A callback type for handling `PlayerManagerEvent` events.
pub type PlayerManagerCallback = CoreCallback<PlayerManagerEvent>;

/// A struct representing changes in the active player.
#[derive(Debug, Display, Clone)]
#[display(fmt = "Active player changed to {}", new_player_id)]
pub struct PlayerChange {
    pub old_player_id: Option<String>,
    pub new_player_id: String,
    pub new_player_name: String,
}

/// A trait for managing multiple players within a multimedia application.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait PlayerManager: Debug + Send + Sync {
    /// Get the active player, if any.
    ///
    /// Returns `Some` containing a weak reference to the currently active player, or `None` if there is no active player.
    fn active_player(&self) -> Option<Weak<Box<dyn Player>>>;

    /// Set the active player by specifying its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `player_id` - A reference to the player ID to set as active.
    fn set_active_player(&self, player_id: &str);

    /// Get a list of players managed by the manager.
    ///
    /// Returns a vector of weak references to player objects.
    fn players(&self) -> Vec<Weak<Box<dyn Player>>>;

    /// Get a player by its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the player to retrieve.
    ///
    /// Returns `Some` containing a weak reference to the player if found, or `None` if no player with the given ID exists.
    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>>;

    /// Register a new player with the manager.
    ///
    /// # Arguments
    ///
    /// * `player` - A boxed trait object implementing `Player` to be registered.
    ///
    /// Returns `true` if the player was successfully registered, or `false` if a player with the same ID already exists.
    fn add_player(&self, player: Box<dyn Player>) -> bool;

    /// Remove a player from the manager by specifying its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `player_id` - The unique identifier of the player to remove.
    fn remove_player(&self, player_id: &str);

    /// Subscribe to receive player manager events through a callback.
    fn subscribe(&self, callback: PlayerManagerCallback) -> CallbackHandle;

    /// Play media content by submitting a play request to the player manager.
    ///
    /// # Arguments
    ///
    /// * `request` - A boxed trait object representing the play request.
    async fn play(&self, request: Box<dyn PlayRequest>);
}

/// A wrapper for PlayerEvent with an optional event and shutdown flag.
///
/// The `PlayerEventWrapper` is used to wrap a `PlayerEvent` with an optional event payload and a flag to indicate
/// whether it represents a shutdown signal.
#[derive(Debug)]
struct PlayerEventWrapper {
    event: Option<PlayerEvent>,
    is_shutdown: bool,
}

impl From<PlayerEvent> for PlayerEventWrapper {
    fn from(value: PlayerEvent) -> Self {
        Self {
            event: Some(value),
            is_shutdown: false,
        }
    }
}

/// A player manager for handling player-related tasks.
///
/// The `DefaultPlayerManager` is responsible for managing player-related tasks such as handling player events and
/// ensuring the proper functioning of the player within the application.
#[derive(Debug)]
pub struct DefaultPlayerManager {
    inner: Arc<InnerPlayerManager>,
    _runtime: Runtime,
}

impl DefaultPlayerManager {
    /// Create a new `DefaultPlayerManager` instance.
    ///
    /// # Arguments
    ///
    /// * `application_config` - An Arc wrapped Mutex containing the application configuration.
    /// * `event_publisher` - An Arc wrapped EventPublisher for publishing player-related events.
    /// * `torrent_stream_server` - An Arc wrapped Box of a trait object implementing TorrentStreamServer.
    /// * `screen_service` - An Arc wrapped Box of a trait object implementing ScreenService.
    ///
    /// # Returns
    ///
    /// A new `DefaultPlayerManager` instance.
    pub fn new(
        application_config: Arc<ApplicationConfig>,
        event_publisher: Arc<EventPublisher>,
        torrent_manager: Arc<Box<dyn TorrentManager>>,
        torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
        screen_service: Arc<Box<dyn ScreenService>>,
    ) -> Self {
        let runtime = Runtime::new().unwrap();
        let (listener_sender, listener_receiver) = channel::<PlayerEventWrapper>();
        let inner = Arc::new(InnerPlayerManager::new(
            application_config,
            listener_sender,
            event_publisher,
            torrent_manager,
            torrent_stream_server,
            screen_service,
        ));

        let receiver_manager = inner.clone();
        runtime.spawn(async move {
            for received in listener_receiver {
                if let Some(event) = received.event {
                    receiver_manager.handle_player_event(event);
                }
                if received.is_shutdown {
                    trace!("Received shutdown signal for the player event receiver");
                    break;
                }
            }

            debug!("Player manager event loop has been closed");
        });

        Self {
            inner,
            _runtime: runtime,
        }
    }
}

#[async_trait]
impl PlayerManager for DefaultPlayerManager {
    fn active_player(&self) -> Option<Weak<Box<dyn Player>>> {
        self.inner.active_player()
    }

    fn set_active_player(&self, player_id: &str) {
        self.inner.set_active_player(player_id)
    }

    fn players(&self) -> Vec<Weak<Box<dyn Player>>> {
        self.inner.players()
    }

    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>> {
        self.inner.by_id(id)
    }

    fn add_player(&self, player: Box<dyn Player>) -> bool {
        self.inner.add_player(player)
    }

    fn remove_player(&self, player_id: &str) {
        self.inner.remove_player(player_id)
    }

    fn subscribe(&self, callback: PlayerManagerCallback) -> CallbackHandle {
        self.inner.subscribe(callback)
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        self.inner.play(request).await
    }
}

impl Drop for DefaultPlayerManager {
    fn drop(&mut self) {
        self.inner
            .listener_sender
            .send(PlayerEventWrapper {
                event: None,
                is_shutdown: true,
            })
            .expect("expected the sender to send a shutdown signal");
    }
}

/// A default implementation of the `PlayerManager` trait.
#[derive(Debug)]
struct InnerPlayerManager {
    application_config: Arc<ApplicationConfig>,
    active_player: Mutex<Option<String>>,
    last_known_player_info: Arc<Mutex<PlayerData>>,
    players: RwLock<Vec<Arc<Box<dyn Player>>>>,
    listener_id: Mutex<Option<CallbackHandle>>,
    listener_sender: Sender<PlayerEventWrapper>,
    torrent_manager: Arc<Box<dyn TorrentManager>>,
    torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
    screen_service: Arc<Box<dyn ScreenService>>,
    callbacks: CoreCallbacks<PlayerManagerEvent>,
    event_publisher: Arc<EventPublisher>,
}

impl InnerPlayerManager {
    fn new(
        application_config: Arc<ApplicationConfig>,
        listener_sender: Sender<PlayerEventWrapper>,
        event_publisher: Arc<EventPublisher>,
        torrent_manager: Arc<Box<dyn TorrentManager>>,
        torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
        screen_service: Arc<Box<dyn ScreenService>>,
    ) -> Self {
        let instance = Self {
            application_config,
            active_player: Mutex::default(),
            last_known_player_info: Arc::new(Default::default()),
            players: RwLock::default(),
            listener_id: Default::default(),
            listener_sender,
            torrent_manager,
            torrent_stream_server,
            screen_service,
            callbacks: CoreCallbacks::default(),
            event_publisher,
        };

        instance
    }

    fn contains(&self, player_id: &str) -> bool {
        self.players
            .read()
            .unwrap()
            .iter()
            .any(|e| e.id() == player_id)
    }

    fn update_player_listener(&self, old_player_id: Option<&String>) {
        if let Some(old_player) = old_player_id
            .and_then(|player_id| self.by_id(player_id.as_str()))
            .and_then(|player_ref| player_ref.upgrade())
        {
            if let Some(callback_handle) = block_in_place(self.listener_id.lock()).as_ref() {
                trace!(
                    "Removing internal player callback handle {}",
                    callback_handle
                );
                old_player.remove_callback(callback_handle.clone());
            }
        }

        if let Some(new_player) = block_in_place(self.active_player.lock())
            .as_ref()
            .and_then(|e| self.by_id(e.as_str()))
            .and_then(|e| e.upgrade())
        {
            trace!(
                "Registering new internal player callback listener to {}",
                new_player
            );
            let sender = self.listener_sender.clone();
            let callback_handle = new_player.add_callback(Box::new(move |e| {
                let wrapper = PlayerEventWrapper::from(e);
                if let Err(e) = sender.send(wrapper) {
                    error!("Failed to send player event, {}", e);
                }
            }));

            let mut listener_id = block_in_place(self.listener_id.lock());
            trace!("Updating listener callback id to {}", callback_handle);
            *listener_id = Some(callback_handle);
        }
    }

    fn handle_player_event(&self, event: PlayerEvent) {
        match event {
            PlayerEvent::DurationChanged(e) => self.handle_player_duration_event(e),
            PlayerEvent::TimeChanged(e) => self.handle_player_time_event(e),
            PlayerEvent::StateChanged(e) => self.handle_player_state_changed(e),
            PlayerEvent::VolumeChanged(_) => {}
        }
    }

    fn handle_player_duration_event(&self, new_duration: u64) {
        if new_duration > 0 {
            let mut mutex = block_in_place(self.last_known_player_info.lock());
            trace!("Updating last known player duration to {}", new_duration);
            mutex.duration = Some(new_duration.clone());
        }

        self.callbacks
            .invoke(PlayerManagerEvent::PlayerDurationChanged(new_duration));
    }

    fn handle_player_time_event(&self, new_time: u64) {
        if new_time > 0 {
            let mut mutex = block_in_place(self.last_known_player_info.lock());
            trace!("Updating last known player time to {}", new_time);
            mutex.time = Some(new_time.clone());
        }

        self.callbacks
            .invoke(PlayerManagerEvent::PlayerTimeChanged(new_time));
    }

    fn handle_player_state_changed(&self, new_state: PlayerState) {
        debug!("Player state changed to {}", new_state);

        if let PlayerState::Stopped = &new_state {
            let duration: u64;

            {
                let mut mutex = block_in_place(self.last_known_player_info.lock());
                trace!("Last known player info {:?}", mutex);
                duration = mutex.duration.take().unwrap_or(0);
                let event = Event::PlayerStopped(PlayerStoppedEvent {
                    url: mutex.url.take().unwrap_or(String::new()),
                    media: mutex.media.take(),
                    time: mutex.time.take(),
                    duration: Some(duration),
                });

                debug!("Publishing player stopped event {:?}", event);
                self.event_publisher.publish(event);
            }

            if let Some(player) = self.active_player().and_then(|e| e.upgrade()) {
                trace!("Last known player duration was {}", duration);
                if duration > 0 {
                    if let Some(request) = player.request().and_then(|e| e.upgrade()).map(|e| {
                        trace!("Last known playback request {:?}", e);
                        e
                    }) {
                        if let Some(stream) = request
                            .downcast_ref::<PlayMediaRequest>()
                            .and_then(|e| e.torrent_stream.upgrade())
                        {
                            debug!("Stopping player stream of {}", stream);
                            self.torrent_stream_server
                                .stop_stream(stream.stream_handle());
                            debug!("Stopping torrent download of {}", stream.handle());
                            self.torrent_manager.remove(stream.handle());
                        }
                    } else {
                        warn!(
                            "Unable to determine last playback request for player {}",
                            player
                        );
                    }

                    trace!("Player stopped event resulted in Event::ClosePlayer");
                    self.event_publisher.publish(Event::ClosePlayer);
                } else {
                    trace!(
                        "Skipping player stopped event, last known duration is {}",
                        duration
                    );
                }
            }
        }

        self.callbacks
            .invoke(PlayerManagerEvent::PlayerStateChanged(new_state))
    }

    fn handle_fullscreen_mode(&self) {
        let is_fullscreen_enabled: bool;
        {
            let settings = self.application_config.user_settings();
            is_fullscreen_enabled = settings.playback_settings.fullscreen.clone();
        }

        debug!("Playback fullscreen mode is {}", is_fullscreen_enabled);
        if is_fullscreen_enabled {
            self.screen_service.fullscreen(is_fullscreen_enabled);
        }
    }
}

#[async_trait]
impl PlayerManager for InnerPlayerManager {
    fn active_player(&self) -> Option<Weak<Box<dyn Player>>> {
        block_in_place(self.active_player.lock())
            .as_ref()
            .and_then(|id| self.by_id(id.as_str()))
            .map(|e| e)
    }

    fn set_active_player(&self, player_id: &str) {
        if let Some(player) = self.by_id(player_id).and_then(|player| player.upgrade()) {
            trace!("Setting active player to {}", player_id);
            let old_player_id: Option<String>;
            let player_name = player.name().to_string();

            // reduce the lock time as much as possible
            {
                let mut active_player = block_in_place(self.active_player.lock());

                // check the current player id
                if let Some(active_player) = active_player.as_ref() {
                    if active_player.as_str() == player_id {
                        debug!("Active player is already {}", player_id);
                        return;
                    }
                }

                old_player_id = active_player.clone();
                debug!("Updating active player to {}", player_id);
                *active_player = Some(player_id.to_string());
            }

            debug!("Updating internal player listener");
            self.update_player_listener(old_player_id.as_ref());

            trace!("Publishing player changed event for {}", player_id);
            self.callbacks
                .invoke(PlayerManagerEvent::ActivePlayerChanged(PlayerChange {
                    old_player_id: old_player_id.clone(),
                    new_player_id: player_id.to_string(),
                    new_player_name: player_name.clone(),
                }));
            self.event_publisher
                .publish(Event::PlayerChanged(PlayerChangedEvent {
                    old_player_id,
                    new_player_id: player_id.to_string(),
                    new_player_name: player_name,
                }));

            info!("Active player has changed to {}", player_id);
        } else {
            warn!(
                "Unable to set {} as active player, player not found",
                player_id
            );
        }
    }

    fn players(&self) -> Vec<Weak<Box<dyn Player>>> {
        trace!("Retrieving registered players");
        let players = self.players.read().unwrap();
        trace!("Lock acquired");
        players.iter().map(Arc::downgrade).collect()
    }

    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>> {
        trace!("Retrieving player by id {}", id);
        self.players
            .read()
            .unwrap()
            .iter()
            .find(|e| e.id() == id)
            .map(Arc::downgrade)
    }

    fn add_player(&self, player: Box<dyn Player>) -> bool {
        trace!("Trying to register new player {}", player.id());
        let id = player.id();

        if !self.contains(id) {
            {
                debug!("Registering player {}", player.id());
                let mut players = self.players.write().unwrap();
                let player_info = player.to_string();

                trace!(
                    "Adding new player {} to player manager",
                    player_info.as_str()
                );
                players.push(Arc::new(player));
                info!("New player {} has been added", player_info.as_str());
            }

            self.callbacks.invoke(PlayerManagerEvent::PlayersChanged);
            return true;
        }

        warn!("Player with id {} has already been registered", id);
        false
    }

    fn remove_player(&self, player_id: &str) {
        let mut players = self.players.write().unwrap();
        let index = players.iter().position(|e| e.id() == player_id);

        if let Some(index) = index {
            let player = players.remove(index);
            info!("Removed player {}", player);

            drop(players);
            self.callbacks.invoke(PlayerManagerEvent::PlayersChanged);
        } else {
            warn!("Unable to remove player {}, player not found", player_id);
        }
    }

    fn subscribe(&self, callback: PlayerManagerCallback) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        trace!("Processing play request {:?}", request);
        {
            let mut mutex = self.last_known_player_info.lock().await;
            mutex.url = Some(request.url().to_string());

            if let Some(e) = request.downcast_ref::<PlayMediaRequest>() {
                mutex.media = e.media.clone_identifier();
            }
        }

        if let Some(player) = self.active_player().and_then(|e| e.upgrade()) {
            debug!("Starting playback of {} in {}", request.url(), player);
            let player_started_event = PlayerStartedEvent::from(&request);

            player.play(request).await;

            self.event_publisher
                .publish(Event::PlayerStarted(player_started_event));
            if let Some(request) = player.request() {
                // invoke the playback changed event
                self.callbacks
                    .invoke(PlayerManagerEvent::PlayerPlaybackChanged(request));
            }
        } else {
            error!("Unable to start playback, no active player found");
        }

        // verify if we need to active the fullscreen mode
        self.handle_fullscreen_mode();
    }
}

#[derive(Debug, Default)]
struct PlayerData {
    url: Option<String>,
    media: Option<Box<dyn MediaIdentifier>>,
    duration: Option<u64>,
    time: Option<u64>,
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{channel, RecvTimeoutError};
    use std::time::Duration;

    use async_trait::async_trait;
    use tempfile::tempdir;

    use crate::core::config::{PlaybackSettings, PopcornSettings};
    use crate::core::event::DEFAULT_ORDER;
    use crate::core::media::MockMediaIdentifier;
    use crate::core::players::{PlaySubtitleRequest, PlayUrlRequest, PlayUrlRequestBuilder};
    use crate::core::screen::MockScreenService;
    use crate::core::torrents::{
        MockTorrentManager, MockTorrentStreamServer, TorrentHandle, TorrentStream,
    };
    use crate::core::{CallbackHandle, Handle};
    use crate::testing::{init_logger, MockPlayer, MockTorrentStream};

    use super::*;

    #[derive(Debug, Display, Clone)]
    #[display(fmt = "DummyPlayer")]
    struct DummyPlayer {
        id: String,
        callbacks: CoreCallbacks<PlayerEvent>,
    }

    impl DummyPlayer {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                callbacks: Default::default(),
            }
        }
    }

    impl Callbacks<PlayerEvent> for DummyPlayer {
        fn add_callback(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
            self.callbacks.add_callback(callback)
        }

        fn remove_callback(&self, handle: CallbackHandle) {
            self.callbacks.remove_callback(handle)
        }
    }

    #[async_trait]
    impl Player for DummyPlayer {
        fn id(&self) -> &str {
            self.id.as_str()
        }

        fn name(&self) -> &str {
            "DummyPlayer"
        }

        fn description(&self) -> &str {
            "DummyPlayer description"
        }

        fn graphic_resource(&self) -> Vec<u8> {
            Vec::new()
        }

        fn state(&self) -> PlayerState {
            PlayerState::Unknown
        }

        fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
            todo!()
        }

        async fn play(&self, _: Box<dyn PlayRequest>) {
            todo!()
        }

        fn pause(&self) {
            todo!()
        }

        fn resume(&self) {
            todo!()
        }

        fn seek(&self, _: u64) {
            todo!()
        }

        fn stop(&self) {
            todo!()
        }
    }

    #[test]
    fn test_active_player() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("Foo".to_string());
        player.expect_add_callback().return_const(1245i64);
        let player = Box::new(player) as Box<dyn Player>;
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");
        manager.set_active_player(player.upgrade().unwrap().id());
        let result = manager.active_player();

        assert!(
            result.is_some(),
            "expected an active player to have been returned"
        );
    }

    #[test]
    fn test_set_active_player() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "FooBar654";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player
            .expect_name()
            .return_const("FooBar player".to_string());
        player.expect_add_callback().return_const(1245i64);
        let player = Box::new(player) as Box<dyn Player>;
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            event_publisher.clone(),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        event_publisher.register(
            Box::new(move |e| {
                match &e {
                    Event::PlayerChanged(id) => tx.send(id.clone()).unwrap(),
                    _ => {}
                }

                Some(e)
            }),
            DEFAULT_ORDER,
        );
        manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");
        manager.set_active_player(player.upgrade().unwrap().id());

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(
            player_id,
            result.new_player_id.as_str(),
            "expected the ID event to be the same"
        );
    }

    #[test]
    fn test_set_active_player_twice() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "FooBar654";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player
            .expect_name()
            .return_const("FooBar player".to_string());
        player.expect_add_callback().return_const(1245i64);
        let player = Box::new(player) as Box<dyn Player>;
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            event_publisher.clone(),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        event_publisher.register(
            Box::new(move |e| {
                match &e {
                    Event::PlayerChanged(id) => tx.send(id.clone()).unwrap(),
                    _ => {}
                }

                Some(e)
            }),
            DEFAULT_ORDER,
        );
        manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");

        manager.set_active_player(player.upgrade().unwrap().id());
        rx.recv_timeout(Duration::from_millis(100))
            .expect("expected the PlayerChanged event to have been published");

        manager.set_active_player(player.upgrade().unwrap().id());
        let result = rx.recv_timeout(Duration::from_millis(100));
        assert_eq!(
            Err(RecvTimeoutError::Timeout),
            result,
            "expected the PlayerChanged to only have been published once"
        );
    }

    #[test]
    fn test_set_active_player_switch_listener() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player2_id = "Id2";
        let player1 = Box::new(DummyPlayer::new("Id1"));
        let player2 = Box::new(DummyPlayer::new(player2_id));
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            event_publisher.clone(),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        manager.subscribe(Box::new(move |e| {
            if let PlayerManagerEvent::PlayerDurationChanged(_) = &e {
                tx.send(e).unwrap();
            }
        }));
        manager.add_player(player1.clone());
        manager.add_player(player2);
        manager.set_active_player(player1.id());
        player1
            .callbacks
            .invoke(PlayerEvent::DurationChanged(25000));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlayerManagerEvent::PlayerDurationChanged(e) = result {
            assert_eq!(
                25000, e,
                "expected the duration of the player event to match"
            );
        } else {
            assert!(
                false,
                "expected PlayerManagerEvent::PlayerDurationChanged, got {} instead",
                result
            )
        }

        manager.set_active_player(player2_id);
        player1
            .callbacks
            .invoke(PlayerEvent::DurationChanged(25000));
        let result = rx.recv_timeout(Duration::from_millis(200));
        assert!(result.is_err(), "expected the PlayerManagerEvent::PlayerDurationChanged to not have been invoked a 2nd time")
    }

    #[test]
    fn test_register_new_player() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        manager.add_player(player);
        let result = manager.by_id(player_id);

        assert!(
            result.is_some(),
            "expected the player to have been registered"
        );
    }

    #[test]
    fn test_register_duplicate_player_id() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id().return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let mut player2 = MockPlayer::default();
        player2.expect_id().return_const(player_id.to_string());
        let player2 = Box::new(player2) as Box<dyn Player>;
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        manager.add_player(player);
        let result = manager.by_id(player_id);
        assert!(
            result.is_some(),
            "expected the player to have been registered"
        );

        manager.add_player(player2);
        let players = manager.inner.players.read().unwrap();
        assert_eq!(
            1,
            players.len(),
            "expected the duplicate id player to not have been registered"
        )
    }

    #[test]
    fn test_player_stopped_event() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "SomeId123";
        let torrent_handle = TorrentHandle::new();
        let stream_handle = Handle::new();
        let (tx, rx) = channel();
        let mut stream = MockTorrentStream::new();
        stream.expect_handle().return_const(torrent_handle);
        stream.expect_stream_handle().return_const(stream_handle);
        let stream = Arc::new(Box::new(stream) as Box<dyn TorrentStream>);
        let request: Arc<Box<dyn PlayRequest>> = Arc::new(Box::new(PlayMediaRequest {
            base: PlayUrlRequest {
                url: "".to_string(),
                title: "".to_string(),
                caption: None,
                thumb: None,
                background: None,
                auto_resume_timestamp: None,
                subtitle: PlaySubtitleRequest {
                    enabled: false,
                    info: None,
                    subtitle: None,
                },
            },
            parent_media: None,
            media: Box::new(MockMediaIdentifier::new()),
            quality: "".to_string(),
            torrent_stream: Arc::downgrade(&stream),
        }));
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("MyPlayer".to_string());
        player.expect_add_callback().returning(move |e| {
            tx.send(e).unwrap();
            Handle::new()
        });
        player
            .expect_request()
            .times(1)
            .returning(Box::new(move || Some(Arc::downgrade(&request))));
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_remove()
            .times(1)
            .withf(move |e| e == &torrent_handle)
            .return_const(());
        let mut torrent_stream_server = MockTorrentStreamServer::new();
        torrent_stream_server
            .expect_stop_stream()
            .times(1)
            .withf(move |handle| handle.clone() == stream_handle)
            .return_const(());
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        let result = manager.add_player(Box::new(player));
        assert!(result, "expected the player to have been added");
        manager.set_active_player(player_id);

        let callback = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        callback(PlayerEvent::StateChanged(PlayerState::Stopped));
    }

    #[test]
    fn test_play() {
        init_logger();
        let url = "MyUrl";
        let title = "FooBar";
        let player_id = "LoremIpsumPlayer";
        let request = PlayUrlRequestBuilder::builder()
            .url(url)
            .title(title)
            .subtitles_enabled(false)
            .build();
        let request_ref = Arc::new(Box::new(request.clone()) as Box<dyn PlayRequest>);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("FooBar".to_string());
        player.expect_add_callback().returning(|_| Handle::new());
        player.expect_play().times(1).returning(move |e| {
            tx.send(e).unwrap();
        });
        player
            .expect_request()
            .return_const(Arc::downgrade(&request_ref));
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let (tx_screen, rx_screen) = channel();
        let mut screen_service = MockScreenService::new();
        screen_service
            .expect_fullscreen()
            .times(1)
            .returning(move |fullscreen| {
                tx_screen.send(fullscreen).unwrap();
            });
        let settings = Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .settings(PopcornSettings {
                    subtitle_settings: Default::default(),
                    ui_settings: Default::default(),
                    server_settings: Default::default(),
                    torrent_settings: Default::default(),
                    playback_settings: PlaybackSettings {
                        quality: None,
                        fullscreen: true,
                        auto_play_next_episode_enabled: false,
                    },
                    tracking_settings: Default::default(),
                })
                .build(),
        );
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            Arc::new(Box::new(screen_service) as Box<dyn ScreenService>),
        );

        manager.add_player(Box::new(player));
        manager.set_active_player(player_id);

        block_in_place(manager.play(Box::new(request) as Box<dyn PlayRequest>));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(url, result.url());
        assert_eq!(title, result.title());

        let result = rx_screen.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn test_remove() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id().return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let torrent_manager = MockTorrentManager::new();
        let torrent_stream_server = MockTorrentStreamServer::new();
        let screen_service = Arc::new(Box::new(MockScreenService::new()) as Box<dyn ScreenService>);
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let manager = DefaultPlayerManager::new(
            settings,
            Arc::new(EventPublisher::default()),
            Arc::new(Box::new(torrent_manager)),
            Arc::new(Box::new(torrent_stream_server)),
            screen_service,
        );

        manager.add_player(player);
        assert!(
            manager.by_id(player_id).is_some(),
            "expected the player to have been registered"
        );

        manager.remove_player(player_id);
        assert!(
            manager.by_id(player_id).is_none(),
            "expected the player to have been removed"
        );
    }
}
