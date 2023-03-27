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
    let event_publisher = popcorn_fx.event_publisher().clone();
    popcorn_fx.runtime().spawn(async move {
        event_publisher.publish(event);
    });
}

#[cfg(test)]
mod test {
    use std::ptr;

    use tempfile::tempdir;

    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::PlayVideoEventC;
    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_handle_player_stopped_event() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let event = EventC::PlayVideo(PlayVideoEventC {
            url: into_c_string("http://localhost/video.mp4".to_string()),
            title: into_c_string("Lorem ipsum dolor".to_string()),
            show_name: ptr::null_mut(),
            thumb: into_c_string("http://localhost/thumb.jpg".to_string()),
        });

        publish_event(&mut instance, event);
    }
}