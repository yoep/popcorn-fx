use log::trace;

use crate::ffi::{LoaderEventC, LoaderEventCallback};
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
pub extern "C" fn register_loader_callback(instance: &mut PopcornFX, callback: LoaderEventCallback) {
    trace!("Registering new loader callback");
    instance.media_loader().subscribe(Box::new(move |e| {
        trace!("Invoking loader C callback for {}", e);
        callback(LoaderEventC::from(e));
    }));
}

#[cfg(test)]
mod tests {
    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::core::media::MovieDetails;
    use popcorn_fx_core::core::playlists::PlaylistItem;
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    extern "C" fn loader_callback(event: LoaderEventC) {
        info!("Received loader event {:?}", event);
    }

    #[test]
    fn test_register_loader_callback() {
        init_logger();
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
            thumb: None,
            parent_media: None,
            media: Some(Box::new(movie)),
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_loader_callback(&mut instance, loader_callback);
        let result = block_in_place(instance.media_loader().load_playlist_item(item));

        assert_eq!(Ok(()), result);
    }
}