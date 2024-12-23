use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};

use crate::core::config::ApplicationConfig;
use crate::core::loader;
use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingState, LoadingStrategy,
};
use crate::core::torrents::TorrentManager;

#[derive(Display)]
#[display(fmt = "Torrent loading strategy")]
pub struct TorrentLoadingStrategy {
    torrent_manager: Arc<Box<dyn TorrentManager>>,
    application_settings: Arc<ApplicationConfig>,
}

impl TorrentLoadingStrategy {
    pub fn new(
        torrent_manager: Arc<Box<dyn TorrentManager>>,
        application_settings: Arc<ApplicationConfig>,
    ) -> Self {
        Self {
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
    async fn process(
        &self,
        mut data: LoadingData,
        context: &LoadingTaskContext,
    ) -> loader::LoadingResult {
        if let Some(uri) = data.url.as_ref() {
            if let Some(torrent_file_info) = data.torrent_file_info.as_ref() {
                trace!("Processing torrent info of {:?}", torrent_file_info);
                context.send_event(LoadingEvent::StateChanged(LoadingState::Connecting));

                match self
                    .torrent_manager
                    .create(uri.as_ref(), torrent_file_info, true)
                    .await
                {
                    Ok(torrent) => {
                        debug!("Enhancing playlist item with torrent");
                        data.torrent = Some(torrent);
                    }
                    Err(e) => return loader::LoadingResult::Err(LoadingError::TorrentError(e)),
                }
            }
        }

        loader::LoadingResult::Ok(data)
    }

    async fn cancel(&self, mut data: LoadingData) -> CancellationResult {
        if let Some(torrent) = data.torrent.take() {
            debug!("Cancelling the torrent downloading");
            self.torrent_manager.remove(torrent.handle());
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::loader::LoadingResult;
    use crate::core::playlist::{PlaylistItem, PlaylistTorrent};
    use crate::core::torrents::{
        MockTorrent, MockTorrentManager, Torrent, TorrentHandle, TorrentInfo,
    };
    use crate::{create_loading_task, init_logger};

    use super::*;

    #[test]
    fn test_process() {
        init_logger!();
        let torrent_info = TorrentInfo {
            info_hash: String::new(),
            uri: String::new(),
            name: "".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let item = PlaylistItem {
            url: Some("".to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: PlaylistTorrent {
                info: Some(torrent_info.clone()),
                file_info: None,
            },
        };
        let data = LoadingData::from(item);
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let torrent_manager = MockTorrentManager::new();
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = TorrentLoadingStrategy::new(Arc::new(Box::new(torrent_manager)), settings);

        let result = runtime.block_on(strategy.process(data.clone(), &*context));

        assert_eq!(LoadingResult::Ok(data), result);
    }

    #[test]
    fn test_cancel() {
        init_logger!();
        let handle = TorrentHandle::new();
        let mut data = LoadingData::from(PlaylistItem {
            url: Some("".to_string()),
            title: "MyTorrent".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(handle);
        let torrent = Box::new(torrent) as Box<dyn Torrent>;
        data.torrent = Some(torrent);
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let (tx, rx) = channel();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_remove()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
            });
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = TorrentLoadingStrategy::new(Arc::new(Box::new(torrent_manager)), settings);

        let result = runtime.block_on(strategy.cancel(data));
        if let Ok(result) = result {
            assert!(
                result.torrent.is_none(),
                "expected the torrent to have been removed from the data"
            );
        } else {
            assert!(
                false,
                "expected CancellationResult::Ok, but got {:?} instead",
                result
            );
        }

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(handle.to_string(), result);
    }
}
