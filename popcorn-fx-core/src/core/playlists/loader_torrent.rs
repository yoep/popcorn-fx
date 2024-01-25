use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::debug;

use crate::core::playlists::{LoadingStrategy, PlaylistItem};
use crate::core::torrent::TorrentManager;

const MAGNET_PREFIX: &str = "magnet://";

#[derive(Debug, Display)]
#[display(fmt = "Torrent loading strategy")]
pub struct TorrentLoadingStrategy {
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentLoadingStrategy {
    pub fn new(torrent_manager: Arc<Box<dyn TorrentManager>>) -> Self {
        Self {
            torrent_manager,
        }
    }
}

#[async_trait]
impl LoadingStrategy for TorrentLoadingStrategy {
    async fn process(&self, item: PlaylistItem) -> Option<PlaylistItem> {
        if let Some(url) = item.url.as_ref()
            .filter(|url| url.starts_with(MAGNET_PREFIX)) {
            debug!("Loading torrent data for playlist item {}", item);
        }

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::torrent::MockTorrentManager;
    use crate::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_process() {
        init_logger();
        let item = PlaylistItem {
            url: Some("magnet://MyTorrent".to_string()),
            title: "Lorem ipsum".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let torrent_manager = MockTorrentManager::new();
        let strategy = TorrentLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result = strategy.process(item.clone()).await;

        assert_eq!(Some(item), result);
    }
}