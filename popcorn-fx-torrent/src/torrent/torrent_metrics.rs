use crate::torrent::format_bytes;
use crate::torrent::metrics::{Counter, Gauge, Metric};
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub upload: Counter,
    pub upload_useful: Counter,
    pub download: Counter,
    pub download_useful: Counter,
    /// The amount of wasted bytes for the torrent.
    pub wasted: Counter,
    /// The amount of pieces in which the torrent is interested.
    pub wanted_pieces: Gauge,
    /// The amount of interested pieces which have been completed & validated for the torrent.
    pub wanted_completed_pieces: Gauge,
    /// The amount of bytes for the interested pieces of the torrent.
    pub wanted_size: Gauge,
    /// The amount of completed & validated bytes for the interested pieces of the torrent.
    pub wanted_completed_size: Gauge,
    /// The amount of completed & validated pieces of the torrent.
    pub completed_pieces: Counter,
    /// The amount of completed & validated bytes of the torrent.
    pub completed_size: Counter,
    /// The amount of connected peers to the torrent.
    pub peers: Gauge,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the progress, as a percentage, of the torrent download.
    /// The value returned is between 0.00 and 1.00.
    pub fn progress(&self) -> f32 {
        let wanted_pieces = self.wanted_pieces.get();
        if wanted_pieces == 0 {
            return 1.0;
        }

        let progress = self.wanted_completed_pieces.get() as f32 / wanted_pieces as f32;
        (progress * 1000.0).floor() / 1000.0
    }

    /// Get the amount of bytes remaining which need to be downloaded by the torrent.
    /// This is based on the interested pieces of the torrent and not the total pieces.
    pub fn bytes_remaining(&self) -> u64 {
        let wanted_size = self.wanted_size.get();
        if wanted_size == 0 {
            return 0;
        }

        wanted_size.saturating_sub(self.wanted_completed_size.get())
    }
}

impl Metric for Metrics {
    fn is_snapshot(&self) -> bool {
        self.upload.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            upload: self.upload.snapshot(),
            upload_useful: self.upload_useful.snapshot(),
            download: self.download.snapshot(),
            download_useful: self.download_useful.snapshot(),
            wasted: self.wasted.snapshot(),
            wanted_pieces: self.wanted_pieces.snapshot(),
            wanted_completed_pieces: self.wanted_completed_pieces.snapshot(),
            wanted_size: self.wanted_size.snapshot(),
            wanted_completed_size: self.wanted_completed_size.snapshot(),
            completed_pieces: self.completed_pieces.snapshot(),
            completed_size: self.completed_size.snapshot(),
            peers: self.peers.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.upload.tick(interval);
        self.upload_useful.tick(interval);
        self.download.tick(interval);
        self.download_useful.tick(interval);
        self.wasted.tick(interval);
        self.wanted_pieces.tick(interval);
        self.wanted_completed_pieces.tick(interval);
        self.wanted_size.tick(interval);
        self.wanted_completed_size.tick(interval);
        self.completed_pieces.tick(interval);
        self.completed_size.tick(interval);
        self.peers.tick(interval);
    }
}

impl PartialEq for Metrics {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:.2}%) {}/{}, {}/{} completed pieces, {} peers, up: {}/s, down: {}/s",
            self.progress() * 100f32,
            format_bytes(self.wanted_completed_size.get() as usize),
            format_bytes(self.wanted_size.get() as usize),
            self.wanted_completed_pieces.get(),
            self.wanted_pieces.get(),
            self.peers.get(),
            format_bytes(self.upload.rate() as usize),
            format_bytes(self.download.rate() as usize),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress() {
        let metrics = Metrics::new();

        let result = metrics.progress();
        assert_eq!(result, 1.0, "expected the initial progress to be 100%");

        // set the metric data for calculating the progress
        metrics.wanted_pieces.set(10);
        metrics.wanted_completed_pieces.set(6);

        let result = metrics.progress();
        assert_eq!(result, 0.6, "expected the initial progress to be 60%");
    }
}
