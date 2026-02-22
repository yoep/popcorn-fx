use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingProgress, LoadingResult,
    LoadingState, LoadingStrategy, Result,
};
use crate::core::stream;
use crate::core::stream::{StreamEvent, StreamServer, StreamState};
use crate::core::torrents::{TorrentManager, TorrentStreamingResource};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, info, trace};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::select;

#[derive(Display)]
#[display("Torrent stream loading strategy")]
pub struct TorrentStreamLoadingStrategy {
    torrent_manager: Arc<dyn TorrentManager>,
    stream_server: Arc<StreamServer>,
}

impl TorrentStreamLoadingStrategy {
    pub fn new(torrent_manager: Arc<dyn TorrentManager>, stream_server: Arc<StreamServer>) -> Self {
        Self {
            torrent_manager,
            stream_server,
        }
    }

    /// Handle the given stream event.
    /// This function checks if the stream is ready to be started based on the received torrent stream event.
    ///
    /// It returns `Ok(true)` when the stream is ready to start, else `Ok(false)` or the error that occurred.
    async fn handle_event(
        &self,
        event: &StreamEvent,
        context: &LoadingTaskContext,
    ) -> Result<bool> {
        match &*event {
            StreamEvent::StateChanged(state) => match state {
                StreamState::Preparing => {
                    debug!("Waiting for the torrent stream to be ready");
                    Ok(false)
                }
                StreamState::Streaming => {
                    debug!("Torrent stream is ready");
                    Ok(true)
                }
                StreamState::Stopped => Err(LoadingError::StreamError(stream::Error::InvalidState)),
            },
            StreamEvent::StatsChanged(status) => {
                context.send_event(LoadingEvent::ProgressChanged(LoadingProgress::from(
                    status.clone(),
                )));
                Ok(false)
            }
        }
    }
}

