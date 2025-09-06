use std::time::Duration;

pub use counter::*;
pub use gauge::*;

mod counter;
mod gauge;

pub trait Metric: Sized {
    /// Check if the current metric is immutable, aka a snapshot metric.
    fn is_snapshot(&self) -> bool;

    /// Get an immutable snapshot (same type), unaffected by future ticks.
    fn snapshot(&self) -> Self;

    /// Call once per tick (typically once per second), providing a tick interval.
    fn tick(&self, interval: Duration);
}
