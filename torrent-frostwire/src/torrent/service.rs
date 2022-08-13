use popcorn_fx::backend::torrent::model::SessionState;
use popcorn_fx::backend::torrent::service::{TorrentService, TorrentServiceListener};
use popcorn_fx::observer::Observable;
use popcorn_fx::property::ReadonlyProperty;

/// The [TorrentService] implementation for libtorrent.
pub struct LibtorrentTorrentService<'a> {
    /// The session state of the service.
    state: SessionState,
    /// The observers which are registered to this service.
    observers: Vec<&'a dyn TorrentServiceListener>,
}

impl<'a> LibtorrentTorrentService<'a> {
    /// Create a new [TorrentService] with the libtorrent implementation.
    pub fn new() -> Self {
        Self {
            state: SessionState::CREATING,
            observers: vec!(),
        }
    }
}

impl<'a> Observable<'a, &dyn TorrentServiceListener> for LibtorrentTorrentService<'a> {
    fn register(&mut self, observer: &'a dyn TorrentServiceListener) {
        self.observers.push(observer)
    }

    fn unregister(&mut self, observer: &'a dyn TorrentServiceListener) {
        if let Some(index) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(index)
        }
    }
}

impl<'a> TorrentService<'a, dyn TorrentServiceListener> for LibtorrentTorrentService<'a> {
    fn session_state(&self) -> SessionState {
        self.state
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx::backend::torrent::model::SessionState;
    use popcorn_fx::backend::torrent::service::TorrentService;

    use crate::torrent::service::LibtorrentTorrentService;

    #[test]
    fn test_session_state_should_return_current_state() {
        let service = LibtorrentTorrentService::new();

        let result = service.session_state();

        assert_eq!(SessionState::CREATING, result)
    }

    #[test]
    fn test_session_state_property_should_return_the_current_state_property() {
        let service = LibtorrentTorrentService::new();

        let property = service.session_state_property();
        let result = property.get();

        assert_eq!(true, result.is_some());
        assert_eq!(SessionState::CREATING, result.unwrap());
    }
}