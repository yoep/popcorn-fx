use crate::core::config::{
    ConfigError, MediaTrackingSyncState, PlaybackSettings, PopcornProperties, PopcornSettings,
    ServerSettings, SubtitleSettings, TorrentSettings, Tracker, TrackingSettings, UiSettings,
};
use crate::core::storage::Storage;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

const DEFAULT_SETTINGS_FILENAME: &str = "settings.json";

/// The config result type for all results returned by the config package.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// The events that can occur within the application settings.
#[derive(Debug, Clone, Display)]
pub enum ApplicationConfigEvent {
    /// Invoked when the settings have been loaded or reloaded
    #[display("Settings have been loaded")]
    Loaded,
    /// Invoked when the settings have been saved
    #[display("Settings have been saved")]
    Saved,
    /// Invoked when any of the subtitle settings have been changed
    #[display("Subtitle settings have been changed")]
    SubtitleSettingsChanged(SubtitleSettings),
    /// Invoked when any of the torrent settings have been changed
    #[display("Torrent settings have been changed")]
    TorrentSettingsChanged(TorrentSettings),
    #[display("UI settings have been changed")]
    /// Invoked when the ui settings have been changed
    UiSettingsChanged(UiSettings),
    /// Invoked when the server settings have been changed
    #[display("Server settings have been changed")]
    ServerSettingsChanged(ServerSettings),
    /// Invoked when the playback settings have been changed
    #[display("Playback settings have been changed")]
    PlaybackSettingsChanged(PlaybackSettings),
    /// Invoked when the tracking settings have been changed
    #[display("Tracking settings have changed")]
    TrackingSettingsChanged(TrackingSettings),
}

/// The application properties & settings of Popcorn FX.
/// This is the main entry into the config data of [PopcornProperties] & [PopcornSettings].
///
/// The [PopcornProperties] are static options that don't change during the lifecycle of the application.
/// The [PopcornSettings] on the other hand might change during the application lifecycle
/// as it contains the user preferences.
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    inner: Arc<InnerApplicationConfig>,
}

