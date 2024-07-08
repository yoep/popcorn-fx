use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, info, trace};
use tokio::sync::Mutex;

use crate::core::config::ApplicationConfig;
use crate::core::storage::Storage;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;
use crate::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};

/// The callback to listen on events of the subtitle manager.
pub type SubtitleCallback = CoreCallback<SubtitleEvent>;

/// Represents events related to subtitles.
#[derive(Debug, Clone, Display)]
pub enum SubtitleEvent {
    /// Indicates a change in subtitle preference.
    #[display(fmt = "subtitle preference changed to {}", _0)]
    PreferenceChanged(SubtitlePreference),
}

/// Represents user preferences for subtitles.
#[repr(C)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum SubtitlePreference {
    /// Specifies a preferred subtitle language.
    #[display(fmt = "preferred language {}", _0)]
    Language(SubtitleLanguage),
    /// Indicates subtitles are disabled.
    #[display(fmt = "disabled")]
    Disabled,
}

#[async_trait]
pub trait SubtitleManager: Debug + Callbacks<SubtitleEvent> + Send + Sync {
    /// Retrieves the current subtitle preference.
    fn preference(&self) -> SubtitlePreference;

    /// Asynchronously retrieves the current subtitle preference.
    async fn preference_async(&self) -> SubtitlePreference;

    /// Updates the subtitle preference.
    ///
    /// # Arguments
    ///
    /// * `preference` - The new subtitle preference to set.
    fn update_preference(&self, preference: SubtitlePreference);

    /// Selects a subtitle from the available list or returns a default if none match the preference.
    ///
    /// # Arguments
    ///
    /// * `subtitles` - Available subtitle options to choose from.
    ///
    /// # Returns
    ///
    /// The selected subtitle information, or a default if none match the preference.
    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo;

    /// Resets the current selected subtitle information.
    fn reset(&self);

    /// Cleans up stored subtitle files.
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
    pub fn new(settings: Arc<ApplicationConfig>) -> Self {
        let instance = Arc::new(InnerSubtitleManager::new(settings));
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
    fn preference(&self) -> SubtitlePreference {
        self.inner.preference()
    }

    async fn preference_async(&self) -> SubtitlePreference {
        self.inner.preference_async().await
    }

    fn update_preference(&self, preference: SubtitlePreference) {
        self.inner.update_preference(preference);
    }

    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        self.inner.select_or_default(subtitles)
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
    preference: Arc<Mutex<SubtitlePreference>>,
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
            preference: Arc::new(Mutex::new(Self::default_preference(&settings))),
            callbacks: Default::default(),
            settings,
        }
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

    fn default_preference(settings: &Arc<ApplicationConfig>) -> SubtitlePreference {
        let preferred_language = settings
            .user_settings()
            .subtitle_settings
            .default_subtitle
            .clone();
        SubtitlePreference::Language(preferred_language)
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
    fn preference(&self) -> SubtitlePreference {
        block_in_place(self.preference_async())
    }

    async fn preference_async(&self) -> SubtitlePreference {
        let mutex = self.preference.lock().await;
        mutex.clone()
    }

    fn update_preference(&self, preference: SubtitlePreference) {
        let mut mutex = block_in_place(self.preference.lock());
        *mutex = preference;
    }

    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        trace!("Selecting subtitle out of {:?}", subtitles);
        let mut subtitle = SubtitleInfo::none();
        let preference = self.preference();

        if let SubtitlePreference::Language(language) = preference {
            if let Some(subtitle_info) = subtitles.iter().find(|e| e.language() == &language) {
                subtitle = subtitle_info.clone();
            } else {
                trace!("Subtitle preference language {} not found, using default subtitle language instead", language);
                subtitle = self
                    .find_for_default_subtitle_language(subtitles)
                    .or_else(|| self.find_for_interface_language(subtitles))
                    .unwrap_or(SubtitleInfo::none());
            }
        }

        debug!("Selected subtitle {:?}", &subtitle);
        subtitle
    }

    /// Reset the player to its default state for the next media playback.
    fn reset(&self) {
        self.update_preference(Self::default_preference(&self.settings));
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

    use tempfile::tempdir;

    use crate::core::config::{
        DecorationType, PopcornProperties, PopcornSettings, SubtitleFamily, SubtitleSettings,
        UiScale, UiSettings,
    };
    use crate::core::media::Category;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_update_preference_language() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Language(SubtitleLanguage::Dutch);
        let manager = DefaultSubtitleManager::new(settings);

        manager.update_preference(preference.clone());

        assert_eq!(preference, manager.preference())
    }

    #[test]
    fn test_update_preference_disabled() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Disabled;
        let manager = DefaultSubtitleManager::new(settings);

        manager.update_preference(preference.clone());

        assert_eq!(preference, manager.preference())
    }

    #[test]
    fn test_reset() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Language(SubtitleLanguage::Bulgarian);
        let manager = DefaultSubtitleManager::new(settings);

        manager.update_preference(preference.clone());
        manager.reset();

        assert_eq!(
            SubtitlePreference::Language(SubtitleLanguage::English),
            manager.preference()
        )
    }

    #[test]
    fn test_select_or_default_select_for_default_subtitle_language() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let manager = DefaultSubtitleManager::new(settings);
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
        let settings = default_settings(temp_path, true);
        settings.update_ui(UiSettings {
            default_language: "fr".to_string(),
            ui_scale: UiScale::new(1.0).unwrap(),
            start_screen: Category::Movies,
            maximized: false,
            native_window_enabled: false,
        });
        let manager = DefaultSubtitleManager::new(settings);
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
        let manager = DefaultSubtitleManager::new(settings);
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
                        default_subtitle: SubtitleLanguage::English,
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
