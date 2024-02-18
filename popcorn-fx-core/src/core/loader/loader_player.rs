use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState, LoadingStrategy};
use crate::core::players::{PlayerManager, PlayMediaRequest, PlayRequest, PlayUrlRequest};

/// A loading strategy specifically designed for player loading.
/// This strategy will translate the [PlaylistItem] into a [PlayRequest] which is invoked on the [PlayerManager].
#[derive(Display)]
#[display(fmt = "Player loading strategy")]
pub struct PlayerLoadingStrategy {
    player_manager: Arc<Box<dyn PlayerManager>>,
}

impl PlayerLoadingStrategy {
    /// Creates a new instance of `PlayerLoadingStrategy`.
    ///
    /// # Arguments
    ///
    /// * `player_manager` - An Arc reference to a PlayerManager.
    ///
    /// # Returns
    ///
    /// A new `PlayerLoadingStrategy` instance.
    pub fn new(player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        Self {
            player_manager,
        }
    }

    /// Converts the loading data into a play request.
    ///
    /// # Arguments
    ///
    /// * `data` - The loading data.
    ///
    /// # Returns
    ///
    /// A result containing a boxed `PlayRequest` if successful, or a `LoadingError` if an error occurs.
    fn convert(&self, data: LoadingData) -> Result<Box<dyn PlayRequest>, LoadingError> {
        if data.media.is_some() {
            trace!("Trying to start media playback for {:?}", data);
            return if data.torrent_stream.is_some() {
                Ok(Box::new(PlayMediaRequest::from(data)))
            } else {
                Err(LoadingError::InvalidData(format!("Missing torrent stream for {:?}", data.media)))
            };
        }

        Ok(Box::new(PlayUrlRequest::from(data)))
    }
}

impl Debug for PlayerLoadingStrategy {
    /// Formats the `PlayerLoadingStrategy` for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// A result containing the formatted output.
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
    async fn process(&self, data: LoadingData, event_channel: Sender<LoadingEvent>, _: CancellationToken) -> LoadingResult {
        if let Some(url) = data.url.as_ref() {
            debug!("Starting playlist item playback for {}", url);
            return match self.convert(data) {
                Ok(request) => {
                    event_channel.send(LoadingEvent::StateChanged(LoadingState::Playing)).unwrap();
                    self.player_manager.play(request).await;
                    LoadingResult::Completed
                }
                Err(err) => LoadingResult::Err(err),
            }
        }

        debug!("No playlist item url is present, playback won't be started");
        LoadingResult::Ok(data)
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

    #[test]
    fn test_process_media_item_no_torrent_stream() {
        init_logger();
        let url = "http://localhost:8090/MyVideo.mkv";
        let expected_error_message = "Missing torrent stream for";
        let movie = MovieDetails {
            title: "FooBar".to_string(),
            imdb_id: "tt123456".to_string(),
            year: "2015".to_string(),
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
            caption: None,
            thumb: None,
            parent_media: None,
            media: Some(Box::new(movie.clone())),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some("1080p".to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let data = LoadingData::from(item);
        let (tx_event, _rx_event) = channel();
        let mut manager = MockPlayerManager::new();
        manager.expect_play()
            .times(0)
            .return_const(());
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        let result = block_in_place(strategy.process(data, tx_event, CancellationToken::new()));

        if let LoadingResult::Err(err) = result {
            if let LoadingError::InvalidData(e) = err {
                assert!(e.contains(expected_error_message), "expected the error message to contain \"{}\", but got {}", expected_error_message, e);
            } else {
                assert!(false, "expected LoadingError::InvalidData, but got {:?} instead", err);
            }
        } else {
            assert!(false, "expected LoadingResult::Err, but got {:?} instead", result);
        }
    }
}