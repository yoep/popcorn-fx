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
    fn preferred_subtitle(&self) -> Option<SubtitleInfo>;

    fn preferred_language(&self) -> SubtitleLanguage;

    /// Checks if the subtitle has been disabled by the user.
    ///
    /// This function checks whether the subtitle is disabled by the user and returns `true` if it is disabled,
    /// or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the subtitle is disabled, `false` otherwise.
    fn is_disabled(&self) -> bool;

    async fn is_disabled_async(&self) -> bool;

    fn update_subtitle(&self, subtitle: SubtitleInfo);

    fn update_custom_subtitle(&self, subtitle_file: &str);

    fn disable_subtitle(&self);

    fn reset(&self);

    fn cleanup(&self);
}

/// The subtitle manager manages the subtitle for the [Media] item playbacks.
#[derive(Debug)]
pub struct DefaultSubtitleManager {
    /// The known info of the selected subtitle if applicable
    subtitle_info: Arc<Mutex<Option<SubtitleInfo>>>,
    /// The preferred language for the subtitle
    preferred_language: Arc<Mutex<SubtitleLanguage>>,
    /// Indicates if the subtitle has been disabled by the user
    disabled_by_user: Mutex<bool>,
    callbacks: CoreCallbacks<SubtitleEvent>,
    settings: Arc<ApplicationConfig>,
}

impl DefaultSubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    pub fn new(settings: Arc<ApplicationConfig>) -> Self {
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

impl Callbacks<SubtitleEvent> for DefaultSubtitleManager {
    fn add(&self, callback: CoreCallback<SubtitleEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl SubtitleManager for DefaultSubtitleManager {
    /// Retrieves the current preferred subtitle for [Media] item playbacks.
    ///
    /// # Returns
    ///
    /// The preferred [SubtitleInfo] if present.
    fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
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

        self.update_subtitle(SubtitleInfo::builder()
            .language(SubtitleLanguage::Custom)
            .files(vec![
                SubtitleFile::builder()
                    .file_id(1)
                    .name(filename)
                    .url(subtitle_uri)
                    .score(0.0)
                    .downloads(0)
                    .build()
            ])
            .build());
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
        let mut mutex_subtitle = block_in_place(self.subtitle_info.lock());
        let language_value = mutex_language.deref_mut();

        *language_value = SubtitleLanguage::None;
        self.update_disabled_state(false);

        if mutex_subtitle.is_some() {
            let _ = mutex_subtitle.take();
        }

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

impl Drop for DefaultSubtitleManager {
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
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt1111")
            .language(SubtitleLanguage::Croatian)
            .build();
        let manager = DefaultSubtitleManager::new(settings);

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
        let manager = DefaultSubtitleManager::new(settings);

        manager.add(Box::new(move |event| {
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
        let manager = DefaultSubtitleManager::new(settings);

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
        let manager = DefaultSubtitleManager::new(settings);

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
        let manager = DefaultSubtitleManager::new(settings);

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_subtitle(subtitle);
        manager.disable_subtitle();
        manager.reset();

        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(false, manager.is_disabled(), "expected the subtitle to not be disabled")
    }

    #[test]
    fn test_drop_cleanup_subtitles() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let manager = DefaultSubtitleManager::new(settings);
        let filepath = copy_test_file(temp_path, "example.srt", None);

        drop(manager);

        assert_eq!(false, PathBuf::from(filepath).exists(), "expected the file to have been removed");
        assert_eq!(true, PathBuf::from(temp_path).exists(), "expected the subtitle directory to not have been removed");
    }

    fn default_settings(temp_path: &str, auto_cleaning_enabled: bool) -> Arc<ApplicationConfig> {
        Arc::new(ApplicationConfig::builder()
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
            })
            .build())
    }
}