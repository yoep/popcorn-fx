use std::ops::DerefMut;
use std::sync::Arc;

use derive_more::Display;
use log::{debug, error, info, trace};
use tokio::sync::Mutex;

use crate::core::{block_in_place, Callbacks, CoreCallback, CoreCallbacks};
use crate::core::config::ApplicationConfig;
use crate::core::storage::Storage;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;

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

/// The subtitle manager manages the subtitle for the [Media] item playbacks.
#[derive(Debug)]
pub struct SubtitleManager {
    /// The known info of the selected subtitle if applicable
    subtitle_info: Arc<Mutex<Option<SubtitleInfo>>>,
    /// The preferred language for the subtitle
    preferred_language: Arc<Mutex<SubtitleLanguage>>,
    /// The filepath to the custom selected subtitle file
    custom_subtitle_file: Mutex<Option<String>>,
    /// Indicates if the subtitle has been disabled by the user
    disabled_by_user: Mutex<bool>,
    callbacks: CoreCallbacks<SubtitleEvent>,
    settings: Arc<Mutex<ApplicationConfig>>,
}

impl SubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    pub fn new(settings: Arc<Mutex<ApplicationConfig>>) -> Self {
        Self {
            subtitle_info: Arc::new(Mutex::new(None)),
            preferred_language: Arc::new(Mutex::new(SubtitleLanguage::None)),
            custom_subtitle_file: Mutex::new(None),
            disabled_by_user: Mutex::new(false),
            callbacks: Default::default(),
            settings,
        }
    }

    /// Retrieves the current preferred subtitle for [Media] item playbacks.
    ///
    /// # Returns
    ///
    /// The preferred [SubtitleInfo] if present.
    pub fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
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
    pub fn preferred_language(&self) -> SubtitleLanguage {
        let arc = self.preferred_language.clone();
        let mutex = futures::executor::block_on(arc.lock());
        *mutex
    }

    /// Retrieves the configured custom subtitle filepath if one is present.
    ///
    /// # Returns
    ///
    /// The subtitle filepath, if available.
    pub fn custom_subtitle(&self) -> Option<String> {
        let mutex_file = self.custom_subtitle_file.blocking_lock();

        if mutex_file.is_some() {
            let filepath = mutex_file.as_ref().unwrap();
            Some(filepath.clone())
        } else {
            None
        }
    }

    /// Checks if the subtitle has been disabled by the user.
    ///
    /// This function checks whether the subtitle is disabled by the user and returns `true` if it is disabled,
    /// or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the subtitle is disabled, `false` otherwise.
    pub fn is_disabled(&self) -> bool {
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
    pub async fn is_disabled_async(&self) -> bool {
        *self.disabled_by_user.lock().await
    }

    /// Updates the [SubtitleInfo] for the next [Media] item playback.
    ///
    /// # Arguments
    ///
    /// * `subtitle` - The new subtitle information.
    pub fn update_subtitle(&self, subtitle: SubtitleInfo) {
        let subtitle_text = subtitle.to_string();
        let language = subtitle.language().clone();

        self.update_subtitle_info(subtitle);
        self.update_language(language);
        self.update_disabled_state(false);
        info!("Subtitle has been updated to {}", subtitle_text);
    }

    /// Updates the active subtitle to a custom selected subtitle file.
    ///
    /// # Arguments
    ///
    /// * `subtitle_file` - The custom subtitle filepath.
    pub fn update_custom_subtitle(&self, subtitle_file: &str) {
        let mut mutex = self.custom_subtitle_file.blocking_lock();
        let _ = mutex.insert(subtitle_file.to_string());

        self.update_language(SubtitleLanguage::Custom);
        self.update_disabled_state(false);
        info!("Subtitle custom file applied for {}", subtitle_file)
    }

    /// Disables the subtitle for the next video playback.
    /// This will make the `is_disabled()` function return `true`
    pub fn disable_subtitle(&self) {
        self.update_subtitle_info(SubtitleInfo::none());
        self.update_language(SubtitleLanguage::None);
        self.update_disabled_state(true);
        info!("Subtitle track has been disabled")
    }

    /// Resets the subtitle for the next [Media] item playback.
    pub fn reset(&self) {
        let mut mutex_language = block_in_place(self.preferred_language.lock());
        let mut mutex_file = block_in_place(self.custom_subtitle_file.lock());
        let mut mutex_subtitle = block_in_place(self.subtitle_info.lock());
        let language_value = mutex_language.deref_mut();

        *language_value = SubtitleLanguage::None;
        self.update_disabled_state(false);

        if mutex_file.is_some() {
            let _ = mutex_file.take();
        }
        if mutex_subtitle.is_some() {
            let _ = mutex_subtitle.take();
        }

        info!("Subtitle has been reset for next media playback")
    }

    /// Registers a new callback listener for the [SubtitleEvent]s.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to register.
    pub fn register(&self, callback: SubtitleCallback) {
        self.callbacks.add(callback);
    }

    /// Cleans the subtitle directory.
    ///
    /// This operation removes all subtitle files from the file system.
    ///
    /// # Safety
    ///
    /// This method performs file system operations and may have side effects. Use with caution.
    pub fn cleanup(&self) {
        let mutex = self.settings.blocking_lock();
        let path = mutex.settings.subtitle_settings.directory();
        let absolute_path = path.to_str().unwrap();

        debug!("Cleaning subtitle directory {}", absolute_path);
        if let Err(e) = Storage::clean_directory(path.as_path()) {
            error!("Failed to clean subtitle directory, {}", e);
        } else {
            info!("Subtitle directory {} has been cleaned", absolute_path);
        }
    }

    fn update_language(&self, preferred_language: SubtitleLanguage) {
        let arc = self.preferred_language.clone();
        let mut mutex = futures::executor::block_on(arc.lock());
        let value = mutex.deref_mut();
        let language_text = preferred_language.to_string();

        *value = preferred_language;
        info!("Subtitle language has been updated to {}", language_text);
        self.callbacks.invoke(SubtitleEvent::PreferredLanguageChanged(mutex.clone()));
    }

    fn update_subtitle_info(&self, subtitle: SubtitleInfo) {
        let mut mutex = block_in_place(self.subtitle_info.lock());
        let _ = mutex.insert(subtitle);
        self.callbacks.invoke(SubtitleEvent::SubtitleInfoChanged(mutex.clone()));
    }

    fn update_disabled_state(&self, new_state: bool) {
        let mut mutex = block_in_place(self.disabled_by_user.lock());
        let value = mutex.deref_mut();
        *value = new_state;
    }
}

