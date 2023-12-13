use std::sync::Arc;

use log::{debug, info, trace};
use tokio::sync::Mutex;

use crate::core::events::{DEFAULT_ORDER, Event, EventPublisher, PlayVideoEvent};
use crate::core::playlists::{Playlist, PlaylistItem};

#[derive(Debug)]
pub struct PlaylistManager {
    inner: Arc<InnerPlaylistManager>,
}

impl PlaylistManager {
    pub fn new(event_publisher: Arc<EventPublisher>) -> Self {
        let manager = Self {
            inner: Arc::new(InnerPlaylistManager {
                playlist: Mutex::default(),
                event_publisher,
            }),

        };

        let event_manager = manager.inner.clone();
        event_manager.clone().event_publisher.register(Box::new(move |event| {
            if let Event::PlayerStopped(_) = &event {
                event_manager.play_next();
            }

            Some(event)
        }), DEFAULT_ORDER - 10);

        manager
    }

    pub fn playlist(&self) -> Playlist {
        todo!()
    }

    pub fn play(&self, playlist: Playlist) {
        self.inner.play(playlist)
    }
}

#[derive(Debug)]
struct InnerPlaylistManager {
    playlist: Mutex<Playlist>,
    event_publisher: Arc<EventPublisher>,
}

impl InnerPlaylistManager {
    fn play(&self, playlist: Playlist) {
        trace!("Starting new playlist with {:?}", playlist);
        let mut mutex = futures::executor::block_on(self.playlist.lock());
        debug!("Replacing playlist with {:?}", playlist);
        *mutex = playlist
    }

    fn play_next(&self) {
        let mut mutex = futures::executor::block_on(self.playlist.lock());

        if let Some(item) = mutex.next() {
            drop(mutex);

            trace!("Processing next item in playlist {}", item);
            if item.url.is_some() {
                self.play_item(&item);
            }
        } else {
            debug!("End of playlist has been reached")
        }
    }

    fn play_item(&self, item: &PlaylistItem) {
        let url = if let Some(e) = &item.url {
            e.clone()
        } else {
            "".to_string()
        };

        info!("Playing next playlist item {}", item);
        self.event_publisher.publish(Event::PlayVideo(PlayVideoEvent {
            url,
            title: item.title.clone(),
            subtitle: None,
            thumb: item.thumb.clone(),
        }))
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::events::PlayerStoppedEvent;

    use super::*;

    #[test]
    fn test_player_stopped_event_item_url() {
        let url = "https://www.youtube.com";
        let mut playlist = Playlist::default();
        let (tx, rc) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = PlaylistManager::new(event_publisher.clone());

        playlist.add(PlaylistItem {
            url: Some(url.to_string()),
            title: "Foo Bar".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        manager.play(playlist);
        event_publisher.register(Box::new(move |event| {
            match event {
                Event::PlayerStopped(_) => {}
                _ => tx.send(event.clone()).unwrap(),
            }
            Some(event)
        }), DEFAULT_ORDER);

        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: url.to_string(),
            media: None,
            time: None,
            duration: None,
        }));
        let result = rc.recv_timeout(Duration::from_millis(250)).unwrap();

        match result {
            Event::PlayVideo(e) => assert_eq!(url, e.url.as_str()),
            _ => assert!(false, "Expected Event::PlayVideo, but got {} instead", result)
        }
    }
}