impl ApplicationConfig {
    /// Creates a new [ApplicationConfig] with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `storage` - The [Storage] instance to use for storing and retrieving data.
    /// * `properties` - The static [PopcornProperties] of the application.
    /// * `settings` - The initial [PopcornSettings] of the application, if none given, defaults to [PopcornSettings::default].
    /// * `settings_save_on_change` - Whether to save the settings on change or not.
    pub fn new(
        storage: Storage,
        properties: PopcornProperties,
        settings: Option<PopcornSettings>,
        settings_save_on_change: bool,
    ) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerApplicationConfig {
            storage,
            properties,
            settings: RwLock::new(settings.unwrap_or(PopcornSettings::default())),
            settings_save_on_change,
            callbacks: MultiThreadedCallback::new(),
            command_sender,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver).await;
        });

        Self { inner }
    }

    /// Creates a new `ApplicationConfigBuilder` with which a new [ApplicationConfig] can be
    /// initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::config::{ApplicationConfig, PopcornProperties};
    ///
    /// let config = ApplicationConfig::builder()
    ///     .storage("storage-path") // This field is required and will panic when not set
    ///     .properties(PopcornProperties::default())
    ///     .build();
    /// ```
    pub fn builder() -> ApplicationConfigBuilder {
        ApplicationConfigBuilder::default()
    }

    /// The popcorn properties of the application.
    /// These are static and can't be changed during the lifetime of the application.
    pub fn properties(&self) -> PopcornProperties {
        self.properties_ref().clone()
    }

    /// Get a reference to the static application properties.
    pub fn properties_ref(&self) -> &PopcornProperties {
        &self.inner.properties
    }

    /// The popcorn user settings of the application.
    /// These are mutable and can be changed during the lifetime of the application.
    /// They're most of the time managed by the user based on preferences.
    pub async fn user_settings(&self) -> PopcornSettings {
        (*self.inner.settings.read().await).clone()
    }

    /// Read the user settings and get the specified value from the settings.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use popcorn_fx_core::core::config::ApplicationConfig;
    ///
    /// async fn example(config: &ApplicationConfig) -> bool {
    ///     config.user_settings_ref(|e| e.ui_settings.maximized).await
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a read guard to the settings and returns the desired value.
    ///
    /// # Returns
    ///
    /// It returns the value returned by the function.
    pub async fn user_settings_ref<O>(&self, f: impl FnOnce(&PopcornSettings) -> O) -> O {
        f(&*self.inner.settings.read().await)
    }

    /// Update the subtitle settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub async fn update_subtitle(&self, settings: SubtitleSettings) {
        let mut subtitle_settings: Option<SubtitleSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.subtitle_settings != settings {
                mutex.subtitle_settings = settings;
                subtitle_settings = Some(mutex.subtitle().clone());
                debug!("Subtitle settings have been updated");
            }
        }

        if let Some(settings) = subtitle_settings {
            self.inner
                .invoke(ApplicationConfigEvent::SubtitleSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        }
    }

    /// Update the torrent settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub async fn update_torrent(&self, settings: TorrentSettings) {
        let mut torrent_settings: Option<TorrentSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.torrent_settings != settings {
                mutex.torrent_settings = settings;
                torrent_settings = Some(mutex.torrent().clone());
                debug!("Torrent settings have been updated");
            }
        }

        if let Some(settings) = torrent_settings {
            self.inner
                .invoke(ApplicationConfigEvent::TorrentSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        }
    }

    /// Update the ui settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub async fn update_ui(&self, settings: UiSettings) {
        let mut ui_settings: Option<UiSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.ui_settings != settings {
                mutex.ui_settings = settings;
                ui_settings = Some(mutex.ui().clone());
                debug!("UI settings have been updated");
            }
        }

        if let Some(settings) = ui_settings {
            self.inner
                .invoke(ApplicationConfigEvent::UiSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        }
    }

    /// Update the api server settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub async fn update_server(&self, settings: ServerSettings) {
        let mut server_settings: Option<ServerSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.server_settings != settings {
                mutex.server_settings = settings;
                server_settings = Some(mutex.server().clone());
                debug!("Server settings have been updated");
            }
        }

        if let Some(settings) = server_settings {
            self.inner
                .invoke(ApplicationConfigEvent::ServerSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        }
    }

    /// Update the playback settings of the application.
    /// The update will be ignored if no fields have been changed.
    pub async fn update_playback(&self, settings: PlaybackSettings) {
        trace!("Updating playback settings");
        let mut playback_settings: Option<PlaybackSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.playback_settings != settings {
                mutex.playback_settings = settings;
                playback_settings = Some(mutex.playback().clone());
                debug!("Playback settings have been updated");
            }
        }

        if let Some(settings) = playback_settings {
            self.inner
                .invoke(ApplicationConfigEvent::PlaybackSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        }
    }

    /// Update the tracking settings of the application.
    /// This will update an individual tracker of the application without affecting any other trackers.
    pub async fn update_tracker(&self, name: &str, tracker: Tracker) {
        trace!("Updating tracker info of {}", name);
        let settings: TrackingSettings;
        {
            let mut mutex = self.inner.settings.write().await;
            mutex.tracking_mut().update(name, tracker);
            settings = mutex.tracking().clone();
        }
        debug!("Tracking settings of {} have been updated", name);

        self.inner
            .invoke(ApplicationConfigEvent::TrackingSettingsChanged(settings));
        if self.inner.settings_save_on_change {
            self.inner.send_command(ApplicationConfigCommand::Save);
        }
    }

    /// Update the state the currently configured tracker.
    pub async fn update_tracker_state(&self, state: MediaTrackingSyncState) {
        trace!("Updating the tracker state to {}", state);
        let settings: TrackingSettings;
        {
            let mut mutex = self.inner.settings.write().await;
            mutex.tracking_mut().update_state(state);
            settings = mutex.tracking().clone();
        }
        debug!("Tracking settings have been updated");

        self.inner
            .invoke(ApplicationConfigEvent::TrackingSettingsChanged(settings));
        if self.inner.settings_save_on_change {
            self.inner.send_command(ApplicationConfigCommand::Save);
        }
    }

    /// Remove a specific tracker from the application.
    /// This will only remove the specified tracker when present, it not, not callbacks will be triggered.
    pub async fn remove_tracker(&self, name: &str) {
        trace!("Removing tracker info of {}", name);
        let mut settings: Option<TrackingSettings> = None;
        {
            let mut mutex = self.inner.settings.write().await;
            if mutex.tracking_mut().remove(name) {
                settings = Some(mutex.tracking().clone());
            }
        }
        debug!("Tracking settings of {} have been updated", name);

        if let Some(settings) = settings {
            self.inner
                .invoke(ApplicationConfigEvent::TrackingSettingsChanged(settings));
            if self.inner.settings_save_on_change {
                self.inner.send_command(ApplicationConfigCommand::Save);
            }
        } else {
            trace!(
                "Tracker {} wasn't found, not triggering TrackingSettingsChanged callback",
                name
            );
        }
    }

    /// Reload the application config.
    pub fn reload(&self) {
        self.inner.send_command(ApplicationConfigCommand::Reload);
    }

    /// Save the application configuration to the storage device.
    pub async fn save(&self) {
        self.inner.save().await
    }
}

