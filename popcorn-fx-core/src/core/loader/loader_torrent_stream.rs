use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio_util::sync::CancellationToken;

use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingProgress, LoadingResult,
    LoadingState, LoadingStrategy,
};
use crate::core::torrents::{
    TorrentError, TorrentStreamEvent, TorrentStreamServer, TorrentStreamState,
};

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
    async fn process(
        &self,
        mut data: LoadingData,
        event_channel: Sender<LoadingEvent>,
        cancel_token: CancellationToken,
    ) -> LoadingResult {
        if let Some(torrent) = data.torrent.take() {
            trace!("Processing torrent stream for {:?}", torrent);
            event_channel
                .send(LoadingEvent::StateChanged(LoadingState::Starting))
                .unwrap();
            match self.torrent_stream_server.start_stream(torrent) {
                Ok(stream) => {
                    if let Some(stream) = stream.upgrade() {
                        let (tx, rx) = channel();
                        trace!("Updating playlist item url to stream {}", stream.url());
                        data.url = Some(stream.url().to_string());
                        event_channel
                            .send(LoadingEvent::StateChanged(LoadingState::Downloading))
                            .unwrap();

                        let event_channel_stream = event_channel.clone();
                        let callback_id = stream.subscribe_stream(Box::new(move |event| {
                            if cancel_token.is_cancelled() {
                                debug!("Cancelling the torrent stream loading process");
                                tx.send(Ok(())).unwrap();
                            }

                            match event {
                                TorrentStreamEvent::StateChanged(state) => match state {
                                    TorrentStreamState::Preparing => {
                                        debug!("Waiting for the torrent stream to be ready")
                                    }
                                    TorrentStreamState::Streaming => {
                                        debug!("Torrent stream is ready");
                                        tx.send(Ok(())).unwrap();
                                    }
                                    TorrentStreamState::Stopped => tx
                                        .send(Err(LoadingError::TorrentError(
                                            TorrentError::InvalidStreamState(state),
                                        )))
                                        .unwrap(),
                                },
                                TorrentStreamEvent::DownloadStatus(status) => {
                                    event_channel_stream
                                        .send(LoadingEvent::ProgressChanged(LoadingProgress::from(
                                            status,
                                        )))
                                        .unwrap();
                                }
                            }
                        }));
                        match rx.recv() {
                            Ok(_) => {
                                event_channel
                                    .send(LoadingEvent::StateChanged(
                                        LoadingState::DownloadFinished,
                                    ))
                                    .unwrap();
                                stream.unsubscribe_stream(callback_id);
                                trace!("Received stream ready signal");
                            }
                            Err(e) => {
                                return LoadingResult::Err(LoadingError::TimeoutError(
                                    e.to_string(),
                                ))
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

            if let Some(stream) =
                stream.downcast_ref::<crate::core::torrents::stream::DefaultTorrentStream>()
            {
                data.torrent = Some(Arc::downgrade(&stream.torrent()));
            }
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::core::{block_in_place, Handle};
    use crate::core::playlists::PlaylistItem;
    use crate::core::torrents::{MockTorrentStreamServer, TorrentStream};
    use crate::testing::{init_logger, MockTorrentStream};

    use super::*;

    #[test]
    fn test_cancel() {
        init_logger();
        let handle = "MyTorrentHandle";
        let stream_handle = Handle::new();
        let mut data = LoadingData::from(PlaylistItem {
            url: None,
            title: "MyStream".to_string(),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        let mut stream = MockTorrentStream::new();
        stream.expect_handle().return_const(handle.to_string());
        stream
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
        let strategy = TorrentStreamLoadingStrategy {
            torrent_stream_server: Arc::new(Box::new(stream_server) as Box<dyn TorrentStreamServer>),
        };

        let result = block_in_place(strategy.cancel(data));
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
