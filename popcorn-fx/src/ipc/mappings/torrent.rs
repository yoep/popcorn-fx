use crate::ipc::proto::torrent::torrent;
use popcorn_fx_torrent::torrent::{TorrentHealth, TorrentHealthState};

impl From<&TorrentHealth> for torrent::Health {
    fn from(value: &TorrentHealth) -> Self {
        Self {
            state: torrent::health::State::from(&value.state).into(),
            ratio: value.ratio,
            seeds: value.seeds,
            leechers: value.leechers,
            special_fields: Default::default(),
        }
    }
}

impl From<&TorrentHealthState> for torrent::health::State {
    fn from(value: &TorrentHealthState) -> Self {
        match value {
            TorrentHealthState::Unknown => Self::UNKNOWN,
            TorrentHealthState::Bad => Self::BAD,
            TorrentHealthState::Medium => Self::MEDIUM,
            TorrentHealthState::Good => Self::GOOD,
            TorrentHealthState::Excellent => Self::EXCELLENT,
        }
    }
}
