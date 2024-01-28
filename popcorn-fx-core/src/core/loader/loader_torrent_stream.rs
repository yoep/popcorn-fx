use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::mpsc::channel;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
use tokio::sync::Mutex;

use crate::core::block_in_place;
use crate::core::loader::{LoadingError, LoadingResult, LoadingStrategy, UpdateState};
use crate::core::playlists::PlaylistItem;
use crate::core::torrents::{TorrentError, TorrentStreamEvent, TorrentStreamServer, TorrentStreamState};

#[derive(Display)]
#[display(fmt = "Torrent stream loading strategy")]
pub struct TorrentStreamLoadingStrategy {
    state_update: Mutex<UpdateState>,
    torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
}

impl TorrentStreamLoadingStrategy {
    pub fn new(torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>) -> Self {
        Self {
            state_update: Mutex::new(Box::new(|_| warn!("state_update has not been configured"))),
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
    fn on_state_update(&self, state_update: UpdateState) {
        let mut state = block_in_place(self.state_update.lock());
        *state = state_update;
    }

    async fn process(&self, mut item: PlaylistItem) -> LoadingResult {
        if let Some(torrent) = item.torrent.take() {
            trace!("Processing torrent stream for {:?}", torrent);
            match self.torrent_stream_server.start_stream(torrent) {
                Ok(stream) => {
                    if let Some(stream) = stream.upgrade() {
                        let (tx, rx) = channel();
                        trace!("Updating playlist item url to stream {}", stream.url());
                        item.url = Some(stream.url().to_string());

                        stream.subscribe(Box::new(move |event| {
                            if let TorrentStreamEvent::StateChanged(state) = event {
                                match state {
                                    TorrentStreamState::Preparing => debug!("Waiting for the torrent stream to be ready"),
                                    TorrentStreamState::Streaming => {
                                        debug!("Torrent stream is ready");
                                        tx.send(Ok(())).unwrap();
                                    }
                                    TorrentStreamState::Stopped => tx.send(Err(LoadingError::TorrentError(TorrentError::InvalidStreamState(state)))).unwrap(),
                                }
                            }
                        }));
                        match rx.recv() {
                            Ok(_) => trace!("Received stream ready signal"),
                            Err(e) => return LoadingResult::Err(LoadingError::TimeoutError(e.to_string())),
                        }
                    } else {
                        warn!("Unable to update playlist item url, stream has already been dropped");
                    }

                    item.torrent_stream = Some(stream);
                }
                Err(e) => return LoadingResult::Err(LoadingError::TorrentError(e)),
            }
        }

        LoadingResult::Ok(item)
    }
}