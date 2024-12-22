use std::fmt::{Debug, Formatter};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy,
};
use crate::core::media::{
    Episode, MediaIdentifier, MediaType, MovieDetails, DEFAULT_AUDIO_LANGUAGE,
};
use crate::core::torrents::{TorrentFileInfo, TorrentInfo, TorrentManager};

const MAGNET_PREFIX: &str = "magnet:?";

/// Represent the loading strategy for loading the torrent information from a media item or a magnet link.
#[derive(Display)]
#[display(fmt = "Torrent info loading strategy")]
pub struct TorrentInfoLoadingStrategy {
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentInfoLoadingStrategy {
    pub fn new(torrent_manager: Arc<Box<dyn TorrentManager>>) -> Self {
        Self { torrent_manager }
    }

    async fn resolve_torrent_info(
        &self,
        url: &str,
        event_channel: Sender<LoadingEvent>,
    ) -> Result<TorrentInfo, LoadingError> {
        event_channel
            .send(LoadingEvent::StateChanged(LoadingState::Starting))
            .unwrap();
        match self.torrent_manager.info(url).await {
            Ok(info) => {
                debug!("Resolved magnet url to {:?}", info);
                Ok(info)
            }
            Err(e) => {
                error!("Failed to start playlist playback, {}", e);
                Err(LoadingError::TorrentError(e))
            }
        }
    }

    async fn resolve_torrent_file_from_media(
        &self,
        info: &TorrentInfo,
        media: &Box<dyn MediaIdentifier>,
        quality: &str,
    ) -> Result<TorrentFileInfo, LoadingError> {
        match media.media_type() {
            MediaType::Movie => media
                .downcast_ref::<MovieDetails>()
                .and_then(|movie| movie.torrents().get(&DEFAULT_AUDIO_LANGUAGE.to_string()))
                .and_then(|media_torrents| media_torrents.get(&quality.to_string()))
                .and_then(|media_torrent| {
                    media_torrent
                        .file()
                        .and_then(|filename| info.by_filename(filename.as_str()))
                        .or(info.largest_file())
                })
                .ok_or(LoadingError::MediaError(format!(
                    "failed to resolve torrent file for {}",
                    media
                ))),
            MediaType::Episode => media
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
                .and_then(|media_torrent| {
                    media_torrent
                        .file()
                        .and_then(|filename| {
                            trace!("Searching for torrent file by filename {}", filename);
                            info.by_filename(filename.as_str())
                        })
                        .or_else(|| {
                            trace!(
                                "Torrent file by filename not found, using largest file instead"
                            );
                            info.largest_file()
                        })
                })
                .ok_or(LoadingError::MediaError(format!(
                    "failed to resolve torrent file for {} with quality {}",
                    media, quality
                ))),
            _ => Err(LoadingError::MediaError(format!(
                "unsupported media type {}",
                media.media_type()
            ))),
        }
    }
}

impl Debug for TorrentInfoLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentInfoLoadingStrategy")
            .field("torrent_manager", &self.torrent_manager)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentInfoLoadingStrategy {
    async fn process(
        &self,
        mut data: LoadingData,
        event_channel: Sender<LoadingEvent>,
        _: CancellationToken,
    ) -> LoadingResult {
        let mut url: Option<String> = None;

        if data.torrent_info.is_none() {
            trace!(
                "Processing item url {:?} for torrent loading strategy",
                data.url
            );
            if let Some(item_url) = data
                .url
                .as_ref()
                .filter(|url| url.starts_with(MAGNET_PREFIX))
                .cloned()
            {
                url = Some(item_url);
            } else {
                debug!(
                    "Playlist item url {:?} is not a magnet, torrent loading is skipped",
                    data.url
                );
            }
        }

        if let Some(url) = url {
            debug!("Loading torrent information of {}", url);
            let torrent_info = self
                .resolve_torrent_info(url.as_str(), event_channel.clone())
                .await;

            match torrent_info {
                Ok(info) => {
                    if let Some(media) = data.media.as_ref() {
                        if let Some(quality) = data.quality.as_ref() {
                            trace!(
                                "Updating torrent file info for media {} with quality {}",
                                media,
                                quality
                            );
                            match self
                                .resolve_torrent_file_from_media(&info, media, quality.as_str())
                                .await
                            {
                                Ok(torrent_file) => {
                                    debug!("Updating torrent file info to {}", torrent_file);
                                    data.torrent_file_info = Some(torrent_file);
                                }
                                Err(e) => return LoadingResult::Err(e),
                            }
                        }
                    }

                    debug!("Updating torrent info to {:?}", info);
                    // TODO: verify
                    // data.url = None; // remove the original url as the item has been enhanced with the data of it
                    data.torrent_info = Some(info);
                }
                Err(e) => return LoadingResult::Err(e),
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
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tokio_util::sync::CancellationToken;

    use crate::core::media::ShowOverview;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia};
    use crate::core::torrents::{MockTorrentManager, TorrentInfo};
    use crate::core::{block_in_place, media};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process_url() {
        init_logger();
        let magnet_url = "magnet:?MyTorrent";
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let info = TorrentInfo {
            info_hash: String::new(),
            uri: String::new(),
            name: "MyTorrentInfo".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let mut data = LoadingData::from(item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let manager_info = info.clone();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager.expect_info().returning(move |e| {
            tx.send(e.to_string()).unwrap();
            Ok(manager_info.clone())
        });
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result =
            block_in_place(strategy.process(data.clone(), tx_event, CancellationToken::new()));
        let resolve_url = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        data.url = None;
        data.torrent_info = Some(info);

        assert_eq!(magnet_url.to_string(), resolve_url);
        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_process_media_url() {
        init_logger();
        let magnet_url = "magnet:?MyFullShowTorrent";
        let expected_torrent_file_info = TorrentFileInfo {
            filename: "MySecondFile".to_string(),
            file_path: "MySecondFile".to_string(),
            file_size: 25000,
            file_index: 2,
        };
        let show = ShowOverview {
            imdb_id: "tt000111".to_string(),
            tvdb_id: "".to_string(),
            title: "MyShow".to_string(),
            year: "2013".to_string(),
            num_seasons: 2,
            images: Default::default(),
            rating: None,
        };
        let episode = Episode {
            season: 1,
            episode: 2,
            first_aired: 0,
            title: "MySecondEpisode".to_string(),
            overview: "".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "".to_string(),
            thumb: None,
            torrents: vec![(
                "720p".to_string(),
                media::TorrentInfo::builder()
                    .url("magnet:?MyEpisodeTorrentUrl")
                    .provider("MyProvider")
                    .source("MySource")
                    .title("MyTitle")
                    .quality("720p")
                    .seed(10)
                    .peer(5)
                    .file("MySecondFile")
                    .build(),
            )]
            .into_iter()
            .collect(),
        };
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: Some(Box::new(show)),
                media: Some(Box::new(episode)),
            },
            quality: Some("720p".to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let info = TorrentInfo {
            info_hash: String::new(),
            uri: String::new(),
            name: "MyShowTorrentInfo".to_string(),
            directory_name: None,
            total_files: 2,
            files: vec![
                TorrentFileInfo {
                    filename: "MyFirstFile".to_string(),
                    file_path: "".to_string(),
                    file_size: 25000,
                    file_index: 1,
                },
                expected_torrent_file_info.clone(),
            ],
        };
        let data = LoadingData::from(item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let manager_info = info.clone();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager.expect_info().returning(move |e| {
            tx.send(e.to_string()).unwrap();
            Ok(manager_info.clone())
        });
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result = block_in_place(strategy.process(data, tx_event, CancellationToken::new()));
        let resolve_url = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(magnet_url.to_string(), resolve_url);
        if let LoadingResult::Ok(result) = result {
            assert_eq!(Some(expected_torrent_file_info), result.torrent_file_info);
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            )
        }
    }

    #[test]
    fn test_process_non_magnet_url() {
        init_logger();
        let magnet_url = "https://www.youtube.com/v/qwe5485";
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let info = TorrentInfo {
            info_hash: String::new(),
            uri: String::new(),
            name: "MyTorrentInfo".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let data = LoadingData::from(item);
        let (tx_event, _rx_event) = channel();
        let manager_info = info.clone();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_info()
            .times(0)
            .returning(move |_| Ok(manager_info.clone()));
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result =
            block_in_place(strategy.process(data.clone(), tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Ok(data), result);
    }
}
