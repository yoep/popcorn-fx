use crate::ipc::proto::media::media;
use crate::ipc::proto::settings;
use crate::ipc::proto::settings::application_settings;
use crate::ipc::proto::settings::application_settings::uisettings::Scale;
use crate::ipc::proto::settings::application_settings::{
    playback_settings, subtitle_settings, PlaybackSettings, ServerSettings, UISettings,
};
use crate::ipc::proto::subtitle::subtitle::Language;
use crate::ipc::{enum_into, Error, Result};
use popcorn_fx_core::core::config::{
    CleaningMode, DecorationType, PopcornSettings, Quality, SubtitleFamily, SubtitleSettings,
    TorrentSettings,
};
use protobuf::MessageField;
use std::path::PathBuf;

impl From<&PopcornSettings> for settings::ApplicationSettings {
    fn from(value: &PopcornSettings) -> Self {
        let mut ui = UISettings::new();
        ui.default_language = value.ui_settings.default_language.clone();
        let mut scale = Scale::new();
        scale.factor = value.ui_settings.ui_scale.value();
        ui.scale = MessageField::some(scale);
        ui.start_screen = media::Category::from(&value.ui_settings.start_screen).into();
        ui.maximized = value.ui_settings.maximized;
        ui.native_window_enabled = value.ui_settings.native_window_enabled;

        let mut server = ServerSettings::new();
        server.api_server = value.server_settings.api_server.clone();

        let mut playback = PlaybackSettings::new();
        playback.quality = value
            .playback_settings
            .quality
            .as_ref()
            .map(|e| playback_settings::Quality::from(e).into());
        playback.fullscreen = value.playback_settings.fullscreen;
        playback.auto_play_next_episode_enabled =
            value.playback_settings.auto_play_next_episode_enabled;

        Self {
            subtitle_settings: MessageField::some(application_settings::SubtitleSettings::from(
                &value.subtitle_settings,
            )),
            ui_settings: MessageField::some(ui),
            server_settings: MessageField::some(server),
            torrent_settings: MessageField::some(application_settings::TorrentSettings::from(
                &value.torrent_settings,
            )),
            playback_settings: MessageField::some(playback),
            tracking_settings: MessageField::some(application_settings::TrackingSettings {
                special_fields: Default::default(),
            }),
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
            connections_limit: value.connections_limit as u32,
            download_rate_limit: value.download_rate_limit as u32,
            upload_rate_limit: value.upload_rate_limit as u32,
        })
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
            Quality::P480 => playback_settings::Quality::P480,
            Quality::P720 => playback_settings::Quality::P720,
            Quality::P1080 => playback_settings::Quality::P1080,
            Quality::P2160 => playback_settings::Quality::P2160,
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
