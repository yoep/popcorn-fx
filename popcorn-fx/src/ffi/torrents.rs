use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::torrents::TorrentHealth;
use popcorn_fx_core::{from_c_string, into_c_owned};

use crate::ffi::{ResultC, TorrentErrorC, TorrentStreamEventC};
use crate::PopcornFX;

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
) -> ResultC<TorrentHealth, TorrentErrorC> {
    trace!("Retrieving torrent health of uri from C");
    let uri = from_c_string(uri);
    let runtime = popcorn_fx.runtime();

    ResultC::from(
        block_in_place_runtime(popcorn_fx.torrent_manager().health_from_uri(&uri), runtime)
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

    use tempfile::tempdir;

    use popcorn_fx_core::testing::copy_test_file;
    use popcorn_fx_core::{assert_timeout_eq, init_logger};

    use crate::test::new_instance;

    use super::*;

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
