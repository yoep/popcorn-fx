use crate::torrents::torrent_request_buffer::PendingRequest;
use crate::torrents::{InnerTorrent, RequestStrategy};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Display)]
#[display(fmt = "priority request strategy")]
pub struct PriorityRequestStrategy {}

impl PriorityRequestStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RequestStrategy for PriorityRequestStrategy {
    async fn supports(&self, _: &InnerTorrent) -> bool {
        true
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
                if let Some(a_piece) = pieces.iter().find(|e| e.index == a.piece()) {
                    if let Some(b_piece) = pieces.iter().find(|e| e.index == b.piece()) {
                        return a_piece
                            .priority
                            .partial_cmp(&b_piece.priority)
                            .unwrap_or(Ordering::Equal);
                    }
                }

                Ordering::Equal
            })
            .collect()
    }
}
