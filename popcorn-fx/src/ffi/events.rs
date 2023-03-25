use log::trace;

use popcorn_fx_core::core::events::PlayerStoppedEvent;

use crate::ffi::PlayerStoppedEventC;
use crate::PopcornFX;

/// Handle the player stopped event.
/// The event data will be cleaned by this fn, reuse of the data is thereby not possible.
///
/// * `event`   - The C event instance of the player stopped data.
#[no_mangle]
pub extern "C" fn handle_event(popcorn_fx: &mut PopcornFX, event: PlayerStoppedEventC) {
    trace!("Handling the player stopped event {:?}", event);
    let event = PlayerStoppedEvent::from(&event);
    popcorn_fx.auto_resume_service().player_stopped(&event);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handle_player_stopped_event() {
        
    }
}