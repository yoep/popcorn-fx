use crate::torrent::metrics::{Counter, Gauge, Metric};
use std::time::Duration;

/// The DHT tracker metrics.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub total_nodes: Gauge,
    pub total_router_nodes: Gauge,
    pub total_pending_queries: Gauge,
    pub total_errors: Counter,
    pub total_bytes_in: Counter,
    pub total_bytes_out: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_nodes: Default::default(),
            total_router_nodes: Default::default(),
            total_pending_queries: Default::default(),
            total_errors: Default::default(),
            total_bytes_in: Default::default(),
            total_bytes_out: Default::default(),
        }
    }
}

impl Metric for Metrics {
    fn is_snapshot(&self) -> bool {
        self.total_nodes.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            total_nodes: self.total_nodes.snapshot(),
            total_router_nodes: self.total_router_nodes.snapshot(),
            total_pending_queries: self.total_pending_queries.snapshot(),
            total_errors: self.total_errors.snapshot(),
            total_bytes_in: self.total_bytes_in.snapshot(),
            total_bytes_out: self.total_bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.total_nodes.tick(interval);
        self.total_router_nodes.tick(interval);
        self.total_pending_queries.tick(interval);
        self.total_errors.tick(interval);
        self.total_bytes_in.tick(interval);
        self.total_bytes_out.tick(interval);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