impl Debug for TorrentStreamLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentStreamLoadingStrategy")
            .field("torrent_stream_server", &self.stream_server)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentStreamLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        let filename = match data.filename.as_ref() {
            Some(filename) => filename,
            None => return LoadingResult::Ok,
        };
        let torrent = match data.torrent.take() {
            Some(torrent) => torrent,
            _ => {
                return LoadingResult::Err(LoadingError::InvalidData(
                    "expected a torrent to be present".to_string(),
                ))
            }
        };
        let torrent_handle = torrent.handle();

        trace!("Processing torrent stream for {:?}", torrent);
        context.send_event(LoadingEvent::StateChanged(LoadingState::Starting));
        let resource =
            TorrentStreamingResource::new(filename, torrent, self.torrent_manager.clone()).await;
        match self.stream_server.start_stream(resource).await {
            Ok(stream) => {
                trace!("Updating playlist item url to stream {}", stream.url());
                data.url = Some(stream.url().to_string());
                context.send_event(LoadingEvent::StateChanged(LoadingState::Downloading));

                let mut receiver = match self
                    .stream_server
                    .subscribe_stream(stream.filename.as_str())
                    .await
                {
                    Err(e) => return LoadingResult::Err(LoadingError::StreamError(e)),
                    Ok(e) => e,
                };
                loop {
                    select! {
                        _ = context.cancelled() => break,
                        event = receiver.recv() => {
                            if let Some(event) = event {
                                match self.handle_event(&*event, context).await {
                                    Ok(ready) => {
                                        if ready {
                                            break;
                                        }
                                    },
                                    Err(e) => return LoadingResult::Err(e),
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }

                info!("Streaming {}", stream.url());
                data.stream = Some(stream)
            }
            Err(e) => {
                self.torrent_manager.remove(&torrent_handle).await;
                return LoadingResult::Err(LoadingError::StreamError(e));
            }
        }

        LoadingResult::Ok
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        if let Some(stream) = data.stream.take() {
            trace!("Cancelling torrent stream for {:?}", stream);
            self.stream_server
                .stop_stream(stream.filename.as_str())
                .await;
            debug!("Stream {} loading has been cancelled", stream.filename);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stream::tests::MockStreamingResource;
    use crate::core::stream::StreamServerEvent;
    use crate::core::torrents::{MockTorrent, MockTorrentManager, TorrentHandle};
    use crate::{create_loading_data, create_loading_task, init_logger, recv_timeout};
    use fx_callback::Callback;
    use fx_torrent::TorrentFileInfo;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::oneshot;

    mod process {
        use super::*;
        use crate::core::torrents::MockTorrent;
        use crate::create_loading_data;
        use tokio::time::timeout;

        #[tokio::test]
        async fn test_torrent_file_unknown() {
            init_logger!();
            let mut data = create_loading_data!(Box::new(MockTorrent::new()));
            let torrent_manager = MockTorrentManager::new();
            let stream_server = StreamServer::new().await.unwrap();
            let task = create_loading_task!();
            let context = task.context();
            let strategy = TorrentStreamLoadingStrategy::new(
                Arc::new(torrent_manager),
                Arc::new(stream_server),
            );

            // when no specific torrent file has been specified,
            // the torrent stream should never be started even when a torrent is present
            let result = strategy.process(&mut data, &*context).await;

            if let LoadingResult::Ok = result {
                assert!(
                    data.torrent.is_some(),
                    "expected the torrent data to be present"
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
        async fn test_start_stream_failed() {
            init_logger!();
            let filename = "MyTorrentFilename.mp4";
            let handle = TorrentHandle::new();
            let torrent = create_torrent(handle);
            let mut data = create_loading_data!(Box::new(torrent), filename);
            let (tx, rx) = oneshot::channel();
            let mut torrent_manager = MockTorrentManager::new();
            torrent_manager
                .expect_remove()
                .times(1)
                .return_once(move |handle| {
                    let _ = tx.send(*handle);
                });
            let stream_server = StreamServer::new().await.unwrap();
            let mut stream_resource = MockStreamingResource::new();
            stream_resource
                .expect_filename()
                .return_const(filename.to_string());
            let _ = stream_server.start_stream(stream_resource).await;
            let task = create_loading_task!();
            let context = task.context();
            let strategy = TorrentStreamLoadingStrategy::new(
                Arc::new(torrent_manager),
                Arc::new(stream_server),
            );

            // process the data, which should fail on the InvalidStreamState
            let result = strategy.process(&mut data, &*context).await;
            assert_ne!(LoadingResult::Ok, result);

            let result = timeout(Duration::from_millis(200), rx)
                .await
                .expect("expected the remove fn to have been called")
                .unwrap();
            assert_eq!(handle, result, "expected the torrent handle to match");
        }
    }

    #[tokio::test]
    async fn test_cancel() {
        init_logger!();
        let handle = TorrentHandle::new();
        let filename = "MyTorrentFilename.mp4";
        let torrent_manager = MockTorrentManager::new();
        let stream_server = Arc::new(StreamServer::new().await.unwrap());
        let strategy =
            TorrentStreamLoadingStrategy::new(Arc::new(torrent_manager), stream_server.clone());

        // subscribe to the stream server events
        let mut receiver = stream_server.subscribe();

        // start a new stream
        let mut resource = MockStreamingResource::new();
        resource
            .expect_filename()
            .return_const(filename.to_string());
        resource.expect_stop().times(1).return_const(());
        let stream = stream_server
            .start_stream(resource)
            .await
            .expect("expected the stream to start");

        // wait for the stream to be started
        let result = recv_timeout!(&mut receiver, Duration::from_millis(200));
        match &*result {
            StreamServerEvent::StreamStarted(result) => {
                assert_eq!(
                    filename,
                    result.filename.as_str(),
                    "expected the stream to have been started"
                );
            }
            _ => assert!(
                false,
                "expected StreamServerEvent::StreamStarted, but got {:?}",
                result
            ),
        }

        // create the loading data with the returned stream
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(handle);
        let mut data = create_loading_data!(Box::new(torrent), filename, stream);

        // cancel the loading data
        let _ = strategy
            .cancel(&mut data)
            .await
            .expect("expected the cancellation to succeed");
        if let Some(torrent) = data.torrent {
            assert_eq!(
                handle,
                torrent.handle(),
                "expected the stream to have been set as torrent for next cancellation"
            );
        } else {
            assert!(
                false,
                "expected TorrentData::Torrent, but got {:?} instead",
                data.torrent
            );
        }

        let result = recv_timeout!(&mut receiver, Duration::from_millis(200));
        match &*result {
            StreamServerEvent::StreamStopped(result) => {
                assert_eq!(
                    filename,
                    result.as_str(),
                    "expected the stream of \"{}\" to be stopped",
                    filename
                );
            }
            _ => assert!(
                false,
                "expected StreamServerEvent::StreamStopped, but got {:?}",
                result
            ),
        }
    }

    fn create_torrent(handle: TorrentHandle) -> MockTorrent {
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(handle);
        torrent.expect_file_by_name().returning(|_| {
            Some(fx_torrent::File {
                index: 0,
                torrent_path: Default::default(),
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
                pieces: Default::default(),
            })
        });
        torrent.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        torrent
    }
}
