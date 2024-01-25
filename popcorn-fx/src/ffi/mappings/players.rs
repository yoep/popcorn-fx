use std::{mem, ptr};
use std::os::raw::c_char;
use std::sync::{Arc, Weak};

use derive_more::Display;
use log::trace;

use popcorn_fx_core::{from_c_string, into_c_owned, into_c_string, to_c_vec};
use popcorn_fx_core::core::{Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerManagerEvent, PlayerState};

use crate::ffi::{ByteArray, PlayerChangedEventC};

pub type PlayerManagerEventCallback = extern "C" fn(PlayerManagerEventC);

/// A C-compatible struct representing a player.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerC {
    /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
    pub id: *const c_char,
    /// A pointer to a null-terminated C string representing the name of the player.
    pub name: *const c_char,
    /// A pointer to a null-terminated C string representing the description of the player.
    pub description: *const c_char,
    /// A pointer to a `ByteArray` struct representing the graphic resource associated with the player.
    ///
    /// This field can be a null pointer if no graphic resource is associated with the player.
    pub graphic_resource: *mut ByteArray,
    /// The state of the player.
    pub state: PlayerState,
    /// Indicates whether embedded playback is supported by the player.
    pub embedded_playback_supported: bool,
}

impl From<Arc<Box<dyn Player>>> for PlayerC {
    fn from(value: Arc<Box<dyn Player>>) -> Self {
        trace!("Converting Player to PlayerC");
        let id = into_c_string(value.id().to_string());
        let name = into_c_string(value.name().to_string());
        let description = into_c_string(value.description().to_string());
        let graphic_resource = if !value.graphic_resource().is_empty() {
            into_c_owned(ByteArray::from(value.graphic_resource()))
        } else {
            ptr::null_mut()
        };
        let embedded_playback_supported = if let Some(e) = value.downcast_ref::<PlayerWrapper>() {
            e.embedded_playback_supported.clone()
        } else {
            false
        };

        Self {
            id,
            name,
            description,
            graphic_resource,
            state: value.state().clone(),
            embedded_playback_supported,
        }
    }
}

#[repr(C)]
#[derive(Debug, Display, Clone)]
#[display(fmt = "id: {}, name: {}", id, name)]
pub struct PlayerWrapper {
    id: String,
    name: String,
    description: String,
    graphic_resource: Vec<u8>,
    state: PlayerState,
    embedded_playback_supported: bool,
    callbacks: CoreCallbacks<PlayerEvent>,
}

impl PlayerWrapper {
    pub fn invoke(&self, event: PlayerEvent) {
        self.callbacks.invoke(event);
    }
}

impl Callbacks<PlayerEvent> for PlayerWrapper {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> i64 {
        self.callbacks.add(callback)
    }

    fn remove(&self, callback_id: i64) {
        self.callbacks.remove(callback_id)
    }
}

impl Player for PlayerWrapper {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }

    fn graphic_resource(&self) -> Vec<u8> {
        self.graphic_resource.clone()
    }

    fn state(&self) -> &PlayerState {
        &self.state
    }
}

