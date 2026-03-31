use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy,
};
use crate::core::media::{Episode, MediaIdentifier, MovieDetails, ShowDetails};
use crate::core::subtitles;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::subtitles::{SubtitleError, SubtitleManager, SubtitlePreference};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace, warn};
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

/// Represents a strategy for loading subtitles.
#[derive(Debug, Display)]
#[display("Subtitles loading strategy")]
pub struct SubtitlesLoadingStrategy {
    manager: Arc<SubtitleManager>,
}

impl SubtitlesLoadingStrategy {
    /// Create a new `SubtitlesLoadingStrategy` instance.
    pub fn new(manager: Arc<SubtitleManager>) -> Self {
        Self { manager }
    }

    /// Updates to the default subtitle based on the loading data.
    ///
    /// # Arguments
    ///
    /// * `data` - The loading data.
    async fn update_to_default_subtitle(&self, data: &mut LoadingData) {
        let subtitles = self.retrieve_available_subtitles(&data).await;

        if let Ok(subtitles) = subtitles {
            let subtitle = self.manager.select_or_default(subtitles.as_slice()).await;

            debug!("Updating subtitle to {} for {:?}", subtitle, data);
            data.subtitle.info = Some(subtitle);
        }
    }

    /// Retrieves the available subtitles for the given loading data.
    ///
    /// # Arguments
    ///
    /// * `data` - The loading data.
    ///
    /// # Returns
    ///
    /// A result containing the available subtitles if found, else the subtitle retrieval error.
    async fn retrieve_available_subtitles(
        &self,
        data: &LoadingData,
    ) -> subtitles::Result<Vec<SubtitleInfo>> {
        debug!("Loading subtitles for {:?}", data);
        let subtitles: subtitles::Result<Vec<SubtitleInfo>>;

        if let Some(media) = data.media.as_ref() {
            if let Some(parent_media) = data.parent_media.as_ref() {
                subtitles = self.handle_episode_subtitle(parent_media, media).await
            } else {
                subtitles = self.handle_movie_subtitles(media).await
            }
        } else if let Some(filename) = data.filename.as_ref() {
            subtitles = self.manager.file_subtitles(filename.as_str()).await
        } else {
            warn!("Unable to retrieve subtitles, no information known about the played item");
            return Err(SubtitleError::SearchFailed(
                "no media information known".to_string(),
            ));
        }

        return subtitles;
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
    async fn handle_movie_subtitles(
        &self,
        movie: &Box<dyn MediaIdentifier>,
    ) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading movie subtitles for playlist item");
        if let Some(movie) = movie.downcast_ref::<MovieDetails>() {
            self.manager.movie_subtitles(movie).await
        } else {
            warn!(
                "Unable to load playlist item subtitle, expected MovieDetails but got {} instead",
                movie
            );
            Err(SubtitleError::ParseUrlError(
                "Unable to load playlist item subtitle, expected MovieDetails".to_string(),
            ))
        }
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
    async fn handle_episode_subtitle(
        &self,
        show: &Box<dyn MediaIdentifier>,
        episode: &Box<dyn MediaIdentifier>,
    ) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading episode subtitles for playlist item");
        if let Some(show) = show.downcast_ref::<ShowDetails>() {
            if let Some(episode) = episode.downcast_ref::<Episode>() {
                self.manager.episode_subtitles(show, episode).await
            } else {
                warn!(
                    "Unable to load playlist item subtitle, expected Episode but got {} instead",
                    episode
                );
                Err(SubtitleError::ParseUrlError(
                    "Unable to load playlist item subtitle, expected Episode".to_string(),
                ))
            }
        } else {
            warn!(
                "Unable to load playlist item subtitle, expected ShowDetails but got {} instead",
                show
            );
            Err(SubtitleError::ParseUrlError(
                "Unable to load playlist item subtitle, expected ShowDetails".to_string(),
            ))
        }
    }

