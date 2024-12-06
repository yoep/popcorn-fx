use crate::torrent::torrent_request_buffer::PendingRequest;
use crate::torrent::{
    Piece, PiecePriority, TorrentContext, TorrentFlags, TorrentOperation, TorrentState,
};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace};
use tokio::sync::RwLockReadGuard;

/// The creation of pending requests which should be retrieved from peers operations.
/// This operation is responsible for creating pending requests for pieces that are marked as wanted.
#[derive(Debug, Display)]
#[display(fmt = "create pending requests operation")]
pub struct TorrentPendingRequestsOperation {}

impl TorrentPendingRequestsOperation {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_pending_requests(&self, torrent: &TorrentContext) {
        let pieces = torrent.pieces_lock().read().await;
        let wanted_pieces: Vec<_> = Self::wanted_pieces(&pieces);

        if wanted_pieces.len() > 0 {
            torrent.update_state(TorrentState::Downloading).await;

            trace!("{} pieces are wanted for {}", wanted_pieces.len(), torrent);
            let mut pending_requests = torrent.pending_requests().write().await;
            let new_pending_requests: Vec<PendingRequest> = wanted_pieces
                .into_iter()
                // only create pending requests for pieces that are not already pending
                .filter(|e| !pending_requests.is_pending(&e.index))
                // order by priority
                .sorted_by(|a, b| (b.priority as u8).cmp(&(a.priority as u8)))
                .map(|piece| PendingRequest::new(piece.index, piece.parts.clone()))
                .collect();

            if new_pending_requests.len() > 0 {
                debug!(
                    "Queueing {} pending requests for {}",
                    new_pending_requests.len(),
                    torrent
                );
                pending_requests.push_all(new_pending_requests);
            }
        } else {
            torrent.update_state(TorrentState::Finished).await;
        }
    }

    /// Get all wanted pieces of the torrent that have not yet been completed
    fn wanted_pieces<'a>(pieces: &'a RwLockReadGuard<Vec<Piece>>) -> Vec<&'a Piece> {
        pieces
            .iter()
            // filter out any piece that isn't wanted
            .filter(|e| e.priority != PiecePriority::None)
            // filter out pieces that have already been completed
            .filter(|e| !e.is_completed())
            .collect()
    }

    /// Check if new pending requests are wanted
    async fn is_pending_requests_wanted(torrent: &TorrentContext) -> bool {
        torrent.pending_requests().read().await.len() == 0
    }
}

#[async_trait]
impl TorrentOperation for TorrentPendingRequestsOperation {
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        if torrent
            .options()
            .read()
            .await
            .contains(TorrentFlags::DownloadMode)
        {
            if Self::is_pending_requests_wanted(torrent).await {
                self.create_pending_requests(&torrent).await;
            }
        } else {
            torrent.cancel_all_pending_requests().await;
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentPendingRequestsOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::operation::TorrentPiecesOperation;
    use crate::torrent::{PiecePriority, Torrent, TorrentInfo};
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_execute() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .operations(vec![])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build();
        let operation = TorrentPendingRequestsOperation::new();
        let inner = torrent.unwrap().instance().unwrap();

        runtime.block_on(async {
            let pieces_operation = TorrentPiecesOperation::new();

            // create the pieces
            pieces_operation.execute(&*inner).await;

            // modify the priorities
            let mut priorities = Vec::new();
            for piece in 5..inner.total_pieces().await {
                priorities.push((piece, PiecePriority::None));
            }
            inner.prioritize_pieces(priorities).await;

            let result = operation.execute(&*inner).await;
            assert_eq!(Some(&*inner), result);

            let result = inner.pending_requests().read().await;
            assert_eq!(
                5,
                result.len(),
                "expected pending requests to have been created"
            );
        });
    }
}
