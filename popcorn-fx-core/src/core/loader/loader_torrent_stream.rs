use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingProgress, LoadingResult,
    LoadingState, LoadingStrategy, Result, TorrentData,
};
use crate::core::torrents::stream::DefaultTorrentStream;
use crate::core::torrents::{
    Error, TorrentStream, TorrentStreamEvent, TorrentStreamServer, TorrentStreamState,
};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::Callback;
use log::{debug, info, trace};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::select;

#[derive(Display)]
#[display("Torrent stream loading strategy")]
pub struct TorrentStreamLoadingStrategy {
    torrent_stream_server: Arc<dyn TorrentStreamServer>,
}

impl TorrentStreamLoadingStrategy {
    pub fn new(torrent_stream_server: Arc<dyn TorrentStreamServer>) -> Self {
        Self {
            torrent_stream_server,
        }
    }

    /// Handle the given stream event.
    /// This function checks if the stream is ready to be started based on the received torrent strean event.
    ///
    /// It returns `Ok(true)` when the stream is ready to start, else `Ok(false)` or the error that occurred.
    async fn handle_stream_event(
        &self,
        event: Arc<TorrentStreamEvent>,
        context: &LoadingTaskContext,
    ) -> Result<bool> {
        match &*event {
            TorrentStreamEvent::StateChanged(state) => match state {
                TorrentStreamState::Preparing => {
                    debug!("Waiting for the torrent stream to be ready");
                    Ok(false)
                }
                TorrentStreamState::Streaming => {
                    debug!("Torrent stream is ready");
                    Ok(true)
                }
                TorrentStreamState::Stopped => Err(LoadingError::TorrentError(
                    Error::InvalidStreamState(*state),
                )),
            },
            TorrentStreamEvent::DownloadStatus(status) => {
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
            .field("torrent_stream_server", &self.torrent_stream_server)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentStreamLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        // check if the current loading data can be streamed as a torrent
        if let Some(filename) = data.torrent_file.as_ref() {
            if let Some(TorrentData::Torrent(torrent)) = data.torrent.take() {
                trace!("Processing torrent stream for {:?}", torrent);
                context.send_event(LoadingEvent::StateChanged(LoadingState::Starting));
                match self
                    .torrent_stream_server
                    .start_stream(torrent, filename)
                    .await
                {
                    Ok(stream) => {
                        trace!("Updating playlist item url to stream {}", stream.url());
                        data.url = Some(stream.url().to_string());
                        context.send_event(LoadingEvent::StateChanged(LoadingState::Downloading));

                        let mut stream_receiver =
                            Callback::<TorrentStreamEvent>::subscribe(&*stream);
                        loop {
                            select! {
                                _ = context.cancelled() => break,
                                event = stream_receiver.recv() => {
                                    if let Some(event) = event {
                                        match self.handle_stream_event(event, context).await {
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

                        match stream.downcast::<DefaultTorrentStream>() {
                            Ok(stream) => {
                                info!("Streaming {}", stream.url());
                                data.torrent = Some(TorrentData::Stream(stream))
                            }
                            Err(e) => {
                                return LoadingResult::Err(LoadingError::ParseError(format!(
                                    "expected DefaultTorrentStream, got {:?} instead",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => return LoadingResult::Err(LoadingError::TorrentError(e)),
                }
            }
        }

        LoadingResult::Ok
    }

    async fn cancel(&self, data: &mut LoadingData) -> CancellationResult {
        if let Some(TorrentData::Stream(stream)) = data.torrent.take() {
            let handle = stream.handle();
            trace!("Cancelling torrent download & stream for {}", handle);
            self.torrent_stream_server.stop_stream(handle).await;
            data.torrent = Some(TorrentData::Torrent(stream));
            debug!("Stream {} loading has been cancelled", handle);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::torrents::{MockTorrentStreamServer, TorrentHandle};
    use crate::testing::MockTorrentStream;
    use crate::{create_loading_task, init_logger, recv_timeout};

    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process_torrent_file_unknown() {
        init_logger!();
        let mut data = create_loading_data(MockTorrentStream::new());
        let mut stream_server = MockTorrentStreamServer::new();
        stream_server.expect_start_stream().times(0);
        let task = create_loading_task!();
        let context = task.context();
        let strategy = TorrentStreamLoadingStrategy {
            torrent_stream_server: Arc::new(stream_server),
        };

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
    async fn test_cancel() {
        init_logger!();
        let handle = TorrentHandle::new();
        let mut stream = MockTorrentStream::new();
        stream.inner.expect_handle().return_const(handle);
        stream.inner.expect_stream_handle().return_const(handle);
        let mut data = create_loading_data(stream);
        let (tx, mut rx) = unbounded_channel();
        let mut stream_server = MockTorrentStreamServer::new();
        stream_server
            .expect_stop_stream()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
            });
        let strategy = TorrentStreamLoadingStrategy {
            torrent_stream_server: Arc::new(stream_server),
        };

        let _ = strategy
            .cancel(&mut data)
            .await
            .expect("expected the cancellation to succeed");
        if let Some(TorrentData::Torrent(torrent)) = data.torrent {
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

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(handle, result);
    }

    fn create_loading_data(stream: MockTorrentStream) -> LoadingData {
        LoadingData {
            url: None,
            title: None,
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Some(TorrentData::Stream(Box::new(stream))),
            torrent_file: None,
        }
    }
}
