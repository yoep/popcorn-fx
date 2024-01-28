use std::fmt::Debug;
use std::sync::{Arc, RwLock, Weak};
use std::sync::mpsc::{channel, Sender};

use derive_more::Display;
use log::{debug, error, info, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::{block_in_place, Callbacks, CoreCallback, CoreCallbacks};
use crate::core::events::{Event, EventPublisher, PlayerChangedEvent, PlayerStartedEvent};
use crate::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};

/// An event representing changes to the player manager.
#[derive(Debug, Clone, Display)]
pub enum PlayerManagerEvent {
    /// Event indicating that the active player has changed.
    #[display(fmt = "Active player changed")]
    ActivePlayerChanged(PlayerChange),
    /// Indicates that the list of players has changed.
    #[display(fmt = "Available players have been changed")]
    PlayersChanged,
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
    fn subscribe(&self, callback: PlayerManagerCallback);

    /// Play media content by submitting a play request to the player manager.
    ///
    /// # Arguments
    ///
    /// * `request` - A boxed trait object representing the play request.
    fn play(&self, request: Box<dyn PlayRequest>);
}

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

/// A default implementation of the `PlayerManager` trait.
#[derive(Debug)]
pub struct DefaultPlayerManager {
    active_player: Mutex<Option<String>>,
    players: RwLock<Vec<Arc<Box<dyn Player>>>>,
    listener_id: Mutex<i64>,
    listener_sender: Sender<PlayerEventWrapper>,
    callbacks: CoreCallbacks<PlayerManagerEvent>,
    event_publisher: Arc<EventPublisher>,
    _runtime: Runtime,
}

impl DefaultPlayerManager {
    pub fn new(event_publisher: Arc<EventPublisher>) -> Self {
        let runtime = Runtime::new().unwrap();
        let (listener_sender, listener_receiver) = channel::<PlayerEventWrapper>();
        let callbacks = CoreCallbacks::<PlayerManagerEvent>::default();

        let receiver_callbacks = callbacks.clone();
        runtime.spawn(async move {
            for received in listener_receiver {
                if let Some(event) = received.event {
                    match event {
                        PlayerEvent::DurationChanged(e) => receiver_callbacks.invoke(PlayerManagerEvent::PlayerDurationChanged(e)),
                        PlayerEvent::TimeChanged(e) => receiver_callbacks.invoke(PlayerManagerEvent::PlayerTimeChanged(e)),
                        PlayerEvent::StateChanged(e) => receiver_callbacks.invoke(PlayerManagerEvent::PlayerStateChanged(e)),
                        PlayerEvent::VolumeChanged(_) => {}
                    }
                }
                if received.is_shutdown {
                    break;
                }
            }

            debug!("Player manager event loop has been closed");
        });

        let instance = Self {
            active_player: Mutex::default(),
            players: RwLock::default(),
            listener_id: Default::default(),
            listener_sender,
            callbacks,
            event_publisher,
            _runtime: runtime,
        };

        instance
    }

    fn contains(&self, player_id: &str) -> bool {
        self.players.read().unwrap()
            .iter()
            .any(|e| e.id() == player_id)
    }

    fn update_player_listener(&self, old_player_id: Option<&String>) {
        if let Some(old_player) = old_player_id
            .and_then(|player_id| self.by_id(player_id.as_str()))
            .and_then(|player_ref| player_ref.upgrade()) {
            let listener_id = self.listener_id.blocking_lock();
            trace!("Removing internal player callback listener {}", listener_id);
            old_player.remove(listener_id.clone());
        }

        if let Some(new_player) = self.active_player.blocking_lock()
            .as_ref()
            .and_then(|e| self.by_id(e.as_str()))
            .and_then(|e| e.upgrade()) {
            trace!("Registering new internal player callback listener to {}", new_player);
            let sender = self.listener_sender.clone();
            let callback_id = new_player.add(Box::new(move |e| {
                let wrapper = PlayerEventWrapper::from(e);
                if let Err(e) = sender.send(wrapper) {
                    error!("Failed to send player event, {}", e);
                }
            }));

            let mut listener_id = self.listener_id.blocking_lock();
            trace!("Updating listener callback id to {}", callback_id);
            *listener_id = callback_id;
        }
    }
}

