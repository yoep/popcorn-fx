use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio_util::sync::CancellationToken;

use crate::core::{loader, subtitles};
use crate::core::loader::{CancellationResult, LoadingData, LoadingEvent, LoadingState, LoadingStrategy};
use crate::core::media::{Episode, MediaIdentifier, MovieDetails, ShowDetails};
use crate::core::playlists::PlaylistItem;
use crate::core::subtitles::{SubtitleError, SubtitleManager, SubtitleProvider};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;

#[derive(Display)]
#[display(fmt = "Subtitles loading strategy")]
pub struct SubtitlesLoadingStrategy {
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
    subtitle_manager: Arc<SubtitleManager>,
}

impl SubtitlesLoadingStrategy {
    pub fn new(subtitle_provider: Arc<Box<dyn SubtitleProvider>>, subtitle_manager: Arc<SubtitleManager>) -> Self {
        Self {
            subtitle_provider,
            subtitle_manager,
        }
    }

    async fn update_to_default_subtitle(&self, item: &PlaylistItem) {
        debug!("Loading subtitles for playlist item {}", item);
        let subtitles: subtitles::Result<Vec<SubtitleInfo>>;

        if let Some(media) = item.media.as_ref() {
            if let Some(parent_media) = item.parent_media.as_ref() {
                subtitles = self.handle_episode_subtitle(parent_media, media).await
            } else {
                subtitles = self.handle_movie_subtitles(media).await
            }
        } else {
            todo!()
        }

        if let Ok(subtitles) = subtitles {
            let subtitle = self.subtitle_provider.select_or_default(subtitles.as_slice());

            debug!("Updating subtitle to {} for playlist item {}", subtitle, item);
            self.subtitle_manager.update_subtitle(subtitle);
        }
    }

    async fn handle_movie_subtitles(&self, movie: &Box<dyn MediaIdentifier>) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading movie subtitles for playlist item");
        return if let Some(movie) = movie.downcast_ref::<MovieDetails>() {
            self.subtitle_provider.movie_subtitles(movie).await
        } else {
            warn!("Unable to load playlist item subtitle, expected MovieDetails but got {} instead", movie);
            Err(SubtitleError::ParseUrlError("Unable to load playlist item subtitle, expected MovieDetails".to_string()))
        };
    }

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
    async fn process(&self, data: LoadingData, event_channel: Sender<LoadingEvent>, _: CancellationToken) -> loader::LoadingResult {
        if !self.subtitle_manager.is_disabled_async().await {
            if self.subtitle_manager.preferred_language() == SubtitleLanguage::None {
                trace!("Processing subtitle for {:?}", data);
                event_channel.send(LoadingEvent::StateChanged(LoadingState::RetrievingSubtitles)).unwrap();
                self.update_to_default_subtitle(&data.item).await;
            }
        }

        loader::LoadingResult::Ok(data)
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

    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use crate::core::block_in_place;
    use crate::core::config::ApplicationConfig;
    use crate::core::loader::LoadingResult;
    use crate::core::storage::Storage;
    use crate::core::subtitles::MockSubtitleProvider;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process_subtitle_disabled() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        }));
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
        let manager = Arc::new(SubtitleManager::new(settings));
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager.clone());

        manager.disable_subtitle();
        let result = block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));

        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_cancel() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        }));
        let data = LoadingData::from(PlaylistItem {
            url: None,
            title: "CancelledItem".to_string(),
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
        let manager = Arc::new(SubtitleManager::new(settings));
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager.clone());

        let result = block_in_place(loader.cancel(data.clone()));
        assert_eq!(CancellationResult::Ok(data), result);
    }
}