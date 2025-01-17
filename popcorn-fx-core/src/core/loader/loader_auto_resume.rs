use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingResult, LoadingStrategy,
};
use crate::core::media::resume::AutoResumeService;

/// Represents a strategy for loading auto resume timestamps.
#[derive(Display)]
#[display(fmt = "Auto resume timestamp loading strategy")]
pub struct AutoResumeLoadingStrategy {
    auto_resume: Arc<Box<dyn AutoResumeService>>,
}

impl AutoResumeLoadingStrategy {
    /// Creates a new `AutoResumeLoadingStrategy` instance.
    ///
    /// # Arguments
    ///
    /// * `auto_resume` - An `Arc` pointer to a `AutoResumeService` trait object.
    ///
    /// # Returns
    ///
    /// A new `AutoResumeLoadingStrategy` instance.
    pub fn new(auto_resume: Arc<Box<dyn AutoResumeService>>) -> Self {
        Self { auto_resume }
    }

    /// Normalizes the given value by trimming and converting it to lowercase.
    fn normalize(value: &str) -> String {
        value.trim().to_lowercase()
    }
}

impl Debug for AutoResumeLoadingStrategy {
    /// Formats the `AutoResumeLoadingStrategy` for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// A result containing the formatted output.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoResumeLoadingStrategy")
            .field("auto_resume", &self.auto_resume)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for AutoResumeLoadingStrategy {
    async fn process(&self, mut data: LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        trace!("Processing auto resume timestamp for {:?}", data);
        let mut id: Option<&str> = None;
        let mut filename: Option<String> = None;

        // try to get the filename from the torrent
        if let Some(torrent) = data.torrent.as_ref() {
            if let Some(torrent_filename) = data.torrent_file.as_ref() {
                let files = torrent.files().await;
                filename = files
                    .into_iter()
                    .find(|e| {
                        Self::normalize(e.filename.as_str()) == Self::normalize(torrent_filename)
                    })
                    .map(|e| e.filename);
            } else {
                // get the largest files from the torrent
                filename = torrent
                    .largest_file()
                    .await
                    .map(|e| e.filename().to_string());
            }
        }

        if context.is_cancelled() {
            return LoadingResult::Err(LoadingError::Cancelled);
        }
        if let Some(media) = data.media.as_ref() {
            debug!(
                "Using media id {} for retrieving auto resume timestamp",
                media.imdb_id()
            );
            id = Some(media.imdb_id());
        }

        if context.is_cancelled() {
            return LoadingResult::Err(LoadingError::Cancelled);
        }
        trace!(
            "Retrieving auto resume timestamp for id: {:?}, filename: {:?}",
            id,
            filename
        );
        if let Some(timestamp) = self
            .auto_resume
            .resume_timestamp(id.map(|e| e.to_string()), filename)
        {
            debug!("Using auto resume timestamp {} for {:?}", timestamp, data);
            data.auto_resume_timestamp = Some(timestamp)
        } else {
            debug!("No auto resume timestamp could be found for {:?}", data);
        }

        LoadingResult::Ok(data)
    }

    async fn cancel(&self, mut data: LoadingData) -> CancellationResult {
        let _ = data.auto_resume_timestamp.take();
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use super::*;
    use crate::core::media::resume::MockAutoResumeService;
    use crate::core::media::MovieOverview;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};
    use crate::core::torrents::TorrentFileInfo;
    use crate::{create_loading_task, init_logger};

    #[test]
    fn test_process() {
        init_logger!();
        let imdb_id = "tt100200";
        let timestamp = 65000u64;
        let filename = "MyFilename.mp4";
        let movie = MovieOverview {
            imdb_id: imdb_id.to_string(),
            title: "FooBar".to_string(),
            year: "".to_string(),
            rating: None,
            images: Default::default(),
        };
        let item = PlaylistItem {
            url: Some("http://localhost:8080/MyVideo.mp4".to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(movie)),
            },
            quality: None,
            auto_resume_timestamp: Some(24000),
            subtitle: PlaylistSubtitle {
                enabled: false,
                info: None,
            },
            torrent: PlaylistTorrent {
                info: None,
                file_info: Some(TorrentFileInfo {
                    filename: filename.to_string(),
                    file_path: "".to_string(),
                    file_size: 1254788,
                    file_index: 0,
                }),
            },
        };
        let data = LoadingData::from(item);
        let (tx, rx) = channel();
        let (tx_filename, rx_filename) = channel();
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let mut auto_resume = MockAutoResumeService::new();
        auto_resume
            .expect_resume_timestamp()
            .times(1)
            .returning(move |id, filename| {
                tx.send(id.map(|e| e.to_string())).unwrap();
                tx_filename.send(filename.map(|e| e.to_string())).unwrap();
                Some(timestamp)
            });
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(
            Box::new(auto_resume) as Box<dyn AutoResumeService>
        ));

