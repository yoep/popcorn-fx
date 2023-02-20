use log::{debug, warn};

use crate::core::config::{ConfigError, PopcornProperties, PopcornSettings};
use crate::core::storage::Storage;

const DEFAULT_SETTINGS_FILENAME: &str = "settings.json";

/// The config result type for all results returned by the config package.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// The application properties & settings of Popcorn FX.
/// This is the main entry into the config data of [PopcornProperties] & [PopcornSettings].
///
/// The [PopcornProperties] are static options that don't change during the lifecycle of the application.
/// The [PopcornSettings] on the other hand might change during the application lifecycle as
/// it contains the user preferences.
#[derive(Debug)]
pub struct ApplicationConfig {
    /// The storage to use for reading the settings
    pub storage: Storage,
    /// The static application properties
    pub properties: PopcornProperties,
    /// The user settings for the application
    pub settings: PopcornSettings,
}

impl ApplicationConfig {
    /// Create new [Settings] which will look for the [DEFAULT_CONFIG_FILENAME] config file.
    /// It will parse the config file if found, else uses the defaults instead.
    pub fn new_auto(storage_directory: &str) -> Self {
        let storage = Storage::from(storage_directory);
        let settings = match storage.read::<PopcornSettings>(DEFAULT_SETTINGS_FILENAME) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read settings from storage, using default settings instead, {}", e);
                PopcornSettings::default()
            }
        };

        Self {
            storage,
            properties: PopcornProperties::new_auto(),
            settings,
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

    /// Reload the application config.
    pub fn reload(&mut self) {}

    /// Save the application settings.
    pub fn save(&self) {
        debug!("Saving the application settings");
        // todo
    }
}

impl PartialEq for ApplicationConfig {
    fn eq(&self, other: &Self) -> bool {
        self.properties == other.properties
            && self.settings == other.settings
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::config::{SubtitleFamily, SubtitleSettings};
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_new_should_return_valid_instance() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let result = ApplicationConfig::new_auto(temp_path);
        let expected_result = "https://api.opensubtitles.com/api/v1".to_string();

        assert_eq!(&expected_result, result.properties().subtitle().url())
    }

    #[test]
    fn test_new_auto_should_read_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json");
        let application = ApplicationConfig::new_auto(temp_path);
        let expected_result = PopcornSettings {
            subtitle_settings: SubtitleSettings::new(
                None,
                Some(true),
                Some(SubtitleLanguage::English),
                Some(SubtitleFamily::Arial),
            ),
            ui_settings: Default::default(),
            server_settings: Default::default(),
            torrent_settings: Default::default(),
        };

        let result = application.user_settings();

        assert_eq!(&expected_result, result)
    }

    #[test]
    fn test_new_auto_settings_do_not_exist() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let application = ApplicationConfig::new_auto(temp_path);
        let expected_result = PopcornSettings::default();

        let result = application.user_settings();

        assert_eq!(&expected_result, result)
    }
}