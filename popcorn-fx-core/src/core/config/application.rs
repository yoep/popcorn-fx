use derive_more::Display;
use log::{debug, error, info, trace, warn};

use crate::core::{block_in_place, CoreCallback, CoreCallbacks};
use crate::core::config::{ConfigError, PlaybackSettings, PopcornProperties, PopcornSettings, ServerSettings, SubtitleSettings, TorrentSettings, UiSettings};
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
    #[display(fmt = "UI settings have been changed")]
    /// Invoked when the ui settings have been changed
    UiSettingsChanged(UiSettings),
    /// Invoked when the server settings have been changed
    #[display(fmt = "Server settings have been changed")]
    ServerSettingsChanged(ServerSettings),
    /// Invoked when the playback settings have been changed
    #[display(fmt = "Playback settings have been changed")]
    PlaybackSettingsChanged(PlaybackSettings),
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
    /// Creates a new `ApplicationConfigBuilder` with which a new [ApplicationConfig] can be
    /// initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, ApplicationConfigCallback, PopcornProperties};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage-path") // This field is required and will panic when not set
    ///     .properties(PopcornProperties::default())
    ///     .with_callback(callback)
    ///     .build();
    /// ```
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
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
            self.save();
        }
    }

    /// Update the torrent settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_torrent(&mut self, torrent_settings: TorrentSettings) {
        if self.settings.torrent_settings != torrent_settings {
            self.settings.torrent_settings = torrent_settings;
            debug!("Torrent settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::TorrentSettingsChanged(self.settings.torrent_settings.clone()));
            self.save();
        }
    }

    /// Update the ui settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_ui(&mut self, ui_settings: UiSettings) {
        if self.settings.ui_settings != ui_settings {
            self.settings.ui_settings = ui_settings;
            debug!("UI settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::UiSettingsChanged(self.settings.ui().clone()));
            self.save();
        }
    }

    /// Update the api server settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_server(&mut self, settings: ServerSettings) {
        if self.settings.server_settings != settings {
            self.settings.server_settings = settings;
            debug!("Server settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::ServerSettingsChanged(self.settings.server().clone()));
            self.save();
        }
    }

    /// Update the playback settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub fn update_playback(&mut self, settings: PlaybackSettings) {
        if self.settings.playback_settings != settings {
            self.settings.playback_settings = settings;
            debug!("Playback settings have been updated");
            self.callbacks.invoke(ApplicationConfigEvent::PlaybackSettingsChanged(self.settings.playback().clone()));
            self.save();
        }
    }

    /// Reload the application config.
    pub fn reload(&mut self) {
        trace!("Reloading application settings");
        match self.storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .read::<PopcornSettings>() {
            Ok(e) => {
                debug!("Application settings have been read from storage");
                let old_settings = self.settings.clone();

                self.settings = e;
                info!("Settings have been reloaded");

                // start invoking events
                self.callbacks.invoke(ApplicationConfigEvent::SettingsLoaded);

                if old_settings.subtitle_settings != self.settings.subtitle_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::SubtitleSettingsChanged(self.settings.subtitle().clone()));
                }
                if old_settings.torrent_settings != self.settings.torrent_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::TorrentSettingsChanged(self.settings.torrent().clone()))
                }
                if old_settings.ui_settings != self.settings.ui_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::UiSettingsChanged(self.settings.ui().clone()))
                }
                if old_settings.server_settings != self.settings.server_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::ServerSettingsChanged(self.settings.server().clone()))
                }
                if old_settings.playback_settings != self.settings.playback_settings {
                    self.callbacks.invoke(ApplicationConfigEvent::PlaybackSettingsChanged(self.settings.playback().clone()))
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
        block_in_place(self.save_async(self.user_settings()))
    }

    async fn save_async(&self, settings: &PopcornSettings) {
        trace!("Saving application settings");
        match self.storage.options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write_async(settings).await {
            Ok(_) => info!("Settings have been saved"),
            Err(e) => error!("Failed to save settings, {}", e)
        }
    }
}

impl PartialEq for ApplicationConfig {
    fn eq(&self, other: &Self) -> bool {
        self.properties == other.properties
            && self.settings == other.settings
    }
}

impl Drop for ApplicationConfig {
    fn drop(&mut self) {
        debug!("Saving settings on exit");
        self.save()
    }
}

/// The builder for the [ApplicationConfig].
#[derive(Debug, Default)]
pub struct ApplicationConfigBuilder {
    storage: Option<Storage>,
    properties: Option<PopcornProperties>,
    settings: Option<PopcornSettings>,
    callbacks: CoreCallbacks<ApplicationConfigEvent>,
}

