use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, loader};
use crate::core::loader::{LoadingData, LoadingState, LoadingStrategy, UpdateState};
use crate::core::players::{PlayerManager, PlayMediaRequest, PlayRequest, PlayUrlRequest};

/// A loading strategy specifically designed for player loading.
/// This strategy will translate the [PlaylistItem] into a [PlayRequest] which is invoked on the [PlayerManager].
#[derive(Display)]
#[display(fmt = "Player loading strategy")]
pub struct PlayerLoadingStrategy {
    state_update: Mutex<UpdateState>,
    player_manager: Arc<Box<dyn PlayerManager>>,
}

impl PlayerLoadingStrategy {
    /// Create a new instance of `PlayerLoadingStrategy`.
    ///
    /// # Arguments
    ///
    /// * `player_manager` - An Arc reference to a PlayerManager.
    pub fn new(player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        Self {
            state_update: Mutex::new(Box::new(|_| warn!("state_update has not been configured"))),
            player_manager,
        }
    }

    fn convert(&self, data: LoadingData) -> Box<dyn PlayRequest> {
        if data.item.media.is_some() {
            return Box::new(PlayMediaRequest::from(data));
        }

        Box::new(PlayUrlRequest::from(data.item))
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
    /// Handle a state update.
    ///
    /// # Arguments
    ///
    /// * `state_update` - The update state function.
    fn on_state_update(&self, state_update: UpdateState) {
        let mut state = block_in_place(self.state_update.lock());
        *state = state_update;
    }

    /// Process the given playlist item.
    ///
    /// # Arguments
    ///
    /// * `item` - The playlist item to process.
    async fn process(&self, data: LoadingData) -> loader::LoadingResult {
        if let Some(url) = data.item.url.as_ref() {
            trace!("Starting playlist item playback for {}", url);
            {
                let state_update = self.state_update.lock().await;
                state_update(LoadingState::Playing);
            }

            self.player_manager.play(self.convert(data));
            return loader::LoadingResult::Completed;
        }

        debug!("No playlist item url is present, playback won't be started");
        loader::LoadingResult::Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::block_in_place;
    use crate::core::loader::LoadingData;
    use crate::core::media::MovieDetails;
    use crate::core::players::MockPlayerManager;
    use crate::core::playlists::PlaylistItem;
    use crate::core::torrents::{MockTorrentStream, TorrentStream};
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
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let data = LoadingData::from(item);
        let (tx, rx) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(data));
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
        let stream: Arc<dyn TorrentStream> = Arc::new(MockTorrentStream::new());
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: "RRoll".to_string(),
            thumb: None,
            parent_media: None,
            media: Some(Box::new(movie.clone())),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let mut data = LoadingData::from(item);
        data.torrent_stream = Some(Arc::downgrade(&stream));
        let (tx, rx) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(data));
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