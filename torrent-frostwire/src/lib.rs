use popcorn_fx::backend::torrent::model::SessionState;
use popcorn_fx::backend::torrent::service::TorrentService;

use crate::torrent::service::LibtorrentTorrentService;

pub mod torrent;

/// Create a new libtorrent service.
// #[no_mangle]
// pub extern "C" fn new_libtorrent_service() -> Box<LibtorrentTorrentService> {
//     Box::new(LibtorrentTorrentService::new())
// }

/// Get the current state of the torrent service.
#[no_mangle]
pub extern "C" fn torrent_service_session_state(service: &LibtorrentTorrentService) -> SessionState {
    service.session_state().clone()
}