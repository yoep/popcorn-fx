use log::trace;

use popcorn_fx_core::core::events::{Event, LOWEST_ORDER};

use crate::ffi::{EventC, EventCCallback};
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

    use tempfile::tempdir;

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::core::events::HIGHEST_ORDER;
    use popcorn_fx_core::core::media::{Images, MovieOverview};
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::{CArray, MediaItemC, PlayerStoppedEventC, TorrentInfoC};
    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_handle_player_stopped_event() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let url = "https://localhost:8090/dummy.mp4";
        let movie = MovieOverview {
            title: "MyMovie".to_string(),
            imdb_id: "tt00011123".to_string(),
            year: "2015".to_string(),
            rating: None,
            images: Images {
                poster: "https://image".to_string(),
                fanart: "https://image".to_string(),
                banner: "https://image".to_string(),
            },
        };
        let mut instance = PopcornFX::new(default_args(temp_path));
        let event = EventC::PlayerStopped(PlayerStoppedEventC {
            url: into_c_string(url.to_string()),
            time: 20000 as *const i64,
            duration: 25000 as *const i64,
            media: into_c_owned(MediaItemC::from(movie)),
        });

        instance.event_publisher().register(Box::new(move |e| {
            tx.send(e).unwrap();
            None
        }), HIGHEST_ORDER);
        publish_event(&mut instance, event);
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let Event::PlayerStopped(result) = result {
            assert_eq!(url, result.url());
            assert_eq!(Some(&20000u64), result.time());
            assert_eq!(Some(&25000u64), result.duration());
        } else {
            assert!(false, "expected Event::PlayerStopped, but got {} instead", result);
        }
    }

    #[test]
    fn test_dispose_event_value() {
        dispose_event_value(EventC::TorrentDetailsLoaded(TorrentInfoC {
            name: into_c_string("Foo".to_string()),
            directory_name: into_c_string("Bar".to_string()),
            total_files: 20,
            files: CArray::from(vec![]),
        }))
    }
}