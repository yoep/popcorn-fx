use std::fs::File;
use std::io::Read;

use derive_more::Display;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};

use crate::core::config::{DEFAULT_HOME_DIRECTORY, SubtitleSettings, UiSettings};

const DEFAULT_SETTINGS_FILENAME: &str = "settings.json";
const DEFAULT_SUBTITLES: fn() -> SubtitleSettings = || SubtitleSettings::default();
const DEFAULT_UI: fn() -> UiSettings = || UiSettings::default();

#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "subtitle_settings: {}, ui_settings: {}", subtitle_settings, ui_settings)]
pub struct PopcornSettings {
    #[serde(default = "DEFAULT_SUBTITLES")]
    subtitle_settings: SubtitleSettings,
    #[serde(default = "DEFAULT_UI")]
    ui_settings: UiSettings,
}

impl PopcornSettings {
    pub fn new(subtitle_settings: SubtitleSettings, ui_settings: UiSettings) -> Self {
        Self {
            subtitle_settings,
            ui_settings,
        }
    }

    /// Create new settings which will search for the [DEFAULT_SETTINGS_FILENAME].
    /// It will be parsed if found and valid, else the defaults will be returned.
    pub fn new_auto() -> Self {
        Self::from_filename(DEFAULT_SETTINGS_FILENAME)
    }

    /// Create new settings from the given filename.
    /// This file will be searched within the home directory of the user.
    pub fn from_filename(filename: &str) -> Self {
        let mut data = String::new();
        let path = home::home_dir().unwrap()
            .join(DEFAULT_HOME_DIRECTORY)
            .join(filename);


        match File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut data).expect("Unable to read the settings file");
                Self::from_str(data.as_str())
            }
            Err(err) => {
                warn!("Failed to read settings file {}, {}, using defaults instead", path.to_str().unwrap(), err.to_string());
                Self::default()
            }
        }
    }

    /// Create new settings from the given string.
    /// If the `value` is invalid, the defaults will be returned.
    pub fn from_str(value: &str) -> Self {
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

    /// Create settings from the given values.
    pub fn from(subtitle_settings: SubtitleSettings, ui_settings: UiSettings) -> Self {
        Self {
            subtitle_settings,
            ui_settings,
        }
    }

    /// The default settings for the application.
    pub fn default() -> Self {
        Self {
            subtitle_settings: DEFAULT_SUBTITLES(),
            ui_settings: DEFAULT_UI(),
        }
    }

    pub fn subtitle(&self) -> &SubtitleSettings {
        &self.subtitle_settings
    }

    pub fn ui(&self) -> &UiSettings {
        &self.ui_settings
    }
}

impl Default for PopcornSettings {
    fn default() -> Self {
        Self {
            subtitle_settings: SubtitleSettings::default(),
            ui_settings: UiSettings::default(),
        }
    }
}


#[cfg(test)]
mod test {
    use crate::core::config::SubtitleFamily;
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::test::init_logger;

    use super::*;

    #[test]
    fn test_settings_from_str_when_valid_should_return_expected_result() {
        init_logger();
        let value = "{\"subtitle_settings\":{\"directory\":\"my-path/to-subtitles\",\"auto_cleaning_enabled\":false,\"default_subtitle\":\"ENGLISH\",\"font_family\":\"ARIAL\",\"font_size\":32,\"decoration\":\"OUTLINE\",\"bold\":false}}";
        let expected_result = PopcornSettings::from(
            SubtitleSettings::new("my-path/to-subtitles".to_string(), false, SubtitleLanguage::English, SubtitleFamily::Arial),
            UiSettings::default());

        let result = PopcornSettings::from_str(value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_settings_from_str_when_invalid_should_return_defaults() {
        init_logger();
        let value = "{something: \"value\"}";
        let expected_result = PopcornSettings::default();

        let result = PopcornSettings::from_str(value);

        assert_eq!(expected_result, result)
    }
}