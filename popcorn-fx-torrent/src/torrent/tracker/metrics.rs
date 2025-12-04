use crate::torrent::metrics::{Counter, Gauge, Metric};
use std::time::Duration;

/// Aggregated I/O metrics for a [`TrackerClient`].
///
/// Tracks the total number of bytes received (`bytes_in`) and sent (`bytes_out`)
/// across all trackers managed by the client.
#[derive(Debug, Default, Clone)]
pub struct TrackerClientMetrics {
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl Metric for TrackerClientMetrics {
    fn is_snapshot(&self) -> bool {
        self.bytes_in.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            bytes_in: self.bytes_in.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.bytes_in.tick(interval);
        self.bytes_out.tick(interval);
    }
}

#[derive(Debug, Default, Clone)]
pub struct TrackerMetrics {
    pub peers: Gauge,
    pub seeders: Gauge,
    pub leechers: Gauge,
    pub confirmed: Counter,
    pub errors: Counter,
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl Metric for TrackerMetrics {
    fn is_snapshot(&self) -> bool {
        self.errors.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            peers: self.peers.snapshot(),
            seeders: self.seeders.snapshot(),
            leechers: self.leechers.snapshot(),
            confirmed: self.confirmed.snapshot(),
            errors: self.errors.snapshot(),
            bytes_in: self.bytes_in.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.peers.tick(interval);
        self.seeders.tick(interval);
        self.leechers.tick(interval);
        self.confirmed.tick(interval);
        self.errors.tick(interval);
        self.bytes_in.tick(interval);
        self.bytes_out.tick(interval);
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConnectionMetrics {
    pub timeouts: Counter,
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl Metric for ConnectionMetrics {
    fn is_snapshot(&self) -> bool {
        self.bytes_in.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            timeouts: self.timeouts.snapshot(),
            bytes_in: self.bytes_in.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.timeouts.tick(interval);
        self.bytes_in.tick(interval);
        self.bytes_out.tick(interval);
    }
}
