use crate::core::event::{Event, EventPublisher, PlayerStartedEvent, PlayerStoppedEvent};
use crate::core::media::MediaIdentifier;
use crate::core::players::{
    ManagerError, ManagerResult, PlayRequest, Player, PlayerEvent, PlayerState,
};
use crate::core::stream::StreamServer;
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
#[cfg(any(test, feature = "testing"))]
pub use mock::*;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, Weak};
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// An event representing changes to the player manager.
#[derive(Debug, Clone, Display)]
pub enum PlayerManagerEvent {
    /// Event indicating that the active player has changed.
    #[display("Active player changed")]
    ActivePlayerChanged(PlayerChange),
    /// Indicates that the list of players has changed.
    #[display("Available players have been changed")]
    PlayersChanged,
    /// Indicates that the active player playback has been changed with a new [PlayRequest].
    #[display("Player playback changed to {:?}", _0)]
    PlayerPlaybackChanged(PlayRequest),
    /// Indicates that the duration of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display("Active player duration changed to {}", _0)]
    PlayerDurationChanged(u64),
    /// Indicates that the time of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display("Active player time changed to {}", _0)]
    PlayerTimeChanged(u64),
    /// Indicates that the state of the active player has changed.
    ///
    /// This event acts as a convenient wrapper around the [Player]'s [PlayerEvent] callbacks,
    /// automatically switching to the new active player whenever it changes.
    #[display("Active player state changed to {}", _0)]
    PlayerStateChanged(PlayerState),
}

impl PartialEq for PlayerManagerEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PlayerManagerEvent::ActivePlayerChanged(a),
                PlayerManagerEvent::ActivePlayerChanged(b),
            ) => a == b,
            (PlayerManagerEvent::PlayersChanged, PlayerManagerEvent::PlayersChanged) => true,
            (
                PlayerManagerEvent::PlayerPlaybackChanged(_),
                PlayerManagerEvent::PlayerPlaybackChanged(_),
            ) => true,
            (
                PlayerManagerEvent::PlayerDurationChanged(a),
                PlayerManagerEvent::PlayerDurationChanged(b),
            ) => a == b,
            (
                PlayerManagerEvent::PlayerTimeChanged(a),
                PlayerManagerEvent::PlayerTimeChanged(b),
            ) => a == b,
            (
                PlayerManagerEvent::PlayerStateChanged(a),
                PlayerManagerEvent::PlayerStateChanged(b),
            ) => a == b,
            _ => false,
        }
    }
}

/// A struct representing changes in the active player.
#[derive(Debug, Display, Clone, PartialEq)]
#[display("Active player changed to {}", new_player_id)]
pub struct PlayerChange {
    pub old_player_id: Option<String>,
    pub new_player_id: String,
    pub new_player_name: String,
}

/// A trait for managing multiple players within a multimedia application.
#[async_trait]
pub trait PlayerManager: Debug + Callback<PlayerManagerEvent> + Send + Sync {
    /// Get the active player, if any.
    ///
    /// Returns `Some` containing a weak reference to the currently active player, or `None` if there is no active player.
    async fn active_player(&self) -> Option<Weak<Box<dyn Player>>>;

    /// Set the active player by specifying its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `player_id` - A reference to the player ID to set as active.
    async fn set_active_player(&self, player_id: &str);

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
    fn add_player(&self, player: Box<dyn Player>) -> ManagerResult<()>;

    /// Remove a player from the manager by specifying its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `player_id` - The unique identifier of the player to remove.
    fn remove_player(&self, player_id: &str);

    /// Play media content by submitting a play request to the player manager.
    ///
    /// # Arguments
    ///
    /// * `request` - A boxed trait object representing the play request.
    async fn play(&self, request: PlayRequest);
}

/// A player manager for handling player-related tasks.
///
/// The `DefaultPlayerManager` is responsible for managing player-related tasks such as handling player events and
/// ensuring the proper functioning of the player within the application.
#[derive(Debug, Clone)]
pub struct DefaultPlayerManager {
    inner: Arc<InnerPlayerManager>,
}

impl DefaultPlayerManager {
    /// Create a new `DefaultPlayerManager` instance.
    pub fn new(event_publisher: EventPublisher, stream_server: Arc<StreamServer>) -> Self {
        let (player_event_sender, player_event_receiver) = unbounded_channel();
        let inner = Arc::new(InnerPlayerManager::new(
            player_event_sender,
            event_publisher,
            stream_server,
        ));

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(player_event_receiver).await;
        });

        Self { inner }
    }
}

