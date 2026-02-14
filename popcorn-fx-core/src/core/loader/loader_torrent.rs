use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::core::config::ApplicationConfig;
use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingProgress, LoadingState,
    LoadingStrategy, Result, TorrentData,
};
use crate::core::torrents::{Torrent, TorrentEvent, TorrentManager, TorrentState};
use crate::core::{loader, torrents};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use tokio::select;

#[derive(Display)]
#[display("Torrent loading strategy")]
pub struct TorrentLoadingStrategy {
    torrent_manager: Arc<dyn TorrentManager>,
    application_settings: ApplicationConfig,
}

impl TorrentLoadingStrategy {
    pub fn new(
        torrent_manager: Arc<dyn TorrentManager>,
        application_settings: ApplicationConfig,
    ) -> Self {
        Self {
            torrent_manager,
            application_settings,
        }
    }

    async fn create_torrent(
        &self,
        torrent: &Box<dyn Torrent>,
        torrent_filename: &str,
        context: &LoadingTaskContext,
    ) -> Result<Box<dyn Torrent>> {
        debug!("Starting download of {}", torrent_filename);
        let handle = torrent.handle();
        let mut receiver = self
            .torrent_manager
            .find_by_handle(&handle)
            .await
            .map(|e| e.subscribe())
            .ok_or(LoadingError::TorrentError(torrents::Error::InvalidHandle(
                handle.to_string(),
            )))?;
        let mut download_future = self.torrent_manager.download(&handle, torrent_filename);

        loop {
            select! {
                _ = context.cancelled() => return Err(LoadingError::Cancelled),
                Some(event) = receiver.recv() => Self::handle_event(&*event, context),
                _ = &mut download_future => break,
            }
        }

        debug!("Enhancing playlist item with torrent");
        select! {
            _ = context.cancelled() => return Err(LoadingError::Cancelled),
            result = self.torrent_manager.find_by_handle(&handle) =>
                result.ok_or(LoadingError::TorrentError(torrents::Error::InvalidHandle(handle.to_string()))),
        }
    }

    fn handle_event(event: &TorrentEvent, context: &LoadingTaskContext) {
        match event {
            TorrentEvent::StateChanged(state) => {
                match state {
                    TorrentState::Initializing => {
                        context.send_event(LoadingEvent::StateChanged(LoadingState::Initializing))
                    }
                    TorrentState::RetrievingMetadata => context
                        .send_event(LoadingEvent::StateChanged(LoadingState::RetrievingMetadata)),
                    TorrentState::CheckingFiles => {
                        context.send_event(LoadingEvent::StateChanged(LoadingState::VerifyingFiles))
                    }
                    TorrentState::Downloading => {
                        context.send_event(LoadingEvent::StateChanged(LoadingState::Downloading))
                    }
                    TorrentState::Finished => context
                        .send_event(LoadingEvent::StateChanged(LoadingState::DownloadFinished)),
                    _ => {}
                }
            }
            TorrentEvent::Stats(status) => context.send_event(LoadingEvent::ProgressChanged(
                LoadingProgress::from(status.clone()),
            )),
            _ => {}
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
        data: &mut LoadingData,
        context: &LoadingTaskContext,
    ) -> loader::LoadingResult {
        if let Some(TorrentData::Torrent(torrent)) = data.torrent.as_ref() {
            if let Some(torrent_filename) = data.torrent_file.as_ref() {
                trace!("Processing torrent info of {:?}", torrent_filename);
                context.send_event(LoadingEvent::StateChanged(LoadingState::Connecting));

                match self
                    .create_torrent(torrent, torrent_filename.as_str(), context)
                    .await
                {
                    Ok(torrent) => {
                        data.torrent = Some(TorrentData::Torrent(torrent));
                    }
                    Err(err) => {
                        return loader::LoadingResult::Err(err);
                    }
                }
            }
        }

        loader::LoadingResult::Ok
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        if let Some(torrent) = data.torrent.take() {
            debug!("Cancelling the torrent downloading");
            let handle = torrent.handle();
            self.torrent_manager.remove(&handle).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::loader::LoadingResult;
    use crate::core::playlist::{PlaylistItem, PlaylistTorrent};
    use crate::core::torrents::{MockTorrent, MockTorrentManager, Torrent, TorrentHandle};
    use crate::{create_loading_task, init_logger, recv_timeout};
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_process() {
        init_logger!();
        let magnet_uri = "magnet:?xt=urn:btih:9a5c24e8164dfe5a98d2437b7f4d6ec9a7e2e045&dn=Another%20Example%20File&tr=http%3A%2F%2Ftracker.anotherexample.com%3A56789%2Fannounce&xl=987654321&sf=Another%20Folder";
        let item = PlaylistItem {
            url: Some(magnet_uri.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: PlaylistTorrent { filename: None },
        };
        let mut data = LoadingData::from(item);
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ApplicationConfig::builder().storage(temp_path).build();
        let torrent_manager = MockTorrentManager::new();
        let task = create_loading_task!();
        let context = task.context();
        let strategy = TorrentLoadingStrategy::new(Arc::new(torrent_manager), settings);

        let result = strategy.process(&mut data, &*context).await;

        assert_eq!(LoadingResult::Ok, result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cancel() {
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
        data.torrent = Some(TorrentData::Torrent(torrent));
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ApplicationConfig::builder().storage(temp_path).build();
        let (tx, mut rx) = unbounded_channel();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_remove()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
            });
        let strategy = TorrentLoadingStrategy::new(Arc::new(torrent_manager), settings);

        let _ = strategy
            .cancel(&mut data)
            .await
            .expect("expected the cancellation process to succeed");
        assert!(
            data.torrent.is_none(),
            "expected the torrent to have been stopped"
        );

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(handle.to_string(), result);
    }
}
