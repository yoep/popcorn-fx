use std::fmt::{Debug, Formatter};
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingStrategy,
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
    async fn process(
        &self,
        mut data: LoadingData,
        _: Sender<LoadingEvent>,
        cancel: CancellationToken,
    ) -> LoadingResult {
        if let Some(media) = data.media.as_ref() {
            if let Some(quality) = data.quality.as_ref() {
                debug!(
                    "Processing media torrent url for {} and quality {}",
                    media, quality
                );
                let media_torrent_info: Option<TorrentInfo>;

                if cancel.is_cancelled() {
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

                if cancel.is_cancelled() {
                    return LoadingResult::Err(LoadingError::Cancelled);
                }
                if let Some(torrent_info) = media_torrent_info {
                    let url = torrent_info.url().to_string();
                    debug!("Updating playlist item url to {} for media {}", url, media);
                    data.url = Some(url.clone());
                    data.media_torrent_info = Some(torrent_info);
                    info!("Loading media url {}", url);
                } else {
                    return LoadingResult::Err(LoadingError::MediaError(format!(
                        "failed to resolve media torrent url for {}",
                        media
                    )));
                }
            }
        }

        LoadingResult::Ok(data)
    }

    async fn cancel(&self, data: LoadingData) -> CancellationResult {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::mpsc::channel;

    use crate::core::block_in_place;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process_movie() {
        init_logger();
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
        let data = LoadingData::from(item);
        let (tx, _) = channel();
        let strategy = MediaTorrentUrlLoadingStrategy::new();

        let result = block_in_place(strategy.process(data, tx, CancellationToken::new()));

        if let LoadingResult::Ok(result) = result {
            assert_eq!(Some(torrent_url.to_string()), result.url);
            assert_eq!(Some(torrent_info), result.media_torrent_info);
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
        let strategy = MediaTorrentUrlLoadingStrategy::new();

        let result = block_in_place(strategy.cancel(data.clone()));

        assert_eq!(Ok(data), result);
    }
}