impl Callback<PlayerManagerEvent> for DefaultPlayerManager {
    fn subscribe(&self) -> Subscription<PlayerManagerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlayerManagerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

#[async_trait]
impl PlayerManager for DefaultPlayerManager {
    async fn active_player(&self) -> Option<Weak<Box<dyn Player>>> {
        self.inner.active_player().await
    }

    async fn set_active_player(&self, player_id: &str) {
        self.inner.set_active_player(player_id).await
    }

    fn players(&self) -> Vec<Weak<Box<dyn Player>>> {
        self.inner.players()
    }

    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>> {
        self.inner.by_id(id)
    }

    fn add_player(&self, player: Box<dyn Player>) -> ManagerResult<()> {
        self.inner.add_player(player)
    }

    fn remove_player(&self, player_id: &str) {
        self.inner.remove_player(player_id)
    }

    async fn play(&self, request: PlayRequest) {
        self.inner.play(request).await
    }
}

impl Drop for DefaultPlayerManager {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel()
    }
}

/// A default implementation of the `PlayerManager` trait.
#[derive(Debug)]
struct InnerPlayerManager {
    active_player: Mutex<Option<String>>,
    last_known_player_info: Arc<Mutex<PlayerData>>,
    players: RwLock<Vec<Arc<Box<dyn Player>>>>,
    player_listener_sender: UnboundedSender<PlayerEvent>,
    player_listener_cancellation: Mutex<CancellationToken>,
    stream_server: Arc<StreamServer>,
    callbacks: MultiThreadedCallback<PlayerManagerEvent>,
    event_publisher: EventPublisher,
    cancellation_token: CancellationToken,
}

impl InnerPlayerManager {
    fn new(
        listener_sender: UnboundedSender<PlayerEvent>,
        event_publisher: EventPublisher,
        stream_server: Arc<StreamServer>,
    ) -> Self {
        let instance = Self {
            active_player: Mutex::default(),
            last_known_player_info: Arc::new(Default::default()),
            players: RwLock::default(),
            player_listener_sender: listener_sender,
            player_listener_cancellation: Mutex::new(CancellationToken::new()),
            stream_server,
            callbacks: MultiThreadedCallback::new(),
            event_publisher,
            cancellation_token: Default::default(),
        };

        instance
    }