    async fn download_subtitle(
        &self,
        subtitle: &SubtitleInfo,
        data: &LoadingData,
    ) -> Option<Subtitle> {
        let filename = data.filename.clone().or_else(|| {
            data.url.clone().map(|e| {
                debug!("Retrieving filename from url {}", e);
                Path::new(e.as_str())
                    .file_stem()
                    .and_then(|e| e.to_str())
                    .unwrap_or_else(|| {
                        warn!("Unable to retrieve filename from {}", e);
                        ""
                    })
                    .to_string()
            })
        });
        let matcher = SubtitleMatcher::from_string(filename, data.quality.clone());

        match self.manager.download(subtitle, &matcher).await {
            Ok(subtitle) => Some(subtitle),
            Err(e) => {
                error!("Failed to download subtitle, {}", e);
                None
            }
        }
    }
}

#[async_trait]
impl LoadingStrategy for SubtitlesLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        if data.subtitle.enabled.unwrap_or(false) {
            trace!("Subtitle manager state {:?}", self.manager);
            if context.is_cancelled() {
                return LoadingResult::Err(LoadingError::Cancelled);
            }

            let subtitle_preference = self.manager.preference().await;
            // check if the subtitle preference is a custom subtitle and that a subtitle has been passed
            // if so, continue with the loader without executing any action regarding subtitles
            if subtitle_preference == SubtitlePreference::Language(SubtitleLanguage::Custom)
                && data.subtitle.info.is_some()
            {
                return LoadingResult::Ok;
            }

