use crate::torrent::torrent_request_buffer::PendingRequestBuffer;
use crate::torrent::{PendingRequestContext, TorrentContext, TorrentFlags, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::warn;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::{select, time};

/// The maximum amount of time to wait for a request operation to complete.
const REQUEST_OPERATION_TIMEOUT: Duration = Duration::from_secs(2);
const TIME_BETWEEN_EXECUTIONS: Duration = Duration::from_secs(3);

/// The retrieve pending requests operation is responsible for retrieving pending requests from peers.
/// It executes the known request strategies of the torrent to data fo wanted pieces from peers.
#[derive(Debug, Display)]
#[display(fmt = "retrieve pending request operation")]
pub struct TorrentRetrievePendingRequestsOperation {
    last_execution: RwLock<Option<Instant>>,
}

impl TorrentRetrievePendingRequestsOperation {
    /// Create a new instance of the retrieve pending requests operation.
    pub fn new() -> Self {
        Self {
            last_execution: Default::default(),
        }
    }

    /// Start processing pending requests from peers.
    async fn request_pieces<'a>(
        &self,
        torrent: &TorrentContext,
        pending_requests: RwLockReadGuard<'a, PendingRequestBuffer>,
    ) {
        let strategies = torrent.request_strategies_ref();
        let peers = torrent.peer_pool().peers.read().await;
        let pieces = torrent.pieces_lock().read().await;
        let ctx = PendingRequestContext {
            pending_requests_buffer: pending_requests,
            peers,
            pieces,
        };

        for strategy in strategies {
            if !strategy.supports(torrent).await {
                continue;
            }

            let max_requests = ctx.pending_requests_buffer.available_permits();
            if max_requests == 0 {
                break;
            }

            self.last_execution.write().await.replace(Instant::now());
            select! {
                _ = time::sleep(REQUEST_OPERATION_TIMEOUT) => {
                    warn!("Request strategy \"{}\" timed out after {:?}", strategy, REQUEST_OPERATION_TIMEOUT);
                    break;
                },
                _ = strategy.execute(&ctx, max_requests) => {}
            }
        }
    }

    /// Check if requesting data from peers is allowed for the torrent.
    /// Returns true if requests can be made to the torrent, else false.
    async fn are_requests_allowed(&self, torrent: &TorrentContext) -> bool {
        let total_peers = torrent.peer_pool().peers.read().await.len();
        Self::is_torrent_running(torrent).await
            && total_peers > 0
            && self.is_execution_allowed().await
    }

    /// Check if it is time to request data from peers.
    async fn is_execution_allowed(&self) -> bool {
        let last_execution = self.last_execution.read().await;
        last_execution
            .map(|e| e.elapsed() >= TIME_BETWEEN_EXECUTIONS)
            .unwrap_or(true)
    }

    /// Check if the torrent is running and not paused.
    async fn is_torrent_running(torrent: &TorrentContext) -> bool {
        !torrent
            .options()
            .read()
            .await
            .contains(TorrentFlags::Paused)
    }
}

#[async_trait]
impl TorrentOperation for TorrentRetrievePendingRequestsOperation {
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        let should_request_piece_data = self.are_requests_allowed(torrent).await;
        if should_request_piece_data {
            let buffer = torrent.pending_requests().read().await;
            let available_permits = buffer.available_permits();
            let pending_requests_len = buffer.len();

            if available_permits > 0 && pending_requests_len > 0 {
                self.request_pieces(torrent, buffer).await;
            }
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(Self::new())
    }
}
