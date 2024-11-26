use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::core::torrents::{TorrentHealth, TorrentState};
use popcorn_fx_core::core::{block_in_place, Handle};
use popcorn_fx_core::{from_c_string, into_c_owned};

use crate::ffi::{
    DownloadStatusC, ResolveTorrentCallback, ResultC, TorrentErrorC, TorrentFileInfoC,
    TorrentStreamEventC, TorrentStreamEventCallback,
};
use crate::PopcornFX;

/// Registers a new torrent stream event callback.
///
/// This function registers a callback function to receive torrent stream events.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `stream_handle` - The handle of the torrent stream.
/// * `callback` - The callback function to be invoked when torrent stream events occur.
///
/// # Returns
///
/// A pointer to an integer value representing the handle of the registered callback, or a null pointer if registration fails.
#[no_mangle]
pub extern "C" fn register_torrent_stream_event_callback(
    popcorn_fx: &mut PopcornFX,
    stream_handle: i64,
    callback: TorrentStreamEventCallback,
) -> *const i64 {
    trace!(
        "Registering a new torrent stream event callback for handle {}",
        stream_handle
    );
    let handle = Handle::from(stream_handle);
    popcorn_fx
        .torrent_stream_server()
        .subscribe(
            handle,
            Box::new(move |event| {
                trace!("Invoking torrent stream event C callback for {:?}", event);
                callback(TorrentStreamEventC::from(event))
            }),
        )
        .map(|handle| handle.value() as *const i64)
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn remove_torrent_stream_event_callback(
    popcorn_fx: &mut PopcornFX,
    stream_handle: *const i64,
    callback_handle: *const i64,
) {
    trace!(
        "Removing torrent event stream callback handle {:?} of {:?}",
        callback_handle,
        stream_handle
    );
    let callback_handle = Handle::from(callback_handle as i64);
    let handle = Handle::from(stream_handle as i64);
    popcorn_fx
        .torrent_stream_server()
        .unsubscribe(handle, callback_handle);
}

/// Calculates the health of a torrent based on its magnet link.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `uri` - The magnet link of the torrent.
///
/// # Returns
///
/// Returns the health of the torrent.
#[no_mangle]
pub extern "C" fn torrent_health_from_uri(
    popcorn_fx: &mut PopcornFX,
    uri: *const c_char,
) -> ResultC<TorrentHealth, TorrentErrorC> {
    trace!("Retrieving torrent health of uri from C");
    let uri = from_c_string(uri);

    ResultC::from(
        block_in_place(popcorn_fx.torrent_manager().health_from_uri(&uri))
            .map_err(|e| TorrentErrorC::from(e)),
    )
}

/// Calculates the health of a torrent based on the number of seeds and leechers.
///
/// # Returns
///
/// Returns the health of the torrent.
#[no_mangle]
pub extern "C" fn calculate_torrent_health(
    popcorn_fx: &mut PopcornFX,
    seeds: u32,
    leechers: u32,
) -> *mut TorrentHealth {
    trace!(
        "Calculating torrent health from C with seeds {} and leechers {}",
        seeds,
        leechers
    );
    into_c_owned(
        popcorn_fx
            .torrent_manager()
            .calculate_health(seeds, leechers),
    )
}

/// Clean the torrents directory.
/// This will remove all existing torrents from the system.
#[no_mangle]
pub extern "C" fn cleanup_torrents_directory(popcorn_fx: &mut PopcornFX) {
    trace!("Cleaning torrents directory from C");
    popcorn_fx.torrent_manager().cleanup();
}

#[no_mangle]
pub extern "C" fn dispose_torrent_stream_event_value(event: TorrentStreamEventC) {
    trace!("Disposing torrent stream event from C {:?}", event);
    drop(event);
}

#[no_mangle]
pub extern "C" fn dispose_torrent_health(health: Box<TorrentHealth>) {
    trace!("Disposing torrent health from C {:?}", health);
    drop(health);
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;

    use log::info;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::core::torrents::{
        MockTorrent, Torrent, TorrentEvent, TorrentFileInfo, TorrentManager,
    };
    use popcorn_fx_core::testing::{copy_test_file, init_logger};
    use popcorn_fx_core::{assert_timeout_eq, into_c_string};

    use crate::ffi::TorrentC;
    use crate::test::{default_args, new_instance};

    use super::*;

    #[no_mangle]
    extern "C" fn has_bytes_callback(_: i32, _: *mut u64) -> bool {
        true
    }

    #[no_mangle]
    extern "C" fn has_piece_callback(_: u32) -> bool {
        true
    }

    #[no_mangle]
    extern "C" fn total_pieces_callback() -> i32 {
        10
    }

    #[no_mangle]
    extern "C" fn prioritize_bytes_callback(_: i32, _: *mut u64) {}

    #[no_mangle]
    extern "C" fn prioritize_pieces_callback(_: i32, _: *mut u32) {}

    #[no_mangle]
    extern "C" fn sequential_mode_callback() {}

    #[no_mangle]
    extern "C" fn torrent_state_callback() -> TorrentState {
        TorrentState::Downloading
    }

    #[no_mangle]
    extern "C" fn torrent_stream_event_callback(event: TorrentStreamEventC) {
        info!("Received torrent stream event {:?}", event);
    }

    #[no_mangle]
    extern "C" fn torrent_resolve_callback(
        file_info: TorrentFileInfoC,
        _: *mut c_char,
        _: bool,
    ) -> TorrentC {
        info!("Received torrent resolve callback for {:?}", file_info);
        TorrentC {
            handle: into_c_string("MyHandle"),
            filepath: into_c_string("/tmp/pmy-path"),
            has_byte_callback: has_bytes_callback,
            has_piece_callback,
            total_pieces: total_pieces_callback,
            prioritize_bytes: prioritize_bytes_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
            torrent_state: torrent_state_callback,
        }
    }

    #[test]
    fn test_cleanup_torrents_directory() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let mut torrent_settings = instance.settings().user_settings().torrent_settings;
        torrent_settings.directory = PathBuf::from(temp_path);
        instance.settings().update_torrent(torrent_settings);
        let filepath = copy_test_file(
            temp_path,
            "example.mp4",
            Some("torrents/subdir/example.mp4"),
        );

        cleanup_torrents_directory(&mut instance);

        assert_timeout_eq!(
            Duration::from_millis(200),
            false,
            PathBuf::from(filepath.clone()).exists()
        );
        assert_eq!(
            true,
            instance
                .settings()
                .user_settings()
                .torrent_settings
                .directory
                .exists(),
            "expected the torrent directory to still exist"
        );
    }

    #[test]
    fn test_remove_torrent_stream_event_callback() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut torrent = MockTorrent::new();
        torrent.expect_file().return_const(PathBuf::from(temp_path));
        torrent.expect_total_pieces().return_const(Some(10));
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent.expect_prioritize_pieces().return_const(());
        let torrent = Box::new(torrent) as Box<dyn Torrent>;
        let mut instance = new_instance(temp_path);

        let stream = instance
            .torrent_stream_server()
            .start_stream(torrent)
            .expect("expected a stream to have been returned")
            .upgrade()
            .expect("expected the stream instance to still be valid");

        let stream_handle_value = stream.stream_handle().value();
        let callback = register_torrent_stream_event_callback(
            &mut instance,
            stream_handle_value,
            torrent_stream_event_callback,
        ) as i64;
        remove_torrent_stream_event_callback(
            &mut instance,
            stream_handle_value as *const i64,
            callback as *const i64,
        );
    }
}
