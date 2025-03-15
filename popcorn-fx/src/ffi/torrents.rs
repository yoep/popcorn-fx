use crate::ffi::{ResultC, TorrentErrorC, TorrentEventC, TorrentEventCallback, TorrentHealthC};
use crate::PopcornFX;
use log::{trace, warn};
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::{from_c_string, into_c_owned};
use popcorn_fx_torrent::torrent::TorrentHandle;
use std::os::raw::c_char;

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
    popcorn_fx: &PopcornFX,
    uri: *const c_char,
) -> ResultC<TorrentHealthC, TorrentErrorC> {
    trace!("Retrieving torrent health of uri from C");
    let uri = from_c_string(uri);
    let runtime = popcorn_fx.runtime();

    ResultC::from(
        block_in_place_runtime(popcorn_fx.torrent_manager().health_from_uri(&uri), runtime)
            .map(|e| TorrentHealthC::from(e))
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
    popcorn_fx: &PopcornFX,
    seeds: u32,
    leechers: u32,
) -> *mut TorrentHealthC {
    trace!(
        "Calculating torrent health from C with seeds {} and leechers {}",
        seeds,
        leechers
    );
    let health = popcorn_fx
        .torrent_manager()
        .calculate_health(seeds, leechers);

    into_c_owned(TorrentHealthC::from(health))
}

/// Register a new callback for torrent events.
#[no_mangle]
pub extern "C" fn register_torrent_event_callback(
    popcorn_fx: &mut PopcornFX,
    handle: i64,
    callback: TorrentEventCallback,
) {
    trace!("Registering torrent event callback from C");
    let handle = TorrentHandle::from(handle);
    let runtime = popcorn_fx.runtime();

    match block_in_place_runtime(
        popcorn_fx.torrent_manager().find_by_handle(&handle),
        runtime,
    ) {
        None => warn!("Failed to find torrent {} for event callback", handle),
        Some(torrent) => {
            let mut receiver = torrent.subscribe();
            runtime.spawn(async move {
                while let Some(event) = receiver.recv().await {
                    match TorrentEventC::try_from(&*event) {
                        Ok(e) => callback(e),
                        Err(e) => {
                            warn!("Failed to convert torrent event to C: {}", e);
                        }
                    }
                }
            });
        }
    }
}

/// Clean the torrents directory.
/// This will remove all existing torrents from the system.
#[no_mangle]
pub extern "C" fn cleanup_torrents_directory(popcorn_fx: &mut PopcornFX) {
    trace!("Cleaning torrents directory from C");
    let torrent_manager = popcorn_fx.torrent_manager().clone();
    popcorn_fx.runtime().spawn(async move {
        torrent_manager.cleanup().await;
    });
}

#[no_mangle]
pub extern "C" fn dispose_torrent_stream_event_value(event: TorrentEventC) {
    trace!("Disposing torrent stream event from C {:?}", event);
    drop(event);
}

#[no_mangle]
pub extern "C" fn dispose_torrent_health(health: Box<TorrentHealthC>) {
    trace!("Disposing torrent health from C {:?}", health);
    drop(health);
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    use popcorn_fx_core::testing::copy_test_file;
    use popcorn_fx_core::{assert_timeout_eq, init_logger, into_c_string};

    use crate::test::new_instance;

    use super::*;

    #[test]
    fn test_torrent_health_from_uri() {
        init_logger!();
        let magnet_uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        let result = torrent_health_from_uri(&mut instance, into_c_string(magnet_uri));

        if let ResultC::Ok(result) = result {
            assert_ne!(0, result.leechers, "expected leechers to be greater than 0");
        } else {
            assert!(false, "expected ResultC::Ok, but got {:?} instead", result);
        }
    }

    #[test]
    fn test_cleanup_torrents_directory() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let settings = instance.settings().clone();
        let mut torrent_settings = block_in_place_runtime(
            settings.user_settings_ref(|e| e.torrent_settings.clone()),
            instance.runtime(),
        );
        torrent_settings.directory = PathBuf::from(temp_path);
        block_in_place_runtime(
            settings.update_torrent(torrent_settings.clone()),
            instance.runtime(),
        );
        let filepath = copy_test_file(
            temp_path,
            "example.mp4",
            Some("torrents/subdir/example.mp4"),
        );

        cleanup_torrents_directory(&mut instance);

        block_in_place_runtime(
            async {
                assert_timeout_eq!(
                    Duration::from_millis(500),
                    false,
                    PathBuf::from(filepath.clone()).exists()
                );
            },
            instance.runtime(),
        );

        let result = block_in_place_runtime(
            instance
                .settings()
                .user_settings_ref(|e| e.torrent_settings.directory.clone()),
            instance.runtime(),
        )
        .exists();
        assert_eq!(
            true, result,
            "expected the torrent directory to still exist"
        );
    }
}
