use log::debug;

pub use error::*;
pub use properties::*;
pub use provider::*;
pub use server_settings::*;
pub use settings::*;
pub use subtitle_settings::*;
pub use torrent_settings::*;
pub use ui_settings::*;

use crate::core::storage::Storage;

mod error;
mod properties;
mod provider;
mod server_settings;
mod settings;
mod subtitle_settings;
mod ui_settings;
mod torrent_settings;

const DEFAULT_HOME_DIRECTORY: &str = ".popcorn-time";

/// The config result type for all results returned by the config package.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Contains the configuration and properties of the application.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Application {
    /// The static application properties
    properties: PopcornProperties,
    /// The user settings for the application
    settings: PopcornSettings,
}

impl Application {
    pub fn new(properties: PopcornProperties, settings: PopcornSettings) -> Self {
        Self { properties, settings }
    }

    /// Create new [Settings] which will look for the [DEFAULT_CONFIG_FILENAME] config file.
    /// It will parse the config file if found, else uses the defaults instead.
    pub fn new_auto(storage: &Storage) -> Self {
        Self {
            properties: PopcornProperties::new_auto(),
            settings: PopcornSettings::new_auto(storage),
        }
    }

    /// The popcorn properties of the application.
    /// These are static and won't change during the lifetime of the application.
    pub fn properties(&self) -> &PopcornProperties {
        &self.properties
    }

    /// The popcorn user settings of the application.
    /// These are mutable and can be changed during the lifetime of the application.
    /// They're most of the time managed by the user based on preferences.
    pub fn user_settings(&self) -> &PopcornSettings {
        &self.settings
    }

    /// Save the application settings.
    pub fn save(&self) {
        debug!("Saving the application settings");
        // todo
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_new_should_return_valid_instance() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage::from(temp_path);
        let result = Application::new_auto(&storage);
        let expected_result = "https://api.opensubtitles.com/api/v1".to_string();

        assert_eq!(&expected_result, result.properties.subtitle().url())
    }

    #[test]
    fn test_default_should_return_default_settings() {
        let application = Application::default();
        let result = application.settings.subtitle();

        assert_eq!(&SubtitleLanguage::None, result.default_subtitle());
        assert_eq!(&true, result.auto_cleaning_enabled())
    }
}