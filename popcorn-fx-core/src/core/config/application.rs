use derive_more::Display;
use log::{debug, info, trace, warn};

use crate::core::{CoreCallback, CoreCallbacks};
use crate::core::config::{ConfigError, PopcornProperties, PopcornSettings, SubtitleSettings, TorrentSettings};
use crate::core::storage::Storage;

const DEFAULT_SETTINGS_FILENAME: &str = "settings.json";

/// The config result type for all results returned by the config package.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// The callback type for the settings.
pub type ApplicationConfigCallback = CoreCallback<ApplicationConfigEvent>;

/// The events that can occur within the application settings.
#[derive(Debug, Clone, Display)]
pub enum ApplicationConfigEvent {
    /// Invoked when the settings have been loaded or reloaded
    #[display(fmt = "Settings have been loaded")]
    SettingsLoaded,
    /// Invoked when any of the subtitle settings have been changed
    #[display(fmt = "Subtitle settings have been changed")]
    SubtitleSettingsChanged(SubtitleSettings),
    /// Invoked when any of the torrent settings have been changed
    #[display(fmt = "Torrent settings have been changed")]
    TorrentSettingsChanged(TorrentSettings),
}

/// The application properties & settings of Popcorn FX.
/// This is the main entry into the config data of [PopcornProperties] & [PopcornSettings].
///
/// The [PopcornProperties] are static options that don't change during the lifecycle of the application.
/// The [PopcornSettings] on the other hand might change during the application lifecycle
/// as it contains the user preferences.
#[derive(Debug)]
pub struct ApplicationConfig {
    /// The storage to use for reading the settings
    pub storage: Storage,
    /// The static application properties
    pub properties: PopcornProperties,
    /// The user settings for the application
    pub settings: PopcornSettings,
    /// The callbacks for this application config
    pub callbacks: CoreCallbacks<ApplicationConfigEvent>,
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
            callbacks: CoreCallbacks::default(),
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

    /// Update the subtitle settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_subtitle(&mut self, subtitle_settings: SubtitleSettings) {
        if self.settings.subtitle_settings != subtitle_settings {
            self.settings.subtitle_settings = subtitle_settings;
            debug!("Subtitle settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::SubtitleSettingsChanged(self.settings.subtitle_settings.clone()));
        }
    }

    /// Update the torrent settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_torrent(&mut self, torrent_settings: TorrentSettings) {
        if self.settings.torrent_settings != torrent_settings {
            self.settings.torrent_settings = torrent_settings;
            debug!("Torrent settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::TorrentSettingsChanged(self.settings.torrent_settings.clone()));
        }
    }

    /// Reload the application config.
    pub fn reload(&mut self) {
        trace!("Reloading application settings");
        match self.storage.read::<PopcornSettings>(DEFAULT_SETTINGS_FILENAME) {
            Ok(e) => {
                debug!("Application settings have been read from storage");
                let old_settings = self.settings.clone();

                self.settings = e;
                info!("Settings have been reloaded");

                // start invoking events
                self.callbacks.invoke(ApplicationConfigEvent::SettingsLoaded);

                if old_settings.subtitle_settings != self.settings.subtitle_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::SubtitleSettingsChanged(self.settings.subtitle_settings.clone()));
                }
                if old_settings.torrent_settings != self.settings.torrent_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::TorrentSettingsChanged(self.settings.torrent_settings.clone()))
                }
            }
            Err(e) => warn!("Failed to reload settings from storage, {}", e)
        }
    }

    /// Register a new callback with this instance.
    pub fn register(&self, callback: ApplicationConfigCallback) {
        self.callbacks.add(callback)
    }

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
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::core::config::{DecorationType, SubtitleFamily, SubtitleSettings};
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_new_should_return_valid_instance() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let result = ApplicationConfig::new_auto(temp_path);
        let expected_result = "https://api.opensubtitles.com/api/v1".to_string();

        assert_eq!(&expected_result, result.properties().subtitle().url())
    }

    #[test]
    fn test_new_auto_should_read_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let application = ApplicationConfig::new_auto(temp_path);
        let expected_result = PopcornSettings {
            subtitle_settings: SubtitleSettings::new(
                None,
                Some(true),
                Some(SubtitleLanguage::English),
                Some(SubtitleFamily::Arial),
                Some(28),
                Some(DecorationType::Outline),
                Some(true),
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
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let application = ApplicationConfig::new_auto(temp_path);
        let expected_result = PopcornSettings::default();

        let result = application.user_settings();

        assert_eq!(&expected_result, result)
    }

    #[test]
    fn test_reload_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut application = ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        };
        application.storage.write(DEFAULT_SETTINGS_FILENAME, &PopcornSettings::default())
            .expect("expected the test file to have been written");

        application.register(Box::new(move |event| {
            tx.send(event).unwrap();
        }));
        application.reload();
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::SettingsLoaded => {}
            _ => assert!(false, "expected ApplicationConfigEvent::SettingsLoaded event")
        }
    }

    #[test]
    fn test_reload_subtitle_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut application = ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        };
        let expected_result = SubtitleSettings {
            directory: "my-directory".to_string(),
            auto_cleaning_enabled: false,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::Arial,
            font_size: 24,
            decoration: DecorationType::None,
            bold: true,
        };
        application.storage.write(DEFAULT_SETTINGS_FILENAME, &PopcornSettings {
            subtitle_settings: expected_result.clone(),
            ui_settings: Default::default(),
            server_settings: Default::default(),
            torrent_settings: Default::default(),
        })
            .expect("expected the test file to have been written");

        application.register(Box::new(move |event| {
            match event {
                ApplicationConfigEvent::SubtitleSettingsChanged(_) => tx.send(event).unwrap(),
                _ => {}
            }
        }));
        application.reload();
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::SubtitleSettingsChanged(settings) => assert_eq!(expected_result, settings),
            _ => assert!(false, "expected ApplicationConfigEvent::SettingsLoaded event")
        }
    }

    #[test]
    fn test_update_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let directory = "/tmp/lorem/subtitles";
        let settings = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: true,
            default_subtitle: SubtitleLanguage::Polish,
            font_family: SubtitleFamily::Arial,
            font_size: 22,
            decoration: DecorationType::None,
            bold: false,
        };
        let mut application = ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        };
        let (tx, rx) = channel();

        application.register(Box::new(move |event| {
            tx.send(event).unwrap()
        }));
        application.update_subtitle(settings.clone());
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::SubtitleSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().subtitle_settings);
            }
            _ => assert!(false, "expected ApplicationConfigEvent::SubtitleSettingsChanged")
        }
    }

    #[test]
    fn test_update_torrent() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let directory = "/tmp/lorem/torrents";
        let settings = TorrentSettings {
            directory: PathBuf::from(directory),
            auto_cleaning_enabled: false,
            connections_limit: 100,
            download_rate_limit: 0,
            upload_rate_limit: 0,
        };
        let mut application = ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        };
        let (tx, rx) = channel();

        application.register(Box::new(move |event| {
            tx.send(event).unwrap()
        }));
        application.update_torrent(settings.clone());
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::TorrentSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().torrent_settings);
            }
            _ => assert!(false, "expected ApplicationConfigEvent::TorrentSettingsChanged")
        }
    }
}