            // check if the subtitle preference is disabled
            // if not, try to download the preferred subtitle
            if subtitle_preference != SubtitlePreference::Disabled {
                // update the current state to retrieving subtitles
                context.send_event(LoadingEvent::StateChanged(
                    LoadingState::RetrievingSubtitles,
                ));

                if subtitle_preference == SubtitlePreference::Language(SubtitleLanguage::None) {
                    trace!("Processing subtitle info for {:?}", data);
                    self.update_to_default_subtitle(data).await;
                } else {
                    debug!(
                        "Current subtitle preference {:?} for {:?}",
                        subtitle_preference, data
                    );
                    if data.subtitle.info.is_none() {
                        if let Ok(subtitles) = self.retrieve_available_subtitles(&data).await {
                            data.subtitle.info =
                                Some(self.manager.select_or_default(&subtitles).await);
                        }
                    }
                }

                if let Some(info) = data.subtitle.info.as_ref() {
                    if context.is_cancelled() {
                        return LoadingResult::Err(LoadingError::Cancelled);
                    }

                    context.send_event(LoadingEvent::StateChanged(
                        LoadingState::DownloadingSubtitle,
                    ));
                    trace!("Downloading subtitle for {:?}", data);
                    if let Some(subtitle) = self.download_subtitle(&info, &data).await {
                        let subtitle_filename = subtitle.file().to_string();
                        data.subtitle.subtitle = Some(subtitle);
                        info!(
                            "Subtitle {} has been downloaded for {:?}",
                            subtitle_filename, data.url
                        );

                        if context.is_cancelled() {
                            return LoadingResult::Err(LoadingError::Cancelled);
                        }
                    }
                }
            } else {
                debug!("Subtitle has been disabled by the user for {:?}", data);
            }
        } else {
            debug!("Subtitles have been disabled for {:?}", data);
        }

        if context.is_cancelled() {
            return LoadingResult::Err(LoadingError::Cancelled);
        }
        LoadingResult::Ok
    }

    async fn cancel(&self, _: &mut LoadingData) -> CancellationResult {
        debug!("Cancelling the subtitle loader");
        self.manager.reset().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{
        ApplicationConfig, DecorationType, PopcornProperties, PopcornSettings, SubtitleFamily,
        SubtitleSettings,
    };
    use crate::core::loader::LoadingResult;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};
    use crate::core::subtitles::{MockSubtitleProvider, SubtitleFile};
    use crate::testing::copy_test_file;
    use crate::{create_loading_task, init_logger, recv_timeout};
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process_movie_subtitles() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
        let subtitle_path = copy_test_file(temp_path, "example.srt", None);
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
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(movie_details.clone())),
            },
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: None,
            },
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(playlist_item);
        let (tx, mut rx) = unbounded_channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(1)
            .returning(move |e| {
                tx.send(e.clone()).unwrap();
                Ok(Vec::new())
            });
        provider
            .expect_file_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        provider
            .expect_download()
            .times(1)
            .return_once(|_, _| Ok(PathBuf::from(subtitle_path)));
        let task = create_loading_task!();
        let context = task.context();
        let loader = SubtitlesLoadingStrategy::new(Arc::new(
            SubtitleManager::builder()
                .settings(settings)
                .provider(provider)
                .build(),
        ));

        let result = loader.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Ok, result);

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(movie_details, result);
    }

    #[tokio::test]
    async fn test_process_filename_subtitles() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
        let subtitle_path = copy_test_file(temp_path, "example.srt", None);
        let filename = "MyTIFile";
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: None,
            },
            torrent: PlaylistTorrent {
                filename: Some(filename.to_string()),
            },
        };
        let mut data = LoadingData::from(playlist_item);
        let (tx, mut rx) = unbounded_channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
                Ok(Vec::new())
            });
        provider
            .expect_download()
            .times(1)
            .return_once(|_, _| Ok(PathBuf::from(subtitle_path)));
        let task = create_loading_task!();
        let context = task.context();
        let loader = SubtitlesLoadingStrategy::new(Arc::new(
            SubtitleManager::builder()
                .settings(settings)
                .provider(provider)
                .build(),
        ));

        let result = loader.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Ok, result);

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(filename.to_string(), result);
    }

    #[tokio::test]
    async fn test_process_subtitle_manager_disabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
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
            media: PlaylistMedia {
                parent: None,
                media: Some(movie),
            },
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: None,
            },
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(playlist_item);
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .returning(|_| panic!("movie_subtitles should not have been invoked"));
        let task = create_loading_task!();
        let context = task.context();
        let subtitle_manager = SubtitleManager::builder()
            .settings(settings)
            .provider(provider)
            .build();
        subtitle_manager
            .update_preference(SubtitlePreference::Disabled)
            .await;
        let loader = SubtitlesLoadingStrategy::new(Arc::new(subtitle_manager));

        let result = loader.process(&mut data, &*context).await;

        assert_eq!(LoadingResult::Ok, result);
    }

    #[tokio::test]
    async fn test_process_playlist_subtitles_disabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(playlist_item);
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        provider
            .expect_episode_subtitles()
            .times(0)
            .returning(|_, _| Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        let task = create_loading_task!();
        let context = task.context();
        let loader = SubtitlesLoadingStrategy::new(Arc::new(
            SubtitleManager::builder()
                .settings(settings)
                .provider(provider)
                .build(),
        ));

        let result = loader.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Ok, result);
    }

    #[tokio::test]
    async fn test_process_custom_subtitle() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: Some(
                    SubtitleInfo::builder()
                        .language(SubtitleLanguage::Custom)
                        .files(vec![SubtitleFile::builder()
                            .file_id(0)
                            .url("/tmp/some/subtitle.srt")
                            .name("Custom")
                            .build()])
                        .build(),
                ),
            },
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(playlist_item);
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        provider
            .expect_episode_subtitles()
            .times(0)
            .returning(|_, _| Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(0)
            .returning(|_| Ok(Vec::new()));
        let task = create_loading_task!();
        let context = task.context();
        let loader = SubtitlesLoadingStrategy::new(Arc::new(
            SubtitleManager::builder()
                .settings(settings)
                .provider(provider)
                .build(),
        ));

        let result = loader.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Ok, result);
    }

    #[tokio::test]
    async fn test_cancel() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path);
        let title = "CancelledItem";
        let mut data = LoadingData::from(PlaylistItem {
            url: None,
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: None,
            },
            torrent: Default::default(),
        });
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .returning(|_| panic!("movie_subtitles should not have been invoked"));
        let loader = SubtitlesLoadingStrategy::new(Arc::new(
            SubtitleManager::builder()
                .settings(settings)
                .provider(provider)
                .build(),
        ));

        let _ = loader
            .cancel(&mut data)
            .await
            .expect("expected the cancel operation to succeed");

        assert_eq!(
            Some(title.to_string()),
            data.title,
            "expected the title data to be unmodified"
        );
    }

    fn default_settings(temp_path: &str) -> ApplicationConfig {
        ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties::default())
            .settings(PopcornSettings {
                subtitle_settings: SubtitleSettings {
                    directory: temp_path.to_string(),
                    auto_cleaning_enabled: false,
                    default_subtitle: SubtitleLanguage::English,
                    font_family: SubtitleFamily::Arial,
                    font_size: 28,
                    decoration: DecorationType::None,
                    bold: false,
                },
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
                tracking_settings: Default::default(),
            })
            .build()
    }
}
