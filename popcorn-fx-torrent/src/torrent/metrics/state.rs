use crate::torrent::metrics::Metric;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct State {
    inner: Arc<InnerState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerState::Mutable {
                value: AtomicBool::new(false),
            }),
        }
    }

    /// Get the current state of the metric.
    pub fn get(&self) -> bool {
        match &*self.inner {
            InnerState::Mutable { value } => value.load(Ordering::Relaxed),
            InnerState::Snapshot { value } => *value,
        }
    }

    /// Update the state of the metric.
    pub fn set(&self, state: bool) {
        match &*self.inner {
            InnerState::Mutable { value } => {
                value.store(state, Ordering::Relaxed);
            }
            InnerState::Snapshot { .. } => {
                // no-op
            }
        }
    }
}

impl Metric for State {
    fn is_snapshot(&self) -> bool {
        match &*self.inner {
            InnerState::Mutable { .. } => false,
            InnerState::Snapshot { .. } => true,
        }
    }

    fn snapshot(&self) -> Self {
        Self {
            inner: Arc::new(InnerState::Snapshot { value: self.get() }),
        }
    }

    fn tick(&self, _: Duration) {
        // no-op
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum InnerState {
    Mutable { value: AtomicBool },
    Snapshot { value: bool },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_snapshot() {
        let state = State::new();

        state.set(true);
        assert_eq!(true, state.get(), "expected the correct state");

        let snapshot = state.snapshot();
        state.set(false);
        assert_eq!(
            false,
            state.get(),
            "expected the state to have been updated"
        );
        assert_eq!(
            true,
            snapshot.get(),
            "expected the snapshot to still be true"
        );
    }
}