impl Drop for SubtitleManager {
    fn drop(&mut self) {
        let mutex = self.settings.blocking_lock();
        let settings = mutex.user_settings().subtitle();

        if *settings.auto_cleaning_enabled() {
            drop(mutex);
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

    use crate::core::config::{DecorationType, PopcornProperties, PopcornSettings, SubtitleFamily, SubtitleSettings};
    use crate::core::subtitles::language::SubtitleLanguage::English;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_update_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let subtitle = SubtitleInfo::new(
            "tt1111".to_string(),
            SubtitleLanguage::Croatian,
        );
        let manager = SubtitleManager::new(settings);

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
        let subtitle = SubtitleInfo::new(
            "tt1234555".to_string(),
            SubtitleLanguage::Spanish,
        );
        let (tx_info, rx_info) = channel();
        let (tx_lang, rx_lang) = channel();
        let manager = SubtitleManager::new(settings);

        manager.register(Box::new(move |event| {
            match event {
                SubtitleEvent::SubtitleInfoChanged(info) => tx_info.send(info).unwrap(),
                SubtitleEvent::PreferredLanguageChanged(lang) => tx_lang.send(lang).unwrap(),
            }
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
        let manager = SubtitleManager::new(settings);

        manager.update_custom_subtitle(filepath);
        let result = manager.custom_subtitle();

        assert_eq!(Some(filepath.to_string()), result)
    }

    #[test]
    fn test_disable_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let manager = SubtitleManager::new(settings);

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
        let subtitle = SubtitleInfo::new(
            "tt121212".to_string(),
            SubtitleLanguage::Lithuanian,
        );
        let manager = SubtitleManager::new(settings);

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_subtitle(subtitle);
        manager.disable_subtitle();
        manager.reset();

        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(None, manager.custom_subtitle());
        assert_eq!(false, manager.is_disabled(), "expected the subtitle to not be disabled")
    }

    #[test]
    fn test_drop_cleanup_subtitles() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let manager = SubtitleManager::new(settings);
        let filepath = copy_test_file(temp_path, "example.srt", None);

        drop(manager);

        assert_eq!(false, PathBuf::from(filepath).exists(), "expected the file to have been removed");
        assert_eq!(true, PathBuf::from(temp_path).exists(), "expected the subtitle directory to not have been removed");
    }

    fn default_settings(temp_path: &str, auto_cleaning_enabled: bool) -> Arc<Mutex<ApplicationConfig>> {
        Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
            },
            settings: PopcornSettings {
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
            },
            callbacks: Default::default(),
        }))
    }
}