        let result = runtime.block_on(strategy.process(data, &*context));

        if let LoadingResult::Ok(result) = result {
            assert_eq!(Some(timestamp), result.auto_resume_timestamp);

            let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
            assert_eq!(
                Some(imdb_id.to_string()),
                result,
                "expected the media id to have been given"
            );

            let result = rx_filename
                .recv_timeout(Duration::from_millis(200))
                .unwrap();
            assert_eq!(
                Some(filename.to_string()),
                result,
                "expected torrent file info to have been given"
            );
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            );
        }
    }

    #[test]
    fn test_process_no_media() {
        init_logger!();
        let timestamp = 86000u64;
        let filename = "FooBar.mp4";
        let item = PlaylistItem {
            url: Some("http://localhost:8080/MyVideo.mp4".to_string()),
            title: "FooBar".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia::default(),
            quality: None,
            auto_resume_timestamp: Some(24000),
            subtitle: PlaylistSubtitle {
                enabled: false,
                info: None,
            },
            torrent: PlaylistTorrent {
                info: None,
                file_info: Some(TorrentFileInfo {
                    filename: filename.to_string(),
                    file_path: "".to_string(),
                    file_size: 1254788,
                    file_index: 0,
                }),
            },
        };
        let data = LoadingData::from(item);
        let (tx, rx) = channel();
        let (tx_filename, rx_filename) = channel();
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let mut auto_resume = MockAutoResumeService::new();
        auto_resume
            .expect_resume_timestamp()
            .times(1)
            .returning(move |id, filename| {
                tx.send(id.map(|e| e.to_string())).unwrap();
                tx_filename.send(filename.map(|e| e.to_string())).unwrap();
                Some(timestamp)
            });
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(
            Box::new(auto_resume) as Box<dyn AutoResumeService>
        ));

        let result = runtime.block_on(strategy.process(data, &*context));

        if let LoadingResult::Ok(result) = result {
            assert_eq!(Some(timestamp), result.auto_resume_timestamp);

            let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
            assert_eq!(None, result, "expected no media id to have been given");

            let result = rx_filename
                .recv_timeout(Duration::from_millis(200))
                .unwrap();
            assert_eq!(
                Some(filename.to_string()),
                result,
                "expected torrent file info to have been given"
            );
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            );
        }
    }

    #[test]
    fn test_cancel() {
        init_logger!();
        let url = "http://localhost:8520/video.mp4";
        let title = "LoremIpsumDolor";
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia::default(),
            quality: None,
            auto_resume_timestamp: Some(24000),
            subtitle: PlaylistSubtitle {
                enabled: false,
                info: None,
            },
            torrent: PlaylistTorrent {
                info: None,
                file_info: None,
            },
        };
        let mut data = LoadingData::from(item);
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let auto_resume = MockAutoResumeService::new();
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(
            Box::new(auto_resume) as Box<dyn AutoResumeService>
        ));

        let result = runtime.block_on(strategy.cancel(data.clone()));
        data.auto_resume_timestamp = None;

        assert_eq!(Ok(data), result);
    }
}
