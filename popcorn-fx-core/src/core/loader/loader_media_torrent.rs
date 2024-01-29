use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::block_in_place;
use crate::core::loader::{LoadingData, LoadingError, LoadingResult, LoadingStrategy, UpdateState};
use crate::core::media::{DEFAULT_AUDIO_LANGUAGE, Episode, MediaType, MovieDetails, TorrentInfo};

#[derive(Display)]
#[display(fmt = "Media torrent utl loading strategy")]
pub struct MediaTorrentUrlLoadingStrategy {
    state_update: Mutex<UpdateState>,
}

impl MediaTorrentUrlLoadingStrategy {
    pub fn new() -> Self {
        Self {
            state_update: Mutex::new(Box::new(|_| warn!("state_update has not been configured"))),
        }
    }
}

impl Debug for MediaTorrentUrlLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaTorrentUrlLoadingStrategy")
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for MediaTorrentUrlLoadingStrategy {
    fn on_state_update(&self, state_update: UpdateState) {
        let mut state = block_in_place(self.state_update.lock());
        *state = state_update;
    }

    async fn process(&self, mut data: LoadingData) -> LoadingResult {
        if let Some(media) = data.item.media.as_ref() {
            if let Some(quality) = data.item.quality.as_ref() {
                debug!("Processing media torrent url for {} and quality {}", media, quality);
                let media_torrent_info: Option<TorrentInfo>;

                match media.media_type() {
                    MediaType::Movie => {
                        trace!("Processing movie details for torrent information of {:?}", media);
                        media_torrent_info = media.downcast_ref::<MovieDetails>()
                            .and_then(|movie| movie.torrents().get(&DEFAULT_AUDIO_LANGUAGE.to_string()))
                            .and_then(|media_torrents| media_torrents.get(&quality.to_string()))
                            .cloned();
                    }
                    MediaType::Episode => {
                        trace!("Processing episode for torrent information");
                        media_torrent_info = media.downcast_ref::<Episode>()
                            .and_then(|episode| {
                                let episode_torrents = episode.torrents();
                                trace!("Retrieving {} from episode torrents {:?}", quality, episode_torrents);
                                episode_torrents.get(&quality.to_string())
                            })
                            .cloned();
                    }
                    _ => {
                        return LoadingResult::Err(LoadingError::MediaError(format!("media type {} is not supported", media.media_type())));
                    }
                }

                if let Some(torrent_info) = media_torrent_info {
                    let url = torrent_info.url().to_string();
                    debug!("Updating playlist item url to {} for media {}", url, media);
                    data.item.url = Some(url);
                    data.media_torrent_info = Some(torrent_info);
                } else {
                    return LoadingResult::Err(LoadingError::MediaError(format!("failed to resolve media torrent url for {}", media)));
                }
            }
        }

        LoadingResult::Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::playlists::PlaylistItem;
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
            thumb: None,
            parent_media: None,
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
                torrents: HashMap::from([
                    (DEFAULT_AUDIO_LANGUAGE.to_string(), HashMap::from([
                        (quality.to_string(), torrent_info.clone()),
                    ])),
                ]),
            })),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let data = LoadingData::from(item);
        let strategy = MediaTorrentUrlLoadingStrategy::new();

        let result = block_in_place(strategy.process(data));

        if let LoadingResult::Ok(result) = result {
            assert_eq!(Some(torrent_url.to_string()), result.item.url);
            assert_eq!(Some(torrent_info), result.media_torrent_info);
        } else {
            assert!(false, "expected LoadingResult::Ok, but got {:?} instead", result);
        }
    }
}