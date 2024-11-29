use crate::torrents::torrent_request_buffer::PendingRequest;
use crate::torrents::{InnerTorrent, RequestStrategy, TorrentFlags};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Display)]
#[display(fmt = "availability request strategy")]
pub struct RequestAvailabilityStrategy {}

impl RequestAvailabilityStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RequestStrategy for RequestAvailabilityStrategy {
    async fn supports(&self, torrent: &InnerTorrent) -> bool {
        !torrent
            .options()
            .await
            .contains(TorrentFlags::SequentialDownload)
    }

    async fn order(
        &self,
        pending_requests: Vec<PendingRequest>,
        torrent: &InnerTorrent,
    ) -> Vec<PendingRequest> {
        let pieces = torrent.pieces_read_lock().await;

        pending_requests
            .into_iter()
            .sorted_by(|a, b| {
                if let Some(a) = pieces.iter().find(|e| e.index == a.piece()) {
                    if let Some(b) = pieces.iter().find(|e| e.index == b.piece()) {
                        if a.priority == b.priority {
                            return a.availability().cmp(&b.availability());
                        }
                    }
                }

                Ordering::Equal
            })
            .collect()
    }
}
