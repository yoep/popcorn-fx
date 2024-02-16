use log::trace;

use popcorn_fx_core::core::events::LOWEST_ORDER;

use crate::ffi::{EventC, EventCCallback};
use crate::PopcornFX;

/// Publish a new application event over the FFI layer.
/// This will invoke the [popcorn_fx_core::core::events::EventPublisher] publisher on the backend.
///
/// _Please keep in mind that the consumption of the event chain is not communicated over the FFI layer_
#[no_mangle]
pub extern "C" fn publish_event(popcorn_fx: &mut PopcornFX, event: EventC) {
    trace!("Handling EventPublisher bridge event of C for {:?}", event);
    if let Some(event) = event.into_event() {
        let event_publisher = popcorn_fx.event_publisher().clone();
        popcorn_fx.runtime().spawn(async move {
            event_publisher.publish(event);
        });
    }
}

/// Register an event callback with the PopcornFX event publisher.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `callback` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A C-compatible function pointer representing the callback to be registered.
#[no_mangle]
pub extern "C" fn register_event_callback(popcorn_fx: &mut PopcornFX, callback: EventCCallback) {
    popcorn_fx.event_publisher().register(Box::new(move |e| {
        trace!("Executing EventPublisher bridge event callback for {}", e);
        callback(EventC::from(e));
        None // consume the event
    }), LOWEST_ORDER);
}

/// Dispose of the given event from the event bridge.
///
/// This function takes ownership of a boxed `EventC` object, releasing its resources.
///
/// # Arguments
///
/// * `event` - A boxed `EventC` object to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_event_value(event: EventC) {
    trace!("Disposing EventC {:?}", event);
    drop(event)
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::core::events::{DEFAULT_ORDER, Event};
    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::{CArray, TorrentInfoC};
    use crate::test::default_args;

    use super::*;

    extern "C" fn event_callback(event: EventC) {
        info!("Event callback received {:?}", event);
    }

    #[test]
    fn test_publish() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.event_publisher().register(Box::new(move |e| {
            tx.send(e).unwrap();
            None
        }), DEFAULT_ORDER);
        publish_event(&mut instance, EventC::ClosePlayer);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(Event::ClosePlayer, result);
    }

    #[test]
    fn test_register_event_callback() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_event_callback(&mut instance, event_callback);
        instance.event_publisher().publish(Event::ClosePlayer);
    }

    #[test]
    fn test_dispose_event_value() {
        dispose_event_value(EventC::TorrentDetailsLoaded(TorrentInfoC {
            uri: into_c_string("magnet:?Lorem".to_string()),
            name: into_c_string("Foo".to_string()),
            directory_name: into_c_string("Bar".to_string()),
            total_files: 20,
            files: CArray::from(vec![]),
        }))
    }
}