use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingResult, LoadingStrategy,
};
use crate::core::stream::{FileStreamingResource, StreamServer, StreamingResource};
use async_trait::async_trait;
use derive_more::Display;
use log::trace;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Display)]
#[display("File loading strategy")]
pub struct FileLoadingStrategy {
    stream_server: Arc<StreamServer>,
}

impl FileLoadingStrategy {
    /// Create a new file loading strategy.
    pub fn new(stream_server: Arc<StreamServer>) -> Self {
        Self { stream_server }
    }

    /// Returns `true` if the given path is a file that can be reached on the system or network.
    fn is_video_file<S: AsRef<Path>>(url: S) -> bool {
        let path = url.as_ref();
        path.exists() && path.is_file()
    }
}

#[async_trait]
impl LoadingStrategy for FileLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, _: &LoadingTaskContext) -> LoadingResult {
        let url = match data.url.as_ref() {
            None => return LoadingResult::Ok,
            Some(url) => url,
        };
        let path = match PathBuf::from_str(url.as_str()) {
            Err(_) => return LoadingResult::Ok,
            Ok(path) => path,
        };

        if !Self::is_video_file(&path) {
            trace!("Url \"{}\" is not a valid video file", url);
            return LoadingResult::Ok;
        }

        let resource = match FileStreamingResource::new(&path) {
            Err(e) => return LoadingResult::Err(LoadingError::StreamError(e)),
            Ok(e) => e,
        };
        let filename = resource.filename().to_string();

        match self.stream_server.start_stream(resource).await {
            Err(e) => LoadingResult::Err(LoadingError::StreamError(e)),
            Ok(stream) => {
                // replace the url with the new server stream url
                data.url = Some(stream.url.to_string());
                data.filename = Some(filename);
                data.stream = Some(stream);
                LoadingResult::Ok
            }
        }
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        if let Some(filename) = data.filename.as_ref() {
            self.stream_server.stop_stream(filename).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_loading_task;
    use crate::init_logger;
    use crate::recv_timeout;
    use std::time::Duration;
    use tempfile::tempdir;

    mod process {
        use super::*;
        use crate::testing::copy_test_file;

        #[tokio::test]
        async fn test_http_url() {
            init_logger!();
            let url = "https://example.com/video.mp4";
            let mut data = create_loading_data(url);
            let task = create_loading_task!();
            let context = task.context();
            let server = StreamServer::new().await.unwrap();
            let strategy = FileLoadingStrategy::new(Arc::new(server));

            let result = strategy.process(&mut data, &*context).await;
            assert_eq!(LoadingResult::Ok, result);

            // verify that the url was not changed
            assert_eq!(
                url,
                data.url.unwrap().as_str(),
                "expected the url to not have been changed"
            );
        }

        #[tokio::test]
        async fn test_file() {
            init_logger!();
            let filename = "large-[123].txt";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let filepath = copy_test_file(temp_path, filename, None);
            let mut data = create_loading_data(filepath.as_str());
            let task = create_loading_task!();
            let context = task.context();
            let server = StreamServer::new().await.unwrap();
            let strategy = FileLoadingStrategy::new(Arc::new(server));

            let result = strategy.process(&mut data, &*context).await;
            assert_eq!(LoadingResult::Ok, result);

            // verify that the url was changed
            assert_ne!(
                filepath,
                data.url.unwrap().as_str(),
                "expected the url to have been changed"
            );
            assert_ne!(
                None, data.filename,
                "expected the filename to have been set"
            );
            assert_ne!(None, data.stream, "expected the stream to have been set");
        }
    }

    mod cancel {
        use super::*;
        use crate::core::stream::StreamServerEvent;
        use crate::testing::copy_test_file;
        use fx_callback::Callback;

        #[tokio::test]
        async fn test_cancel() {
            init_logger!();
            let filename = "large-[123].txt";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let filepath = copy_test_file(temp_path, filename, None);
            let mut data = create_loading_data(filepath.as_str());
            let task = create_loading_task!();
            let context = task.context();
            let server = StreamServer::new().await.unwrap();

            // subscribe to the stream server events
            let mut receiver = server.subscribe();

            // process the loading data
            let strategy = FileLoadingStrategy::new(Arc::new(server));
            let result = strategy.process(&mut data, &*context).await;
            assert_eq!(LoadingResult::Ok, result);

            let event = recv_timeout!(&mut receiver, Duration::from_millis(200));
            match &*event {
                StreamServerEvent::StreamStarted(stream) => {
                    assert_eq!(
                        filename,
                        stream.filename.as_str(),
                        "expected the stream to start"
                    );
                }
                _ => assert!(
                    false,
                    "expected StreamServerEvent::StreamStarted, but got {:?}",
                    event
                ),
            }

            // cancel the loader
            let result = strategy.cancel(&mut data).await;
            assert_eq!(Ok(()), result, "expected the cancellation to succeed");

            let event = recv_timeout!(&mut receiver, Duration::from_millis(200));
            match &*event {
                StreamServerEvent::StreamStopped(filename) => {
                    assert_eq!(filename, filename.as_str(), "expected the stream to stop");
                }
                _ => assert!(
                    false,
                    "expected StreamServerEvent::StreamStopped, but got {:?}",
                    event
                ),
            }
        }
    }

    fn create_loading_data(url: &str) -> LoadingData {
        LoadingData {
            url: Some(url.to_string()),
            title: None,
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: None,
            filename: None,
            stream: None,
        }
    }
}
