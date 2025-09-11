use crate::torrent::metrics::{Counter, Gauge, Metric, State};
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub available_pieces: Gauge,
    pub client_interested: State,
    pub remote_interested: State,
    pub client_choked: State,
    pub remote_choked: State,
    pub bytes_in: Counter,
    pub bytes_in_useful: Counter,
    pub bytes_out: Counter,
    pub bytes_out_useful: Counter,
    pub rejects: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for Metrics {
    fn is_snapshot(&self) -> bool {
        self.bytes_in.is_snapshot()
    }

    fn snapshot(&self) -> Self {
        Self {
            available_pieces: self.available_pieces.snapshot(),
            client_interested: self.client_interested.snapshot(),
            remote_interested: self.remote_interested.snapshot(),
            client_choked: self.client_choked.snapshot(),
            remote_choked: self.remote_choked.snapshot(),
            bytes_in: self.bytes_in.snapshot(),
            bytes_in_useful: self.bytes_in_useful.snapshot(),
            bytes_out: self.bytes_out.snapshot(),
            bytes_out_useful: self.bytes_out_useful.snapshot(),
            rejects: self.rejects.snapshot(),
        }
    }

    fn tick(&self, interval: Duration) {
        self.available_pieces.tick(interval);
        self.client_interested.tick(interval);
        self.remote_interested.tick(interval);
        self.client_choked.tick(interval);
        self.remote_choked.tick(interval);
        self.bytes_in.tick(interval);
        self.bytes_in_useful.tick(interval);
        self.bytes_out.tick(interval);
        self.bytes_out_useful.tick(interval);
        self.rejects.tick(interval);
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "available pieces: {}, up: {}/s, down: {}/s",
            self.available_pieces.get(),
            self.bytes_out.rate(),
            self.bytes_in.rate()
        )
    }
}
