use std::sync::Arc;

use tokio::sync::Mutex;

use crate::core::events::EventPublisher;
use crate::core::playlist::Playlist;

#[derive(Debug)]
pub struct PlaylistPlayer {
    inner: Arc<InnerPlaylistPlayer>,
    event_publisher: Arc<EventPublisher>,
}

impl PlaylistPlayer {
    pub fn new(event_publisher: Arc<EventPublisher>) -> Self {
        Self {
            inner: Arc::new(InnerPlaylistPlayer::default()),
            event_publisher,
        }
    }

    pub fn playlist(&self) -> Playlist {
        todo!()
    }
}

#[derive(Debug, Default)]
struct InnerPlaylistPlayer {
    playlist: Mutex<Playlist>,
}

impl InnerPlaylistPlayer {}

#[cfg(test)]
mod test {
    use super::*;
}