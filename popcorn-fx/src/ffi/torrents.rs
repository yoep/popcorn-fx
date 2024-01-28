use log::trace;

use popcorn_fx_core::{into_c_owned, into_c_string};
use popcorn_fx_core::core::torrents::{TorrentInfo, TorrentState, TorrentWrapper};
use popcorn_fx_torrent::torrent::DefaultTorrentManager;
use popcorn_fx_torrent_stream::{TorrentC, TorrentWrapperC};

use crate::ffi::{ResolveTorrentCallback, ResolveTorrentInfoCallback, TorrentFileInfoC};
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

/// Registers a new C-compatible resolve torrent callback function with PopcornFX.
///
/// This function allows registering a callback that will be invoked when torrent resolution is complete.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `callback` - The C-compatible resolve torrent callback function to be registered.
///
/// # Example
///
/// ```c
/// void resolve_callback(TorrentInfoC info) {
///     // Handle resolved torrent information
/// }
///
/// // Register the C-compatible callback with PopcornFX
/// torrent_resolve_callback(popcorn_fx, resolve_callback);
/// ```
///
/// This function registers a callback that receives resolved torrent information in the form of a `TorrentInfoC` struct.
/// You can then handle this information as needed within your callback function.
///
/// Note: This function is intended for C integration with PopcornFX.
///
/// # Safety
///
/// This function performs unsafe operations, as it deals with raw C-compatible function pointers.
#[no_mangle]
pub extern "C" fn torrent_resolve_info_callback(popcorn_fx: &mut PopcornFX, callback: ResolveTorrentInfoCallback) {
    trace!("Registering new C resolve torrent info callback");
    if let Some(manager) = popcorn_fx.torrent_manager().downcast_ref::<DefaultTorrentManager>() {
        manager.register_resolve_info_callback(Box::new(move |e| {
            trace!("Executing resolve magnet callback for {}", e);
            let info_c = callback(into_c_string(e));
            trace!("Received {:?} as resolve magnet callback result", info_c);
            TorrentInfo::from(info_c)
        }));
    }
}

/// A callback function for resolving torrents.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a `PopcornFX` instance and a `ResolveTorrentCallback` function as arguments.
///
/// The function registers the provided callback function with the `DefaultTorrentManager` from the `PopcornFX` instance.
/// When the callback function is invoked by the manager, it converts the arguments and the result between Rust and C types.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with C-compatible code and dereferences raw pointers.
/// Users of this function should ensure that they provide a valid `PopcornFX` instance and a valid `ResolveTorrentCallback`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The `ResolveTorrentCallback` function to be registered.
#[no_mangle]
pub extern "C" fn torrent_resolve_callback(popcorn_fx: &mut PopcornFX, callback: ResolveTorrentCallback) {
    trace!("Registering new C resolve torrent callback");
    if let Some(manager) = popcorn_fx.torrent_manager().downcast_ref::<DefaultTorrentManager>() {
        manager.register_resolve_callback(Box::new(move |file_info, torrent_directory, auto_start| {
            trace!("Executing resolve torrent callback for {:?}", file_info);
            let torrent_file_info = TorrentFileInfoC::from(file_info.clone());
            let torrent_directory = into_c_string(torrent_directory.to_string());
            let torrent = callback(torrent_file_info, torrent_directory, auto_start);
            trace!("Received {:?} as resolve torrent callback result", torrent);
            TorrentWrapper::from(torrent)
        }));
    }
}

/// Clean the torrents directory.
/// This will remove all existing torrents from the system.
#[no_mangle]
pub extern "C" fn cleanup_torrents_directory(popcorn_fx: &mut PopcornFX) {
    trace!("Cleaning torrents directory from C");
    popcorn_fx.torrent_manager().cleanup();
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, into_c_string};
    use popcorn_fx_core::core::torrents::{Torrent, TorrentEvent};
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use crate::test::{default_args, new_instance};

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
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
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

    #[test]
    fn test_cleanup_torrents_directory() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let filepath = copy_test_file(temp_path, "example.mp4", Some("torrents/subdir/example.mp4"));

        cleanup_torrents_directory(&mut instance);

        assert_eq!(false, PathBuf::from(filepath).exists(), "expected the torrent file to have been cleaned");
        assert_eq!(true, instance.settings().settings.torrent_settings.directory.exists(), "expected the torrent directory to still exist");
    }
}