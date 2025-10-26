use crate::torrent::metrics::{Counter, Metric};
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct TrackerManagerMetrics {
    pub bytes_in: Counter,
    pub bytes_out: Counter,
}

impl Metric for TrackerManagerMetrics {
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
            confirmed: self.confirmed.snapshot(),
            errors: self.errors.snapshot(),
            bytes_in: self.bytes_in.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
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
