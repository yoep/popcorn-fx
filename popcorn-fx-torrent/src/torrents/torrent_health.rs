use derive_more::Display;

use crate::torrents::trackers::Announcement;

/// Represents the different states of torrent health.
#[repr(u8)]
#[derive(Debug, Default, Display, Clone, PartialEq)]
pub enum TorrentHealthState {
    /// Unknown health state, indicating that the health of the torrent could not be determined.
    #[default]
    #[display(fmt = "unknown")]
    Unknown,
    /// Bad health state, indicating that the torrent is in poor condition.
    #[display(fmt = "bad")]
    Bad,
    /// Medium health state, indicating that the torrent is in a moderate condition.
    #[display(fmt = "medium")]
    Medium,
    /// Good health state, indicating that the torrent is in good condition.
    #[display(fmt = "good")]
    Good,
    /// Excellent health state, indicating that the torrent is in excellent condition.
    #[display(fmt = "excellent")]
    Excellent,
}

/// Represents the health statistics of a torrent.
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentHealth {
    /// The health state of the torrent.
    pub state: TorrentHealthState,
    /// The ratio of uploaded data to downloaded data for the torrent.
    pub ratio: f32,
    /// The number of seeders (peers with a complete copy of the torrent).
    pub seeds: u64,
    /// The number of leechers currently downloading or sharing the torrent.
    pub leechers: u64,
}

impl From<&Announcement> for TorrentHealth {
    fn from(announcement: &Announcement) -> Self {
        // the seeds that have completed the download
        let seeds = announcement.total_seeders as f64;
        // the leechers that have partially downloaded the torrent
        let leechers = announcement.total_leechers as f64;

        let ratio = if leechers > 0.0 {
            seeds / leechers
        } else {
            seeds
        };

        // Precompute constants
        const RATIO_WEIGHT: f64 = 0.6;
        const SEEDS_WEIGHT: f64 = 0.4;

        // Normalize the data
        let normalized_ratio = f64::min(ratio / 5.0 * 100.0, 100.0);
        let normalized_seeds = f64::min(seeds / 30.0 * 100.0, 100.0);

        // Weight the metrics
        let weighted_total = (normalized_ratio * RATIO_WEIGHT) + (normalized_seeds * SEEDS_WEIGHT);
        let scaled_total = (weighted_total * 3.0 / 100.0).round() as u64;

        // Determine the health state
        let health_state = if seeds == 0f64 && leechers == 0f64 {
            TorrentHealthState::Unknown
        } else {
            match scaled_total {
                0 => TorrentHealthState::Bad,
                1 => TorrentHealthState::Medium,
                2 => TorrentHealthState::Good,
                3 => TorrentHealthState::Excellent,
                _ => TorrentHealthState::Unknown,
            }
        };

        Self {
            state: health_state,
            ratio: ratio as f32,
            seeds: seeds as u64,
            leechers: leechers as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_health_from() {
        let expected_result = TorrentHealth {
            state: Default::default(),
            ratio: 0.0,
            seeds: 0,
            leechers: 0,
        };
        let announcement = Announcement {
            total_leechers: 0,
            total_seeders: 0,
            peers: vec![],
        };
        let result = TorrentHealth::from(&announcement);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Bad,
            ratio: 0.5,
            seeds: 5,
            leechers: 10,
        };
        let announcement = Announcement {
            total_leechers: 10,
            total_seeders: 5,
            peers: vec![],
        };
        let result = TorrentHealth::from(&announcement);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Medium,
            ratio: 1.0,
            seeds: 10,
            leechers: 10,
        };
        let announcement = Announcement {
            total_leechers: 10,
            total_seeders: 10,
            peers: vec![],
        };
        let result = TorrentHealth::from(&announcement);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Good,
            ratio: 3.5,
            seeds: 35,
            leechers: 10,
        };
        let announcement = Announcement {
            total_leechers: 10,
            total_seeders: 35,
            peers: vec![],
        };
        let result = TorrentHealth::from(&announcement);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Excellent,
            ratio: 5.0,
            seeds: 50,
            leechers: 10,
        };
        let announcement = Announcement {
            total_leechers: 10,
            total_seeders: 50,
            peers: vec![],
        };
        let result = TorrentHealth::from(&announcement);
        assert_eq!(expected_result, result);
    }
}
