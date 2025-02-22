use derive_more::Display;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};

use crate::core::config::{
    PlaybackSettings, ServerSettings, SubtitleSettings, TorrentSettings, TrackingSettings,
    UiSettings,
};

const DEFAULT_SUBTITLES: fn() -> SubtitleSettings = SubtitleSettings::default;
const DEFAULT_UI: fn() -> UiSettings = UiSettings::default;
const DEFAULT_SERVER: fn() -> ServerSettings = ServerSettings::default;
const DEFAULT_TORRENT: fn() -> TorrentSettings = TorrentSettings::default;
const DEFAULT_PLAYBACK: fn() -> PlaybackSettings = PlaybackSettings::default;
const DEFAULT_TRACKING: fn() -> TrackingSettings = TrackingSettings::default;

/// The Popcorn FX user settings.
/// These contain the preferences of the user for the application.
#[derive(Debug, Display, Default, Clone, Serialize, Deserialize, PartialEq)]
#[display(
    fmt = "subtitle_settings: {}, ui_settings: {}, server_settings: {}, torrent_settings: {}, playback_settings: {}, tracking_settings: {}",
    subtitle_settings,
    ui_settings,
    server_settings,
    torrent_settings,
    playback_settings,
    tracking_settings
)]
pub struct PopcornSettings {
    #[serde(default = "DEFAULT_SUBTITLES")]
    pub subtitle_settings: SubtitleSettings,
    #[serde(default = "DEFAULT_UI")]
    pub ui_settings: UiSettings,
    #[serde(default = "DEFAULT_SERVER")]
    pub server_settings: ServerSettings,
    #[serde(default = "DEFAULT_TORRENT")]
    pub torrent_settings: TorrentSettings,
    #[serde(default = "DEFAULT_PLAYBACK")]
    pub playback_settings: PlaybackSettings,
    #[serde(default = "DEFAULT_TRACKING")]
    pub tracking_settings: TrackingSettings,
}

impl PopcornSettings {
    /// Retrieve the subtitle settings of the application.
    pub fn subtitle(&self) -> &SubtitleSettings {
        &self.subtitle_settings
    }

    /// Retrieve the UI settings of the application.
    pub fn ui(&self) -> &UiSettings {
        &self.ui_settings
    }

    /// Retrieve the server settings of the application.
    pub fn server(&self) -> &ServerSettings {
        &self.server_settings
    }

    /// Retrieve the torrent settings of the application.
    pub fn torrent(&self) -> &TorrentSettings {
        &self.torrent_settings
    }

    /// Retrieve the playback settings of the application.
    pub fn playback(&self) -> &PlaybackSettings {
        &self.playback_settings
    }

    /// Retrieve the media tracking settings of the application.
    pub fn tracking(&self) -> &TrackingSettings {
        &self.tracking_settings
    }

    /// Retrieve a mutable reference to the media tracking settings of the application.
    pub fn tracking_mut(&mut self) -> &mut TrackingSettings {
        &mut self.tracking_settings
    }
}

impl From<&str> for PopcornSettings {
    /// Create new settings from the given json string.
    /// If the `value` is invalid, the [PopcornSettings::default] will be returned.
    fn from(value: &str) -> Self {
        trace!("Parsing application settings \"{}\"", value);
        match serde_json::from_str(value) {
            Ok(e) => {
                debug!("Application settings parsed, {:?}", &e);
                e
            }
            Err(err) => {
                warn!(
                    "Failed to deserialize settings, {}, using defaults instead",
                    err.to_string()
                );
                PopcornSettings::default()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::{DecorationType, SubtitleFamily};
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::init_logger;

    use super::*;

    #[test]
    fn test_settings_from_str_when_valid_should_return_expected_result() {
        init_logger!();
        let value = r#"{
  "subtitle_settings": {
    "directory": "my-path/to-subtitles",
    "auto_cleaning_enabled": false,
    "default_subtitle": "ENGLISH",
    "font_family": "ARIAL",
    "font_size": 32,
    "decoration": "OUTLINE",
    "bold": false
  }
}"#;
        let expected_result = PopcornSettings {
            subtitle_settings: SubtitleSettings {
                directory: "my-path/to-subtitles".to_string(),
                auto_cleaning_enabled: false,
                default_subtitle: SubtitleLanguage::English,
                font_family: SubtitleFamily::Arial,
                font_size: 32,
                decoration: DecorationType::Outline,
                bold: false,
            },
            ui_settings: Default::default(),
            server_settings: Default::default(),
            torrent_settings: Default::default(),
            playback_settings: Default::default(),
            tracking_settings: Default::default(),
        };

        let result = PopcornSettings::from(value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_settings_from_str_when_invalid_should_return_defaults() {
        init_logger!();
        let value = "{something: \"value\"}";
        let expected_result = PopcornSettings::default();

        let result = PopcornSettings::from(value);

        assert_eq!(expected_result, result)
    }
}