impl PlayerManager for DefaultPlayerManager {
    fn active_player(&self) -> Option<Weak<Box<dyn Player>>> {
        block_in_place(self.active_player.lock())
            .as_ref()
            .and_then(|id| self.by_id(id.as_str()))
            .map(|e| e)
    }

    fn set_active_player(&self, player_id: &str) {
        if let Some(player) = self.by_id(player_id).and_then(|player| player.upgrade()) {
            let old_player_id: Option<String>;
            let player_name = player.name().to_string();

            // reduce the lock time as much as possible
            {
                let mut active_player = self.active_player.blocking_lock();
                old_player_id = active_player.clone();
                debug!("Updating active player to {}", player_id);
                *active_player = Some(player_id.to_string());
            }

            debug!("Updating internal player listener");
            self.update_player_listener(old_player_id.as_ref());

            trace!("Publishing player changed event for {}", player_id);
            self.callbacks.invoke(PlayerManagerEvent::ActivePlayerChanged(PlayerChange {
                old_player_id: old_player_id.clone(),
                new_player_id: player_id.to_string(),
                new_player_name: player_name.clone(),
            }));
            self.event_publisher.publish(Event::PlayerChanged(PlayerChangedEvent {
                old_player_id,
                new_player_id: player_id.to_string(),
                new_player_name: player_name,
            }));
        } else {
            warn!("Unable to set {} as active player, player not found", player_id);
        }
    }

    fn players(&self) -> Vec<Weak<Box<dyn Player>>> {
        trace!("Retrieving registered players");
        let players = self.players.read().unwrap();
        trace!("Lock acquired");
        players.iter()
            .map(Arc::downgrade)
            .collect()
    }

    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>> {
        trace!("Retrieving player by id {}", id);
        self.players.read().unwrap().iter()
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

                trace!("Adding new player {} to player manager", player_info.as_str());
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
        let index = players.iter()
            .position(|e| e.id() == player_id);

        if let Some(index) = index {
            let player = players.remove(index);
            info!("Removed player {}", player);

            drop(players);
            self.callbacks.invoke(PlayerManagerEvent::PlayersChanged);
        } else {
            warn!("Unable to remove player {}, player not found", player_id);
        }
    }

    fn subscribe(&self, callback: PlayerManagerCallback) {
        self.callbacks.add(callback);
    }

    fn play(&self, request: Box<dyn PlayRequest>) {
        if let Some(player) = self.active_player()
            .and_then(|e| e.upgrade()) {
            debug!("Starting playback of {} in {}", request.url(), player);
            let player_started_event = PlayerStartedEvent::from(&request);
            player.play(request);
            self.event_publisher.publish(Event::PlayerStarted(player_started_event));
        } else {
            error!("Unable to start playback, no active player found");
        }
    }
}

impl Drop for DefaultPlayerManager {
    fn drop(&mut self) {
        self.listener_sender.send(PlayerEventWrapper {
            event: None,
            is_shutdown: true,
        }).expect("expected the sender to send a shutdown signal");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::events::DEFAULT_ORDER;
    use crate::core::players::MockPlayer;
    use crate::testing::init_logger;

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
        fn add(&self, callback: CoreCallback<PlayerEvent>) -> i64 {
            self.callbacks.add(callback)
        }

        fn remove(&self, callback_id: i64) {
            self.callbacks.remove(callback_id)
        }
    }

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

        fn state(&self) -> &PlayerState {
            &PlayerState::Unknown
        }

