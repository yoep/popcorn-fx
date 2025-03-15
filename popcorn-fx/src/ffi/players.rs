use log::{debug, error, info, trace, warn};
use std::os::raw::c_char;
use std::ptr;

use crate::ffi::{
    PlayerC, PlayerEventC, PlayerManagerEventC, PlayerManagerEventCallback, PlayerRegistrationC,
    PlayerSet, PlayerWrapper, PlayerWrapperC,
};
use crate::PopcornFX;
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::players::{Player, PlayerEvent};
use popcorn_fx_core::{from_c_string, into_c_owned};

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
    trace!("Retrieving active player from C");
    let player_manager = popcorn_fx.player_manager();
    let runtime = popcorn_fx.runtime();
    block_in_place_runtime(
        async {
            match player_manager
                .active_player()
                .await
                .and_then(|e| e.upgrade())
            {
                None => ptr::null_mut(),
                Some(e) => {
                    let player = PlayerC::from(e).await;
                    into_c_owned(player)
                }
            }
        },
        runtime,
    )
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
pub extern "C" fn set_active_player(popcorn_fx: &mut PopcornFX, player_id: *mut c_char) {
    let player_id = from_c_string(player_id);
    trace!("Updating active player from C to {}", player_id);

    let player_manager = popcorn_fx.player_manager();
    let runtime = popcorn_fx.runtime();
    block_in_place_runtime(
        async { player_manager.set_active_player(player_id.as_str()).await },
        runtime,
    )
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
    let player_manager = popcorn_fx.player_manager();
    let runtime = popcorn_fx.runtime();
    let players = block_in_place_runtime(
        async {
            let mut players = vec![];
            for player in player_manager
                .players()
                .into_iter()
                .filter_map(|e| e.upgrade())
            {
                players.push(PlayerC::from(player).await);
            }
            players
        },
        runtime,
    );

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
pub extern "C" fn player_by_id(popcorn_fx: &mut PopcornFX, player_id: *mut c_char) -> *mut PlayerC {
    let player_id = from_c_string(player_id);
    trace!("Retrieving C player by id {}", player_id);

    let player_manager = popcorn_fx.player_manager();
    let runtime = popcorn_fx.runtime();
    block_in_place_runtime(
        async {
            match player_manager
                .by_id(player_id.as_str())
                .and_then(|e| e.upgrade())
            {
                None => ptr::null_mut(),
                Some(e) => {
                    let player = PlayerC::from(e).await;
                    into_c_owned(player)
                }
            }
        },
        runtime,
    )
}

/// Retrieves a pointer to a `PlayerWrapperC` instance by its unique identifier (ID) from the PopcornFX player manager.
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
/// Returns a pointer to a `PlayerWrapperC` instance representing the player if found, or a null pointer if no player with the given ID exists.
#[no_mangle]
pub extern "C" fn player_pointer_by_id(
    popcorn_fx: &mut PopcornFX,
    player_id: *mut c_char,
) -> *mut PlayerWrapperC {
    let player_id = from_c_string(player_id);
    trace!("Retrieving C player wrapper for {}", player_id);
    popcorn_fx
        .player_manager()
        .by_id(player_id.as_str())
        .map(|e| PlayerWrapperC::from(e))
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
pub extern "C" fn register_player_callback(
    popcorn_fx: &mut PopcornFX,
    callback: PlayerManagerEventCallback,
) {
    trace!("Registering new player manager callback");
    let mut receiver = popcorn_fx.player_manager().subscribe();
    popcorn_fx.runtime().spawn(async move {
        loop {
            if let Some(event) = receiver.recv().await {
                callback(PlayerManagerEventC::from((*event).clone()))
            } else {
                break;
            }
        }
    });
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
/// * `player` - A `PlayerRegistrationC` instance to be registered with the player manager.
///
/// # Notes
///
/// This function registers a player with the PopcornFX player manager using the provided `PlayerC` instance.
/// It logs an info message if the registration is successful and a warning message if registration fails.
#[no_mangle]
pub extern "C" fn register_player(popcorn_fx: &mut PopcornFX, player: PlayerRegistrationC) {
    trace!("Registering new C player {:?}", player);
    let player = PlayerWrapper::from(player);
    let id = player.id().to_string();

    if popcorn_fx.player_manager().add_player(Box::new(player)) {
        info!("Registered new C player {}", id);
    } else {
        warn!("Failed to register C player {}", id);
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
pub extern "C" fn remove_player(popcorn_fx: &mut PopcornFX, player_id: *mut c_char) {
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
pub extern "C" fn invoke_player_event(player: &mut PlayerWrapperC, event: PlayerEventC) {
    trace!(
        "Received player event from C {:?} for player {}",
        event,
        player.id()
    );
    match player.instance() {
        Some(player) => {
            player.downcast_ref::<PlayerWrapper>().map(|wrapper| {
                let event = PlayerEvent::from(event);
                trace!("Invoking player event from C {}", event);
                wrapper.invoke(event);
            }).unwrap_or_else(|| {
                error!("Unable to process C player event, player instance is not of type PlayerWrapper");
            });
        }
        None => {
            warn!(
                "Unable to process C player event, player {} has been disposed",
                player.id()
            );
        }
    }
}

/// Pauses the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
#[no_mangle]
pub extern "C" fn player_pause(player: &mut PlayerWrapperC) {
    trace!("Pausing player from C {:?}", player);
    if let Some(player) = player.instance() {
        trace!("Pausing player {}", player);
        player.pause();
    } else {
        warn!("Unable to pause player from C, player instance has been disposed");
    }
}

/// Resumes the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
#[no_mangle]
pub extern "C" fn player_resume(player: &mut PlayerWrapperC) {
    trace!("Resuming player from C {:?}", player);
    if let Some(player) = player.instance() {
        trace!("Resuming player {}", player);
        player.resume();
    } else {
        warn!("Unable to resume player from C, player instance has been disposed");
    }
}

/// Seeks the player associated with the given `PlayerWrapperC` instance to the specified time position.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
/// * `time` - The time position to seek to, in milliseconds.
#[no_mangle]
pub extern "C" fn player_seek(player: &mut PlayerWrapperC, time: u64) {
    trace!("Seeking player time from C {:?}", player);
    if let Some(player) = player.instance() {
        trace!("Seeking player time for {} with {}", player, time);
        player.seek(time);
    } else {
        warn!("Unable to seek player from C, player instance has been disposed");
    }
}

/// Stops the player associated with the given `PlayerWrapperC` instance.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A mutable reference to a `PlayerWrapperC` instance.
#[no_mangle]
pub extern "C" fn player_stop(player: &mut PlayerWrapperC) {
    trace!("Stopping player from C {:?}", player);
    if let Some(player) = player.instance() {
        trace!("Stopping player {}", player);
        player.stop();
    } else {
        warn!("Unable to stop player from C, player instance has been disposed");
    }
}

/// Dispose of a C-compatible player manager event.
///
/// This function is responsible for cleaning up resources associated with a C-compatible player manager event.
///
/// # Arguments
///
/// * `event` - A C-compatible player manager event to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_player_manager_event(event: PlayerManagerEventC) {
    trace!("Disposing C player manager event {:?}", event);
    drop(event);
}

/// Disposes of the `PlayerEventC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `event` pointer.
///
/// # Arguments
///
/// * `event` - A box containing the `PlayerEventC` instance to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_player_event_value(event: PlayerEventC) {
    trace!("Disposing player event value from C for {:?}", event);
    drop(event);
}

/// Disposes of the `PlayerWrapperC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `ptr` pointer.
///
/// # Arguments
///
/// * `ptr` - A box containing the `PlayerWrapperC` instance to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_player_pointer(ptr: Box<PlayerWrapperC>) {
    trace!("Disposing player pointer {:?}", ptr);
    drop(ptr);
}

/// Disposes of the `PlayerC` instance and deallocates its memory.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++),
/// and the caller is responsible for ensuring the safety of the provided `player` pointer.
///
/// # Arguments
///
/// * `player` - A box containing the `PlayerC` instance to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_player(player: Box<PlayerC>) {
    trace!("Disposing player info {:?}", player);
    drop(player);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use popcorn_fx_core::core::players::{PlayerManagerEvent, PlayerState};
    use popcorn_fx_core::testing::MockPlayer;
    use popcorn_fx_core::{
        from_c_owned, from_c_vec, init_logger, into_c_string, into_c_vec, recv_timeout,
    };
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::ffi::PlayRequestC;
    use crate::test::default_args;

    use super::*;

    #[no_mangle]
    extern "C" fn play_registration_callback(_: PlayRequestC) {
        // no-op
    }

    #[no_mangle]
    extern "C" fn pause_registration_callback() {
        // no-op
    }

    #[no_mangle]
    extern "C" fn resume_registration_callback() {
        // no-op
    }

    #[no_mangle]
    extern "C" fn seek_registration_callback(_: u64) {
        // no-op
    }

    #[no_mangle]
    extern "C" fn stop_registration_callback() {
        // no-op
    }

    #[test]
    fn test_active_player() {
        init_logger!();
        let player_id = "Lorem";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerWrapper::from(PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            graphic_resource_len: 0,
            state: PlayerState::Playing,
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
        });

        instance.player_manager().add_player(Box::new(player));
        set_active_player(&mut instance, into_c_string(player_id.to_string()));
        let result = from_c_owned(active_player(&mut instance));

        assert_eq!(player_id.to_string(), from_c_string(result.id));
    }

    #[test]
    fn test_players() {
        init_logger!();
        let player_id = "MyPlayerId999";
        let graphic_resource_vec = vec![80, 20];
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let (graphic_resource, graphic_resource_len) = into_c_vec(graphic_resource_vec.clone());
        let player = PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource,
            graphic_resource_len,
            state: PlayerState::Paused,
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
        };

        register_player(&mut instance, player);
        let set = from_c_owned(players(&mut instance));
        let players = from_c_vec(set.players, set.len);

        let result = players.get(0).unwrap();
        let bytes = from_c_vec(result.graphic_resource, result.graphic_resource_len);
        assert_eq!(1, players.len());
        assert_eq!(player_id.to_string(), from_c_string(result.id));
        assert_eq!(graphic_resource_vec, bytes);
    }

    #[test]
    fn test_player_by_id() {
        init_logger!();
        let player_id = "MyId666";
        let name = "VlcPlayer";
        let graphic_resource_vec = vec![155, 30, 16];
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let (graphic_resource, graphic_resource_len) = into_c_vec(graphic_resource_vec.clone());
        let player = PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string(name.to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource,
            graphic_resource_len,
            state: PlayerState::Paused,
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
        };

        register_player(&mut instance, player);
        let player = from_c_owned(player_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        let bytes = from_c_vec(player.graphic_resource, player.graphic_resource_len);
        assert_eq!(player_id.to_string(), from_c_string(player.id));
        assert_eq!(name.to_string(), from_c_string(player.name));
        assert_eq!(graphic_resource_vec, bytes);
    }

    #[test]
    fn test_register_player() {
        init_logger!();
        let player_id = "Id123";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            graphic_resource_len: 0,
            state: PlayerState::Error,
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
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
        init_logger!();
        let player_id = "Id123";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let player = PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("FooBar".to_string()),
            description: into_c_string("Lorem ipsum".to_string()),
            graphic_resource: ptr::null_mut(),
            graphic_resource_len: 0,
            state: PlayerState::Buffering,
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
        };

        register_player(&mut instance, player);
        assert_eq!(1, instance.player_manager().players().len());
        remove_player(&mut instance, into_c_string(player_id.to_string()));
        assert_eq!(
            0,
            instance.player_manager().players().len(),
            "expected the player to have been removed from the player manager"
        );
    }

    #[tokio::test]
    async fn test_invoke_player_event() {
        init_logger!();
        let expected_result = 240;
        let player = PlayerWrapper::from(PlayerRegistrationC {
            id: ptr::null_mut(),
            name: ptr::null_mut(),
            description: ptr::null_mut(),
            graphic_resource: ptr::null_mut(),
            graphic_resource_len: 0,
            state: Default::default(),
            embedded_playback_supported: false,
            play_callback: play_registration_callback,
            pause_callback: pause_registration_callback,
            resume_callback: resume_registration_callback,
            seek_callback: seek_registration_callback,
            stop_callback: stop_registration_callback,
        });
        let (tx, mut rx) = unbounded_channel();
        let event_c = PlayerEventC::DurationChanged(expected_result.clone());
        let player = Arc::new(Box::new(player) as Box<dyn Player>);
        let mut wrapper = PlayerWrapperC::from(Arc::downgrade(&player));

        let mut receiver = player.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap()
            }
        });

        invoke_player_event(&mut wrapper, event_c);
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        if let PlayerEvent::DurationChanged(e) = result {
            assert_eq!(expected_result, e);
        } else {
            assert!(
                false,
                "expected PlayerEvent::DurationChanged, but got {} instead",
                result
            );
        }
    }

    #[test]
    fn test_player_pause() {
        init_logger!();
        let player_id = "TestPlayer";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_pause().times(1).return_const(());
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.player_manager().add_player(Box::new(player));
        let mut ptr = from_c_owned(player_pointer_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        player_pause(&mut ptr);
    }

    #[test]
    fn test_player_resume() {
        init_logger!();
        let player_id = "TestPlayer";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_resume().times(1).return_const(());
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.player_manager().add_player(Box::new(player));
        let mut ptr = from_c_owned(player_pointer_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        player_resume(&mut ptr);
    }

    #[test]
    fn test_player_seek() {
        init_logger!();
        let player_id = "TestPlayer";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_seek().times(1).return_const(());
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.player_manager().add_player(Box::new(player));
        let mut ptr = from_c_owned(player_pointer_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        player_seek(&mut ptr, 28000);
    }

    #[test]
    fn test_player_stop() {
        init_logger!();
        let player_id = "TestPlayer";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_stop().times(1).return_const(());
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.player_manager().add_player(Box::new(player));
        let mut ptr = from_c_owned(player_pointer_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        player_stop(&mut ptr);
    }

    #[test]
    fn test_dispose_player_manager_event() {
        init_logger!();
        let event = PlayerManagerEventC::from(PlayerManagerEvent::PlayerTimeChanged(20000));

        dispose_player_manager_event(event);
    }

    #[test]
    fn test_dispose_player_event_value() {
        init_logger!();
        let event = PlayerEventC::DurationChanged(20000);

        dispose_player_event_value(event);
    }

    #[test]
    fn test_dispose_player_pointer() {
        init_logger!();
        let player_id = "TestPlayer";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut player = MockPlayer::new();
        player.expect_id().return_const(player_id.to_string());
        player.expect_resume().times(1).return_const(());
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.player_manager().add_player(Box::new(player));
        let ptr = from_c_owned(player_pointer_by_id(
            &mut instance,
            into_c_string(player_id.to_string()),
        ));

        dispose_player_pointer(Box::new(ptr));
    }

    #[tokio::test]
    async fn test_dispose_player() {
        init_logger!();
        let mut player = MockPlayer::new();
        player.expect_id().return_const("MyPlayerId".to_string());
        player.expect_name().return_const("MyPlayer".to_string());
        player
            .expect_description()
            .return_const("SomeRandomDescription".to_string());
        player.expect_graphic_resource().return_const(vec![]);
        player.expect_state().return_const(PlayerState::Playing);
        let player_c = PlayerC::from(Arc::new(Box::new(player) as Box<dyn Player>)).await;

        dispose_player(Box::new(player_c));
    }
}
