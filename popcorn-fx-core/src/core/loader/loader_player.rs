use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info};

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy,
};
use crate::core::players::{PlayRequest, PlayerManager};

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
    fn convert(&self, data: &mut LoadingData) -> Result<PlayRequest, LoadingError> {
        PlayRequest::try_from(data).map_err(|e| LoadingError::ParseError(e.to_string()))
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

        let media = result.media().unwrap();
        if let Some(details) = media.downcast_ref::<MovieDetails>() {
            assert_eq!(movie, *details);
            assert_eq!(Some(quality.to_string()), result.quality());
        } else {
            assert!(
                false,
                "expected MovieDetails, but got {:?} instead",
                result.media()
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

        assert_eq!(Some(quality.to_string()), result.quality());
    }
}
