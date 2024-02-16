use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use tokio_util::sync::CancellationToken;

use crate::core::events::{Event, EventPublisher};
use crate::core::loader::{CancellationResult, LoadingData, LoadingEvent, LoadingResult, LoadingStrategy};

/// Represents a loading strategy for handling torrent details.
///
/// This strategy is responsible for processing torrent-related data and publishing events when torrent details are loaded.
#[derive(Display)]
#[display(fmt = "Torrent details loading strategy")]
pub struct TorrentDetailsLoadingStrategy {
    event_publisher: Arc<EventPublisher>,
}

impl TorrentDetailsLoadingStrategy {
    /// Creates a new instance of `TorrentDetailsLoadingStrategy`.
    ///
    /// # Arguments
    ///
    /// * `event_publisher` - An `EventPublisher` for publishing events related to torrent details.
    pub fn new(event_publisher: Arc<EventPublisher>) -> Self {
        Self {
            event_publisher,
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
    async fn process(&self, data: LoadingData, _: Sender<LoadingEvent>, _: CancellationToken) -> LoadingResult {
        trace!("Processing torrent details strategy for {:?}", data);
        if let Some(torrent_info) = data.torrent_info.as_ref() {
            if let None = data.torrent_file_info.as_ref() {
                self.event_publisher.publish(Event::TorrentDetailsLoaded(torrent_info.clone()));
                return LoadingResult::Completed;
            } else {
                debug!("Torrent file info present, torrent details won't be shown");
            }
        } else {
            debug!("No torrent information present, torrent details won't be loaded");
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

    use crate::core::block_in_place;
    use crate::core::loader::loading_chain::DEFAULT_ORDER;
    use crate::core::torrents::TorrentInfo;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_process() {
        init_logger();
        let torrent_info = TorrentInfo {
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
            torrent_info: Some(torrent_info.clone()),
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: None,
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };
        let (tx, rx) = channel();
        let (tx_event, _) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let strategy = TorrentDetailsLoadingStrategy::new(event_publisher.clone());

        event_publisher.register(Box::new(move |event| {
            tx.send(event).unwrap();
            None
        }), DEFAULT_ORDER);

        let result = block_in_place(strategy.process(data, tx_event, CancellationToken::new()));
        assert_eq!(LoadingResult::Completed, result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        if let Event::TorrentDetailsLoaded(result) = result {
            assert_eq!(torrent_info, result);
        } else {
            assert!(false, "expected Event::TorrentDetailsLoaded, but got {:?} instead", result)
        }
    }

    #[test]
    fn test_cancel() {
        let data = LoadingData {
            url: None,
            title: Some("MyTorrentDetails".to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: None,
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };
        let event_publisher = Arc::new(EventPublisher::default());
        let strategy = TorrentDetailsLoadingStrategy::new(event_publisher);

        let result = block_in_place(strategy.cancel(data.clone()));

        assert_eq!(CancellationResult::Ok(data), result);
    }
}