        fn play(&self, request: Box<dyn PlayRequest>) {
            todo!()
        }
    }

    #[test]
    fn test_active_player() {
        init_logger();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::default();
        player.expect_id()
            .return_const(player_id.to_string());
        player.expect_name()
            .return_const("Foo".to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let manager = DefaultPlayerManager::new(Arc::new(EventPublisher::default()));

        manager.add_player(player);
        let player = manager.by_id(player_id).expect("expected the player to have been found");
        manager.set_active_player(player.upgrade().unwrap().id());
        let result = manager.active_player();

        assert!(result.is_some(), "expected an active player to have been returned");
    }

    #[test]
    fn test_set_active_player() {
        init_logger();
        let player_id = "FooBar654";
        let mut player = MockPlayer::default();
        player.expect_id()
            .return_const(player_id.to_string());
        player.expect_name()
            .return_const("FooBar player".to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultPlayerManager::new(event_publisher.clone());

        event_publisher.register(Box::new(move |e| {
            match &e {
                Event::PlayerChanged(id) => tx.send(id.clone()).unwrap(),
                _ => {}
            }

            Some(e)
        }), DEFAULT_ORDER);
        manager.add_player(player);
        let player = manager.by_id(player_id).expect("expected the player to have been found");
        manager.set_active_player(player.upgrade().unwrap().id());

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(player_id, result.new_player_id.as_str(), "expected the ID event to be the same");
    }

    #[test]
    fn test_set_active_player_switch_listener() {
        init_logger();
        let player2_id = "Id2";
        let player1 = Box::new(DummyPlayer::new("Id1"));
        let player2 = Box::new(DummyPlayer::new(player2_id));
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultPlayerManager::new(event_publisher.clone());

        manager.subscribe(Box::new(move |e| {
            if let PlayerManagerEvent::PlayerDurationChanged(_) = &e {
                tx.send(e).unwrap();
            }
        }));
        manager.add_player(player1.clone());
        manager.add_player(player2);
        manager.set_active_player(player1.id());
        player1.callbacks.invoke(PlayerEvent::DurationChanged(25000));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlayerManagerEvent::PlayerDurationChanged(e) = result {
            assert_eq!(25000, e, "expected the duration of the player event to match");
        } else {
            assert!(false, "expected PlayerManagerEvent::PlayerDurationChanged, got {} instead", result)
        }

        manager.set_active_player(player2_id);
        player1.callbacks.invoke(PlayerEvent::DurationChanged(25000));
        let result = rx.recv_timeout(Duration::from_millis(200));
        assert!(result.is_err(), "expected the PlayerManagerEvent::PlayerDurationChanged to not have been invoked a 2nd time")
    }

    #[test]
    fn test_register_new_player() {
        init_logger();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::new();
        player.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let manager = DefaultPlayerManager::new(Arc::new(EventPublisher::default()));

        manager.add_player(player);
        let result = manager.by_id(player_id);

        assert!(result.is_some(), "expected the player to have been registered");
    }

    #[test]
    fn test_register_duplicate_player_id() {
        init_logger();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let mut player2 = MockPlayer::default();
        player2.expect_id()
            .return_const(player_id.to_string());
        let player2 = Box::new(player2) as Box<dyn Player>;
        let manager = DefaultPlayerManager::new(Arc::new(EventPublisher::default()));

        manager.add_player(player);
        let result = manager.by_id(player_id);
        assert!(result.is_some(), "expected the player to have been registered");

        manager.add_player(player2);
        let players = manager.players.read().unwrap();
        assert_eq!(1, players.len(), "expected the duplicate id player to not have been registered")
    }

    #[test]
    fn test_remove() {
        init_logger();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let manager = DefaultPlayerManager::new(Arc::new(EventPublisher::default()));

        manager.add_player(player);
        assert!(manager.by_id(player_id).is_some(), "expected the player to have been registered");

        manager.remove_player(player_id);
        assert!(manager.by_id(player_id).is_none(), "expected the player to have been removed");
    }
}