use derive_more::Display;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};

use crate::core::config::{ServerSettings, SubtitleSettings, TorrentSettings, UiSettings};
use crate::core::storage::Storage;

const DEFAULT_SETTINGS_FILENAME: &str = "settings.json";
const DEFAULT_SUBTITLES: fn() -> SubtitleSettings = SubtitleSettings::default;
const DEFAULT_UI: fn() -> UiSettings = UiSettings::default;
const DEFAULT_SERVER: fn() -> ServerSettings = ServerSettings::default;
const DEFAULT_TORRENT: fn() -> TorrentSettings = TorrentSettings::default;

/// The Popcorn FX user settings.
/// These contain the preferences of the user for the application.
#[derive(Debug, Display, Default, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "subtitle_settings: {}, ui_settings: {}", subtitle_settings, ui_settings)]
pub struct PopcornSettings {
    #[serde(default = "DEFAULT_SUBTITLES")]
    subtitle_settings: SubtitleSettings,
    #[serde(default = "DEFAULT_UI")]
    ui_settings: UiSettings,
    #[serde(default = "DEFAULT_SERVER")]
    server_settings: ServerSettings,
    #[serde(default = "DEFAULT_TORRENT")]
    torrent_settings: TorrentSettings,
}

impl PopcornSettings {
    pub fn new(subtitle_settings: SubtitleSettings, ui_settings: UiSettings, server_settings: ServerSettings,
               torrent_settings: TorrentSettings) -> Self {
        Self {
            subtitle_settings,
            ui_settings,
            server_settings,
            torrent_settings,
        }
    }

    /// Create new settings which will search for the [DEFAULT_SETTINGS_FILENAME].
    /// It will be parsed if found and valid, else the defaults will be returned.
    pub fn new_auto(storage: &Storage) -> Self {
        Self::from_filename(DEFAULT_SETTINGS_FILENAME, storage)
    }

    /// Create new settings from the given filename.
    /// This file will be searched within the home directory of the user.
    pub fn from_filename(filename: &str, storage: &Storage) -> Self {
        match storage.read::<Self>(filename) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read settings file {}, using defaults instead", e);
                Self::default()
            }
        }
    }

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
                warn!("Failed to deserialize settings, {}, using defaults instead", err.to_string());
                PopcornSettings::default()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::config::SubtitleFamily;
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_new_auto_should_always_return_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage::from(temp_path);

        PopcornSettings::new_auto(&storage);
    }

    #[test]
    fn test_settings_from_str_when_valid_should_return_expected_result() {
        init_logger();
        let value = "{\"subtitle_settings\":{\"directory\":\"my-path/to-subtitles\",\"auto_cleaning_enabled\":false,\"default_subtitle\":\"ENGLISH\",\"font_family\":\"ARIAL\",\"font_size\":32,\"decoration\":\"OUTLINE\",\"bold\":false}}";
        let expected_result = PopcornSettings::new(
            SubtitleSettings::new("my-path/to-subtitles".to_string(), false, SubtitleLanguage::English, SubtitleFamily::Arial),
            UiSettings::default(),
            ServerSettings::default(),
            TorrentSettings::default(),
        );

        let result = PopcornSettings::from(value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_settings_from_str_when_invalid_should_return_defaults() {
        init_logger();
        let value = "{something: \"value\"}";
        let expected_result = PopcornSettings::default();

        let result = PopcornSettings::from(value);

        assert_eq!(expected_result, result)
    }
}