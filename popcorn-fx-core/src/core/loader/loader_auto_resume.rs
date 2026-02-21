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
#[display("Auto resume timestamp loading strategy")]
pub struct AutoResumeLoadingStrategy {
    auto_resume: Arc<dyn AutoResumeService>,
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
    pub fn new(auto_resume: Arc<dyn AutoResumeService>) -> Self {
        Self { auto_resume }
    }

    /// Normalizes the given value by trimming and converting it to lowercase.
    fn normalize<S: AsRef<str>>(value: S) -> String {
        value.as_ref().trim().to_lowercase()
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
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        trace!("Processing auto resume timestamp for {:?}", data);
        let mut id: Option<&str> = None;
        let mut filename: Option<String> = None;

        // try to get the filename from the torrent
        if let Some(torrent) = data.torrent.as_ref() {
            if let Some(torrent_filename) = data.torrent_file.as_ref() {
                let files = torrent.files().await;
                filename = files
                    .into_iter()
                    .find(|e| Self::normalize(e.filename()) == Self::normalize(torrent_filename))
                    .map(|e| e.filename());
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
            .await
        {
            debug!("Using auto resume timestamp {} for {:?}", timestamp, data);
            data.auto_resume_timestamp = Some(timestamp)
        } else {
            debug!("No auto resume timestamp could be found for {:?}", data);
        }

        LoadingResult::Ok
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        let _ = data.auto_resume_timestamp.take();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::media::resume::MockAutoResumeService;
    use crate::core::media::MovieOverview;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};
    use crate::core::torrents::MockTorrent;
    use crate::{create_loading_task, init_logger, recv_timeout};

    use fx_torrent;
    use fx_torrent::TorrentFileInfo;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process() {
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
                filename: Some(filename.to_string()),
            },
        };
        let mut torrent = MockTorrent::new();
        torrent
            .expect_files()
            .returning(|| vec![create_file(filename)]);
        let mut data = LoadingData::from(item);
        data.torrent = Some(Box::new(torrent));
        let (tx, mut rx) = unbounded_channel();
        let (tx_filename, mut rx_filename) = unbounded_channel();
        let task = create_loading_task!();
        let context = task.context();
        let mut auto_resume = MockAutoResumeService::new();
        auto_resume
            .expect_resume_timestamp()
            .times(1)
            .returning(move |id, filename| {
                tx.send(id.map(|e| e.to_string())).unwrap();
                tx_filename.send(filename.map(|e| e.to_string())).unwrap();
                Some(timestamp)
            });
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(auto_resume));

        let result = strategy.process(&mut data, &*context).await;

        if let LoadingResult::Ok = result {
            assert_eq!(Some(timestamp), data.auto_resume_timestamp);

            let result = recv_timeout!(&mut rx, Duration::from_millis(200));
            assert_eq!(
                Some(imdb_id.to_string()),
                result,
                "expected the media id to have been given"
            );

            let result = recv_timeout!(&mut rx_filename, Duration::from_millis(200));
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

    #[tokio::test]
    async fn test_process_no_media() {
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
                filename: Some(filename.to_string()),
            },
        };
        let mut torrent = MockTorrent::new();
        torrent
            .expect_files()
            .returning(|| vec![create_file(filename)]);
        let mut data = LoadingData::from(item);
        data.torrent = Some(Box::new(torrent));
        let (tx, mut rx) = unbounded_channel();
        let (tx_filename, mut rx_filename) = unbounded_channel();
        let task = create_loading_task!();
        let context = task.context();
        let mut auto_resume = MockAutoResumeService::new();
        auto_resume
            .expect_resume_timestamp()
            .times(1)
            .returning(move |id, filename| {
                tx.send(id.map(|e| e.to_string())).unwrap();
                tx_filename.send(filename.map(|e| e.to_string())).unwrap();
                Some(timestamp)
            });
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(auto_resume));

        let result = strategy.process(&mut data, &*context).await;

        if let LoadingResult::Ok = result {
            assert_eq!(Some(timestamp), data.auto_resume_timestamp);

            let result = recv_timeout!(&mut rx, Duration::from_millis(200));
            assert_eq!(None, result, "expected no media id to have been given");

            let result = recv_timeout!(&mut rx_filename, Duration::from_millis(200));
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

    #[tokio::test]
    async fn test_cancel() {
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
            torrent: PlaylistTorrent { filename: None },
        };
        let mut data = LoadingData::from(item);
        let auto_resume = MockAutoResumeService::new();
        let strategy = AutoResumeLoadingStrategy::new(Arc::new(auto_resume));

        let _ = strategy
            .cancel(&mut data)
            .await
            .expect("expected the cancellation to succeed");

        assert_eq!(
            None, data.auto_resume_timestamp,
            "expected the auto resume timestamp to be cleared"
        );
    }

    fn create_file(filename: &str) -> fx_torrent::File {
        fx_torrent::File {
            index: 0,
            torrent_path: PathBuf::from(filename),
            torrent_offset: 0,
            info: TorrentFileInfo {
                length: 0,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
            pieces: 0..100,
        }
    }
}
