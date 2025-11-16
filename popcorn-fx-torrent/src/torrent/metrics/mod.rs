use std::time::Duration;

pub use counter::*;
pub use gauge::*;
pub use state::*;

mod counter;
mod gauge;
mod state;

/// A trait representing a time-advancing metric that can produce immutable snapshots.
///
/// Types implementing this trait behave as *live metrics* whose values may evolve
/// over time. They support three core operations:
///
/// * **`tick`** — advances internal state by a fixed time interval
/// * **`snapshot`** — produces an immutable, self-contained copy of the metric
///   whose values will not change after creation
/// * **`is_snapshot`** — identifies whether the metric instance is already such
///   an immutable snapshot
///
/// This abstraction allows a metric to track moving or rate-based values while
/// still supporting efficient, read-only snapshots for reporting or event emission.
pub trait Metric: Sized {
    /// Returns `true` if this metric instance represents an immutable snapshot.
    ///
    /// Snapshot metrics are not affected by future calls to [`tick`] on the
    /// original live metric. Implementors typically set an internal flag or use
    /// copy-on-write semantics to distinguish between live and snapshot modes.
    fn is_snapshot(&self) -> bool;

    /// Produces an immutable snapshot of this metric.
    ///
    /// A snapshot must contain the current values of the metric and **must not**
    /// be modified by future updates or ticks to the original metric instance.
    ///
    /// This is commonly used when exporting metrics for reporting, logging,
    /// or subscriber callbacks.
    fn snapshot(&self) -> Self;

    /// Advances the metric’s internal state by the provided time interval.
    ///
    /// This is typically called periodically (e.g. once per second) to update
    /// rate counters, decay windows, or any other time-dependent metric logic.
    fn tick(&self, interval: Duration);
}
