use crate::ipc::proto;
use crate::ipc::proto::torrent::{torrent, torrent_event};
use popcorn_fx_core::core::torrents::collection::MagnetInfo;
use popcorn_fx_core::core::torrents::{Error, TorrentStreamState};
use popcorn_fx_torrent::torrent::{
    TorrentEvent, TorrentHealth, TorrentHealthState, TorrentState, TorrentStats,
};
use protobuf::MessageField;

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

impl From<&TorrentState> for torrent::State {
    fn from(value: &TorrentState) -> Self {
        match value {
            TorrentState::Initializing => Self::INITIALIZING,
            TorrentState::RetrievingMetadata => Self::RETRIEVING_METADATA,
            TorrentState::CheckingFiles => Self::CHECKING_FILES,
            TorrentState::Downloading => Self::DOWNLOADING,
            TorrentState::Finished => Self::FINISHED,
            TorrentState::Seeding => Self::SEEDING,
            TorrentState::Paused => Self::PAUSED,
            TorrentState::Error => Self::ERROR,
        }
    }
}

impl From<&TorrentStreamState> for torrent::StreamState {
    fn from(value: &TorrentStreamState) -> Self {
        match value {
            TorrentStreamState::Preparing => Self::PREPARING,
            TorrentStreamState::Streaming => Self::STREAMING,
            TorrentStreamState::Stopped => Self::STOPPED,
        }
    }
}

impl From<&MagnetInfo> for proto::torrent::MagnetInfo {
    fn from(value: &MagnetInfo) -> Self {
        Self {
            name: value.name.clone(),
            magnet_uri: value.magnet_uri.clone(),
            special_fields: Default::default(),
        }
    }
}

impl From<&TorrentEvent> for proto::torrent::TorrentEvent {
    fn from(value: &TorrentEvent) -> Self {
        let mut event = Self::new();

        match value {
            TorrentEvent::StateChanged(_) => {
                event.event = torrent_event::Event::STATE_CHANGED.into();
            }
            TorrentEvent::MetadataChanged => {
                event.event = torrent_event::Event::METADATA_CHANGED.into();
            }
            TorrentEvent::PeerConnected(_) => {
                event.event = torrent_event::Event::PEER_CONNECTED.into();
            }
            TorrentEvent::PeerDisconnected(_) => {
                event.event = torrent_event::Event::PEER_DISCONNECTED.into();
            }
            TorrentEvent::TrackersChanged => {
                event.event = torrent_event::Event::TRACKERS_CHANGED.into();
            }
            TorrentEvent::PiecesChanged => {
                event.event = torrent_event::Event::PIECES_CHANGED.into();
            }
            TorrentEvent::PiecePrioritiesChanged => {
                event.event = torrent_event::Event::PIECE_PRIORITIES_CHANGED.into();
            }
            TorrentEvent::PieceCompleted(piece) => {
                event.event = torrent_event::Event::PIECE_COMPLETED.into();
                event.piece_completed = MessageField::some(torrent_event::PieceCompleted {
                    piece_index: *piece as u64,
                    special_fields: Default::default(),
                });
            }
            TorrentEvent::FilesChanged => {
                event.event = torrent_event::Event::FILES_CHANGED.into();
            }
            TorrentEvent::OptionsChanged => {
                event.event = torrent_event::Event::OPTIONS_CHANGED.into();
            }
            TorrentEvent::Stats(stats) => {
                event.event = torrent_event::Event::STATS.into();
                event.stats = MessageField::some(torrent_event::Stats::from(stats));
            }
        }

        event
    }
}

impl From<&TorrentStats> for torrent_event::Stats {
    fn from(value: &TorrentStats) -> Self {
        Self {
            stats: MessageField::some(torrent::Stats::from(value)),
            special_fields: Default::default(),
        }
    }
}

impl From<&TorrentStats> for torrent::Stats {
    fn from(value: &TorrentStats) -> Self {
        Self {
            progress: value.progress(),
            upload: value.upload as u64,
            upload_rate: value.upload_rate,
            upload_useful: value.upload_useful as u64,
            upload_useful_rate: value.upload_useful_rate,
            download: value.download as u64,
            download_rate: value.download_rate,
            download_useful: value.download_useful as u64,
            download_useful_rate: value.download_useful_rate,
            total_uploaded: value.total_uploaded as u64,
            total_downloaded: value.total_downloaded as u64,
            total_downloaded_useful: value.total_downloaded_useful as u64,
            wanted_pieces: value.wanted_pieces as u64,
            completed_pieces: value.completed_pieces as u64,
            total_size: value.total_size as u64,
            total_completed_size: value.total_completed_size as u64,
            total_peers: value.total_peers as u64,
            special_fields: Default::default(),
        }
    }
}

impl From<&Error> for torrent::Error {
    fn from(value: &Error) -> Self {
        let mut error = Self::new();

        match value {
            Error::InvalidUrl(url) => {
                error.type_ = torrent::error::Type::INVALID_URL.into();
                error.invalid_url = MessageField::some(torrent::error::InvalidUrl {
                    url: url.clone(),
                    special_fields: Default::default(),
                });
            }
            Error::FileNotFound(file) => {
                error.type_ = torrent::error::Type::FILE_NOT_FOUND.into();
                error.file_not_found = MessageField::some(torrent::error::FileNotFound {
                    file: file.clone(),
                    special_fields: Default::default(),
                });
            }
            _ => todo!(),
        }

        error
    }
}
