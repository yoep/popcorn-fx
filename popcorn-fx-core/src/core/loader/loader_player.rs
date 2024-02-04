use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use tokio_util::sync::CancellationToken;

use crate::core::loader;
use crate::core::loader::{CancellationResult, LoadingData, LoadingEvent, LoadingState, LoadingStrategy};
use crate::core::players::{PlayerManager, PlayMediaRequest, PlayRequest, PlayUrlRequest};

/// A loading strategy specifically designed for player loading.
/// This strategy will translate the [PlaylistItem] into a [PlayRequest] which is invoked on the [PlayerManager].
#[derive(Display)]
#[display(fmt = "Player loading strategy")]
pub struct PlayerLoadingStrategy {
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
            player_manager,
        }
    }

    fn convert(&self, data: LoadingData) -> Box<dyn PlayRequest> {
        if data.media.is_some() {
            return Box::new(PlayMediaRequest::from(data));
        }

        return Box::new(PlayUrlRequest::from(data));
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
    /// Process the given playlist item.
    ///
    /// # Arguments
    ///
    /// * `item` - The playlist item to process.
    async fn process(&self, data: LoadingData, event_channel: Sender<LoadingEvent>, _: CancellationToken) -> loader::LoadingResult {
        if let Some(url) = data.url.as_ref() {
            trace!("Starting playlist item playback for {}", url);
            event_channel.send(LoadingEvent::StateChanged(LoadingState::Playing)).unwrap();
            self.player_manager.play(self.convert(data));
            return loader::LoadingResult::Completed;
        }

        debug!("No playlist item url is present, playback won't be started");
        loader::LoadingResult::Ok(data)
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        Ok(data)
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
            caption: None,
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
        let (tx_event, _rx_event) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(data, tx_event, CancellationToken::new()));
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
        let stream = Arc::new(Box::new(MockTorrentStream::new()) as Box<dyn TorrentStream>);
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: "RRoll".to_string(),
            caption: None,
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
        let (tx_event, _rx_event) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .returning(move |e| {
                tx.send(e).unwrap();
                ()
            });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        block_in_place(strategy.process(data, tx_event, CancellationToken::new()));
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