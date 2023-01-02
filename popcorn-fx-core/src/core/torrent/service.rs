use crate::core::torrent::model::SessionState;
use crate::observer::{Observable, Observer};

pub trait TorrentServiceListener: Observer {}

/// The [TorrentService] manages the [Torrent]'s and creation of them.
/// Use this service to resolve magnet url's or start downloading a torrent.
pub trait TorrentService<'a, T: TorrentServiceListener>: Observable<'a, T> {
    /// The current state of the service session.
    fn session_state(&self) -> SessionState;
}