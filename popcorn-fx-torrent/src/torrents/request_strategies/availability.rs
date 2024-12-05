use crate::torrents::torrent_request_buffer::{PendingRequest, PendingRequestBuffer};
use crate::torrents::{
    PendingRequestContext, Piece, RequestStrategy, TorrentContext, TorrentFlags,
};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use log::trace;
use std::cmp::Ordering;
use tokio::sync::RwLockReadGuard;

#[derive(Debug, Display)]
#[display(fmt = "availability request strategy")]
pub struct RequestAvailabilityStrategy {}

impl RequestAvailabilityStrategy {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the pending requests sorted by piece availability
    fn pending_requests_by_availability<'a>(
        buffer: &'a RwLockReadGuard<PendingRequestBuffer>,
        pieces: &RwLockReadGuard<Vec<Piece>>,
    ) -> Vec<&'a PendingRequest> {
        buffer
            .pending_requests()
            .into_iter()
            .filter(|e| {
                pieces
                    .get(e.piece())
                    .map(|piece| piece.availability > 0)
                    .unwrap_or(false)
            })
            .sorted_by(|a, b| {
                if let Some(a) = pieces.get(a.piece()) {
                    if let Some(b) = pieces.get(b.piece()) {
                        return b.availability.cmp(&a.availability);
                    }
                }

                Ordering::Equal
            })
            .collect()
    }
}

#[async_trait]
impl RequestStrategy for RequestAvailabilityStrategy {
    async fn supports(&self, torrent: &TorrentContext) -> bool {
        !torrent
            .options()
            .read()
            .await
            .contains(TorrentFlags::SequentialDownload)
    }

    async fn execute<'a>(&self, ctx: &'a PendingRequestContext<'a>, max_requests: usize) {
        let mut requested_pieces = 0;
        let pending_requests =
            Self::pending_requests_by_availability(&ctx.pending_requests_buffer, &ctx.pieces);
        if pending_requests.len() == 0 {
            return;
        }

        trace!(
            "Trying to request a total of {} available pieces",
            pending_requests.len()
        );
        for request in pending_requests {
            // try to find peers that have the piece available
            let available_peers = ctx.find_available_peers(&request.piece()).await;
            trace!(
                "Piece {} has {} available peers",
                request,
                available_peers.len()
            );
            if available_peers.len() == 0 {
                continue;
            }

            if ctx.accept(request, available_peers).await {
                requested_pieces += 1;

                // check if we're allowed to request more
                if requested_pieces >= max_requests {
                    break;
                }
            }
        }
    }

    fn clone_boxed(&self) -> Box<dyn RequestStrategy> {
        Box::new(Self::new())
    }
}
