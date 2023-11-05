use std::sync::Arc;

use tokio::sync::Mutex;

use crate::core::events::EventPublisher;
use crate::core::playlists::Playlist;

#[derive(Debug)]
pub struct PlaylistManager {
    inner: Arc<InnerPlaylistManager>,
    event_publisher: Arc<EventPublisher>,
}

impl PlaylistManager {
    pub fn new(event_publisher: Arc<EventPublisher>) -> Self {
        Self {
            inner: Arc::new(InnerPlaylistManager::default()),
            event_publisher,
        }
    }

    pub fn playlist(&self) -> Playlist {
        todo!()
    }
}

#[derive(Debug, Default)]
struct InnerPlaylistManager {
    playlist: Mutex<Playlist>,
}

impl InnerPlaylistManager {}

#[cfg(test)]
mod test {
    use super::*;
}