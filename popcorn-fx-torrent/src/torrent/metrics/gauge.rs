use crate::torrent::metrics::Metric;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

const GAUGE_LIMIT: usize = 100;

#[derive(Debug, Clone)]
pub struct Gauge {
    inner: Arc<InnerGauge>,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerGauge::Mutable {
                values: Default::default(),
            }),
        }
    }

    /// Increase the gauge metric by 1.
    pub fn inc(&self) {
        self.inc_by(1)
    }

    /// Increase the gauge metric by the given value.
    pub fn inc_by(&self, value: u64) {
        match &*self.inner {
            InnerGauge::Mutable { values } => {
                if let Ok(mut values) = values.lock() {
                    let last_value = values.last().cloned().unwrap_or(0);
                    values.push(last_value.saturating_add(value));
                    Self::limit(&mut values);
                }
            }
            InnerGauge::Snapshot { .. } => {}
        }
    }

    /// Decrease the gauge metric by 1.
    pub fn dec(&self) {
        match &*self.inner {
            InnerGauge::Mutable { values } => {
                if let Ok(mut values) = values.lock() {
                    let last_value = values.last().cloned().unwrap_or(0);
                    values.push(last_value.saturating_sub(1));
                    Self::limit(&mut values);
                }
            }
            InnerGauge::Snapshot { .. } => {}
        }
    }

    /// Add the given value to the gauge metric.
    pub fn set(&self, value: u64) {
        match &*self.inner {
            InnerGauge::Mutable { values } => {
                if let Ok(mut values) = values.lock() {
                    values.push(value);
                    Self::limit(&mut values);
                }
            }
            InnerGauge::Snapshot { .. } => {}
        }
    }

    /// Get the gauge value for this tick.
    pub fn get(&self) -> u64 {
        match &*self.inner {
            InnerGauge::Mutable { values } => values
                .lock()
                .ok()
                .and_then(|e| e.last().cloned())
                .unwrap_or(0),
            InnerGauge::Snapshot { values } => values.last().cloned().unwrap_or(0),
        }
    }

    /// Get the average value of the gauge for this tick.
    pub fn avg(&self) -> u64 {
        match &*self.inner {
            InnerGauge::Mutable { values } => values
                .lock()
                .ok()
                .map(|e| Self::calculate_avg(e.as_slice()))
                .unwrap_or(0),
            InnerGauge::Snapshot { values } => Self::calculate_avg(values.as_slice()),
        }
    }

    /// Get the peak value of the gauge for this tick.
    pub fn peak(&self) -> u64 {
        match &*self.inner {
            InnerGauge::Mutable { values } => values
                .lock()
                .ok()
                .and_then(|e| e.iter().max().cloned())
                .unwrap_or(0),
            InnerGauge::Snapshot { values } => values.iter().max().cloned().unwrap_or(0),
        }
    }

    fn calculate_avg(values: &[u64]) -> u64 {
        values.iter().sum::<u64>() / values.len() as u64
    }

    fn limit(values: &mut MutexGuard<Vec<u64>>) {
        if values.len() >= GAUGE_LIMIT {
            let _ = values.remove(0);
        }
    }
}

impl Metric for Gauge {
    fn is_snapshot(&self) -> bool {
        match &*self.inner {
            InnerGauge::Mutable { .. } => false,
            InnerGauge::Snapshot { .. } => true,
        }
    }

    fn snapshot(&self) -> Self {
        Self {
            inner: Arc::new(InnerGauge::Snapshot {
                values: match &*self.inner {
                    InnerGauge::Mutable { values } => {
                        values.lock().ok().map(|e| e.clone()).unwrap_or(Vec::new())
                    }
                    InnerGauge::Snapshot { values } => values.clone(),
                },
            }),
        }
    }

    fn tick(&self, _: Duration) {
        // no-op
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum InnerGauge {
    Mutable { values: Mutex<Vec<u64>> },
    Snapshot { values: Vec<u64> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick() {
        let guage = Gauge::new();

        guage.set(10);
        assert_eq!(10, guage.get());

        guage.tick(Duration::from_secs(1));
        assert_eq!(0, guage.get());
    }
}
