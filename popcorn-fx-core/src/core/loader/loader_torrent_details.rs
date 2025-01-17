use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};

use crate::core::event::{Event, EventPublisher};
use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingResult, LoadingStrategy,
};
use crate::core::torrents::TorrentManager;

/// Represents a loading strategy for handling torrent details.
///
/// This strategy is responsible for processing torrent-related data and publishing events when torrent details are loaded.
#[derive(Display)]
#[display(fmt = "Torrent details loading strategy")]
pub struct TorrentDetailsLoadingStrategy {
    event_publisher: Arc<EventPublisher>,
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentDetailsLoadingStrategy {
    /// Creates a new instance of `TorrentDetailsLoadingStrategy`.
    ///
    /// # Arguments
    ///
    /// * `event_publisher` - An `EventPublisher` for publishing events related to torrent details.
    pub fn new(
        event_publisher: Arc<EventPublisher>,
        torrent_manager: Arc<Box<dyn TorrentManager>>,
    ) -> Self {
        Self {
            event_publisher,
            torrent_manager,
        }
    }
}

impl Debug for TorrentDetailsLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentDetailsLoadingStrategy")
            .field("event_publisher", &self.event_publisher)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentDetailsLoadingStrategy {
    async fn process(&self, data: LoadingData, _context: &LoadingTaskContext) -> LoadingResult {
        trace!("Processing torrent details strategy for {:?}", data);
        if let Some(torrent) = data.torrent.as_ref() {
            // check if a specific file has been set to be loaded
            // if not, stop the loading chain and show the retrieved details
            if let None = data.torrent_file.as_ref() {
                let handle = torrent.handle();
                return match self.torrent_manager.info(&handle).await {
                    Ok(torrent_info) => {
                        self.event_publisher
                            .publish(Event::TorrentDetailsLoaded(torrent_info.clone()));
                        LoadingResult::Completed
                    }
                    Err(e) => LoadingResult::Err(LoadingError::TorrentError(e)),
                };
            } else {
                debug!("Torrent file info present, torrent details won't be shown");
            }
        } else {
            debug!("No torrent information present, torrent details won't be loaded");
        }

        LoadingResult::Ok(data)
    }

    async fn cancel(&self, mut data: LoadingData) -> CancellationResult {
        if data.torrent.is_some() {
            if let Some(torrent) = data.torrent.take() {
                let handle = torrent.handle();
                self.torrent_manager.remove(&handle).await;
            }
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::loader::loading_chain::DEFAULT_ORDER;
    use crate::core::loader::SubtitleData;
    use crate::core::torrents::{MockTorrentManager, TorrentHandle, TorrentInfo};
    use crate::{create_loading_task, init_logger};

    use super::*;

    #[test]
    fn test_process() {
        init_logger!();
        let torrent_info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: String::new(),
            name: "MyTorrentName".to_string(),
            directory_name: None,
            total_files: 5,
            files: vec![],
        };
        let data = LoadingData {
            url: None,
            title: Some("MyTorrentDetails".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: None,
            torrent_file: None,
        };
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let torrent_manager =
            Arc::new(Box::new(MockTorrentManager::new()) as Box<dyn TorrentManager>);
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = TorrentDetailsLoadingStrategy::new(event_publisher.clone(), torrent_manager);

        event_publisher.register(
            Box::new(move |event| {
                tx.send(event).unwrap();
                None
            }),
            DEFAULT_ORDER,
        );

        let result = runtime.block_on(strategy.process(data, &*context));
        assert_eq!(LoadingResult::Completed, result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        if let Event::TorrentDetailsLoaded(result) = result {
            assert_eq!(torrent_info, result);
        } else {
            assert!(
                false,
                "expected Event::TorrentDetailsLoaded, but got {:?} instead",
                result
            )
        }
    }

    #[test]
    fn test_cancel() {
        init_logger!();
        let torrent_handle = TorrentHandle::new();
        let data = LoadingData {
            url: None,
            title: Some("MyTorrentDetails".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: None,
            torrent_file: None,
        };
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_remove()
            .times(1)
            .returning(move |handle| tx.send(handle.clone()).unwrap());
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = TorrentDetailsLoadingStrategy::new(
            event_publisher,
            Arc::new(Box::new(torrent_manager)),
        );

        let result = runtime.block_on(strategy.cancel(data.clone()));

        if let Ok(_) = result {
            let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
            assert_eq!(torrent_handle, result);
        } else {
            assert!(false, "expected Ok, got {:?} instead", result);
        }
    }
}
