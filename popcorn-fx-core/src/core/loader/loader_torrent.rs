use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, loader};
use crate::core::config::ApplicationConfig;
use crate::core::loader::{LoadingError, LoadingState, LoadingStrategy, UpdateState};
use crate::core::playlists::PlaylistItem;
use crate::core::torrents::TorrentManager;

#[derive(Display)]
#[display(fmt = "Torrent loading strategy")]
pub struct TorrentLoadingStrategy {
    state_update: Mutex<UpdateState>,
    torrent_manager: Arc<Box<dyn TorrentManager>>,
    application_settings: Arc<Mutex<ApplicationConfig>>,
}

impl TorrentLoadingStrategy {
    pub fn new(torrent_manager: Arc<Box<dyn TorrentManager>>, application_settings: Arc<Mutex<ApplicationConfig>>) -> Self {
        Self {
            state_update: Mutex::new(Box::new(|_| warn!("state_update has not been configured"))),
            torrent_manager,
            application_settings,
        }
    }
}

impl Debug for TorrentLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentLoadingStrategy")
            .field("torrent_manager", &self.torrent_manager)
            .field("application_settings", &self.application_settings)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentLoadingStrategy {
    fn on_state_update(&self, state_update: UpdateState) {
        let mut state = block_in_place(self.state_update.lock());
        *state = state_update;
    }

    async fn process(&self, mut item: PlaylistItem) -> loader::LoadingResult {
        if let Some(torrent_file_info) = item.torrent_file_info.as_ref() {
            {
                let state_update = self.state_update.lock().await;
                state_update(LoadingState::Connecting);
            }

            trace!("Processing torrent info of {:?}", torrent_file_info);
            let settings = self.application_settings.lock().await;
            let torrent_directory = settings.user_settings().torrent().directory().to_str()
                .expect("expected a valid torrent directory");

            match self.torrent_manager.create(torrent_file_info, torrent_directory, true).await {
                Ok(torrent) => {
                    debug!("Enhancing playlist item with torrent {:?}", torrent);
                    item.torrent = Some(torrent);
                }
                Err(e) => return loader::LoadingResult::Err(LoadingError::TorrentError(e)),
            }
        }

        loader::LoadingResult::Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::block_in_place;
    use crate::core::loader::LoadingResult;
    use crate::core::torrents::{MockTorrentManager, TorrentInfo};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process() {
        init_logger();
        let torrent_info = TorrentInfo {
            name: "".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let item = PlaylistItem {
            url: Some("".to_string()),
            title: "Lorem ipsum".to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: Some(torrent_info.clone()),
            torrent_file_info: None,
            torrent: None,
            torrent_stream: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build()));
        let torrent_manager = MockTorrentManager::new();
        let strategy = TorrentLoadingStrategy::new(Arc::new(Box::new(torrent_manager)), settings);

        let result = block_in_place(strategy.process(item.clone()));

        assert_eq!(LoadingResult::Ok(item), result);
    }
}