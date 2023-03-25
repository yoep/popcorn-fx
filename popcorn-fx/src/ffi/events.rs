use log::trace;

use popcorn_fx_core::core::events::Event;

use crate::ffi::EventC;
use crate::PopcornFX;

/// Publish a new application event over the FFI layer.
/// This will invoke the [popcorn_fx_core::core::events::EventPublisher] publisher on the backend.
///
/// _Please keep in mind that the consumption of the event chain is not communicated over the FFI layer_
#[no_mangle]
pub extern "C" fn publish_event(popcorn_fx: &mut PopcornFX, event: EventC) {
    trace!("Handling EventPublisher bridge event of C for {:?}", event);
    let event = Event::from(event);
    popcorn_fx.event_publisher().publish(event);
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_handle_player_stopped_event() {
        init_logger();
    }
}