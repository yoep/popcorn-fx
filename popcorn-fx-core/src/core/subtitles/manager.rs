use std::fmt::Debug;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace};
use tokio::sync::Mutex;

use crate::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use crate::core::config::ApplicationConfig;
use crate::core::events::{DEFAULT_ORDER, Event, EventPublisher};
use crate::core::storage::Storage;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;
use crate::core::subtitles::SubtitleFile;

/// The callback to listen on events of the subtitle manager.
pub type SubtitleCallback = CoreCallback<SubtitleEvent>;

/// The events of the subtitle manager.
#[derive(Debug, Clone, Display)]
pub enum SubtitleEvent {
    /// Invoked when the preferred [SubtitleInfo] is changed.
    ///
    /// * The new subtitle information.
    #[display(fmt = "Subtitle info changed to {:?}", _0)]
    SubtitleInfoChanged(Option<SubtitleInfo>),
    /// Invoked when the preferred [SubtitleLanguage] is changed.
    ///
    /// * The new preferred subtitle language
    #[display(fmt = "Preferred subtitle language changed to {}", _0)]
    PreferredLanguageChanged(SubtitleLanguage),
}

#[async_trait]
pub trait SubtitleManager: Debug + Callbacks<SubtitleEvent> + Send + Sync {
    /// Retrieves the preferred subtitle.
    ///
    /// # Returns
    ///
    /// The preferred subtitle as an `Option<SubtitleInfo>`.
    fn preferred_subtitle(&self) -> Option<SubtitleInfo>;

    /// Retrieves the preferred language for subtitles.
    ///
    /// # Returns
    ///
    /// The preferred language for subtitles.
    fn preferred_language(&self) -> SubtitleLanguage;

    /// Checks if the subtitle has been disabled by the user.
    ///
    /// # Returns
    ///
    /// - `true` if the subtitle is disabled.
    /// - `false` if the subtitle is enabled.
    fn is_disabled(&self) -> bool;

    /// Asynchronously checks if the subtitle has been disabled by the user.
    ///
    /// # Returns
    ///
    /// - `true` if the subtitle is disabled.
    /// - `false` if the subtitle is enabled.
    async fn is_disabled_async(&self) -> bool;

    /// Updates the current subtitle with the provided subtitle information.
    fn update_subtitle(&self, subtitle: SubtitleInfo);

    /// Updates the subtitle with the custom subtitle file.
    fn update_custom_subtitle(&self, subtitle_file: &str);
    
    /// Select one of the available subtitles.
    ///
    /// * `subtitles` - The available subtitle slice to pick from.
    ///
    /// # Returns
    /// 
    /// It returns the default [SubtitleInfo::none] when the preferred subtitle is not present.
    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo;

    /// Disables the subtitle on behalf of the user.
    /// To undo this action, call [reset].
    fn disable_subtitle(&self);

    /// Resets the current selected subtitle information.
    fn reset(&self);

    /// Cleans the stored subtitle files.
    fn cleanup(&self);
}

/// The subtitle manager manages subtitles for media item playbacks.
#[derive(Debug)]
pub struct DefaultSubtitleManager {
    inner: Arc<InnerSubtitleManager>,
}

impl DefaultSubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    pub fn new(settings: Arc<ApplicationConfig>, event_publisher: Arc<EventPublisher>) -> Self {
        let instance = Arc::new(InnerSubtitleManager::new(settings));

        let event_inner = instance.clone();
        event_publisher.register(
            Box::new(move |event| {
                if let Event::PlayerStopped(_) = &event {
                    event_inner.on_player_stopped();
                }

                Some(event)
            }),
            DEFAULT_ORDER,
        );

        Self { inner: instance }
    }
}

impl Callbacks<SubtitleEvent> for DefaultSubtitleManager {
    fn add(&self, callback: CoreCallback<SubtitleEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
    }
}

#[async_trait]
impl SubtitleManager for DefaultSubtitleManager {
    fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
        self.inner.preferred_subtitle()
    }

    fn preferred_language(&self) -> SubtitleLanguage {
        self.inner.preferred_language()
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    async fn is_disabled_async(&self) -> bool {
        self.inner.is_disabled_async().await
    }

    fn update_subtitle(&self, subtitle: SubtitleInfo) {
        self.inner.update_subtitle(subtitle)
    }

    fn update_custom_subtitle(&self, subtitle_file: &str) {
        self.inner.update_custom_subtitle(subtitle_file)
    }

    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        self.inner.select_or_default(subtitles)
    }
    
    fn disable_subtitle(&self) {
        self.inner.disable_subtitle()
    }

    fn reset(&self) {
        self.inner.reset()
    }

    fn cleanup(&self) {
        self.inner.cleanup()
    }
}

