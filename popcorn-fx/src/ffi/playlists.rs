use log::trace;

use popcorn_fx_core::core::playlists::Playlist;

use crate::ffi::PlaylistItemC;
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
    let item = item.to_struct();

    popcorn_fx.playlist_manager().play(Playlist::from(item))
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

#[cfg(test)]
mod test {
    use std::ptr;

    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_dispose_playlist_item() {
        init_logger();
        let item = Box::new(PlaylistItemC {
            url: into_c_string("http://my_url".to_string()),
            title: into_c_string("Foo Bar".to_string()),
            thumb: into_c_string("MyThumb".to_string()),
            media: ptr::null_mut(),
        });

        dispose_playlist_item(item);
    }
}