use log::trace;

use popcorn_fx_core::core::torrent::TorrentState;
use popcorn_fx_core::into_c_owned;
use popcorn_fx_torrent_stream::{TorrentC, TorrentWrapperC};

use crate::PopcornFX;

/// The torrent wrapper for moving data between Rust and FrostWire.
///
/// This is a temporary wrapper until the torrent component is replaced.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
/// The `torrent` pointer must be a valid `TorrentC` instance.
/// The returned pointer to `TorrentWrapperC` should be used carefully to avoid memory leaks.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `torrent` - A pointer to a `TorrentC` instance.
///
/// # Returns
///
/// A pointer to a `TorrentWrapperC` instance.
#[no_mangle]
pub extern "C" fn torrent_wrapper(popcorn_fx: &mut PopcornFX, torrent: TorrentC) -> *mut TorrentWrapperC {
    trace!("Wrapping TorrentC into TorrentWrapperC for {:?}", torrent);
    let wrapper_c = TorrentWrapperC::from(torrent);
    trace!("Registering wrapper in torrent manager");
    popcorn_fx.torrent_manager().add(wrapper_c.wrapper().clone());
    into_c_owned(wrapper_c)
}

/// Inform that the state of the torrent has changed.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `torrent` pointer must be a valid `TorrentWrapperC` instance.
///
/// # Arguments
///
/// * `torrent` - A pointer to a `TorrentWrapperC` instance.
/// * `state` - The new state of the torrent.
#[no_mangle]
pub extern "C" fn torrent_state_changed(torrent: &TorrentWrapperC, state: TorrentState) {
    torrent.state_changed(state)
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, into_c_string};
    use popcorn_fx_core::core::torrent::{Torrent, TorrentEvent};

    use crate::test::default_args;

    use super::*;

    #[no_mangle]
    pub extern "C" fn has_bytes_callback(_: i32, _: *mut u64) -> bool {
        true
    }

    #[no_mangle]
    pub extern "C" fn has_piece_callback(_: u32) -> bool {
        true
    }

    #[no_mangle]
    pub extern "C" fn total_pieces_callback() -> i32 {
        10
    }

    #[no_mangle]
    pub extern "C" fn prioritize_bytes_callback(_: i32, _: *mut u64) {}

    #[no_mangle]
    pub extern "C" fn prioritize_pieces_callback(_: i32, _: *mut u32) {}

    #[no_mangle]
    pub extern "C" fn sequential_mode_callback() {}

    #[no_mangle]
    pub extern "C" fn torrent_state_callback() -> TorrentState {
        TorrentState::Downloading
    }

    #[test]
    fn test_torrent_state_changed() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let torrent = TorrentC {
            filepath: into_c_string("lorem.txt".to_string()),
            has_byte_callback: has_bytes_callback,
            has_piece_callback: has_piece_callback,
            total_pieces: total_pieces_callback,
            prioritize_bytes: prioritize_bytes_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
            torrent_state: torrent_state_callback,
        };
        let (tx, rx) = channel();

        let wrapper = from_c_owned(torrent_wrapper(&mut instance, torrent));
        wrapper.wrapper().register(Box::new(move |e| {
            tx.send(e).unwrap()
        }));
        torrent_state_changed(&wrapper, TorrentState::Starting);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        match result {
            TorrentEvent::StateChanged(state) => assert_eq!(TorrentState::Starting, state),
            _ => {}
        }
    }
}