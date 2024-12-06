use crate::torrent::peer_pool::PeerPool;
use crate::torrent::torrent_request_buffer::{PendingRequest, PendingRequestBuffer};
use crate::torrent::{PendingRequestContext, Piece, RequestStrategy, TorrentContext};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use std::cmp::Ordering;
use tokio::sync::RwLockReadGuard;

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
    async fn supports(&self, _: &TorrentContext) -> bool {
        true
    }

    async fn execute<'a>(&self, ctx: &'a PendingRequestContext<'a>, max_requests: usize) {
        // TODO
    }

    fn clone_boxed(&self) -> Box<dyn RequestStrategy> {
        Box::new(Self::new())
    }
}