impl From<PlayerC> for PlayerWrapper {
    fn from(value: PlayerC) -> Self {
        trace!("Converting PlayerC to PlayerWrapperC");
        let id = from_c_string(value.id);
        let name = from_c_string(value.name);
        let description = from_c_string(value.description);
        let graphic_resource = if !value.graphic_resource.is_null() {
            let bytes = unsafe { value.graphic_resource.read() };
            let result = Vec::from(&bytes);
            mem::forget(bytes);
            result
        } else {
            Vec::new()
        };

        Self {
            id,
            name,
            description,
            graphic_resource,
            state: value.state.clone(),
            embedded_playback_supported: value.embedded_playback_supported.clone(),
            callbacks: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerWrapperC {
    wrapper: Weak<Box<dyn Player>>,
}

impl PlayerWrapperC {
    pub fn instance(&self) -> Option<Arc<Box<dyn Player>>> {
        self.wrapper.upgrade()
    }
}

impl From<Weak<Box<dyn Player>>> for PlayerWrapperC {
    fn from(value: Weak<Box<dyn Player>>) -> Self {
        Self {
            wrapper: value,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerSet {
    pub players: *mut PlayerC,
    pub len: i32,
}

impl From<Vec<PlayerC>> for PlayerSet {
    fn from(value: Vec<PlayerC>) -> Self {
        trace!("Converting C players to PlayerSet");
        let (players, len) = to_c_vec(value);

        Self {
            players,
            len,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum PlayerManagerEventC {
    ActivePlayerChanged(PlayerChangedEventC),
    PlayersChanged,
    PlayerDurationChanged(u64),
    PlayerTimeChanged(u64),
    PlayerStateChanged(PlayerState),
}

impl From<PlayerManagerEvent> for PlayerManagerEventC {
    fn from(value: PlayerManagerEvent) -> Self {
        match value {
            PlayerManagerEvent::ActivePlayerChanged(e) => PlayerManagerEventC::ActivePlayerChanged(PlayerChangedEventC::from(e)),
            PlayerManagerEvent::PlayersChanged => PlayerManagerEventC::PlayersChanged,
            PlayerManagerEvent::PlayerDurationChanged(e) => PlayerManagerEventC::PlayerDurationChanged(e),
            PlayerManagerEvent::PlayerTimeChanged(e) => PlayerManagerEventC::PlayerTimeChanged(e),
            PlayerManagerEvent::PlayerStateChanged(e) => PlayerManagerEventC::PlayerStateChanged(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use popcorn_fx_core::{from_c_owned, from_c_vec};
    use popcorn_fx_core::core::players::{MockPlayer, PlayerChange};
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_player() {
        init_logger();
        let player_id = "FooBar123";
        let player_name = "foo";
        let player_description = "lorem ipsum dolor";
        let graphic_resource = vec![80, 20];
        let mut mock_player = MockPlayer::new();
        mock_player.expect_id()
            .return_const(player_id.to_string());
        mock_player.expect_name()
            .return_const(player_name.to_string());
        mock_player.expect_description()
            .return_const(player_description.to_string());
        mock_player.expect_graphic_resource()
            .return_const(graphic_resource.clone());
        mock_player.expect_state()
            .return_const(PlayerState::Playing);
        let player = Arc::new(Box::new(mock_player) as Box<dyn Player>);

        let result = PlayerC::from(player);

        let bytes = from_c_owned(result.graphic_resource);
        assert_eq!(player_id.to_string(), from_c_string(result.id));
        assert_eq!(player_name.to_string(), from_c_string(result.name));
        assert_eq!(player_description, from_c_string(result.description));
        assert_eq!(graphic_resource, Vec::from(&bytes));
        assert_eq!(PlayerState::Playing, result.state);
    }

    #[test]
    fn test_from_player_for_wrapper() {
        init_logger();
        let state = PlayerState::Stopped;
        let player = Arc::new(Box::new(PlayerWrapper {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            graphic_resource: vec![],
            state: state.clone(),
            embedded_playback_supported: true,
            callbacks: Default::default(),
        }) as Box<dyn Player>);

        let result = PlayerC::from(player);

        assert_eq!(state, result.state);
        assert_eq!(true, result.embedded_playback_supported, "expected the embedded playback value to have been set");
    }

    #[test]
    fn from_players() {
        init_logger();
        let player_id = "player123";
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("my_player".to_string()),
            description: ptr::null(),
            graphic_resource: ptr::null_mut(),
            state: PlayerState::Stopped,
            embedded_playback_supported: false,
        };
        let players = vec![player];

        let set = PlayerSet::from(players);
        assert_eq!(1, set.len);

        let vec = from_c_vec(set.players, set.len);
        let result = vec.get(0).unwrap();
        assert_eq!(player_id.to_string(), from_c_string(result.id));
    }

    #[test]
    fn test_from_player_c() {
        init_logger();
        let player_id = "InternalPlayerId";
        let player_name = "InternalPlayerName";
        let description = "Lorem ipsum dolor esta";
        let resource = vec![84, 78, 90];
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string(player_name.to_string()),
            description: into_c_string(description.to_string()),
            graphic_resource: into_c_owned(ByteArray::from(resource.clone())),
            state: PlayerState::Paused,
            embedded_playback_supported: false,
        };

        let wrapper = PlayerWrapper::from(player);

        assert_eq!(player_id, wrapper.id());
        assert_eq!(player_name, wrapper.name());
        assert_eq!(description, wrapper.description());
        assert_eq!(resource, wrapper.graphic_resource());
    }

    #[test]
    fn test_player_manager_event_c_from() {
        let player_id = "MyId";
        let event = PlayerManagerEvent::ActivePlayerChanged(PlayerChange {
            old_player_id: None,
            new_player_id: player_id.to_string(),
            new_player_name: "".to_string(),
        });

        let result = PlayerManagerEventC::from(event);
        if let PlayerManagerEventC::ActivePlayerChanged(e) = result {
            assert_eq!(player_id.to_string(), from_c_string(e.new_player_id));
        } else {
            assert!(false, "expected PlayerManagerEventC::ActivePlayerChanged, got {:?} instead", result);
        }

        let result = crate::ffi::mappings::players::PlayerManagerEventC::from(PlayerManagerEvent::PlayersChanged);
        if let PlayerManagerEventC::PlayersChanged = result {} else {
            assert!(false, "expected PlayerManagerEventC::PlayersChanged, got {:?} instead", result);
        }
    }
}