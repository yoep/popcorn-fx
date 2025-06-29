use crate::ipc::proto;
use crate::ipc::proto::message;
use crate::ipc::proto::torrent::{torrent, torrent_event};
use popcorn_fx_core::core::torrents::collection::MagnetInfo;
use popcorn_fx_core::core::torrents::{Error, TorrentInfo, TorrentStreamState};
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

impl From<&TorrentInfo> for torrent::Info {
    fn from(value: &TorrentInfo) -> Self {
        Self {
            handle: MessageField::some(message::Handle::from(&value.handle)),
            info_hash: value.info_hash.clone(),
            uri: value.uri.clone(),
            name: value.name.clone(),
            directory_name: value.directory_name.clone(),
            total_files: value.total_files,
            files: value.files.iter().map(torrent::info::File::from).collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&popcorn_fx_torrent::torrent::File> for torrent::info::File {
    fn from(value: &popcorn_fx_torrent::torrent::File) -> Self {
        Self {
            index: value.index as u32,
            filename: value.filename(),
            torrent_path: value.torrent_path.as_os_str().to_string_lossy().to_string(),
            offset: value.offset as u64,
            length: value.length() as u64,
            md5sum: value.info.md5sum.clone(),
            sha1: value.info.sha1.clone(),
            special_fields: Default::default(),
        }
    }
}

impl From<&TorrentEvent> for proto::torrent::TorrentEvent {
    fn from(value: &TorrentEvent) -> Self {
        let mut event = Self::new();

        match value {
            TorrentEvent::StateChanged(state) => {
                event.event = torrent_event::Event::STATE_CHANGED.into();
                event.state_changed = MessageField::some(torrent_event::StateChanged {
                    state: torrent::State::from(state).into(),
                    special_fields: Default::default(),
                });
            }
            TorrentEvent::MetadataChanged(_) => {
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
            TorrentEvent::PiecesChanged(_) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_state_from() {
        assert_eq!(
            torrent::health::State::UNKNOWN,
            torrent::health::State::from(&TorrentHealthState::Unknown)
        );
        assert_eq!(
            torrent::health::State::BAD,
            torrent::health::State::from(&TorrentHealthState::Bad)
        );
        assert_eq!(
            torrent::health::State::MEDIUM,
            torrent::health::State::from(&TorrentHealthState::Medium)
        );
        assert_eq!(
            torrent::health::State::GOOD,
            torrent::health::State::from(&TorrentHealthState::Good)
        );
        assert_eq!(
            torrent::health::State::EXCELLENT,
            torrent::health::State::from(&TorrentHealthState::Excellent)
        );
    }

    #[test]
    fn test_torrent_state_from() {
        assert_eq!(
            torrent::State::INITIALIZING,
            torrent::State::from(&TorrentState::Initializing)
        );
        assert_eq!(
            torrent::State::RETRIEVING_METADATA,
            torrent::State::from(&TorrentState::RetrievingMetadata)
        );
        assert_eq!(
            torrent::State::CHECKING_FILES,
            torrent::State::from(&TorrentState::CheckingFiles)
        );
        assert_eq!(
            torrent::State::DOWNLOADING,
            torrent::State::from(&TorrentState::Downloading)
        );
        assert_eq!(
            torrent::State::FINISHED,
            torrent::State::from(&TorrentState::Finished)
        );
        assert_eq!(
            torrent::State::SEEDING,
            torrent::State::from(&TorrentState::Seeding)
        );
        assert_eq!(
            torrent::State::PAUSED,
            torrent::State::from(&TorrentState::Paused)
        );
        assert_eq!(
            torrent::State::ERROR,
            torrent::State::from(&TorrentState::Error)
        );
    }

    #[test]
    fn test_torrent_stream_state_from() {
        assert_eq!(
            torrent::StreamState::PREPARING,
            torrent::StreamState::from(&TorrentStreamState::Preparing)
        );
        assert_eq!(
            torrent::StreamState::STREAMING,
            torrent::StreamState::from(&TorrentStreamState::Streaming)
        );
        assert_eq!(
            torrent::StreamState::STOPPED,
            torrent::StreamState::from(&TorrentStreamState::Stopped)
        );
    }

    #[test]
    fn test_torrent_event_from_state_changed() {
        let event = TorrentEvent::StateChanged(TorrentState::Downloading);
        let expected_result = proto::torrent::TorrentEvent {
            torrent_handle: Default::default(),
            event: torrent_event::Event::STATE_CHANGED.into(),
            state_changed: MessageField::some(torrent_event::StateChanged {
                state: torrent::State::DOWNLOADING.into(),
                special_fields: Default::default(),
            }),
            peer_connected: Default::default(),
            peer_disconnected: Default::default(),
            piece_completed: Default::default(),
            stats: Default::default(),
            special_fields: Default::default(),
        };

        let result = proto::torrent::TorrentEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_event_from_metadata_changed() {
        let event = TorrentEvent::MetadataChanged;
        let expected_result = proto::torrent::TorrentEvent {
            torrent_handle: Default::default(),
            event: torrent_event::Event::METADATA_CHANGED.into(),
            state_changed: Default::default(),
            peer_connected: Default::default(),
            peer_disconnected: Default::default(),
            piece_completed: Default::default(),
            stats: Default::default(),
            special_fields: Default::default(),
        };

        let result = proto::torrent::TorrentEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_event_from_piece_completed() {
        let event = TorrentEvent::PieceCompleted(87);
        let expected_result = proto::torrent::TorrentEvent {
            torrent_handle: Default::default(),
            event: torrent_event::Event::PIECE_COMPLETED.into(),
            state_changed: Default::default(),
            peer_connected: Default::default(),
            peer_disconnected: Default::default(),
            piece_completed: MessageField::some(torrent_event::PieceCompleted {
                piece_index: 87,
                special_fields: Default::default(),
            }),
            stats: Default::default(),
            special_fields: Default::default(),
        };

        let result = proto::torrent::TorrentEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_event_from_stats() {
        let event = TorrentEvent::Stats(TorrentStats {
            upload: 0,
            upload_rate: 1024,
            upload_useful: 0,
            upload_useful_rate: 0,
            download: 0,
            download_rate: 0,
            download_useful: 0,
            download_useful_rate: 0,
            total_uploaded: 0,
            total_downloaded: 17000,
            total_downloaded_useful: 14000,
            wanted_pieces: 300,
            completed_pieces: 12,
            total_size: 200000,
            total_completed_size: 14000,
            total_peers: 67,
        });
        let expected_result = proto::torrent::TorrentEvent {
            torrent_handle: Default::default(),
            event: torrent_event::Event::STATS.into(),
            state_changed: Default::default(),
            peer_connected: Default::default(),
            peer_disconnected: Default::default(),
            piece_completed: Default::default(),
            stats: MessageField::some(torrent_event::Stats {
                stats: MessageField::some(torrent::Stats {
                    progress: 0.04,
                    upload: 0,
                    upload_rate: 1024,
                    upload_useful: 0,
                    upload_useful_rate: 0,
                    download: 0,
                    download_rate: 0,
                    download_useful: 0,
                    download_useful_rate: 0,
                    total_uploaded: 0,
                    total_downloaded: 17000,
                    total_downloaded_useful: 14000,
                    wanted_pieces: 300,
                    completed_pieces: 12,
                    total_size: 200000,
                    total_completed_size: 14000,
                    total_peers: 67,
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            special_fields: Default::default(),
        };

        let result = proto::torrent::TorrentEvent::from(&event);

        assert_eq!(expected_result, result);
    }
}
