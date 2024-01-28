use log::trace;

use popcorn_fx_core::core::playlists::{Playlist, PlaylistItem};
use popcorn_fx_core::from_c_vec;

use crate::ffi::{CSet, PlaylistItemC};
use crate::PopcornFX;

/// Play a playlist item using PopcornFX.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with external code (C/C++), and
/// the caller is responsible for ensuring the safety of the provided `popcorn_fx` and `item` pointers.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `item` - A reference to a `PlaylistItemC` representing the item to be played.
#[no_mangle]
pub extern "C" fn play_playlist_item(popcorn_fx: &mut PopcornFX, item: &PlaylistItemC) {
    trace!("Playing playlist item from C for {:?}", item);
    let item = PlaylistItem::from(item.clone());

    trace!("Playing playlist item {:?}", item);
    popcorn_fx.playlist_manager().play(Playlist::from(item))
}

#[no_mangle]
pub extern "C" fn play_playlist(popcorn_fx: &mut PopcornFX, playlist: CSet<PlaylistItemC>) {
    trace!("Playing playlist from C for {:?}", playlist);
    let playlist: Playlist = Vec::<PlaylistItemC>::from(playlist).into_iter()
        .map(|e| PlaylistItem::from(e))
        .collect();

    trace!("Starting playback of {:?}", playlist);
    popcorn_fx.playlist_manager().play(playlist);
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

#[no_mangle]
pub extern "C" fn dispose_playlist_set(set: Box<CSet<PlaylistItemC>>) {
    trace!("Disposing playlist set {:?}", set);
    drop(from_c_vec(set.items, set.len));
}

#[cfg(test)]
mod test {
    use std::ptr;

    use tempfile::tempdir;

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_play_playlist_item() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let item = PlaylistItemC {
            url: into_c_string("https://www.youtube.com/".to_string()),
            title: into_c_string("FooBar".to_string()),
            thumb: ptr::null(),
            quality: ptr::null(),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
        };

        play_playlist_item(&mut instance, &item);
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
        };
        let playlist = CSet::<PlaylistItemC>::from(vec![item]);

        dispose_playlist_set(Box::new(playlist));
    }
}