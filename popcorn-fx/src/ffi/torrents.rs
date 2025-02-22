use crate::ffi::{ResultC, TorrentErrorC, TorrentHealthC, TorrentStreamEventC};
use crate::PopcornFX;
use log::trace;
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::{from_c_string, into_c_owned};
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
pub extern "C" fn dispose_torrent_health(health: Box<TorrentHealthC>) {
    trace!("Disposing torrent health from C {:?}", health);
    drop(health);
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

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
}
