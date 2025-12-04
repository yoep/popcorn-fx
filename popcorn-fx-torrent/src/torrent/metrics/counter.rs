use crate::torrent::metrics::Metric;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
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
                avg_5s: Default::default(),
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

    /// Get the 5s average low-pass rate in counter/s.
    /// This is based on the libtorrent stats implementation.
    pub fn rate(&self) -> u32 {
        match &*self.inner {
            InnerCounter::Mutable { avg_5s, .. } => avg_5s.load(Ordering::Relaxed),
            InnerCounter::Snapshot { avg_5s, .. } => *avg_5s,
        }
    }

    /// Increase the counter by 1.
    pub fn inc(&self) {
        self.inc_by(1)
    }

    /// Increase the counter by the given value.
    pub fn inc_by(&self, value: u64) {
        match &*self.inner {
            InnerCounter::Mutable { total, counter, .. } => {
                counter.fetch_add(value, Ordering::Relaxed);
                total.fetch_add(value, Ordering::Relaxed);
            }
            InnerCounter::Snapshot { .. } => {}
        }
    }

    /// Reset the counter to 0.
    pub fn reset(&self) {
        match &*self.inner {
            InnerCounter::Mutable { total, counter, .. } => {
                counter.store(0, Ordering::Relaxed);
                total.store(0, Ordering::Relaxed);
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
                avg_5s: self.rate(),
            }),
        }
    }

    fn tick(&self, interval: Duration) {
        match &*self.inner {
            InnerCounter::Mutable {
                counter, avg_5s, ..
            } => {
                // calculate the 5s average low-pass with `avg = avg*4/5 + sample/5`
                if interval != Duration::from_secs(0) {
                    let sample =
                        counter.load(Ordering::Relaxed) * 1000u64 / (interval.as_millis() as u64);
                    let new_avg = ((avg_5s.load(Ordering::Relaxed) as u64) * 4 / 5)
                        .saturating_add(sample / 5);
                    avg_5s.store(new_avg.min(u64::from(u32::MAX)) as u32, Ordering::Relaxed);
                }

                counter.store(0, Ordering::Relaxed)
            }
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
        /// 5-second low-pass average (bytes/s)
        avg_5s: AtomicU32,
    },
    Snapshot {
        total: u64,
        counter: u64,
        avg_5s: u32,
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

    #[test]
    fn test_rate() {
        let counter = Counter::new();

        for _ in 0..10 {
            counter.inc_by(1000);
            counter.tick(Duration::from_secs(1));
        }

        let result = counter.rate();
        assert_eq!(891, result);
    }
}