#[derive(Debug)]
struct InnerSubtitleManager {
    /// The known info of the selected subtitle if applicable.
    subtitle_info: Arc<Mutex<Option<SubtitleInfo>>>,
    /// The preferred language for the subtitle.
    preferred_language: Arc<Mutex<SubtitleLanguage>>,
    /// Indicates if the subtitle has been disabled by the user.
    disabled_by_user: Mutex<bool>,
    /// Callbacks for handling subtitle events.
    callbacks: CoreCallbacks<SubtitleEvent>,
    /// Application settings.
    settings: Arc<ApplicationConfig>,
}

impl InnerSubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    fn new(settings: Arc<ApplicationConfig>) -> Self {
        Self {
            subtitle_info: Arc::new(Mutex::new(None)),
            preferred_language: Arc::new(Mutex::new(SubtitleLanguage::None)),
            disabled_by_user: Mutex::new(false),
            callbacks: Default::default(),
            settings,
        }
    }

    fn update_language(&self, preferred_language: SubtitleLanguage) {
        let arc = self.preferred_language.clone();
        let mut mutex = futures::executor::block_on(arc.lock());
        let value = mutex.deref_mut();
        let language_text = preferred_language.to_string();

        *value = preferred_language;
        info!("Subtitle language has been updated to {}", language_text);
        self.callbacks
            .invoke(SubtitleEvent::PreferredLanguageChanged(mutex.clone()));
    }

    fn update_subtitle_info(&self, subtitle: SubtitleInfo) {
        trace!("Updating subtitle info to {:?}", subtitle);
        let event_value: Option<SubtitleInfo>;

        {
            let mut mutex = block_in_place(self.subtitle_info.lock());
            let _ = mutex.insert(subtitle);
            debug!("Subtitle info has been updated to {:?}", mutex);
            event_value = mutex.clone();
        }

        self.callbacks
            .invoke(SubtitleEvent::SubtitleInfoChanged(event_value));
    }

    fn update_disabled_state(&self, new_state: bool) {
        let mut mutex = block_in_place(self.disabled_by_user.lock());
        let value = mutex.deref_mut();
        *value = new_state;
    }

    fn reset_subtitle_info(&self) {
        let mut mutex = block_in_place(self.subtitle_info.lock());
        let _ = mutex.take();
        trace!("Subtitle info has been reset");
    }

    fn on_player_stopped(&self) {
        // only reset the subtitle info as we might need the preferred language
        // for the next playback
        self.reset_subtitle_info();
    }

    /// Find the subtitle for the default configured subtitle language.
    /// This uses the [SubtitleSettings::default_subtitle] setting.
    fn find_for_default_subtitle_language(
        &self,
        subtitles: &[SubtitleInfo],
    ) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings();
        let subtitle_language = settings.subtitle().default_subtitle();

        subtitles
            .iter()
            .find(|e| e.language() == subtitle_language)
            .map(|e| e.clone())
    }

    /// Find the subtitle for the interface language.
    /// This uses the [UiSettings::default_language] setting.
    fn find_for_interface_language(&self, subtitles: &[SubtitleInfo]) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings();
        let language = settings.ui().default_language();

        subtitles
            .iter()
            .find(|e| &e.language().code() == language)
            .map(|e| e.clone())
    }
}

