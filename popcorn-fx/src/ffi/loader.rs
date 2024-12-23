use std::os::raw::c_char;

use log::{trace, warn};

use popcorn_fx_core::core::{block_in_place_runtime, Handle};
use popcorn_fx_core::from_c_string;

use crate::ffi::{LoaderEventC, LoaderEventCallback, LoadingHandleC};
use crate::PopcornFX;

/// Register a loader event callback to receive loader state change events.
///
/// This function registers a callback function to receive loader state change events from the
/// PopcornFX instance. When a loader state change event occurs, the provided callback will be invoked.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the PopcornFX instance to register the callback with.
/// * `callback` - A C-compatible callback function that will be invoked when loader state change events occur.
#[no_mangle]
pub extern "C" fn register_loader_callback(
    instance: &mut PopcornFX,
    callback: LoaderEventCallback,
) {
    trace!("Registering new loader callback");

    let mut receiver = instance.media_loader().subscribe();
    instance.runtime().spawn(async move {
        loop {
            if let Some(event) = receiver.recv().await {
                trace!("Invoking loader C callback for {}", event);
                callback(LoaderEventC::from((*event).clone()));
            } else {
                break;
            }
        }
    });
}

/// Load a media item using the media loader from a C-compatible URL.
///
/// This function takes a mutable reference to a `PopcornFX` instance and a C-compatible string (`*mut c_char`) representing the URL of the media item to load.
/// It uses the media loader to load the media item asynchronously and returns a handle (represented as a `LoadingHandleC`) for the loading process.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `url` - A C-compatible string representing the URL of the media item to load.
///
/// # Returns
///
/// A `LoadingHandleC` representing the loading process associated with the loaded item.
#[no_mangle]
pub extern "C" fn loader_load(instance: &mut PopcornFX, url: *mut c_char) -> LoadingHandleC {
    let url = from_c_string(url);
    trace!("Loading new loader url {} from C", url);
    let handle = block_in_place_runtime(
        instance.media_loader().load_url(url.as_str()),
        instance.runtime(),
    );

    trace!("Loader load returned handle {}", handle);
    handle.value() as *const i64
}

/// Cancels the current media loading process initiated by the `MediaLoader`.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn loader_cancel(instance: &mut PopcornFX, handle: LoadingHandleC) {
    if !handle.is_null() {
        trace!("Cancelling the loader");
        let handle = Handle::from(handle as i64);
        instance.media_loader().cancel(handle);
    } else {
        warn!("Unable to cancel the loader, no handle specified");
    }
}

/// Dispose of a C-compatible LoaderEventC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible LoaderEventC value.
///
/// # Arguments
///
/// * `event` - A C-compatible LoaderEventC value to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_loader_event_value(event: LoaderEventC) {
    trace!("Disposing LoaderEventC {:?}", event);
    drop(event);
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::core::loader::{
        LoadingResult, LoadingState, MockLoadingStrategy, HIGHEST_ORDER,
    };
    use popcorn_fx_core::core::media::MovieDetails;
    use popcorn_fx_core::core::playlist::{PlaylistItem, PlaylistMedia};
    use popcorn_fx_core::testing::init_logger;
    use popcorn_fx_core::{init_logger, into_c_string};

    use crate::ffi::CArray;
    use crate::test::default_args;

    use super::*;

    extern "C" fn loader_callback(event: LoaderEventC) {
        info!("Received loader event {:?}", event);
    }

    #[test]
    fn test_register_loader_callback() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let movie = MovieDetails {
            title: "MyMovieTitle".to_string(),
            imdb_id: "t000123".to_string(),
            year: "2014".to_string(),
            runtime: "".to_string(),
            genres: vec![],
            synopsis: "".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "".to_string(),
            torrents: Default::default(),
        };
        let item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(movie)),
            },
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_loader_callback(&mut instance, loader_callback);
        let result = instance
            .runtime()
            .block_on(instance.media_loader().load_playlist_item(item));

        assert_ne!(result.value(), 0);
    }

    #[test]
    fn test_loader_load() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let url = "magnet:?xt=urn:btih:9a5c24e8164dfe5a98d2437b7f4d6ec9a7e2e045&dn=Another%20Example%20File&tr=http%3A%2F%2Ftracker.anotherexample.com%3A56789%2Fannounce&xl=987654321&sf=Another%20Folder";
        let mut instance = PopcornFX::new(default_args(temp_path));

        let result = loader_load(&mut instance, into_c_string(url.to_string()));

        assert_ne!(0i64, result as i64);
    }

    #[test]
    fn test_loader_cancel() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        loader_cancel(&mut instance, 874458i64 as *const i64);
    }

    #[test]
    fn test_dispose_loader_event_value() {
        init_logger!();
        let event = LoaderEventC::StateChanged(84555i64, LoadingState::Downloading);

        dispose_loader_event_value(event);
    }
}
