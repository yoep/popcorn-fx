use std::os::raw::c_char;
use std::ptr;

use log::{debug, error, info, trace, warn};

use popcorn_fx_core::{from_c_string, into_c_owned};
use popcorn_fx_core::core::players::{Player, PlayerEvent};

use crate::ffi::{PlayerC, PlayerManagerEventC, PlayerManagerEventCallback, PlayerSet, PlayerWrapper, PlayerWrapperC};
use crate::PopcornFX;

/// Retrieve a pointer to the active player as a `PlayerC` instance from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns a pointer to a `PlayerC` instance representing the active player, or a null pointer if there is no active player.
#[no_mangle]
pub extern "C" fn active_player(popcorn_fx: &mut PopcornFX) -> *mut PlayerC {
    trace!("Retrieving C active player");
    match popcorn_fx.player_manager().active_player() {
        None => ptr::null_mut(),
        Some(e) => {
            e.upgrade()
                .map(|e| PlayerC::from(e))
                .map(|e| into_c_owned(e))
                .unwrap_or(ptr::null_mut())
        }
    }
}

/// Set the active player in the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
#[no_mangle]
pub extern "C" fn set_active_player(popcorn_fx: &mut PopcornFX, player_id: *const c_char) {
    let player_id = from_c_string(player_id);
    trace!("Updating active player from C to {}", player_id);

    popcorn_fx.player_manager().set_active_player(player_id.as_str());
}

/// Retrieve a pointer to a `PlayerSet` containing information about all players managed by PopcornFX.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns a pointer to a `PlayerSet` containing information about all players managed by PopcornFX.
#[no_mangle]
pub extern "C" fn players(popcorn_fx: &mut PopcornFX) -> *mut PlayerSet {
    trace!("Retrieving players from C");
    let players = popcorn_fx.player_manager().players().into_iter()
        .filter_map(|e| e.upgrade())
        .map(|e| PlayerC::from(e))
        .collect::<Vec<PlayerC>>();

    debug!("Retrieved a total of {} C players", players.len());
    into_c_owned(PlayerSet::from(players))
}

/// Retrieve a pointer to a `PlayerC` instance by its unique identifier (ID) from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
///
/// # Returns
///
/// Returns a pointer to a `PlayerC` instance representing the player if found, or a null pointer if no player with the given ID exists.
#[no_mangle]
pub extern "C" fn player_by_id(popcorn_fx: &mut PopcornFX, player_id: *const c_char) -> *mut PlayerC {
    let player_id = from_c_string(player_id);
    trace!("Retrieving C player by id {}", player_id);

    popcorn_fx.player_manager()
        .by_id(player_id.as_str())
        .and_then(|e| e.upgrade())
        .map(|e| PlayerC::from(e))
        .map(|e| into_c_owned(e))
        .unwrap_or(ptr::null_mut())
}

/// Register a callback function to be notified of player manager events.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` pointer.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A C-compatible callback function that will be invoked when player manager events occur.
#[no_mangle]
pub extern "C" fn register_player_callback(popcorn_fx: &mut PopcornFX, callback: PlayerManagerEventCallback) {
    trace!("Registering new player manager callback");
    popcorn_fx.player_manager().subscribe(Box::new(move |event| {
        callback(PlayerManagerEventC::from(event.clone()))
    }));
}

/// Register a player with the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player` - A `PlayerC` instance to be registered with the player manager.
///
/// # Notes
///
/// This function registers a player with the PopcornFX player manager using the provided `PlayerC` instance.
/// It logs an info message if the registration is successful and a warning message if registration fails.
#[no_mangle]
pub extern "C" fn register_player(popcorn_fx: &mut PopcornFX, player: PlayerC) -> *mut PlayerWrapperC {
    trace!("Registering new C player");
    let player = PlayerWrapper::from(player);
    let id = player.id().to_string();

    if popcorn_fx.player_manager().add_player(Box::new(player)) {
        info!("Registered new C player {}", id);
        popcorn_fx.player_manager().by_id(id.as_str())
            .map(|e| PlayerWrapperC::from(e))
            .map(|e| into_c_owned(e))
            .unwrap_or(ptr::null_mut())
    } else {
        warn!("Failed to register C player {}", id);
        ptr::null_mut()
    }
}

/// Remove a player with the specified ID from the PopcornFX player manager.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `player_id` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `player_id` - A pointer to a null-terminated C string representing the player's unique identifier (ID).
///
/// # Notes
///
/// This function removes a player with the specified ID from the PopcornFX player manager.
/// It converts the `player_id` C string to a Rust String and logs a trace message to indicate the removal.
#[no_mangle]
pub extern "C" fn remove_player(popcorn_fx: &mut PopcornFX, player_id: *const c_char) {
    let id = from_c_string(player_id);

    trace!("Removing C player ID {}", id);
    popcorn_fx.player_manager().remove_player(id.as_str());
}

