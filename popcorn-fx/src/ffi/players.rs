use log::{info, warn};

use popcorn_fx_core::core::players::Player;

use crate::ffi::PlayerC;
use crate::PopcornFX;

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
pub extern "C" fn register_player(popcorn_fx: &mut PopcornFX, player: PlayerC) {
    let id = player.id().to_string();

    if popcorn_fx.player_manager().register(Box::new(player)) {
        info!("Register C player {}", id)
    } else {
        warn!("Failed to register C player {}", id)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

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
        };

        register_player(&mut instance, player);

        let players = instance.player_manager().players();
        if let Some(player) = players.get(0) {
            assert_eq!(player_id, player.upgrade().unwrap().id());
        } else {
            assert!(false, "expected at least one player to be registered")
        }
    }
}