    async fn start(&self, mut player_event_receiver: UnboundedReceiver<PlayerEvent>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(event) = player_event_receiver.recv() => self.handle_player_event(event).await,
            }
        }
        debug!("Player manager main loop ended");
    }

    fn contains(&self, player_id: &str) -> bool {
        self.players
            .read()
            .unwrap()
            .iter()
            .any(|e| e.id() == player_id)
    }

    async fn update_player_listener(&self) {
        // cancel the previous event listener loop
        // this automatically drops the event receiver
        self.player_listener_cancellation.lock().await.cancel();

        if let Some(new_player) = self
            .active_player
            .lock()
            .await
            .as_ref()
            .and_then(|e| self.by_id(e.as_str()))
            .and_then(|e| e.upgrade())
        {
            trace!(
                "Registering new internal player callback listener to {}",
                new_player
            );
            let sender = self.player_listener_sender.clone();

            // replace the existing event listener cancellation token
            let cancellation_token = CancellationToken::new();
            *self.player_listener_cancellation.lock().await = cancellation_token.clone();

            // create a new player event listener
            let mut event_receiver = new_player.subscribe();
            tokio::spawn(async move {
                loop {
                    select! {
                        _ = cancellation_token.cancelled() => break,
                        event = event_receiver.recv() => {
                            if let Some(event) = event {
                                let _ = sender.send((*event).clone());
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
        }
    }

    async fn handle_player_event(&self, event: PlayerEvent) {
        match event {
            PlayerEvent::DurationChanged(e) => self.handle_player_duration_event(e).await,
            PlayerEvent::TimeChanged(e) => self.handle_player_time_event(e).await,
            PlayerEvent::StateChanged(e) => self.handle_player_state_changed(e).await,
            PlayerEvent::VolumeChanged(_) => {}
        }
    }

    async fn handle_player_duration_event(&self, new_duration: u64) {
        if new_duration > 0 {
            let mut mutex = self.last_known_player_info.lock().await;
            trace!("Updating last known player duration to {}", new_duration);
            mutex.duration = Some(new_duration.clone());
        }

        self.callbacks
            .invoke(PlayerManagerEvent::PlayerDurationChanged(new_duration));
    }

    async fn handle_player_time_event(&self, new_time: u64) {
        if new_time > 0 {
            let mut mutex = self.last_known_player_info.lock().await;
            trace!("Updating last known player time to {}", new_time);
            mutex.time = Some(new_time.clone());
        }

        self.callbacks
            .invoke(PlayerManagerEvent::PlayerTimeChanged(new_time));
    }

    async fn handle_player_state_changed(&self, new_state: PlayerState) {
        debug!("Player state changed to {}", new_state);
        if let PlayerState::Stopped = &new_state {
            let duration: u64;

            {
                let mut mutex = self.last_known_player_info.lock().await;
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

            if let Some(player) = self.active_player().await.and_then(|e| e.upgrade()) {
                trace!("Last known player duration was {}", duration);
                if duration > 0 {
                    if let Some(request) = player.request().await {
                        if let Some(stream) = request.stream().as_ref() {
                            debug!("Stopping player stream of {}", stream.filename);
                            self.stream_server
                                .stop_stream(stream.filename.as_str())
                                .await;
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

    async fn active_player(&self) -> Option<Weak<Box<dyn Player>>> {
        self.active_player
            .lock()
            .await
            .as_ref()
            .and_then(|id| self.by_id(id.as_str()))
            .map(|e| e)
    }

    async fn set_active_player(&self, player_id: &str) {
        if let Some(player) = self.by_id(player_id).and_then(|player| player.upgrade()) {
            trace!("Setting active player to {}", player_id);
            let old_player_id: Option<String>;
            let player_name = player.name().to_string();

            // reduce the lock time as much as possible
            {
                let mut active_player = self.active_player.lock().await;

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

            debug!("Player manager is updating internal player listener");
            self.update_player_listener().await;

            trace!(
                "Player manager is publishing player changed event for {}",
                player_id
            );
            self.callbacks
                .invoke(PlayerManagerEvent::ActivePlayerChanged(PlayerChange {
                    old_player_id: old_player_id.clone(),
                    new_player_id: player_id.to_string(),
                    new_player_name: player_name.clone(),
                }));

            info!("Player manager updated active player to {}", player_id);
        } else {
            warn!(
                "Player manager failed to set \"{}\" as active player, player not found",
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

    fn add_player(&self, player: Box<dyn Player>) -> ManagerResult<()> {
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
            return Ok(());
        }

        warn!("Player with id {} has already been registered", id);
        Err(ManagerError::DuplicatePlayer(id.to_string()))
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

    async fn play(&self, request: PlayRequest) {
        trace!("Processing play request {:?}", request);
        {
            let mut mutex = self.last_known_player_info.lock().await;
            mutex.url = Some(request.url().to_string());
            mutex.media = request.media();
        }

        if let Some(player) = self.active_player().await.and_then(|e| e.upgrade()) {
            debug!("Starting playback of {} in {}", request.url(), player);
            let player_started_event = PlayerStartedEvent::from(&request);

            player.play(request).await;

            self.event_publisher
                .publish(Event::PlayerStarted(player_started_event));
            if let Some(request) = player.request().await {
                // invoke the playback changed event
                self.callbacks
                    .invoke(PlayerManagerEvent::PlayerPlaybackChanged(request));
            }
        } else {
            error!("Unable to start playback, no active player found");
        }
    }
}

#[derive(Debug, Default)]
struct PlayerData {
    url: Option<String>,
    media: Option<Box<dyn MediaIdentifier>>,
    duration: Option<u64>,
    time: Option<u64>,
}

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use fx_callback::{Subscriber, Subscription};

    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub PlayerManager {}

        #[async_trait]
        impl PlayerManager for PlayerManager {
            fn add_player(&self, player: Box<dyn Player>) -> ManagerResult<()>;
            fn remove_player(&self, player_id: &str);
            fn players(&self) -> Vec<Weak<Box<dyn Player>>>;
            fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>>;
            async fn active_player(&self) -> Option<Weak<Box<dyn Player>>>;
            async fn set_active_player(&self, player_id: &str);
            async fn play(&self, request: PlayRequest);
        }

        impl Callback<PlayerManagerEvent> for PlayerManager {
            fn subscribe(&self) -> Subscription<PlayerManagerEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<PlayerManagerEvent>);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{event, stream};
    use crate::testing::MockPlayer;
    use crate::{init_logger, recv_timeout};

    use super::*;

    use crate::core::stream::ServerStream;
    use async_trait::async_trait;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::{select, time};
    use url::Url;

    #[derive(Debug, Display, Clone)]
    #[display("DummyPlayer")]
    struct DummyPlayer {
        id: String,
        callbacks: MultiThreadedCallback<PlayerEvent>,
    }

    impl DummyPlayer {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                callbacks: MultiThreadedCallback::new(),
            }
        }
    }

    impl Callback<PlayerEvent> for DummyPlayer {
        fn subscribe(&self) -> Subscription<PlayerEvent> {
            self.callbacks.subscribe()
        }

        fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>) {
            self.callbacks.subscribe_with(subscriber)
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

        async fn state(&self) -> PlayerState {
            PlayerState::Unknown
        }

        async fn request(&self) -> Option<PlayRequest> {
            None
        }

        async fn current_volume(&self) -> Option<u32> {
            None
        }

        async fn play(&self, _: PlayRequest) {
            // no-op
        }

        async fn pause(&self) {
            // no-op
        }

        async fn resume(&self) {
            // no-op
        }

        async fn seek(&self, _: u64) {
            // no-op
        }

        async fn stop(&self) {
            // no-op
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_active_player() {
        init_logger!();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("Foo".to_string());
        player.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let player = Box::new(player) as Box<dyn Player>;
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let _ = manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");
        manager
            .set_active_player(player.upgrade().unwrap().id())
            .await;
        let result = manager.active_player().await;

        assert!(
            result.is_some(),
            "expected an active player to have been returned"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_set_active_player() {
        init_logger!();
        let player_id = "FooBar654";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player
            .expect_name()
            .return_const("FooBar player".to_string());
        player.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let player = Box::new(player) as Box<dyn Player>;
        let (tx, mut rx) = unbounded_channel();
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let PlayerManagerEvent::ActivePlayerChanged(change) = &*event {
                    tx.send(change.clone()).unwrap();
                }
            }
        });
        let _ = manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");
        manager
            .set_active_player(player.upgrade().unwrap().id())
            .await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        assert_eq!(
            player_id,
            result.new_player_id.as_str(),
            "expected the ID event to be the same"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_set_active_player_twice() {
        init_logger!();
        let player_id = "FooBar654";
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player
            .expect_name()
            .return_const("FooBar player".to_string());
        player.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let player = Box::new(player) as Box<dyn Player>;
        let (tx, mut rx) = unbounded_channel();
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let PlayerManagerEvent::ActivePlayerChanged(change) = &*event {
                    tx.send(change.clone()).unwrap();
                }
            }
        });
        let _ = manager.add_player(player);
        let player = manager
            .by_id(player_id)
            .expect("expected the player to have been found");

        manager
            .set_active_player(player.upgrade().unwrap().id())
            .await;
        let _ = recv_timeout!(&mut rx, Duration::from_millis(100));

        manager
            .set_active_player(player.upgrade().unwrap().id())
            .await;
        let result = select! {
            _ = time::sleep(Duration::from_millis(100)) => false,
            Some(_) = rx.recv() => true,
        };
        assert_eq!(
            false, result,
            "expected the PlayerChanged to only have been published once"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_set_active_player_switch_listener() {
        init_logger!();
        let player2_id = "Id2";
        let player1 = Box::new(DummyPlayer::new("Id1"));
        let player2 = Box::new(DummyPlayer::new(player2_id));
        let (tx, mut rx) = unbounded_channel();
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let PlayerManagerEvent::PlayerDurationChanged(_) = &*event {
                        tx.send((*event).clone()).unwrap();
                    }
                } else {
                    break;
                }
            }
        });
        let _ = manager.add_player(player1.clone());
        let _ = manager.add_player(player2);
        manager.set_active_player(player1.id()).await;
        player1
            .callbacks
            .invoke(PlayerEvent::DurationChanged(25000));
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

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

        manager.set_active_player(player2_id).await;
        player1
            .callbacks
            .invoke(PlayerEvent::DurationChanged(25000));
        let result = select! {
            _ = time::sleep(Duration::from_millis(200)) => false,
            Some(_) = rx.recv() => true,
        };
        assert_eq!(false, result, "expected the PlayerManagerEvent::PlayerDurationChanged to not have been invoked a 2nd time")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_register_new_player() {
        init_logger!();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let _ = manager.add_player(player);
        let result = manager.by_id(player_id);

        assert!(
            result.is_some(),
            "expected the player to have been registered"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_register_duplicate_player_id() {
        init_logger!();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id().return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let mut player2 = MockPlayer::default();
        player2.expect_id().return_const(player_id.to_string());
        let player2 = Box::new(player2) as Box<dyn Player>;
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let _ = manager.add_player(player);
        let result = manager.by_id(player_id);
        assert!(
            result.is_some(),
            "expected the player to have been registered"
        );

        let _ = manager.add_player(player2);
        let players = manager.inner.players.read().unwrap();
        assert_eq!(
            1,
            players.len(),
            "expected the duplicate id player to not have been registered"
        )
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_player_stopped_event() {
        init_logger!();
        let player_id = "SomeId123";
        let filename = "my-video.mp4";
        let request = PlayRequest::builder()
            .url("http://localhost/my-video.mkv")
            .title("FooBar")
            .stream(ServerStream {
                url: Url::parse("http://localhost:6000/my-video.mp4").unwrap(),
                filename: filename.to_string(),
            })
            .build();
        let player_callbacks = MultiThreadedCallback::new();
        let player_subscription = player_callbacks.subscribe();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("MyPlayer".to_string());
        player
            .expect_subscribe()
            .times(1)
            .return_once(move || player_subscription);
        player.expect_request().times(1).return_const(request);
        let (tx_player_manager, mut rx_player_manager) = unbounded_channel();
        let (tx_events, mut rx_events) = unbounded_channel();
        let stream_server = Arc::new(StreamServer::new().await.unwrap());
        let manager = DefaultPlayerManager::new(EventPublisher::default(), stream_server.clone());

        // subscribe to the player manager events
        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let PlayerManagerEvent::PlayerStateChanged(_) = &*event {
                    tx_player_manager.send((*event).clone()).unwrap();
                    break;
                }
            }
        });

        // subscribe to the event publisher
        let mut receiver = manager
            .inner
            .event_publisher
            .subscribe(event::DEFAULT_ORDER)
            .unwrap();
        tokio::spawn(async move {
            while let Some(mut event_handler) = receiver.recv().await {
                if let Some(Event::ClosePlayer) = event_handler.event_ref() {
                    tx_events.send(()).unwrap();
                    event_handler.next();
                    break;
                }

                event_handler.next();
            }
        });

        let result = manager.add_player(Box::new(player));
        assert_eq!(Ok(()), result, "expected the player to have been added");
        manager.set_active_player(player_id).await;

        // invoke the player duration changed event, this is needed for the trigger of `Event::ClosePlayer`
        player_callbacks.invoke(PlayerEvent::DurationChanged(456800));
        // invoke the player stopped event
        player_callbacks.invoke(PlayerEvent::StateChanged(PlayerState::Stopped));

        // try to receive the player event through the player manager
        let result = recv_timeout!(
            &mut rx_player_manager,
            Duration::from_millis(500),
            "expected to receive a player event"
        );
        assert_eq!(
            PlayerManagerEvent::PlayerStateChanged(PlayerState::Stopped),
            result
        );

        // try to receive the close player event from the application
        let _ = recv_timeout!(
            &mut rx_events,
            Duration::from_millis(500),
            "expected to have received a close player event"
        );

        // verify if the torrent was removed from the torrent manager
        let result = stream_server
            .state(filename)
            .await
            .err()
            .expect("expected an error");
        assert_eq!(stream::Error::NotFound(filename.to_string()), result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_play() {
        init_logger!();
        let url = "MyUrl";
        let title = "FooBar";
        let player_id = "LoremIpsumPlayer";
        let request = PlayRequest::builder()
            .url(url)
            .title(title)
            .subtitles_enabled(false)
            .build();
        let (tx, mut rx) = unbounded_channel();
        let mut player = MockPlayer::default();
        player.expect_id().return_const(player_id.to_string());
        player.expect_name().return_const("FooBar".to_string());
        player.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        player.expect_play().times(1).returning(move |e| {
            tx.send(e).unwrap();
        });
        player.expect_request().return_const(request.clone());
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let _ = manager.add_player(Box::new(player));
        manager.set_active_player(player_id).await;

        manager.play(request).await;
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        assert_eq!(url, result.url());
        assert_eq!(title, result.title());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_remove() {
        init_logger!();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id().return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let stream_server = StreamServer::new().await.unwrap();
        let manager = DefaultPlayerManager::new(EventPublisher::default(), Arc::new(stream_server));

        let result = manager.add_player(player);
        assert_eq!(Ok(()), result, "expected the player to have been added");
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
