use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::select;

use crate::core::callback::Callback;
use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingProgress, LoadingResult,
    LoadingState, LoadingStrategy, Result,
};
use crate::core::torrents::{Error, TorrentStreamEvent, TorrentStreamServer, TorrentStreamState};

#[derive(Display)]
#[display(fmt = "Torrent stream loading strategy")]
pub struct TorrentStreamLoadingStrategy {
    torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
}

impl TorrentStreamLoadingStrategy {
    pub fn new(torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>) -> Self {
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
    async fn process(&self, mut data: LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        if let Some(torrent) = data.torrent.take() {
            trace!("Processing torrent stream for {:?}", torrent);
            context.send_event(LoadingEvent::StateChanged(LoadingState::Starting));
            match self.torrent_stream_server.start_stream(torrent) {
                Ok(stream) => {
                    if let Some(stream) = stream.upgrade() {
                        trace!("Updating playlist item url to stream {}", stream.url());
                        data.url = Some(stream.url().to_string());
                        context.send_event(LoadingEvent::StateChanged(LoadingState::Downloading));

                        let mut stream_receiver =
                            Callback::<TorrentStreamEvent>::subscribe(&**stream);
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
                    } else {
                        warn!(
                            "Unable to update playlist item url, stream has already been dropped"
                        );
                    }

                    data.torrent_stream = Some(stream);
                }
                Err(e) => return LoadingResult::Err(LoadingError::TorrentError(e)),
            }
        }

        LoadingResult::Ok(data)
    }

    async fn cancel(&self, mut data: LoadingData) -> CancellationResult {
        if let Some(stream) = data.torrent_stream.take().and_then(|e| e.upgrade()) {
            debug!(
                "Cancelling torrent download & stream for {}",
                stream.stream_handle()
            );

            self.torrent_stream_server
                .stop_stream(stream.stream_handle());
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::playlist::PlaylistItem;
    use crate::core::torrents::{MockTorrentStreamServer, TorrentHandle, TorrentStream};
    use crate::core::Handle;
    use crate::testing::MockTorrentStream;
    use crate::{create_loading_task, init_logger};

    use super::*;

    #[test]
    fn test_cancel() {
        init_logger!();
        let handle = TorrentHandle::new();
        let stream_handle = Handle::new();
        let mut data = LoadingData::from(PlaylistItem {
            url: None,
            title: "MyStream".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let mut stream = MockTorrentStream::new();
        stream.inner.expect_handle().return_const(handle);
        stream
            .inner
            .expect_stream_handle()
            .return_const(stream_handle.clone());
        let stream = Arc::new(Box::new(stream) as Box<dyn TorrentStream>);
        data.torrent_stream = Some(Arc::downgrade(&stream));
        let (tx, rx) = channel();
        let mut stream_server = MockTorrentStreamServer::new();
        stream_server
            .expect_stop_stream()
            .times(1)
            .returning(move |e| {
                tx.send(e).unwrap();
            });
        let task = create_loading_task!();
        let context = task.context();
        let runtime = context.runtime();
        let strategy = TorrentStreamLoadingStrategy {
            torrent_stream_server: Arc::new(Box::new(stream_server) as Box<dyn TorrentStreamServer>),
        };

        let result = runtime.block_on(strategy.cancel(data));
        if let Ok(result) = result {
            assert!(
                result.torrent_stream.is_none(),
                "expected the torrent_stream data to have been dropped"
            );
        } else {
            assert!(
                false,
                "expected CancellationResult::Ok, got {:?} instead",
                result
            )
        }

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(stream_handle, result);
    }
}
