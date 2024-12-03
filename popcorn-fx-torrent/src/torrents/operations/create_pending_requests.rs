use crate::torrents::torrent_request_buffer::PendingRequest;
use crate::torrents::{InnerTorrent, TorrentFlags, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};

#[derive(Debug, Display)]
#[display(fmt = "create pending requests operation")]
pub struct TorrentPendingRequestsOperation {}

impl TorrentPendingRequestsOperation {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_pending_requests(&self, torrent: &InnerTorrent) {
        let wanted_pieces = torrent.wanted_pieces().await;

        if wanted_pieces.len() > 0 {
            trace!("{} pieces are wanted for {}", wanted_pieces.len(), torrent);
            let requested_pieces = torrent.pending_requested_pieces().await;
            let mut pending_requests: Vec<PendingRequest> = wanted_pieces
                .into_iter()
                .filter(|e| !requested_pieces.contains(&e.index))
                // .filter(|e| e.availability() > 0)
                .map(|piece| PendingRequest::new(piece.index, piece.parts.clone()))
                .collect();

            debug!(
                "Queueing {} pending requests for {}",
                pending_requests.len(),
                torrent
            );
            for strategy in torrent.request_strategies_ref() {
                if strategy.supports(torrent).await {
                    pending_requests = strategy.order(pending_requests, torrent).await;
                }
            }

            torrent.add_pending_requests(pending_requests).await
        }
    }
}

#[async_trait]
impl TorrentOperation for TorrentPendingRequestsOperation {
    async fn execute<'a>(&self, torrent: &'a InnerTorrent) -> Option<&'a InnerTorrent> {
        if torrent.options().await.contains(TorrentFlags::DownloadMode) {
            self.create_pending_requests(&torrent).await;
        } else {
            torrent.cancel_all_pending_requests().await;
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentPendingRequestsOperation::new())
    }
}
