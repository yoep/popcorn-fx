use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::channel;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::block_in_place;
use crate::core::loader::{LoadingData, LoadingError, LoadingProgress, LoadingResult, LoadingState, LoadingStrategy, UpdateProgress, UpdateState};
use crate::core::torrents::{TorrentError, TorrentStreamEvent, TorrentStreamServer, TorrentStreamState};

#[derive(Display)]
#[display(fmt = "Torrent stream loading strategy")]
pub struct TorrentStreamLoadingStrategy {
    state_updater: Mutex<UpdateState>,
    progress_updater: Mutex<Arc<UpdateProgress>>,
    torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
}

impl TorrentStreamLoadingStrategy {
    pub fn new(torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>) -> Self {
        Self {
            state_updater: Mutex::new(Box::new(|_| warn!("state_updater has not been configured"))),
            progress_updater: Mutex::new(Arc::new(Box::new(|_| warn!("progress_updater has not been configured")))),
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
    fn state_updater(&self, state_update: UpdateState) {
        let mut updater = block_in_place(self.state_updater.lock());
        *updater = state_update;
    }

    fn progress_updater(&self, progress_updater: UpdateProgress) {
        let mut updater = block_in_place(self.progress_updater.lock());
        *updater = Arc::new(progress_updater);
    }

    async fn process(&self, mut data: LoadingData) -> LoadingResult {
        if let Some(torrent) = data.torrent.take() {
            trace!("Processing torrent stream for {:?}", torrent);
            {
                let state_update = block_in_place(self.state_updater.lock());
                state_update(LoadingState::Starting);
            }
            match self.torrent_stream_server.start_stream(torrent) {
                Ok(stream) => {
                    if let Some(stream) = stream.upgrade() {
                        let (tx, rx) = channel();
                        let progress_updater: Arc<UpdateProgress>;
                        trace!("Updating playlist item url to stream {}", stream.url());
                        data.item.url = Some(stream.url().to_string());

                        {
                            let state_update = block_in_place(self.state_updater.lock());
                            state_update(LoadingState::Downloading);
                        }
                        {
                            let updater = block_in_place(self.progress_updater.lock());
                            progress_updater = updater.clone();
                        }

                        let callback_id = stream.subscribe_stream(Box::new(move |event| {
                            match event {
                                TorrentStreamEvent::StateChanged(state) => {
                                    match state {
                                        TorrentStreamState::Preparing => debug!("Waiting for the torrent stream to be ready"),
                                        TorrentStreamState::Streaming => {
                                            debug!("Torrent stream is ready");
                                            tx.send(Ok(())).unwrap();
                                        }
                                        TorrentStreamState::Stopped => tx.send(Err(LoadingError::TorrentError(TorrentError::InvalidStreamState(state)))).unwrap(),
                                    }
                                }
                                TorrentStreamEvent::DownloadStatus(status) => {
                                    progress_updater(LoadingProgress::from(status));
                                }
                            }
                        }));
                        match rx.recv() {
                            Ok(_) => {
                                {
                                    let state_update = block_in_place(self.state_updater.lock());
                                    state_update(LoadingState::DownloadFinished);
                                }
                                stream.unsubscribe_stream(callback_id);
                                trace!("Received stream ready signal");
                            }
                            Err(e) => return LoadingResult::Err(LoadingError::TimeoutError(e.to_string())),
                        }
                    } else {
                        warn!("Unable to update playlist item url, stream has already been dropped");
                    }

                    data.torrent_stream = Some(stream);
                }
                Err(e) => return LoadingResult::Err(LoadingError::TorrentError(e)),
            }
        }

        LoadingResult::Ok(data)
    }
}