impl Callbacks<SubtitleEvent> for InnerSubtitleManager {
    fn add(&self, callback: CoreCallback<SubtitleEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl SubtitleManager for InnerSubtitleManager {
    /// Retrieves the current preferred subtitle for [Media] item playbacks.
    ///
    /// # Returns
    ///
    /// The preferred [SubtitleInfo] if present.
    fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
        trace!("Retrieving preferred subtitle from subtitle manager");
        let mutex = block_in_place(self.subtitle_info.lock());

        if mutex.is_some() {
            mutex.clone()
        } else {
            None
        }
    }

    /// Retrieves the current preferred subtitle language for the [Media] item playback.
    ///
    /// # Returns
    ///
    /// The preferred [SubtitleLanguage].
    fn preferred_language(&self) -> SubtitleLanguage {
        let arc = self.preferred_language.clone();
        let mutex = futures::executor::block_on(arc.lock());
        *mutex
    }

    /// Checks if the subtitle has been disabled by the user.
    ///
    /// This function checks whether the subtitle is disabled by the user and returns `true` if it is disabled,
    /// or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the subtitle is disabled, `false` otherwise.
    fn is_disabled(&self) -> bool {
        block_in_place(self.is_disabled_async())
    }

    /// Asynchronously checks if the subtitle has been disabled by the user.
    ///
    /// This asynchronous function checks whether the subtitle is disabled by the user and returns `true` if it is disabled,
    /// or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the subtitle is disabled, `false` otherwise.
    async fn is_disabled_async(&self) -> bool {
        self.disabled_by_user.lock().await.clone()
    }

    /// Updates the [SubtitleInfo] for the next [Media] item playback.
    ///
    /// # Arguments
    ///
    /// * `subtitle` - The new subtitle information.
    fn update_subtitle(&self, subtitle: SubtitleInfo) {
        let subtitle_text = subtitle.to_string();
        let language = subtitle.language().clone();

        self.update_subtitle_info(subtitle);
        self.update_language(language);
        self.update_disabled_state(false);
        info!("Subtitle has been updated to {}", subtitle_text);
    }

    /// Update the player to use a custom subtitle file.
    ///
    /// # Arguments
    ///
    /// * `subtitle_uri` - The uri of the custom subtitle file.
    fn update_custom_subtitle(&self, subtitle_uri: &str) {
        debug!("Updating to custom subtitle file {}", subtitle_uri);
        let filename = Path::new(subtitle_uri)
            .file_name()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string())
            .unwrap_or(String::new());

        self.update_subtitle(
            SubtitleInfo::builder()
                .language(SubtitleLanguage::Custom)
                .files(vec![SubtitleFile::builder()
                    .file_id(1)
                    .name(filename)
                    .url(subtitle_uri)
                    .score(0.0)
                    .downloads(0)
                    .build()])
                .build(),
        );
    }

    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        trace!("Selecting subtitle out of {:?}", subtitles);
        let subtitle = self
            .find_for_default_subtitle_language(subtitles)
            .or_else(|| self.find_for_interface_language(subtitles))
            .unwrap_or(SubtitleInfo::none());
        debug!("Selected subtitle {:?}", &subtitle);
        subtitle
    }

    /// Disable the subtitle track.
    fn disable_subtitle(&self) {
        self.update_subtitle_info(SubtitleInfo::none());
        self.update_language(SubtitleLanguage::None);
        self.update_disabled_state(true);
        info!("Subtitle track has been disabled")
    }

    /// Reset the player to its default state for the next media playback.
    fn reset(&self) {
        let mut mutex_language = block_in_place(self.preferred_language.lock());
        let language_value = mutex_language.deref_mut();

        *language_value = SubtitleLanguage::None;
        self.update_disabled_state(false);
        self.reset_subtitle_info();

        info!("Subtitle has been reset for next media playback")
    }

    /// Clean up the subtitle directory by removing all files.
    fn cleanup(&self) {
        let settings = self.settings.user_settings();
        let path = settings.subtitle_settings.directory();
        let absolute_path = path.to_str().unwrap();

        debug!("Cleaning subtitle directory {}", absolute_path);
        if let Err(e) = Storage::clean_directory(path.as_path()) {
            error!("Failed to clean subtitle directory, {}", e);
        } else {
            info!("Subtitle directory {} has been cleaned", absolute_path);
        }
    }
}