impl Callback<ApplicationConfigEvent> for ApplicationConfig {
    fn subscribe(&self) -> Subscription<ApplicationConfigEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<ApplicationConfigEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl PartialEq for ApplicationConfig {
    fn eq(&self, other: &Self) -> bool {
        let properties = self.properties_ref();
        let other_properties = other.properties_ref();

        *properties == *other_properties
    }
}

impl Drop for ApplicationConfig {
    fn drop(&mut self) {
        trace!("Application config is being dropped");
        if Arc::strong_count(&self.inner) <= 2 {
            self.inner.cancellation_token.cancel();
        }
    }
}

/// The builder for the [ApplicationConfig].
#[derive(Debug, Default)]
pub struct ApplicationConfigBuilder {
    storage: Option<Storage>,
    properties: Option<PopcornProperties>,
    settings: Option<PopcornSettings>,
    settings_save_on_change: Option<bool>,
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

    /// Sets whether to save the settings on change or not.
    pub fn settings_save_on_change(mut self, save_on_change: bool) -> Self {
        self.settings_save_on_change = Some(save_on_change);
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
        let properties = self
            .properties
            .or_else(|| Some(PopcornProperties::new_auto()))
            .unwrap();

        ApplicationConfig::new(
            storage,
            properties,
            self.settings,
            self.settings_save_on_change.unwrap_or(true),
        )
    }
}

#[derive(Debug, PartialEq)]
enum ApplicationConfigCommand {
    /// Save the application configuration to the storage.
    Save,
    /// Reload the application configuration settings from the storage.
    Reload,
}

#[derive(Debug)]
struct InnerApplicationConfig {
    /// The storage to use for reading the settings
    storage: Storage,
    /// The static application properties
    properties: PopcornProperties,
    /// The user settings for the application
    settings: RwLock<PopcornSettings>,
    /// Whether to save the settings on change
    settings_save_on_change: bool,
    /// The callbacks for this application config
    callbacks: MultiThreadedCallback<ApplicationConfigEvent>,
    /// The async command sender.
    command_sender: UnboundedSender<ApplicationConfigCommand>,
    cancellation_token: CancellationToken,
}

impl InnerApplicationConfig {
    /// Start the main loop of the application configuration.
    /// This will load the settings from the storage and start listening commands.
    async fn start(&self, mut command_receiver: UnboundedReceiver<ApplicationConfigCommand>) {
        {
            let mut settings = self.settings.write().await;
            if *settings == PopcornSettings::default() {
                match self
                    .storage
                    .options()
                    .serializer(DEFAULT_SETTINGS_FILENAME)
                    .read_async::<PopcornSettings>()
                    .await
                {
                    Ok(e) => {
                        *settings = e;
                        self.invoke(ApplicationConfigEvent::Loaded);
                    }
                    Err(e) => warn!(
                        "Failed to read settings from storage, using default settings instead, {}",
                        e
                    ),
                }
            }
        }

        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
            }
        }
        self.save().await;
        debug!("Application config main loop ended");
    }

    async fn handle_command(&self, command: ApplicationConfigCommand) {
        match command {
            ApplicationConfigCommand::Save => self.save().await,
            ApplicationConfigCommand::Reload => self.reload().await,
        }
    }

    async fn save(&self) {
        let settings = self.settings.read().await;
        trace!("Application settings are being saved, {:?}", settings);
        match self
            .storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write_async(&*settings)
            .await
        {
            Ok(_) => {
                info!("Application settings have been saved");
                self.invoke(ApplicationConfigEvent::Saved);
            }
            Err(e) => error!("Application settings failed to save, {}", e),
        }
    }

    async fn reload(&self) {
        trace!("Reloading the application configuration settings");
        match self
            .storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .read_async::<PopcornSettings>()
            .await
        {
            Ok(e) => {
                debug!("Application settings have been read from storage");
                let old_settings: PopcornSettings;
                let new_settings: PopcornSettings;

                {
                    let mut mutex = self.settings.write().await;
                    old_settings = mutex.clone();

                    *mutex = e;
                    new_settings = mutex.clone();
                    info!("Settings have been reloaded");
                }

                // start invoking events
                self.invoke(ApplicationConfigEvent::Loaded);

                if old_settings.subtitle_settings != new_settings.subtitle_settings {
                    self.invoke(ApplicationConfigEvent::SubtitleSettingsChanged(
                        new_settings.subtitle().clone(),
                    ));
                }
                if old_settings.torrent_settings != new_settings.torrent_settings {
                    self.invoke(ApplicationConfigEvent::TorrentSettingsChanged(
                        new_settings.torrent().clone(),
                    ))
                }
                if old_settings.ui_settings != new_settings.ui_settings {
                    self.invoke(ApplicationConfigEvent::UiSettingsChanged(
                        new_settings.ui().clone(),
                    ))
                }
                if old_settings.server_settings != new_settings.server_settings {
                    self.invoke(ApplicationConfigEvent::ServerSettingsChanged(
                        new_settings.server().clone(),
                    ))
                }
                if old_settings.playback_settings != new_settings.playback_settings {
                    self.invoke(ApplicationConfigEvent::PlaybackSettingsChanged(
                        new_settings.playback().clone(),
                    ))
                }
            }
            Err(e) => warn!("Failed to reload settings from storage, {}", e),
        }
    }

    fn invoke(&self, event: ApplicationConfigEvent) {
        self.callbacks.invoke(event);
    }

    fn send_command(&self, command: ApplicationConfigCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Application config failed to send command, {}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::{
        CleaningMode, DecorationType, Quality, SubtitleFamily, SubtitleSettings, UiScale,
    };
    use crate::core::media::Category;
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::testing::copy_test_file;
    use crate::{init_logger, recv_timeout};

    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_new_should_return_valid_instance() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let result = ApplicationConfig::builder().storage(temp_path).build();
        let expected_result = "https://api.opensubtitles.com/api/v1".to_string();

        assert_eq!(&expected_result, result.properties().subtitle().url())
    }

    #[tokio::test]
    async fn test_new_auto_should_read_settings() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let (tx, mut rx) = unbounded_channel();
        let application = ApplicationConfig::builder().storage(temp_path).build();
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
            tracking_settings: Default::default(),
        };

        let mut receiver = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let ApplicationConfigEvent::Loaded = &*event {
                    tx.send(()).unwrap();
                }
            }
        });

        let _ = recv_timeout!(&mut rx, Duration::from_millis(250));
        let result = application.user_settings().await;

        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_new_auto_settings_do_not_exist() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let expected_result = PopcornSettings::default();

        let result = application.user_settings().await;

        assert_eq!(expected_result, result)
    }

    #[ignore] // FIXME: this test is currently very unstable, as it's writing an empty settings.json file
    #[tokio::test]
    async fn test_reload_settings() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None); // copy the initial settings to the test dir
        let (tx, mut rx) = unbounded_channel();
        let application = ApplicationConfig::builder().storage(temp_path).build();
        application
            .inner
            .storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write_async(&PopcornSettings::default())
            .await
            .expect("expected the test file to have been written");

        let mut receiver = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    ApplicationConfigEvent::Loaded => tx.send((*event).clone()).unwrap(),
                    _ => {}
                }
            }
        });

        // wait for the initial load
        let _ = recv_timeout!(
            &mut rx,
            Duration::from_millis(750),
            "expected to receive the initial ApplicationConfigEvent::Loaded event"
        );

        // reload the settings
        application.reload();

        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(750),
            "expected to receive a ApplicationConfigEvent"
        );
        match result {
            ApplicationConfigEvent::Loaded => {}
            _ => assert!(false, "expected ApplicationConfigEvent::Loaded event"),
        }
    }

    #[tokio::test]
    async fn test_reload_subtitle_settings() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let (tx_loaded, mut rx_loaded) = unbounded_channel();
        copy_test_file(temp_path, "settings.json", None);
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let expected_result = SubtitleSettings {
            directory: "my-directory".to_string(),
            auto_cleaning_enabled: false,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::Arial,
            font_size: 24,
            decoration: DecorationType::None,
            bold: true,
        };

        let mut receiver = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    ApplicationConfigEvent::Loaded => tx_loaded.send(()).unwrap(),
                    ApplicationConfigEvent::SubtitleSettingsChanged(_) => {
                        tx.send((*event).clone()).unwrap();
                    }
                    _ => {}
                }
            }
        });

        // wait for the initial settings to be loaded
        let _ = recv_timeout!(
            &mut rx_loaded,
            Duration::from_millis(250),
            "expected the settings to have been loaded"
        );

        // modify the application settings in the storage
        application
            .inner
            .storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .write_async(&PopcornSettings {
                subtitle_settings: expected_result.clone(),
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
                tracking_settings: Default::default(),
            })
            .await
            .expect("expected the test file to have been written");

        // reload the application settings from the storage
        application.reload();

        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(250),
            "expected a subtitle settings change"
        );
        match result {
            ApplicationConfigEvent::SubtitleSettingsChanged(settings) => {
                assert_eq!(expected_result, settings)
            }
            _ => assert!(
                false,
                "expected ApplicationConfigEvent::SettingsLoaded event"
            ),
        }
    }

    #[tokio::test]
    async fn test_update_subtitle() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let directory = "/tmp/lorem/subtitles";
        let expected_settings = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: true,
            default_subtitle: SubtitleLanguage::Polish,
            font_family: SubtitleFamily::Arial,
            font_size: 22,
            decoration: DecorationType::None,
            bold: false,
        };
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let (tx, mut rx) = unbounded_channel();

        let mut reciever = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = reciever.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        application.update_subtitle(expected_settings.clone()).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        match result {
            ApplicationConfigEvent::SubtitleSettingsChanged(result) => {
                let settings = application.user_settings().await;
                assert_eq!(expected_settings, result);
                assert_eq!(expected_settings, settings.subtitle_settings);
            }
            _ => assert!(
                false,
                "expected ApplicationConfigEvent::SubtitleSettingsChanged"
            ),
        }
    }

    #[tokio::test]
    async fn test_update_torrent() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let directory = "/tmp/lorem/torrents";
        let settings = TorrentSettings {
            directory: PathBuf::from(directory),
            cleaning_mode: CleaningMode::Off,
            connections_limit: 100,
            download_rate_limit: 0,
            upload_rate_limit: 0,
        };
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let (tx, mut rx) = unbounded_channel();

        let mut reciever = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = reciever.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        application.update_torrent(settings.clone()).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        match result {
            ApplicationConfigEvent::TorrentSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().await.torrent_settings);
            }
            _ => assert!(
                false,
                "expected ApplicationConfigEvent::TorrentSettingsChanged"
            ),
        }
    }

    #[tokio::test]
    async fn test_update_ui() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = UiSettings {
            default_language: "en".to_string(),
            ui_scale: UiScale::new(1.2).unwrap(),
            start_screen: Category::Favorites,
            maximized: false,
            native_window_enabled: false,
        };
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let (tx, mut rx) = unbounded_channel();

        let mut reciever = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = reciever.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        application.update_ui(settings.clone()).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        match result {
            ApplicationConfigEvent::UiSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().await.ui_settings);
            }
            _ => assert!(false, "expected ApplicationConfigEvent::UiSettingsChanged"),
        }
    }

    #[tokio::test]
    async fn test_update_server() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ServerSettings {
            api_server: Some("http://localhost:8080".to_string()),
        };
        let application = ApplicationConfig::builder().storage(temp_path).build();
        let (tx, mut rx) = unbounded_channel();

        let mut reciever = application.subscribe();
        tokio::spawn(async move {
            while let Some(event) = reciever.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        application.update_server(settings.clone()).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(100));
        match result {
            ApplicationConfigEvent::ServerSettingsChanged(result) => {
                assert_eq!(settings, result);
                assert_eq!(settings, application.user_settings().await.server_settings);
            }
            _ => assert!(
                false,
                "expected ApplicationConfigEvent::ServerSettingsChanged"
            ),
        }
    }

    #[tokio::test]
    async fn test_save() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let playback = PlaybackSettings {
            quality: Some(Quality::P1080),
            fullscreen: true,
            auto_play_next_episode_enabled: true,
        };
        let server = ServerSettings {
            api_server: Some("http://localhost:8080".to_string()),
        };
        let application = ApplicationConfig::builder()
            .storage(temp_path)
            .settings_save_on_change(false)
            .build();

        application.update_server(server.clone()).await;
        application.update_playback(playback.clone()).await;
        application.save().await;

        let settings: PopcornSettings = application
            .inner
            .storage
            .options()
            .serializer(DEFAULT_SETTINGS_FILENAME)
            .read_async()
            .await
            .unwrap();
        assert_eq!(server, settings.server_settings);
        assert_eq!(playback, settings.playback_settings);
    }
}
