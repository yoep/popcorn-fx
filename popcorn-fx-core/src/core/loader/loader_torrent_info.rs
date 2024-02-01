use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState, LoadingStrategy};
use crate::core::media::{DEFAULT_AUDIO_LANGUAGE, Episode, MediaIdentifier, MediaType, MovieDetails};
use crate::core::torrents::{TorrentFileInfo, TorrentInfo, TorrentManager};

const MAGNET_PREFIX: &str = "magnet:?";

#[derive(Display)]
#[display(fmt = "Torrent info loading strategy")]
pub struct TorrentInfoLoadingStrategy {
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentInfoLoadingStrategy {
    pub fn new(torrent_manager: Arc<Box<dyn TorrentManager>>) -> Self {
        Self {
            torrent_manager,
        }
    }

    async fn resolve_torrent_info(&self, url: &str, event_channel: Sender<LoadingEvent>) -> Result<TorrentInfo, LoadingError> {
        event_channel.send(LoadingEvent::StateChanged(LoadingState::Starting)).unwrap();
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

    async fn resolve_torrent_file_from_media(&self, info: &TorrentInfo, media: &Box<dyn MediaIdentifier>, quality: &str) -> Result<TorrentFileInfo, LoadingError> {
        return match media.media_type() {
            MediaType::Movie => {
                media.downcast_ref::<MovieDetails>()
                    .and_then(|movie| movie.torrents().get(&DEFAULT_AUDIO_LANGUAGE.to_string()))
                    .and_then(|media_torrents| media_torrents.get(&quality.to_string()))
                    .and_then(|media_torrent| media_torrent.file()
                        .and_then(|filename| info.by_filename(filename.as_str()))
                        .or(info.largest_file()))
                    .ok_or(LoadingError::MediaError(format!("failed to resolve torrent file for {}", media)))
            }
            MediaType::Episode => {
                media.downcast_ref::<Episode>()
                    .and_then(|episode| {
                        let episode_torrents = episode.torrents();
                        trace!("Retrieving {} from episode torrents {:?}", quality, episode_torrents);

                        episode_torrents.get(&quality.to_string())
                    })
                    .and_then(|media_torrent| media_torrent.file()
                        .and_then(|filename| info.by_filename(filename.as_str()))
                        .or(info.largest_file()))
                    .ok_or(LoadingError::MediaError(format!("failed to resolve torrent file for {} with quality {}", media, quality)))
            }
            _ => {
                Err(LoadingError::MediaError(format!("unsupported media type {}", media.media_type())))
            }
        };
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
    async fn process(&self, mut data: LoadingData, event_channel: Sender<LoadingEvent>, _: CancellationToken) -> LoadingResult {
        if data.item.torrent_info.is_none() {
            trace!("Processing {:?} url for torrent loading strategy", data.item.url);
            if let Some(url) = data.item.url.as_ref()
                .filter(|url| url.starts_with(MAGNET_PREFIX)) {
                debug!("Loading torrent data for playlist item {}", data.item);
                let torrent_info = self.resolve_torrent_info(url.as_str(), event_channel.clone()).await;

                match torrent_info {
                    Ok(e) => {
                        if let Some(media) = data.item.media.as_ref() {
                            if let Some(quality) = data.item.quality.as_ref() {
                                match self.resolve_torrent_file_from_media(&e, media, quality.as_str()).await {
                                    Ok(torrent_file) => {
                                        data.item.torrent_info = Some(e);
                                        data.item.torrent_file_info = Some(torrent_file);
                                    }
                                    Err(e) => return LoadingResult::Err(e),
                                }
                            } else {
                                return LoadingResult::Err(LoadingError::MediaError("Quality information is required".to_string()));
                            }
                        }
                    }
                    Err(e) => return LoadingResult::Err(e),
                }
            } else {
                debug!("Playlist item url is not a magnet, torrent loading is skipped");
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

    use crate::core::playlists::PlaylistItem;
    use crate::core::torrents::{MockTorrentManager, TorrentInfo};
    use crate::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_process() {
        init_logger();
        let magnet_url = "magnet:?MyTorrent";
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
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
        let info = TorrentInfo {
            name: "MyTorrentInfo".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let data = LoadingData::from(item);
        let (tx, rx) = channel();
        let (tx_event, _rx_event) = channel();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager.expect_info()
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
                Ok(info.clone())
            });
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result = strategy.process(data.clone(), tx_event, CancellationToken::new()).await;
        let resolve_url = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert_eq!(magnet_url.to_string(), resolve_url);
        assert_eq!(LoadingResult::Ok(data), result);
    }
}