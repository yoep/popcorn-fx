use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState, LoadingStrategy};
use crate::core::media::{Episode, MediaIdentifier, MovieDetails, ShowDetails};
use crate::core::subtitles;
use crate::core::subtitles::{SubtitleError, SubtitleManager, SubtitleProvider};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;

/// Represents a strategy for loading subtitles.
#[derive(Display)]
#[display(fmt = "Subtitles loading strategy")]
pub struct SubtitlesLoadingStrategy {
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
}

impl SubtitlesLoadingStrategy {
    /// Creates a new `SubtitlesLoadingStrategy` instance.
    ///
    /// # Arguments
    ///
    /// * `subtitle_provider` - An `Arc` pointer to a `SubtitleProvider` trait object.
    /// * `subtitle_manager` - An `Arc` pointer to a `SubtitleManager` instance.
    ///
    /// # Returns
    ///
    /// A new `SubtitlesLoadingStrategy` instance.
    pub fn new(subtitle_provider: Arc<Box<dyn SubtitleProvider>>, subtitle_manager: Arc<Box<dyn SubtitleManager>>) -> Self {
        Self {
            subtitle_provider,
            subtitle_manager,
        }
    }

    /// Updates to the default subtitle based on the loading data.
    ///
    /// # Arguments
    ///
    /// * `data` - The loading data.
    async fn update_to_default_subtitle(&self, data: &LoadingData) {
        debug!("Loading subtitles for {:?}", data);
        let subtitles: subtitles::Result<Vec<SubtitleInfo>>;

        if let Some(media) = data.media.as_ref() {
            if let Some(parent_media) = data.parent_media.as_ref() {
                subtitles = self.handle_episode_subtitle(parent_media, media).await
            } else {
                subtitles = self.handle_movie_subtitles(media).await
            }
        } else if let Some(file_info) = data.torrent_file_info.as_ref() {
            subtitles = self.subtitle_provider.file_subtitles(file_info.filename.as_str()).await
        } else {
            warn!("Unable to retrieve subtitles, no information known about the played item");
            return;
        }

        if let Ok(subtitles) = subtitles {
            let subtitle = self.subtitle_provider.select_or_default(subtitles.as_slice());

            debug!("Updating subtitle to {} for {:?}", subtitle, data);
            self.subtitle_manager.update_subtitle(subtitle);
        }
    }

    /// Handles loading subtitles for a movie.
    ///
    /// # Arguments
    ///
    /// * `movie` - The movie media identifier.
    ///
    /// # Returns
    ///
    /// A result containing a vector of `SubtitleInfo` if successful, or a `SubtitleError` if an error occurs.
    async fn handle_movie_subtitles(&self, movie: &Box<dyn MediaIdentifier>) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading movie subtitles for playlist item");
        return if let Some(movie) = movie.downcast_ref::<MovieDetails>() {
            self.subtitle_provider.movie_subtitles(movie).await
        } else {
            warn!("Unable to load playlist item subtitle, expected MovieDetails but got {} instead", movie);
            Err(SubtitleError::ParseUrlError("Unable to load playlist item subtitle, expected MovieDetails".to_string()))
        };
    }

    /// Handles loading subtitles for an episode.
    ///
    /// # Arguments
    ///
    /// * `show` - The show media identifier.
    /// * `episode` - The episode media identifier.
    ///
    /// # Returns
    ///
    /// A result containing a vector of `SubtitleInfo` if successful, or a `SubtitleError` if an error occurs.
    async fn handle_episode_subtitle(&self, show: &Box<dyn MediaIdentifier>, episode: &Box<dyn MediaIdentifier>) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading episode subtitles for playlist item");
        return if let Some(show) = show.downcast_ref::<ShowDetails>() {
            if let Some(episode) = episode.downcast_ref::<Episode>() {
                self.subtitle_provider.episode_subtitles(show, episode).await
            } else {
                warn!("Unable to load playlist item subtitle, expected Episode but got {} instead", episode);
                Err(SubtitleError::ParseUrlError("Unable to load playlist item subtitle, expected Episode".to_string()))
            }
        } else {
            warn!("Unable to load playlist item subtitle, expected ShowDetails but got {} instead", show);
            Err(SubtitleError::ParseUrlError("Unable to load playlist item subtitle, expected ShowDetails".to_string()))
        };
    }
}


