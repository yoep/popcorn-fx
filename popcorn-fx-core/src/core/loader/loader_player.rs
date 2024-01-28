use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, loader};
use crate::core::loader::{LoadingState, LoadingStrategy, UpdateState};
use crate::core::players::{PlayerManager, PlayMediaRequest, PlayRequest, PlayUrlRequest};
use crate::core::playlists::PlaylistItem;

#[derive(Display)]
#[display(fmt = "Player loading strategy")]
pub struct PlayerLoadingStrategy {
    state_update: Mutex<UpdateState>,
    player_manager: Arc<Box<dyn PlayerManager>>,
}

impl PlayerLoadingStrategy {
    pub fn new(player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        Self {
            state_update: Mutex::new(Box::new(|_| warn!("state_update has not been configured"))),
            player_manager,
        }
    }

    fn convert(&self, item: PlaylistItem) -> Box<dyn PlayRequest> {
        if item.media.is_some() {
            return Box::new(PlayMediaRequest::from(item));
        }

        Box::new(PlayUrlRequest::from(item))
    }
}

impl Debug for PlayerLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerLoadingStrategy")
            .field("player_manager", &self.player_manager)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for PlayerLoadingStrategy {
    fn on_state_update(&self, state_update: UpdateState) {
        let mut state = block_in_place(self.state_update.lock());
        *state = state_update;
    }

    async fn process(&self, item: PlaylistItem) -> loader::LoadingResult {
        trace!("Starting playback of item {}", item);
        {
            let state_update = self.state_update.lock().await;
            state_update(LoadingState::Playing);
        }

        self.player_manager.play(self.convert(item));
        loader::LoadingResult::Completed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::block_in_place;
    use crate::core::media::MovieDetails;
    use crate::core::players::MockPlayerManager;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process_youtube_url() {
        init_logger();
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let title = "RRoll";
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            torrent: None,
            torrent_stream: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let (tx, rx) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(item));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(url, result.url());
        assert_eq!(title, result.title());
    }

    #[test]
    fn test_process_media_item() {
        init_logger();
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let title = "RRol";
        let quality = "1080p";
        let movie = MovieDetails {
            title: title.to_string(),
            imdb_id: "tt0666".to_string(),
            year: "2018".to_string(),
            runtime: "".to_string(),
            genres: vec![],
            synopsis: "".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "".to_string(),
            torrents: Default::default(),
        };
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: "RRoll".to_string(),
            thumb: None,
            parent_media: None,
            media: Some(Box::new(movie.clone())),
            torrent_info: None,
            torrent_file_info: None,
            torrent: None,
            torrent_stream: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let (tx, rx) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(item));
        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        if let Some(result) = result.downcast_ref::<PlayMediaRequest>() {
            if let Some(media) = result.media.downcast_ref::<MovieDetails>() {
                assert_eq!(movie, *media);
                assert_eq!(Some(quality.to_string()), result.quality());
            } else {
                assert!(false, "expected MovieDetails, but got {:?} instead", result.media);
            }
        } else {
            assert!(false, "expected PlayMediaRequest, but got {:?} instead", result);
        }
    }
}