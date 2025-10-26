use crate::torrent::metrics::{Counter, Gauge, Metric};
use std::time::Duration;

/// The metrics of the DHT node tracker.
#[derive(Debug, Default, Clone)]
pub struct DhtMetrics {
    pub nodes: Gauge,
    pub router_nodes: Gauge,
    pub pending_queries: Gauge,
    pub errors: Counter,
    pub discovered_peers: Counter,
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl DhtMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for DhtMetrics {
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

/// The metrics of a DHT node.
#[derive(Debug, Default, Clone)]
pub struct NodeMetrics {
    /// The amount of times the node has successfully responded to a query.
    pub confirmed_queries: Counter,
    /// The number of times the node failed to respond to a query.
    pub timeouts: Counter,
}

impl NodeMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for NodeMetrics {
    fn is_snapshot(&self) -> bool {
        self.confirmed_queries.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            confirmed_queries: self.confirmed_queries.snapshot(),
            timeouts: self.timeouts.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.confirmed_queries.tick(interval);
        self.timeouts.tick(interval);
    }
}
