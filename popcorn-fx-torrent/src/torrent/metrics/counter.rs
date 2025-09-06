use crate::torrent::metrics::Metric;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Counter {
    inner: Arc<InnerCounter>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerCounter::Mutable {
                total: Default::default(),
                counter: Default::default(),
            }),
        }
    }

    /// Get the total accumulated by this counter.
    pub fn total(&self) -> u64 {
        match &*self.inner {
            InnerCounter::Mutable { total, .. } => total.load(Ordering::Relaxed),
            InnerCounter::Snapshot { total, .. } => *total,
        }
    }

    /// Get the counter value for this tick.
    pub fn get(&self) -> u64 {
        match &*self.inner {
            InnerCounter::Mutable { counter, .. } => counter.load(Ordering::Relaxed),
            InnerCounter::Snapshot { counter, .. } => *counter,
        }
    }

    /// Increase the counter by 1.
    pub fn inc(&self) {
        self.inc_by(1)
    }

    /// Increase the counter by the given value.
    pub fn inc_by(&self, value: u64) {
        match &*self.inner {
            InnerCounter::Mutable { total, counter } => {
                counter.fetch_add(value, Ordering::Relaxed);
                total.fetch_add(value, Ordering::Relaxed);
            }
            InnerCounter::Snapshot { .. } => {}
        }
    }
}

impl Metric for Counter {
    fn is_snapshot(&self) -> bool {
        match &*self.inner {
            InnerCounter::Mutable { .. } => false,
            InnerCounter::Snapshot { .. } => true,
        }
    }

    fn snapshot(&self) -> Self {
        Self {
            inner: Arc::new(InnerCounter::Snapshot {
                total: self.total(),
                counter: self.get(),
            }),
        }
    }

    fn tick(&self, _: Duration) {
        match &*self.inner {
            InnerCounter::Mutable { counter, .. } => counter.store(0, Ordering::Relaxed),
            InnerCounter::Snapshot { .. } => {}
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum InnerCounter {
    Mutable {
        total: AtomicU64,
        counter: AtomicU64,
    },
    Snapshot {
        total: u64,
        counter: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot() {
        let counter = Counter::new();

        counter.inc_by(2);
        assert_eq!(2, counter.get());

        let snapshot = counter.snapshot();
        counter.inc_by(1);
        assert_eq!(2, snapshot.get());
        assert_eq!(3, counter.get());
    }

    #[test]
    fn test_tick() {
        let counter = Counter::new();

        counter.inc();
        assert_eq!(1, counter.get());

        counter.tick(Duration::from_secs(1));
        assert_eq!(0, counter.get());
        assert_eq!(1, counter.total());
    }
}
