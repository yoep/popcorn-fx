use log::trace;

use popcorn_fx_core::core::playlists::{Playlist, PlaylistItem};
use popcorn_fx_core::from_c_vec;

use crate::ffi::{CArray, PlaylistItemC};
use crate::PopcornFX;

/// Play a playlist from C by converting it to the Rust data structure and starting playback asynchronously.
///
/// This function takes a mutable reference to a `PopcornFX` instance and a C-compatible array of `PlaylistItemC` items.
/// It converts the C array into a Rust `Playlist` and starts playback asynchronously using the playlist manager.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `playlist` - A C-compatible array of `PlaylistItemC` items representing the playlist to play.
#[no_mangle]
pub extern "C" fn play_playlist(popcorn_fx: &mut PopcornFX, playlist: CArray<PlaylistItemC>) {
    trace!("Converting playlist from C for {:?}", playlist);
    let playlist: Playlist = Vec::<PlaylistItemC>::from(playlist).into_iter()
        .map(|e| PlaylistItem::from(e))
        .collect();

    let playlist_manager = popcorn_fx.playlist_manager().clone();
    popcorn_fx.runtime().spawn(async move {
        trace!("Starting playlist from C for {:?}", playlist);
        playlist_manager.play(playlist);
    });
}

/// Dispose of a playlist item.
///
/// # Arguments
///
/// * `item` - A boxed `PlaylistItemC` representing the item to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_playlist_item(item: Box<PlaylistItemC>) {
    trace!("Disposing playlist item {:?}", item)
}

/// Dispose of a C-style array of playlist items.
///
/// This function takes ownership of a C-style array of `PlaylistItemC` and drops it to free the associated memory.
///
/// # Arguments
///
/// * `set` - A boxed C-style array of `PlaylistItemC` to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_playlist_set(set: Box<CArray<PlaylistItemC>>) {
    trace!("Disposing playlist set {:?}", set);
    drop(from_c_vec(set.items, set.len));
}

#[cfg(test)]
mod test {
    use std::ptr;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::core::playlists::{PlaylistManagerEvent, PlaylistState};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_play_playlist() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let item = PlaylistItemC::from(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        let playlist = CArray::from(vec![item]);
        let (tx, rx) = channel();
        let (tx_state, rx_state) = channel();
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance.playlist_manager().subscribe(Box::new(move |e| {
            match e {
                PlaylistManagerEvent::PlaylistChanged => tx.send(e).unwrap(),
                PlaylistManagerEvent::StateChanged(state) => tx_state.send(state).unwrap(),
                _ => {}
            }
        }));
        play_playlist(&mut instance, playlist);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistManagerEvent::PlaylistChanged, result, "expected the PlaylistChanged event to have been published");

        let result = rx_state.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlaylistState::Playing, result, "expected the state to have changed");
    }

    #[test]
    fn test_dispose_playlist_item() {
        init_logger();
        let item = Box::new(PlaylistItemC {
            url: into_c_string("http://my_url".to_string()),
            title: into_c_string("Foo Bar".to_string()),
            thumb: into_c_string("MyThumb".to_string()),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            quality: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
            subtitles_enabled: false,
        });

        dispose_playlist_item(item);
    }

    #[test]
    fn test_dispose_playlist_set() {
        init_logger();
        let item = PlaylistItemC {
            url: into_c_string("http://my_url".to_string()),
            title: into_c_string("Foo Bar".to_string()),
            thumb: into_c_string("MyThumb".to_string()),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            quality: ptr::null_mut(),
            auto_resume_timestamp: into_c_owned(500u64),
            subtitles_enabled: false,
        };
        let playlist = CArray::<PlaylistItemC>::from(vec![item]);

        dispose_playlist_set(Box::new(playlist));
    }
}