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
use crate::core::torrents::{Torrent, TorrentManager};

/// Represents a loading strategy for handling torrent details.
///
/// This strategy is responsible for processing torrent-related data and publishing events when torrent details are loaded.
#[derive(Display)]
#[display(fmt = "Torrent details loading strategy")]
pub struct TorrentDetailsLoadingStrategy {
    event_publisher: EventPublisher,
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentDetailsLoadingStrategy {
    /// Creates a new instance of `TorrentDetailsLoadingStrategy`.
    ///
    /// # Arguments
    ///
    /// * `event_publisher` - An `EventPublisher` for publishing events related to torrent details.
    pub fn new(
        event_publisher: EventPublisher,
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
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        trace!("Processing torrent details strategy for {:?}", data);
        if let Some(torrent) = data.torrent.as_ref() {
            // check if a specific file has been set to be loaded
            // if not, stop the loading chain and show the retrieved details
            if let None = data.torrent_file.as_ref() {
                let handle = torrent.handle();
                return match self.torrent_manager.info(&handle).await {
                    Ok(torrent_info) => {
                        debug!(
                            "Loading task {} loaded torrent details, {:?}",
                            context, torrent_info
                        );
                        // remove the torrent from the manager
                        self.torrent_manager.remove(&handle).await;
                        // inform the event publisher about the torrent details
                        self.event_publisher
                            .publish(Event::TorrentDetailsLoaded(torrent_info));
                        // end the loading task
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

        LoadingResult::Ok
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        if data.torrent.is_some() {
            if let Some(torrent) = data.torrent.take() {
                let handle = torrent.handle();
                self.torrent_manager.remove(&handle).await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::core::loader::loading_chain::DEFAULT_ORDER;
    use crate::core::loader::{SubtitleData, TorrentData};
    use crate::core::torrents::{MockTorrent, MockTorrentManager, TorrentHandle, TorrentInfo};
    use crate::{create_loading_task, init_logger, recv_timeout};

    use super::*;

    #[tokio::test]
    async fn test_process() {
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
        let torrent_handle = TorrentHandle::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(torrent_handle);
        let mut data = LoadingData {
            url: None,
            title: Some("MyTorrentDetails".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: Some(TorrentData::Torrent(Box::new(torrent))),
            torrent_file: None,
        };
        let (tx, mut rx) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let mut torrent_manager = MockTorrentManager::new();
        let torrent_manager_handle = torrent_handle.clone();
        let torrent_manager_torrent_info = torrent_info.clone();
        torrent_manager.expect_remove().return_const(());
        torrent_manager
            .expect_info()
            .withf(move |e| *e == torrent_manager_handle)
            .times(1)
            .returning(move |_| Ok(torrent_manager_torrent_info.clone()));
        let task = create_loading_task!();
        let context = task.context();
        let strategy = TorrentDetailsLoadingStrategy::new(
            event_publisher.clone(),
            Arc::new(Box::new(torrent_manager)),
        );

        let mut callback = event_publisher.subscribe(DEFAULT_ORDER).unwrap();
        tokio::spawn(async move {
            if let Some(mut handler) = callback.recv().await {
                tx.send(handler.take().unwrap()).unwrap();
                handler.next();
            }
        });

        let result = strategy.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Completed, result);

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
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

    #[tokio::test]
    async fn test_cancel() {
        init_logger!();
        let torrent_handle = TorrentHandle::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(torrent_handle);
        let title = "MyTorrentDetails";
        let caption = "Lorem ipsum dolor";
        let mut data = LoadingData {
            url: None,
            title: Some(title.to_string()),
            caption: Some(caption.to_string()),
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: Some(TorrentData::Torrent(Box::new(torrent))),
            torrent_file: None,
        };
        let (tx, mut rx) = unbounded_channel();
        let event_publisher = EventPublisher::default();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_remove()
            .times(1)
            .returning(move |handle| tx.send(handle.clone()).unwrap());
        let strategy = TorrentDetailsLoadingStrategy::new(
            event_publisher,
            Arc::new(Box::new(torrent_manager)),
        );

        let _ = strategy
            .cancel(&mut data)
            .await
            .expect("expected the cancellation process to succeed");
        assert_eq!(
            Some(title.to_string()),
            data.title,
            "expected the title to be unmodified"
        );
        assert_eq!(
            Some(caption.to_string()),
            data.caption,
            "expected the caption to be unmodified"
        );

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        assert_eq!(torrent_handle, result);
    }
}