impl ApplicationConfigBuilder {
    /// Sets the storage to use for reading the settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, PopcornProperties};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .properties(PopcornProperties::default())
    ///     .build();
    /// ```
    pub fn storage(mut self, storage: &str) -> Self {
        self.storage = Some(Storage::from(storage));
        self
    }

    /// Sets the user settings for the application.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, PopcornSettings};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .settings(PopcornSettings::default())
    ///     .build();
    /// ```
    pub fn settings(mut self, settings: PopcornSettings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Sets the static properties of the application.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, PopcornProperties};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .properties(PopcornProperties::default())
    ///     .build();
    /// ```
    pub fn properties(mut self, properties: PopcornProperties) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Adds an additional callback to the `CoreCallbacks` object for the application config.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::ApplicationConfig;
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .with_callback(callback)
    ///     .build();
    /// ```
    pub fn with_callback(self, callback: ApplicationConfigCallback) -> Self {
        self.callbacks.add(callback);
        self
    }

    /// Builds an `ApplicationConfig` object with the specified parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if the `storage` path has not been set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, PopcornProperties};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .properties(PopcornProperties::default())
    ///     .build();
    /// ```
    pub fn build(self) -> ApplicationConfig {
        let storage = self.storage.expect("storage path has not been set");
        let settings = self.settings
            .or_else(|| {
                match storage.options()
                    .serializer(DEFAULT_SETTINGS_FILENAME)
                    .read::<PopcornSettings>() {
                    Ok(e) => Some(e),
                    Err(e) => {
                        warn!("Failed to read settings from storage, using default settings instead, {}", e);
                        Some(PopcornSettings::default())
                    }
                }
            })
            .unwrap();
        let properties = self.properties
            .or_else(|| Some(PopcornProperties::new_auto()))
            .unwrap();

        ApplicationConfig {
            storage,
            properties,
            settings,
            callbacks: self.callbacks,
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::core::config::{DecorationType, Quality, SubtitleFamily, SubtitleSettings, UiScale};
    use crate::core::media::Category;
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file_as_string};

    use super::*;

    #[test]
    fn test_new_should_return_valid_instance() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let result = ApplicationConfig::builder()
            .storage(temp_path)
            .build();
        let expected_result = "https://api.opensubtitles.com/api/v1".to_string();

        assert_eq!(&expected_result, result.properties().subtitle().url())
    }

    #[test]
    fn test_new_auto_should_read_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let application = ApplicationConfig::builder()
            .storage(temp_path)
            .build();
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
            playback_settings: Default::default(),
        };

        let result = application.user_settings();

        assert_eq!(&expected_result, result)
    }

    #[test]
    fn test_new_auto_settings_do_not_exist() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let application = ApplicationConfig::builder()
            .storage(temp_path)
            .build();
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
        application.storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write(&PopcornSettings::default())
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
        application.storage.options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write(&PopcornSettings {
                subtitle_settings: expected_result.clone(),
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
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

    #[test]
    fn test_update_ui() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = UiSettings {
            default_language: "en".to_string(),
            ui_scale: UiScale::new(1.2).unwrap(),
            start_screen: Category::Favorites,
            maximized: false,
            native_window_enabled: false,
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
        application.update_ui(settings.clone());
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::UiSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().ui_settings);
            }
            _ => assert!(false, "expected ApplicationConfigEvent::UiSettingsChanged")
        }
    }

    #[test]
    fn test_update_server() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ServerSettings {
            api_server: Some("http://localhost:8080".to_string()),
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
        application.update_server(settings.clone());
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match result {
            ApplicationConfigEvent::ServerSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().server_settings);
            }
            _ => assert!(false, "expected ApplicationConfigEvent::ServerSettingsChanged")
        }
    }

    #[test]
    fn test_save() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut application = ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: Default::default(),
            callbacks: Default::default(),
        };
        let playback = PlaybackSettings {
            quality: Some(Quality::P1080),
            fullscreen: true,
            auto_play_next_episode_enabled: true,
        };
        let server = ServerSettings {
            api_server: Some("http://localhost:8080".to_string()),
        };

        application.update_server(server.clone());
        application.update_playback(playback.clone());
        application.save();

        let result = read_temp_dir_file_as_string(&temp_dir, DEFAULT_SETTINGS_FILENAME);
        assert!(!result.is_empty(), "expected a non-empty json file");

        let settings: PopcornSettings = serde_json::from_str(result.as_str()).unwrap();
        assert_eq!(server, settings.server_settings);
        assert_eq!(playback, settings.playback_settings);
    }
}