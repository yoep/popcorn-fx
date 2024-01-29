use std::os::raw::c_char;

use log::{trace, warn};

use popcorn_fx_core::{from_c_string, into_c_string};
use popcorn_fx_core::core::torrents::{TorrentInfo, TorrentState, TorrentWrapper};
use popcorn_fx_torrent::torrent::DefaultTorrentManager;

use crate::ffi::{ResolveTorrentCallback, ResolveTorrentInfoCallback, TorrentFileInfoC};
use crate::PopcornFX;

#[no_mangle]
pub extern "C" fn torrent_state_changed(popcorn_fx: &mut PopcornFX, handle: *const c_char, state: TorrentState) {
    let handle = from_c_string(handle);
    if let Some(torrent) = popcorn_fx.torrent_manager().by_handle(handle.as_str())
        .and_then(|e| e.upgrade()) {
        if let Some(wrapper) = torrent.downcast_ref::<TorrentWrapper>() {
            trace!("Processing C torrent state changed");
            wrapper.state_changed(state);
        }
    } else {
        warn!("Unable to process torrent state changed, handle {} not found", handle);
    }
}

#[no_mangle]
pub extern "C" fn torrent_piece_finished(popcorn_fx: &mut PopcornFX, handle: *const c_char, piece: u32) {
    let handle = from_c_string(handle);
    if let Some(torrent) = popcorn_fx.torrent_manager().by_handle(handle.as_str())
        .and_then(|e| e.upgrade()) {
        if let Some(wrapper) = torrent.downcast_ref::<TorrentWrapper>() {
            trace!("Processing C torrent piece finished");
            wrapper.piece_finished(piece);
        }
    } else {
        warn!("Unable to process torrent piece finished, handle {} not found", handle);
    }
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
    use tokio::sync::Mutex;

    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::core::torrents::{Torrent, TorrentEvent, TorrentFileInfo, TorrentManager};
    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use crate::ffi::TorrentC;
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
        let handle = "MyHandle";
        let torrent = TorrentC {
            handle: into_c_string(handle.to_string()),
            filepath: into_c_string("lorem.txt".to_string()),
            has_byte_callback: has_bytes_callback,
            has_piece_callback: has_piece_callback,
            total_pieces: total_pieces_callback,
            prioritize_bytes: prioritize_bytes_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
            torrent_state: torrent_state_callback,
        };
        let torrent_file_info = TorrentFileInfo {
            filename: "".to_string(),
            file_path: temp_path.to_string(),
            file_size: 18000,
            file_index: 0,
        };

        let (tx, rx) = channel();
        let manager = instance.torrent_manager().clone();
        let torrent_manager = manager.downcast_ref::<DefaultTorrentManager>().unwrap();

        torrent_manager.register_resolve_callback(Box::new(move |_, _, _| {
            let wrapper = TorrentWrapper {
                handle: handle.to_string(),
                filepath: Default::default(),
                has_bytes: Mutex::new(Box::new(|_| true)),
                has_piece: Mutex::new(Box::new(|_| true)),
                total_pieces: Mutex::new(Box::new(|| 10)),
                prioritize_bytes: Mutex::new(Box::new(|_| {})),
                prioritize_pieces: Mutex::new(Box::new(|_| {})),
                sequential_mode: Mutex::new(Box::new(|| {})),
                torrent_state: Mutex::new(Box::new(|| TorrentState::Downloading)),
                callbacks: Default::default(),
            };
            let tx_wrapper = tx.clone();
            wrapper.subscribe(Box::new(move |event| {
                tx_wrapper.send(event).unwrap();
            }));
            wrapper
        }));
        match block_in_place(torrent_manager.create(&torrent_file_info, temp_path, true)) {
            Ok(result) => assert_eq!(handle, result.upgrade().unwrap().handle()),
            Err(e) => assert!(false, "expected torrent to have been created, got {}", e),
        }

        torrent_manager.by_handle(handle).expect("expected the torrent handle to have been found");
        torrent_state_changed(&mut instance, into_c_string(handle.to_string()), TorrentState::Starting);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        match result {
            TorrentEvent::StateChanged(state) => assert_eq!(TorrentState::Starting, state),
            _ => assert!(false, "expected TorrentEvent::StateChanged, but got {} instead", result),
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