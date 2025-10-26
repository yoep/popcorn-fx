use crate::torrent::metrics::{Counter, Metric};
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub bytes_read: Counter,
    pub bytes_written: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for Metrics {
    fn is_snapshot(&self) -> bool {
        self.bytes_read.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            bytes_read: self.bytes_read.snapshot(),
            bytes_written: self.bytes_written.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.bytes_read.tick(interval);
        self.bytes_written.tick(interval);
    }
}