/// Invoke a player event on a wrapped player instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance that wraps a player.
/// * `event` - The player event to invoke.
///
/// # Notes
///
/// This function checks if the `player` instance exists and is of the expected type (`PlayerWrapper`).
/// If the conditions are met, it invokes the specified player event on the wrapped player.
#[no_mangle]
pub extern "C" fn invoke_player_event(player: &mut PlayerWrapperC, event: PlayerEvent) {
    match player.instance() {
        Some(player) => {
            player.downcast_ref::<PlayerWrapper>().map(|wrapper| {
                trace!("Invoking player C event {}", event);
                wrapper.invoke(event);
            }).unwrap_or_else(|| {
                error!("Unable to process C player event, player instance is not of type PlayerWrapper");
            });
        }
        None => {
            warn!("Unable to process C player event, player instance has been disposed");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_vec, into_c_string};
    use popcorn_fx_core::core::Callbacks;
    use popcorn_fx_core::core::players::PlayerState;
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::ByteArray;
    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_active_player() {
        init_logger();
        let player_id = "Lorem";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerWrapper::from(PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            state: PlayerState::Playing,
            embedded_playback_supported: false,
        });

        instance.player_manager().add_player(Box::new(player));
        set_active_player(&mut instance, into_c_string(player_id.to_string()));
        let result = from_c_owned(active_player(&mut instance));

        assert_eq!(player_id.to_string(), from_c_string(result.id));
    }

    #[test]
    fn test_players() {
        init_logger();
        let player_id = "MyPlayerId999";
        let graphic_resource = vec![80, 20];
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: into_c_owned(ByteArray::from(graphic_resource.clone())),
            state: PlayerState::Paused,
            embedded_playback_supported: false,
        };

        register_player(&mut instance, player);
        let set = from_c_owned(players(&mut instance));
        let players = from_c_vec(set.players, set.len);

        let result = players.get(0).unwrap();
        let bytes = from_c_owned(result.graphic_resource);
        assert_eq!(1, players.len());
        assert_eq!(player_id.to_string(), from_c_string(result.id));
        assert_eq!(graphic_resource, Vec::from(&bytes));
    }

    #[test]
    fn test_player_by_id() {
        init_logger();
        let player_id = "MyId666";
        let name = "VlcPlayer";
        let graphic_resource = vec![155, 30, 16];
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string(name.to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: into_c_owned(ByteArray::from(graphic_resource.clone())),
            state: PlayerState::Paused,
            embedded_playback_supported: false,
        };

        register_player(&mut instance, player);
        let player = from_c_owned(player_by_id(&mut instance, into_c_string(player_id.to_string())));

        let bytes = from_c_owned(player.graphic_resource);
        assert_eq!(player_id.to_string(), from_c_string(player.id));
        assert_eq!(name.to_string(), from_c_string(player.name));
        assert_eq!(graphic_resource, Vec::from(&bytes));
    }

    #[test]
    fn test_register_player() {
        init_logger();
        let player_id = "Id123";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            state: PlayerState::Error,
            embedded_playback_supported: false,
        };

        register_player(&mut instance, player);

        let players = instance.player_manager().players();
        if let Some(player) = players.get(0) {
            assert_eq!(player_id, player.upgrade().unwrap().id());
        } else {
            assert!(false, "expected at least one player to be registered")
        }
    }

    #[test]
    fn test_remove_player() {
        init_logger();
        let player_id = "Id123";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            state: PlayerState::Buffering,
            embedded_playback_supported: false,
        };

        register_player(&mut instance, player);
        assert_eq!(1, instance.player_manager().players().len());
        remove_player(&mut instance, into_c_string(player_id.to_string()));
        assert_eq!(0, instance.player_manager().players().len(), "expected the player to have been removed from the player manager");
    }

    #[test]
    fn test_invoke_player_event() {
        init_logger();
        let expected_result = 240;
        let player = PlayerWrapper::from(PlayerC {
            id: ptr::null(),
            name: ptr::null(),
            description: ptr::null(),
            graphic_resource: ptr::null_mut(),
            state: Default::default(),
            embedded_playback_supported: false,
        });
        let (tx, rx) = channel();
        player.add(Box::new(move |e| {
            tx.send(e).unwrap();
        }));
        let player = Arc::new(Box::new(player) as Box<dyn Player>);
        let mut wrapper = PlayerWrapperC::from(Arc::downgrade(&player));

        invoke_player_event(&mut wrapper, PlayerEvent::DurationChanged(expected_result.clone()));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let PlayerEvent::DurationChanged(e) = result {
            assert_eq!(expected_result, e);
        } else {
            assert!(false, "expected PlayerEvent::DurationChanged, but got {} instead", result);
        }
    }
}