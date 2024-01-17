use std::sync::{Arc, RwLock, Weak};

use log::{debug, info, trace, warn};
use tokio::sync::Mutex;

use crate::core::players::Player;

/// A trait for managing multiple players.
pub trait PlayerManager {
    /// Get the identifier of the active player, if any.
    ///
    /// Returns `Some` containing the identifier of the active player, or `None` if there is no active player.
    fn active_player(&self) -> Option<String>;

    /// Set the active player.
    ///
    /// # Arguments
    ///
    /// * `player_id` - A reference to the player id to set as active.
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
    fn register(&self, player: Box<dyn Player>) -> bool;

    /// Remove a player from the manager by its unique identifier (ID).
    ///
    /// # Arguments
    ///
    /// * `player_id` - The unique identifier of the player to remove.
    fn remove(&self, player_id: &str);
}

/// A default implementation of the `PlayerManager` trait.
#[derive(Debug, Default)]
pub struct DefaultPlayerManager {
    active_player: Mutex<Option<String>>,
    players: RwLock<Vec<Arc<Box<dyn Player>>>>,
}

impl PlayerManager for DefaultPlayerManager {
    fn active_player(&self) -> Option<String> {
        let active_player = self.active_player.blocking_lock();
        active_player.as_ref()
            .map(|e| e.clone())
    }

    fn set_active_player(&self, player_id: &str) {
        if self.by_id(player_id).is_some() {
            let mut active_player = self.active_player.blocking_lock();
            debug!("Updating active player to {}", player_id);
            *active_player = Some(player_id.to_string());
        } else {
            warn!("Unable to set {} as active player, player not found", player_id);
        }
    }

    fn players(&self) -> Vec<Weak<Box<dyn Player>>> {
        let players = self.players.read().unwrap();

        players.iter()
            .map(Arc::downgrade)
            .collect()
    }

    fn by_id(&self, id: &str) -> Option<Weak<Box<dyn Player>>> {
        let players = self.players.read().unwrap();

        players.iter()
            .find(|e| e.id() == id)
            .map(Arc::downgrade)
    }

    fn register(&self, player: Box<dyn Player>) -> bool {
        let id = player.id();

        if self.by_id(id).is_none() {
            let mut players = self.players.write().unwrap();
            let player_info = player.to_string();

            trace!("Registering new player {}", player_info.as_str());
            players.push(Arc::new(player));
            info!("New player {} has been added", player_info.as_str());

            return true;
        }

        warn!("Player with id {} has already been registered", id);
        false
    }

    fn remove(&self, player_id: &str) {
        let mut players = self.players.write().unwrap();
        let index = players.iter()
            .position(|e| e.id() == player_id);

        if let Some(index) = index {
            let player = players.remove(index);
            info!("Removed player {}", player)
        } else {
            warn!("Unable to remove player {}, player not found", player_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::players::MockPlayer;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_active_player() {
        init_logger();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::default();
        player.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let mut manager = DefaultPlayerManager::default();

        manager.register(player);
        let player = manager.by_id(player_id).expect("expected the player to have been found");
        manager.set_active_player(player.upgrade().unwrap().id());
        let result = manager.active_player();

        assert!(result.is_some(), "expected an active player to have been returned");
    }

    #[test]
    fn test_register_new_player() {
        init_logger();
        let player_id = "MyPlayerId";
        let mut player = MockPlayer::default();
        player.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player) as Box<dyn Player>;
        let mut manager = DefaultPlayerManager::default();

        manager.register(player);
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
        let mut manager = DefaultPlayerManager::default();

        manager.register(player);
        let result = manager.by_id(player_id);
        assert!(result.is_some(), "expected the player to have been registered");

        manager.register(player2);
        let players = manager.players.read().unwrap();
        assert_eq!(1, players.len(), "expected the duplicate id player to not have been registered")
    }

    #[test]
    fn rest_remove() {
        init_logger();
        let player_id = "SomePlayer123";
        let mut player1 = MockPlayer::default();
        player1.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(player1) as Box<dyn Player>;
        let mut manager = DefaultPlayerManager::default();

        manager.register(player);
        assert!(manager.by_id(player_id).is_some(), "expected the player to have been registered");

        manager.remove(player_id);
        assert!(manager.by_id(player_id).is_none(), "expected the player to have been removed");
    }
}