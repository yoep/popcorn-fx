use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace, warn};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy,
};
use crate::core::media::{Episode, MediaIdentifier, MovieDetails, ShowDetails};
use crate::core::subtitles;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::subtitles::{
    SubtitleError, SubtitleManager, SubtitlePreference, SubtitleProvider,
};

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
    pub fn new(
        subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
        subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    ) -> Self {
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
    async fn update_to_default_subtitle(&self, data: &mut LoadingData) {
        let subtitles = self.retrieve_available_subtitles(&data).await;

        if let Ok(subtitles) = subtitles {
            let subtitle = self
                .subtitle_manager
                .select_or_default(subtitles.as_slice());

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
        } else if let Some(file_info) = data.torrent_file_info.as_ref() {
            subtitles = self
                .subtitle_provider
                .file_subtitles(file_info.filename.as_str())
                .await
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
        return if let Some(movie) = movie.downcast_ref::<MovieDetails>() {
            self.subtitle_provider.movie_subtitles(movie).await
        } else {
            warn!(
                "Unable to load playlist item subtitle, expected MovieDetails but got {} instead",
                movie
            );
            Err(SubtitleError::ParseUrlError(
                "Unable to load playlist item subtitle, expected MovieDetails".to_string(),
            ))
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
    async fn handle_episode_subtitle(
        &self,
        show: &Box<dyn MediaIdentifier>,
        episode: &Box<dyn MediaIdentifier>,
    ) -> subtitles::Result<Vec<SubtitleInfo>> {
        trace!("Loading episode subtitles for playlist item");
        return if let Some(show) = show.downcast_ref::<ShowDetails>() {
            if let Some(episode) = episode.downcast_ref::<Episode>() {
                self.subtitle_provider
                    .episode_subtitles(show, episode)
                    .await
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
        };
    }

    async fn download_subtitle(
        &self,
        subtitle: &SubtitleInfo,
        data: &LoadingData,
    ) -> Option<Subtitle> {
        let filename = data
            .torrent_file_info
            .clone()
            .map(|e| e.filename)
            .or_else(|| {
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

        match self
            .subtitle_provider
            .download_and_parse(subtitle, &matcher)
            .await
        {
            Ok(subtitle) => Some(subtitle),
            Err(e) => {
                error!("Failed to download subtitle, {}", e);
                None
            }
        }
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
    async fn process(
        &self,
        mut data: LoadingData,
        event_channel: Sender<LoadingEvent>,
        cancel: CancellationToken,
    ) -> LoadingResult {
        if data.subtitle.enabled.unwrap_or(false) {
            trace!("Subtitle manager state {:?}", self.subtitle_manager);
            if cancel.is_cancelled() {
                return LoadingResult::Err(LoadingError::Cancelled);
            }

            let subtitle_preference = self.subtitle_manager.preference_async().await;
            // check if the subtitle preference is a custom subtitle and that a subtitle has been passed
            // if so, continue with the loader without executing any action regarding subtitles
            if subtitle_preference == SubtitlePreference::Language(SubtitleLanguage::Custom)
                && data.subtitle.info.is_some()
            {
                return LoadingResult::Ok(data);
            }

            // check if the subtitle preference is disabled
            // if not, try to download the preferred subtitle
            if subtitle_preference != SubtitlePreference::Disabled {
                // update the current state to retrieving subtitles
                event_channel
                    .send(LoadingEvent::StateChanged(
                        LoadingState::RetrievingSubtitles,
                    ))
                    .unwrap();

                if subtitle_preference == SubtitlePreference::Language(SubtitleLanguage::None) {
                    trace!("Processing subtitle info for {:?}", data);
                    self.update_to_default_subtitle(&mut data).await;
                } else {
                    debug!(
                        "Current subtitle preference {:?} for {:?}",
                        subtitle_preference, data
                    );
                    if data.subtitle.info.is_none() {
                        if let Ok(subtitles) = self.retrieve_available_subtitles(&data).await {
                            data.subtitle.info =
                                Some(self.subtitle_manager.select_or_default(&subtitles));
                        }
                    }
                }

                if let Some(info) = data.subtitle.info.as_ref() {
                    if cancel.is_cancelled() {
                        return LoadingResult::Err(LoadingError::Cancelled);
                    }

                    event_channel
                        .send(LoadingEvent::StateChanged(
                            LoadingState::DownloadingSubtitle,
                        ))
                        .unwrap();
                    trace!("Downloading subtitle for {:?}", data);
                    if let Some(subtitle) = self.download_subtitle(&info, &data).await {
                        let subtitle_filename = subtitle.file().to_string();
                        data.subtitle.subtitle = Some(subtitle);
                        info!(
                            "Subtitle {} has been downloaded for {:?}",
                            subtitle_filename, data.url
                        );

                        if cancel.is_cancelled() {
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

        if cancel.is_cancelled() {
            return LoadingResult::Err(LoadingError::Cancelled);
        }
        LoadingResult::Ok(data)
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        debug!("Cancelling the subtitle loader");
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
    use crate::core::playlist::{PlaylistItem, PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};
    use crate::core::subtitles::{MockSubtitleProvider, SubtitleFile};
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
        let data = LoadingData::from(playlist_item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
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
            .return_const(Ok(Vec::new()));
        provider
            .expect_download_and_parse()
            .times(1)
            .return_const(Ok(Subtitle::new(
                vec![],
                None,
                "MySubtitleFile".to_string(),
            )));
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference_async()
            .times(1)
            .return_const(SubtitlePreference::Language(SubtitleLanguage::None));
        manager
            .expect_select_or_default()
            .times(1)
            .returning(|_| SubtitleInfo::none());
        let loader = SubtitlesLoadingStrategy::new(
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(manager)),
        );

        let result =
            block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
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
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: true,
                info: None,
            },
            torrent: PlaylistTorrent {
                info: None,
                file_info: Some(torrent_file_info.clone()),
            },
        };
        let data = LoadingData::from(playlist_item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .return_const(Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
                Ok(Vec::new())
            });
        provider
            .expect_download_and_parse()
            .times(1)
            .return_const(Ok(Subtitle::new(
                vec![],
                None,
                "MySubtitleFile".to_string(),
            )));
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference_async()
            .times(1)
            .return_const(SubtitlePreference::Language(SubtitleLanguage::None));
        manager
            .expect_select_or_default()
            .times(1)
            .returning(|_| SubtitleInfo::none());
        let loader = SubtitlesLoadingStrategy::new(
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(manager)),
        );

        let result =
            block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
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
        let data = LoadingData::from(playlist_item);
        let (tx_event, _) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .returning(|_| panic!("movie_subtitles should not have been invoked"));
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference_async()
            .times(1)
            .return_const(SubtitlePreference::Disabled);
        let manager = Arc::new(Box::new(manager) as Box<dyn SubtitleManager>);
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager);

        let result =
            block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));

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
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let data = LoadingData::from(playlist_item);
        let (tx_event, _) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider
            .expect_episode_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference_async()
            .return_const(SubtitlePreference::Disabled);
        let loader = SubtitlesLoadingStrategy::new(
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(manager)),
        );

        let result =
            block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_process_custom_subtitle() {
        init_logger();
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
        let data = LoadingData::from(playlist_item);
        let (tx_event, _) = channel();
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_movie_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider
            .expect_episode_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        provider
            .expect_file_subtitles()
            .times(0)
            .return_const(subtitles::Result::Ok(Vec::new()));
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference_async()
            .return_const(SubtitlePreference::Language(SubtitleLanguage::Custom));
        let loader = SubtitlesLoadingStrategy::new(
            Arc::new(Box::new(provider)),
            Arc::new(Box::new(manager)),
        );

        let result =
            block_in_place(loader.process(data.clone(), tx_event, CancellationToken::new()));
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
        let mut manager = MockSubtitleManager::new();
        manager.expect_reset().times(1).return_const(());
        let manager = Arc::new(Box::new(manager) as Box<dyn SubtitleManager>);
        let loader = SubtitlesLoadingStrategy::new(Arc::new(Box::new(provider)), manager);

        let result = block_in_place(loader.cancel(data.clone()));
        assert_eq!(Ok(data), result);
    }
}
