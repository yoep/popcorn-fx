use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace};

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingResult, LoadingStrategy,
};
use crate::core::media::{Episode, MediaType, MovieDetails, TorrentInfo, DEFAULT_AUDIO_LANGUAGE};

/// Represents a strategy for loading media torrent URLs.
#[derive(Display)]
#[display(fmt = "Media torrent URL loading strategy")]
pub struct MediaTorrentUrlLoadingStrategy {}

impl MediaTorrentUrlLoadingStrategy {
    /// Creates a new `MediaTorrentUrlLoadingStrategy` instance.
    ///
    /// # Returns
    ///
    /// A new `MediaTorrentUrlLoadingStrategy` instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Debug for MediaTorrentUrlLoadingStrategy {
    /// Formats the `MediaTorrentUrlLoadingStrategy` for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// A result containing the formatted output.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaTorrentUrlLoadingStrategy").finish()
    }
}

#[async_trait]
impl LoadingStrategy for MediaTorrentUrlLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        if let Some(media) = data.media.as_ref() {
            if let Some(quality) = data.quality.as_ref() {
                debug!(
                    "Processing media torrent url for {} and quality {}",
                    media, quality
                );
                let media_torrent_info: Option<TorrentInfo>;

                if context.is_cancelled() {
                    return LoadingResult::Err(LoadingError::Cancelled);
                }
                match media.media_type() {
                    MediaType::Movie => {
                        trace!(
                            "Processing movie details for torrent information of {:?}",
                            media
                        );
                        media_torrent_info = media
                            .downcast_ref::<MovieDetails>()
                            .and_then(|movie| {
                                movie.torrents().get(&DEFAULT_AUDIO_LANGUAGE.to_string())
                            })
                            .and_then(|media_torrents| media_torrents.get(&quality.to_string()))
                            .cloned();
                    }
                    MediaType::Episode => {
                        trace!("Processing episode for torrent information");
                        media_torrent_info = media
                            .downcast_ref::<Episode>()
                            .and_then(|episode| {
                                let episode_torrents = episode.torrents();
                                trace!(
                                    "Retrieving {} from episode torrents {:?}",
                                    quality,
                                    episode_torrents
                                );
                                episode_torrents.get(&quality.to_string())
                            })
                            .cloned();
                    }
                    _ => {
                        return LoadingResult::Err(LoadingError::MediaError(format!(
                            "media type {} is not supported",
                            media.media_type()
                        )));
                    }
                }

                if context.is_cancelled() {
                    return LoadingResult::Err(LoadingError::Cancelled);
                }
                if let Some(torrent_info) = media_torrent_info {
                    let url = torrent_info.url().to_string();
                    debug!("Updating playlist item url to {} for media {}", url, media);
                    data.url = Some(url.clone());
                    data.torrent_file = torrent_info.file().map(|e| e.clone());
                    info!("Loading media url {}", url);
                } else {
                    return LoadingResult::Err(LoadingError::MediaError(format!(
                        "failed to resolve media torrent url for {}",
                        media
                    )));
                }
            }
        }

        LoadingResult::Ok
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::playlist::{PlaylistItem, PlaylistMedia};
    use crate::{create_loading_task, init_logger};

    use super::*;

    #[test]
    fn test_process_movie() {
        init_logger!();
        let quality = "720p";
        let torrent_url = "magnet:?MyUrl";
        let torrent_info = TorrentInfo::new(
            torrent_url.to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            0,
            0,
            None,
            None,
            None,
        );
        let item = PlaylistItem {
            url: None,
            title: "LoremIpsum".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(MovieDetails {
                    title: "".to_string(),
                    imdb_id: "".to_string(),
                    year: "".to_string(),
                    runtime: "".to_string(),
                    genres: vec![],
                    synopsis: "".to_string(),
                    rating: None,
                    images: Default::default(),
                    trailer: "".to_string(),
                    torrents: HashMap::from([(
                        DEFAULT_AUDIO_LANGUAGE.to_string(),
                        HashMap::from([(quality.to_string(), torrent_info.clone())]),
                    )]),
                })),
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let mut data = LoadingData::from(item);
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = MediaTorrentUrlLoadingStrategy::new();

        let result = runtime.block_on(strategy.process(&mut data, &*context));

        if let LoadingResult::Ok = result {
            assert_eq!(Some(torrent_url.to_string()), data.url);
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            );
        }
    }

    #[test]
    fn test_cancel() {
        init_logger!();
        let url = "http://localhost:9090/DolorEsta.mp4";
        let title = "FooBar";
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: Some("720p".to_string()),
            auto_resume_timestamp: Some(50000),
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let data = LoadingData::from(item);
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = MediaTorrentUrlLoadingStrategy::new();

        let result = runtime.block_on(strategy.cancel(data.clone()));

        assert_eq!(Ok(data), result);
    }
}