impl Drop for InnerSubtitleManager {
    fn drop(&mut self) {
        let settings = self.settings.user_settings();
        let subtitle_settings = settings.subtitle();

        if *subtitle_settings.auto_cleaning_enabled() {
            self.cleanup()
        } else {
            trace!("Skipping subtitle directory cleaning")
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::core::config::{DecorationType, PopcornProperties, PopcornSettings, SubtitleFamily, SubtitleSettings, UiScale, UiSettings};
    use crate::core::events::{LOWEST_ORDER, PlayerStoppedEvent};
    use crate::core::media::Category;
    use crate::core::subtitles::language::SubtitleLanguage::English;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_player_stopped_event() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt1111")
            .language(SubtitleLanguage::French)
            .build();
        let (tx, rx) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher.clone());

        event_publisher.register(
            Box::new(move |_| {
                tx.send(()).unwrap();
                None
            }),
            LOWEST_ORDER,
        );

        manager.update_subtitle(subtitle);
        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: "http://localhost/my-video".to_string(),
            media: None,
            time: Some(12000),
            duration: Some(47000),
        }));

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected to receive the player stopped event");
        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::French, manager.preferred_language());
    }

    #[test]
    fn test_update_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt1111")
            .language(SubtitleLanguage::Croatian)
            .build();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);

        manager.disable_subtitle();
        manager.update_subtitle(subtitle.clone());
        let subtitle_result = manager.preferred_subtitle();
        let language_result = manager.preferred_language();

        assert_eq!(Some(subtitle), subtitle_result);
        assert_eq!(SubtitleLanguage::Croatian, language_result);
        assert_eq!(false, manager.is_disabled())
    }

    #[test]
    fn test_subtitle_info_changed() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt1234555")
            .language(SubtitleLanguage::Spanish)
            .build();
        let (tx_info, rx_info) = channel();
        let (tx_lang, rx_lang) = channel();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);

        manager.add(Box::new(move |event| match event {
            SubtitleEvent::SubtitleInfoChanged(info) => tx_info.send(info).unwrap(),
            SubtitleEvent::PreferredLanguageChanged(lang) => tx_lang.send(lang).unwrap(),
        }));
        manager.update_subtitle(subtitle.clone());

        let info = rx_info.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(Some(subtitle), info);

        let lang = rx_lang.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(SubtitleLanguage::Spanish, lang);
    }

    #[test]
    fn test_update_custom_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let filepath = "/home/lorem/ipsum.srt";
        let expected_result = SubtitleInfo::builder()
            .language(SubtitleLanguage::Custom)
            .files(vec![SubtitleFile::builder()
                .file_id(1)
                .name("ipsum.srt")
                .url(filepath)
                .score(0.0)
                .downloads(0)
                .build()])
            .build();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);

        manager.update_custom_subtitle(filepath);
        let result = manager.preferred_subtitle();

        assert_eq!(Some(expected_result), result);
    }

    #[test]
    fn test_disable_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);

        manager.disable_subtitle();
        let result = manager.is_disabled();

        assert!(result, "expected the subtitle to be disabled")
    }

    #[test]
    fn test_reset() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt121212")
            .language(SubtitleLanguage::Lithuanian)
            .build();
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_subtitle(subtitle);
        manager.disable_subtitle();
        manager.reset();

        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(
            false,
            manager.is_disabled(),
            "expected the subtitle to not be disabled"
        )
    }

    #[test]
    fn test_select_or_default_select_for_default_subtitle_language() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("lorem")
            .language(SubtitleLanguage::English)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles);

        assert_eq!(subtitle_info, result)
    }

    #[test]
    fn test_select_or_default_select_for_interface_language() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut settings = default_settings(temp_path, true);
        settings.update_ui(UiSettings {
            default_language: "fr".to_string(),
            ui_scale: UiScale::new(1.0).unwrap(),
            start_screen: Category::Movies,
            maximized: false,
            native_window_enabled: false,
        });
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("ipsum")
            .language(SubtitleLanguage::French)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles);

        assert_eq!(subtitle_info, result)
    }

    #[test]
    fn test_drop_cleanup_subtitles() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultSubtitleManager::new(settings, event_publisher);
        let filepath = copy_test_file(temp_path, "example.srt", None);

        drop(manager);

        assert_eq!(
            false,
            PathBuf::from(filepath).exists(),
            "expected the file to have been removed"
        );
        assert_eq!(
            true,
            PathBuf::from(temp_path).exists(),
            "expected the subtitle directory to not have been removed"
        );
    }

    fn default_settings(temp_path: &str, auto_cleaning_enabled: bool) -> Arc<ApplicationConfig> {
        Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .properties(PopcornProperties::default())
                .settings(PopcornSettings {
                    subtitle_settings: SubtitleSettings {
                        directory: temp_path.to_string(),
                        auto_cleaning_enabled,
                        default_subtitle: English,
                        font_family: SubtitleFamily::Arial,
                        font_size: 28,
                        decoration: DecorationType::None,
                        bold: false,
                    },
                    ui_settings: Default::default(),
                    server_settings: Default::default(),
                    torrent_settings: Default::default(),
                    playback_settings: Default::default(),
                    tracking_settings: Default::default(),
                })
                .build(),
        )
    }
}