impl Debug for SubtitlesLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubtitleLoadingStrategy")
            .field("subtitle_provider", &self.subtitle_provider)
            .field("subtitle_manager", &self.subtitle_manager)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for SubtitlesLoadingStrategy {
    async fn process(&self, data: LoadingData, event_channel: Sender<LoadingEvent>, cancel: CancellationToken) -> LoadingResult {
        if data.subtitles_enabled.unwrap_or(false) {
            trace!("Subtitle manager state {:?}", self.subtitle_manager);

            if cancel.is_cancelled() {
                return LoadingResult::Err(LoadingError::Cancelled);
            }
            if !self.subtitle_manager.is_disabled_async().await {
                if self.subtitle_manager.preferred_language() == SubtitleLanguage::None {
                    trace!("Processing subtitle for {:?}", data);
                    event_channel.send(LoadingEvent::StateChanged(LoadingState::RetrievingSubtitles)).unwrap();
                    self.update_to_default_subtitle(&data).await;
                } else {
                    debug!("Subtitle has already been selected for {:?}", data);
                }
            } else {
                debug!("Subtitle has been disabled by the user for {:?}", data);
            }
        } else {
            debug!("Subtitles have been disabled for {:?}", data);
        }

        if cancel.is_cancelled() {
            return LoadingResult::Err(LoadingError::Cancelled);
        }
        LoadingResult::Ok(data)
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        debug!("Cancelling the subtitle load");
        self.subtitle_manager.reset();
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::block_in_place;
    use crate::core::loader::LoadingResult;
    use crate::core::playlists::PlaylistItem;
    use crate::core::subtitles::MockSubtitleProvider;
    use crate::core::torrents::TorrentFileInfo;
    use crate::testing::{init_logger, MockSubtitleManager};

    use super::*;

    #[test]
    fn test_process_movie_subtitles() {
        init_logger();
        let movie_details = MovieDetails {
            title: "MyMovieTitle".to_string(),
            imdb_id: "tt112233".to_string(),
            year: "2013".to_string(),
            runtime: "80".to_string(),
            genres: vec![],
            synopsis: "Lorem ipsum dolor".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "".to_string(),
            torrents: Default::default(),
        };
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: Some(Box::new(movie_details.clone())),
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };
        let data = LoadingData::from(playlist_item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider.expect_movie_subtitles()
            .times(1)
            .returning(move |e| {
                tx.send(e.clone()).unwrap();
                Ok(Vec::new())
            });
        provider.expect_file_subtitles()
            .times(0)
            .return_const(Ok(Vec::new()));
        provider.expect_select_or_default()
            .times(1)
            .returning(|_| {
                SubtitleInfo::none()
            });
        let mut manager = MockSubtitleManager::new();
        manager.expect_is_disabled_async()
            .times(1)
            .return_const(false);
        manager.expect_preferred_language()
            .times(1)
            .return_const(SubtitleLanguage::None);
        manager.expect_update_subtitle()
            .times(1)
            .return_const(());
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), Arc::new(Box::new(manager)));

        let result = block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Ok(data), result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(movie_details, result);
    }

    #[test]
    fn test_process_filename_subtitles() {
        init_logger();
        let filename = "MyTIFile";
        let torrent_file_info = TorrentFileInfo {
            filename: filename.to_string(),
            file_path: "/tmp/some/random/path".to_string(),
            file_size: 845000,
            file_index: 12,
        };
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: Some(torrent_file_info.clone()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };
        let data = LoadingData::from(playlist_item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider.expect_movie_subtitles()
            .times(0)
            .return_const(Ok(Vec::new()));
        provider.expect_file_subtitles()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
                Ok(Vec::new())
            });
        provider.expect_select_or_default()
            .times(1)
            .returning(|_| {
                SubtitleInfo::none()
            });
        let mut manager = MockSubtitleManager::new();
        manager.expect_is_disabled_async()
            .times(1)
            .return_const(false);
        manager.expect_preferred_language()
            .times(1)
            .return_const(SubtitleLanguage::None);
        manager.expect_update_subtitle()
            .times(1)
            .return_const(());
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), Arc::new(Box::new(manager)));

        let result = block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Ok(data), result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(filename.to_string(), result);
    }

    #[test]
    fn test_process_subtitle_manager_disabled() {
        init_logger();
        let movie = Box::new(MovieDetails {
            title: "".to_string(),
            imdb_id: "tt112233".to_string(),
            year: "".to_string(),
            runtime: "".to_string(),
            genres: vec![],
            synopsis: "".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "".to_string(),
            torrents: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: Some(movie),
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };
        let data = LoadingData::from(playlist_item);
        let (tx_event, _) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider.expect_movie_subtitles()
            .returning(|_| {
                panic!("movie_subtitles should not have been invoked")
            });
        let mut manager = MockSubtitleManager::new();
        manager.expect_is_disabled_async()
            .return_const(true);
        let manager = Arc::new(Box::new(manager) as Box<dyn SubtitleManager>);
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager);

        let result = block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));

        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_process_playlist_subtitles_disabled() {
        init_logger();
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
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
        let data = LoadingData::from(playlist_item);
        let (tx_event, _) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider.expect_movie_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider.expect_episode_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider.expect_file_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        let mut manager = MockSubtitleManager::new();
        manager.expect_is_disabled_async()
            .times(0)
            .return_const(true);
        manager.expect_preferred_subtitle()
            .times(0)
            .return_const(Some(SubtitleInfo::custom()));
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), Arc::new(Box::new(manager)));

        let result = block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_cancel() {
        init_logger();
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "CancelledItem".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        });
        let mut provider = MockSubtitleProvider::new();
        provider.expect_movie_subtitles()
            .returning(|_| {
                panic!("movie_subtitles should not have been invoked")
            });
        let mut manager = MockSubtitleManager::new();
        manager.expect_reset()
            .times(1)
            .return_const(());
        let manager = Arc::new(Box::new(manager) as Box<dyn SubtitleManager>);
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager);

        let result = block_in_place(loader.cancel(data.clone()));
        assert_eq!(Ok(data), result);
    }
}