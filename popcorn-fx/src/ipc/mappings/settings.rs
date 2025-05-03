use crate::ipc::proto::media::media;
use crate::ipc::proto::settings;
use crate::ipc::proto::settings::application_settings;
use crate::ipc::proto::settings::application_settings::tracking_settings::last_sync;
use crate::ipc::proto::settings::application_settings::{
    playback_settings, subtitle_settings, tracking_settings,
};
use crate::ipc::proto::subtitle::subtitle::Language;
use crate::ipc::{enum_into, Error, Result};
use popcorn_fx_core::core::config::{
    CleaningMode, DecorationType, MediaTrackingSyncState, PlaybackSettings, PopcornSettings,
    Quality, ServerSettings, SubtitleFamily, SubtitleSettings, TorrentSettings, TrackingSettings,
    UiScale, UiSettings,
};
use popcorn_fx_core::core::media::Category;
use protobuf::MessageField;
use std::path::PathBuf;

impl From<&PopcornSettings> for settings::ApplicationSettings {
    fn from(value: &PopcornSettings) -> Self {
        Self {
            subtitle_settings: MessageField::some(application_settings::SubtitleSettings::from(
                &value.subtitle_settings,
            )),
            ui_settings: MessageField::some(application_settings::UISettings::from(
                &value.ui_settings,
            )),
            server_settings: MessageField::some(application_settings::ServerSettings::from(
                &value.server_settings,
            )),
            torrent_settings: MessageField::some(application_settings::TorrentSettings::from(
                &value.torrent_settings,
            )),
            playback_settings: MessageField::some(application_settings::PlaybackSettings::from(
                &value.playback_settings,
            )),
            tracking_settings: MessageField::some(application_settings::TrackingSettings::from(
                &value.tracking_settings,
            )),
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitleSettings> for application_settings::SubtitleSettings {
    fn from(value: &SubtitleSettings) -> Self {
        Self {
            directory: value.directory.clone(),
            auto_cleaning_enabled: value.auto_cleaning_enabled,
            default_subtitle: Language::from(&value.default_subtitle).into(),
            font_family: subtitle_settings::Family::from(&value.font_family).into(),
            font_size: value.font_size as i32,
            decoration: subtitle_settings::DecorationType::from(&value.decoration).into(),
            bold: value.bold,
            special_fields: Default::default(),
        }
    }
}

impl From<&UiSettings> for application_settings::UISettings {
    fn from(value: &UiSettings) -> Self {
        Self {
            default_language: value.default_language.clone(),
            scale: MessageField::some(application_settings::uisettings::Scale {
                factor: value.ui_scale.value(),
                special_fields: Default::default(),
            }),
            start_screen: media::Category::from(&value.start_screen).into(),
            maximized: value.maximized,
            native_window_enabled: value.native_window_enabled,
            special_fields: Default::default(),
        }
    }
}

impl TryFrom<&application_settings::UISettings> for UiSettings {
    type Error = Error;

    fn try_from(value: &application_settings::UISettings) -> Result<Self> {
        let ui_scale = value
            .scale
            .as_ref()
            .map(|e| e.factor)
            .map(|e| UiScale::new(e))
            .transpose()
            .map_err(|e| Error::InvalidMessage(format!("{}", e)))?
            .unwrap_or_default();
        let start_screen = value
            .start_screen
            .enum_value()
            .map(|e| Category::from(&e))
            .map_err(|_| Error::UnsupportedEnum)?;

        Ok(Self {
            default_language: value.default_language.clone(),
            ui_scale,
            start_screen,
            maximized: value.maximized,
            native_window_enabled: value.native_window_enabled,
        })
    }
}

impl From<&ServerSettings> for application_settings::ServerSettings {
    fn from(value: &ServerSettings) -> Self {
        Self {
            api_server: value.api_server.clone(),
            special_fields: Default::default(),
        }
    }
}

impl TryFrom<&application_settings::ServerSettings> for ServerSettings {
    type Error = Error;

    fn try_from(value: &application_settings::ServerSettings) -> Result<Self> {
        Ok(Self {
            api_server: value.api_server.clone(),
        })
    }
}

impl From<&TorrentSettings> for application_settings::TorrentSettings {
    fn from(value: &TorrentSettings) -> Self {
        Self {
            directory: value.directory.as_os_str().to_string_lossy().to_string(),
            cleaning_mode: application_settings::torrent_settings::CleaningMode::from(
                &value.cleaning_mode,
            )
            .into(),
            connections_limit: value.connections_limit,
            download_rate_limit: value.download_rate_limit,
            upload_rate_limit: value.upload_rate_limit,
            special_fields: Default::default(),
        }
    }
}

impl TryFrom<&application_settings::TorrentSettings> for TorrentSettings {
    type Error = Error;

    fn try_from(value: &application_settings::TorrentSettings) -> Result<Self> {
        Ok(Self {
            directory: PathBuf::from(value.directory.as_str()),
            cleaning_mode: enum_into(value.cleaning_mode).map(|e| CleaningMode::from(&e))?,
            connections_limit: value.connections_limit,
            download_rate_limit: value.download_rate_limit,
            upload_rate_limit: value.upload_rate_limit,
        })
    }
}

impl From<&TrackingSettings> for application_settings::TrackingSettings {
    fn from(value: &TrackingSettings) -> Self {
        Self {
            last_sync: value
                .last_sync()
                .map(|e| tracking_settings::LastSync {
                    last_synced_millis: e.time.timestamp_millis() as u64,
                    state: last_sync::State::from(&e.state).into(),
                    special_fields: Default::default(),
                })
                .into(),
            special_fields: Default::default(),
        }
    }
}

impl From<&PlaybackSettings> for application_settings::PlaybackSettings {
    fn from(value: &PlaybackSettings) -> Self {
        Self {
            quality: value
                .quality
                .as_ref()
                .map(|e| playback_settings::Quality::from(e).into()),
            fullscreen: value.fullscreen,
            auto_play_next_episode_enabled: value.auto_play_next_episode_enabled,
            special_fields: Default::default(),
        }
    }
}

impl From<&MediaTrackingSyncState> for last_sync::State {
    fn from(value: &MediaTrackingSyncState) -> Self {
        match value {
            MediaTrackingSyncState::Success => Self::SUCCESS,
            MediaTrackingSyncState::Failed => Self::FAILED,
        }
    }
}

impl From<&SubtitleFamily> for subtitle_settings::Family {
    fn from(value: &SubtitleFamily) -> Self {
        match value {
            SubtitleFamily::Arial => subtitle_settings::Family::ARIAL,
            SubtitleFamily::ComicSans => subtitle_settings::Family::COMIC_SANS,
            SubtitleFamily::Georgia => subtitle_settings::Family::GEORGIA,
            SubtitleFamily::Tahoma => subtitle_settings::Family::TAHOMA,
            SubtitleFamily::TrebuchetMs => subtitle_settings::Family::TREBUCHET_MS,
            SubtitleFamily::Verdana => subtitle_settings::Family::VERDANA,
        }
    }
}

impl From<&DecorationType> for subtitle_settings::DecorationType {
    fn from(value: &DecorationType) -> Self {
        match value {
            DecorationType::None => subtitle_settings::DecorationType::NONE,
            DecorationType::Outline => subtitle_settings::DecorationType::OUTLINE,
            DecorationType::OpaqueBackground => {
                subtitle_settings::DecorationType::OPAQUE_BACKGROUND
            }
            DecorationType::SeeThroughBackground => {
                subtitle_settings::DecorationType::SEE_THROUGH_BACKGROUND
            }
        }
    }
}

impl From<&Quality> for playback_settings::Quality {
    fn from(value: &Quality) -> Self {
        match value {
            Quality::P480 => Self::P480,
            Quality::P720 => Self::P720,
            Quality::P1080 => Self::P1080,
            Quality::P2160 => Self::P2160,
        }
    }
}

impl From<&playback_settings::Quality> for Quality {
    fn from(value: &playback_settings::Quality) -> Self {
        match value {
            playback_settings::Quality::P0 | playback_settings::Quality::P480 => Self::P480,
            playback_settings::Quality::P720 => Self::P720,
            playback_settings::Quality::P1080 => Self::P1080,
            playback_settings::Quality::P2160 => Self::P2160,
        }
    }
}

impl From<&application_settings::torrent_settings::CleaningMode> for CleaningMode {
    fn from(value: &application_settings::torrent_settings::CleaningMode) -> Self {
        match value {
            application_settings::torrent_settings::CleaningMode::OFF => CleaningMode::Off,
            application_settings::torrent_settings::CleaningMode::ON_SHUTDOWN => {
                CleaningMode::OnShutdown
            }
            application_settings::torrent_settings::CleaningMode::WATCHED => CleaningMode::Watched,
        }
    }
}

impl From<&CleaningMode> for application_settings::torrent_settings::CleaningMode {
    fn from(value: &CleaningMode) -> Self {
        match value {
            CleaningMode::Off => application_settings::torrent_settings::CleaningMode::OFF,
            CleaningMode::OnShutdown => {
                application_settings::torrent_settings::CleaningMode::ON_SHUTDOWN
            }
            CleaningMode::Watched => application_settings::torrent_settings::CleaningMode::WATCHED,
        }
    }
}
