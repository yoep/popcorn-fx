use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace};

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy,
};
use crate::core::players::{
    PlayMediaRequest, PlayRequest, PlayStreamRequest, PlayUrlRequest, PlayerManager,
};

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
        Self { player_manager }
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
    fn convert(&self, data: &mut LoadingData) -> Result<Box<dyn PlayRequest>, LoadingError> {
        if data.media.is_some() {
            trace!("Trying to start media playback for {:?}", data);
            return if data.torrent.is_some() {
                PlayMediaRequest::try_from(data)
                    .map(|e| Box::new(e) as Box<dyn PlayRequest>)
                    .map_err(|e| LoadingError::ParseError(e.to_string()))
            } else {
                Err(LoadingError::InvalidData(format!(
                    "Missing torrent stream for {:?}",
                    data.media
                )))
            };
        } else if data.torrent.is_some() {
            trace!("Trying to start torrent stream playback for {:?}", data);
            return PlayStreamRequest::try_from(data)
                .map(|e| Box::new(e) as Box<dyn PlayRequest>)
                .map_err(|e| LoadingError::ParseError(e.to_string()));
        }

        trace!("Starting URL playback for {:?}", data);
        PlayUrlRequest::try_from(data)
            .map(|e| Box::new(e) as Box<dyn PlayRequest>)
            .map_err(|e| LoadingError::ParseError(e.to_string()))
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
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        if let Some(url) = data.url.as_ref() {
            let url = url.clone();
            debug!("Starting playlist item playback for {}", url);
            return match self.convert(data) {
                Ok(request) => {
                    context.send_event(LoadingEvent::StateChanged(LoadingState::Playing));
                    self.player_manager.play(request).await;
                    info!("Playback started for {}", url);
                    LoadingResult::Completed
                }
                Err(err) => LoadingResult::Err(err),
            };
        }

        debug!("No playlist item url is present, playback won't be started");
        LoadingResult::Ok
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::loader::{LoadingData, TorrentData};
    use crate::core::media::MovieDetails;
    use crate::core::players::MockPlayerManager;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia};
    use crate::testing::MockTorrentStream;
    use crate::{create_loading_task, init_logger, recv_timeout};

    use super::*;

    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process_youtube_url() {
        init_logger!();
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let title = "RRoll";
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(item);
        let (tx, mut rx) = unbounded_channel();
        let task = create_loading_task!();
        let context = task.context();
        let mut manager = MockPlayerManager::new();
        manager.expect_play().returning(move |e| {
            tx.send(e).unwrap();
            ()
        });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        strategy.process(&mut data, &*context).await;
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        assert_eq!(url, result.url());
        assert_eq!(title, result.title());
    }

    #[tokio::test]
    async fn test_process_media_item() {
        init_logger!();
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
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(movie.clone())),
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(item);
        data.torrent = Some(TorrentData::Stream(Box::new(MockTorrentStream::new())));
        let (tx, mut rx) = unbounded_channel();
        let task = create_loading_task!();
        let context = task.context();
        let mut manager = MockPlayerManager::new();
        manager.expect_play().returning(move |e| {
            tx.send(e).unwrap();
            ()
        });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        strategy.process(&mut data, &*context).await;
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        if let Some(result) = result.downcast_ref::<PlayMediaRequest>() {
            if let Some(media) = result.media.downcast_ref::<MovieDetails>() {
                assert_eq!(movie, *media);
                assert_eq!(Some(quality.to_string()), result.quality());
            } else {
                assert!(
                    false,
                    "expected MovieDetails, but got {:?} instead",
                    result.media
                );
            }
        } else {
            assert!(
                false,
                "expected PlayMediaRequest, but got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_process_media_item_no_torrent_stream() {
        init_logger!();
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
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(movie.clone())),
            },
            quality: Some("1080p".to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(item);
        let task = create_loading_task!();
        let context = task.context();
        let mut manager = MockPlayerManager::new();
        manager.expect_play().times(0).return_const(());
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        let result = strategy.process(&mut data, &*context).await;

        if let LoadingResult::Err(err) = result {
            if let LoadingError::InvalidData(e) = err {
                assert!(
                    e.contains(expected_error_message),
                    "expected the error message to contain \"{}\", but got {}",
                    expected_error_message,
                    e
                );
            } else {
                assert!(
                    false,
                    "expected LoadingError::InvalidData, but got {:?} instead",
                    err
                );
            }
        } else {
            assert!(
                false,
                "expected LoadingResult::Err, but got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_process_torrent_stream() {
        init_logger!();
        let url = "https://localhost:87445/MyVideo.mkv";
        let title = "streaming title";
        let quality = "1080p";
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: None,
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(item);
        data.torrent = Some(TorrentData::Stream(Box::new(MockTorrentStream::new())));
        let (tx, mut rx) = unbounded_channel();
        let task = create_loading_task!();
        let context = task.context();
        let mut manager = MockPlayerManager::new();
        manager.expect_play().returning(move |e| {
            tx.send(e).unwrap();
            ()
        });
        let strategy = PlayerLoadingStrategy::new(Arc::new(Box::new(manager)));

        strategy.process(&mut data, &*context).await;
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));

        if let Some(result) = result.downcast_ref::<PlayStreamRequest>() {
            assert_eq!(Some(quality.to_string()), result.quality());
        } else {
            assert!(
                false,
                "expected PlayMediaRequest, but got {:?} instead",
                result
            );
        }
    }
}
