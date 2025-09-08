use crate::torrent::metrics::{Counter, Gauge, Metric};
use std::time::Duration;

/// The DHT tracker metrics.
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub nodes: Gauge,
    pub router_nodes: Gauge,
    pub pending_queries: Gauge,
    pub errors: Counter,
    pub discovered_peers: Counter,
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for Metrics {
    fn is_snapshot(&self) -> bool {
        self.nodes.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            nodes: self.nodes.snapshot(),
            router_nodes: self.router_nodes.snapshot(),
            pending_queries: self.pending_queries.snapshot(),
            errors: self.errors.snapshot(),
            discovered_peers: self.discovered_peers.snapshot(),
            bytes_in: self.bytes_in.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.nodes.tick(interval);
        self.router_nodes.tick(interval);
        self.pending_queries.tick(interval);
        self.errors.tick(interval);
        self.discovered_peers.tick(interval);
        self.bytes_in.tick(interval);
        self.bytes_out.tick(interval);
    }
}
