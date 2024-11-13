use std::os::raw::c_char;
use std::ptr;

use log::{trace, warn};

use popcorn_fx_core::core::torrents::{
    DownloadStatus, TorrentHealth, TorrentState, TorrentWrapper,
};
use popcorn_fx_core::core::{block_in_place, Handle};
use popcorn_fx_core::{from_c_string, into_c_owned, into_c_string};
use popcorn_fx_torrent::torrents::DefaultTorrentManager;

use crate::ffi::{
    CancelTorrentCallback, DownloadStatusC, ResolveTorrentCallback, ResultC, TorrentErrorC,
    TorrentFileInfoC, TorrentStreamEventC, TorrentStreamEventCallback,
};
use crate::PopcornFX;

/// Callback function for handling changes in the state of a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `state` - The new state of the torrent.
#[no_mangle]
pub extern "C" fn torrent_state_changed(
    popcorn_fx: &mut PopcornFX,
    handle: *mut c_char,
    state: TorrentState,
) {
    let handle = from_c_string(handle);
    if let Some(torrent) = popcorn_fx
        .torrent_manager()
        .by_handle(handle.as_str())
        .and_then(|e| e.upgrade())
    {
        if let Some(wrapper) = torrent.downcast_ref::<TorrentWrapper>() {
            trace!("Processing C torrent state changed");
            wrapper.state_changed(state);
        }
    } else {
        warn!(
            "Unable to process torrent state changed, handle {} not found",
            handle
        );
    }
}

/// Callback function for handling the completion of downloading a piece in a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `piece` - The index of the finished piece.
#[no_mangle]
pub extern "C" fn torrent_piece_finished(
    popcorn_fx: &mut PopcornFX,
    handle: *mut c_char,
    piece: u32,
) {
    let handle = from_c_string(handle);
    if let Some(torrent) = popcorn_fx
        .torrent_manager()
        .by_handle(handle.as_str())
        .and_then(|e| e.upgrade())
    {
        if let Some(wrapper) = torrent.downcast_ref::<TorrentWrapper>() {
            wrapper.piece_finished(piece);
        }
    } else {
        warn!(
            "Unable to process torrent piece finished, handle {} not found",
            handle
        );
    }
}

/// Callback function for handling changes in the download status of a torrent.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `handle` - The handle to the torrent.
/// * `download_status` - The new download status of the torrent.
#[no_mangle]
pub extern "C" fn torrent_download_status(
    popcorn_fx: &mut PopcornFX,
    handle: *mut c_char,
    download_status: DownloadStatusC,
) {
    let handle = from_c_string(handle);
    if let Some(torrent) = popcorn_fx
        .torrent_manager()
        .by_handle(handle.as_str())
        .and_then(|e| e.upgrade())
    {
        if let Some(wrapper) = torrent.downcast_ref::<TorrentWrapper>() {
            trace!("Processing C torrent download status {:?}", download_status);
            wrapper.download_status(DownloadStatus::from(download_status));
        }
    } else {
        warn!(
            "Unable to process torrent download status, handle {} not found",
            handle
        );
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
pub extern "C" fn register_torrent_resolve_callback(
    popcorn_fx: &mut PopcornFX,
    callback: ResolveTorrentCallback,
) {
    trace!("Registering new C resolve torrent callback");
    if let Some(manager) = popcorn_fx
        .torrent_manager()
        .downcast_ref::<DefaultTorrentManager>()
    {
        manager.register_resolve_callback(Box::new(
            move |file_info, torrent_directory, auto_start| {
                trace!("Executing resolve torrent callback for {:?}", file_info);
                let torrent_file_info = TorrentFileInfoC::from(file_info.clone());
                let torrent_directory = into_c_string(torrent_directory.to_string());
                let torrent = callback(torrent_file_info, torrent_directory, auto_start);
                trace!("Received {:?} as resolve torrent callback result", torrent);
                TorrentWrapper::from(torrent)
            },
        ));
    }
}

/// Register a new C-compatible cancel torrent callback with a Rust PopcornFX instance.
///
/// This function registers a callback that handles the cancellation of torrent-related operations.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with C-compatible code and dereferences raw pointers.
/// Users of this function should ensure that they provide a valid `PopcornFX` instance and a valid `CancelTorrentCallback`.
///
/// When the registered callback function is invoked by the manager, it converts the arguments and the result between Rust and C types.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `callback` - A `CancelTorrentCallback` function that will be registered to handle cancel torrent events.
#[no_mangle]
pub extern "C" fn torrent_cancel_callback(
    popcorn_fx: &mut PopcornFX,
    callback: CancelTorrentCallback,
) {
    trace!("Registering new C cancel torrent callback");
    if let Some(manager) = popcorn_fx
        .torrent_manager()
        .downcast_ref::<DefaultTorrentManager>()
    {
        manager.register_cancel_callback(Box::new(move |handle| {
            trace!("Executing cancel torrent callback for {:?}", handle);
            callback(into_c_string(handle));
        }));
    }
}

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
    fn test_torrent_state_changed() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let handle = "MyHandle";
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

        torrent_manager
            .by_handle(handle)
            .expect("expected the torrent handle to have been found");
        torrent_state_changed(
            &mut instance,
            into_c_string(handle.to_string()),
            TorrentState::Starting,
        );

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        match result {
            TorrentEvent::StateChanged(state) => assert_eq!(TorrentState::Starting, state),
            _ => assert!(
                false,
                "expected TorrentEvent::StateChanged, but got {} instead",
                result
            ),
        }
    }

    #[test]
    fn test_torrent_piece_finished() {
        init_logger();
        let handle = "MyHandleId654";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        let manager = instance
            .torrent_manager()
            .downcast_ref::<DefaultTorrentManager>()
            .unwrap();
        manager.register_resolve_callback(Box::new(|_, _, _| TorrentWrapper {
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
        }));

        torrent_piece_finished(&mut instance, into_c_string(handle), 5);
    }

    #[test]
    fn test_register_torrent_resolve_callback() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_torrent_resolve_callback(&mut instance, torrent_resolve_callback);
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
        torrent.expect_total_pieces().return_const(10);
        torrent.expect_subscribe().return_const(Handle::new());
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent.expect_prioritize_pieces().return_const(());
        let torrent = Arc::new(Box::new(torrent) as Box<dyn Torrent>);
        let mut instance = new_instance(temp_path);

        let stream = instance
            .torrent_stream_server()
            .start_stream(Arc::downgrade(&torrent))
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
