use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, loader};
use crate::core::config::ApplicationConfig;
use crate::core::loader::{LoadingData, LoadingError, LoadingState, LoadingStrategy, UpdateState};
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

    async fn process(&self, mut data: LoadingData) -> loader::LoadingResult {
        if let Some(torrent_file_info) = data.item.torrent_file_info.as_ref() {
            {
                let state_update = self.state_update.lock().await;
                state_update(LoadingState::Connecting);
            }

            trace!("Processing torrent info of {:?}", torrent_file_info);
            let torrent_directory: String;

            {
                let settings = self.application_settings.lock().await;
                torrent_directory = settings.user_settings().torrent().directory()
                    .to_str()
                    .map(|e| e.to_string())
                    .expect("expected a valid torrent directory from the user settings");
            }

            match self.torrent_manager.create(torrent_file_info, torrent_directory.as_str(), true).await {
                Ok(torrent) => {
                    debug!("Enhancing playlist item with torrent");
                    data.torrent = Some(torrent);
                }
                Err(e) => return loader::LoadingResult::Err(LoadingError::TorrentError(e)),
            }
        }

        loader::LoadingResult::Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::block_in_place;
    use crate::core::loader::LoadingResult;
    use crate::core::playlists::PlaylistItem;
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
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };
        let data = LoadingData::from(item);
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build()));
        let torrent_manager = MockTorrentManager::new();
        let strategy = TorrentLoadingStrategy::new(Arc::new(Box::new(torrent_manager)), settings);

        let result = block_in_place(strategy.process(data.clone()));

        assert_eq!(LoadingResult::Ok(data), result);
    }
}