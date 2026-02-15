use crate::core::config::ApplicationConfig;
use crate::core::storage::Storage;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// The callback to listen on events of the subtitle manager.
pub type SubtitleCallback = Subscription<SubtitleEvent>;

/// Represents events related to subtitles.
#[derive(Debug, Clone, Display)]
pub enum SubtitleEvent {
    #[display("subtitle preference changed to {}", _0)]
    PreferenceChanged(SubtitlePreference),
}

/// Represents user preferences for subtitles.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum SubtitlePreference {
    /// Specifies a preferred subtitle language.
    #[display("preferred language {}", _0)]
    Language(SubtitleLanguage),
    /// Indicates subtitles are disabled.
    #[display("disabled")]
    Disabled,
}

#[async_trait]
pub trait SubtitleManager: Debug + Callback<SubtitleEvent> + Send + Sync {
    /// Retrieves the current subtitle preference.
    async fn preference(&self) -> SubtitlePreference;

    /// Asynchronously retrieves the current subtitle preference.
    async fn preference_async(&self) -> SubtitlePreference;

    /// Updates the subtitle preference.
    ///
    /// # Arguments
    ///
    /// * `preference` - The new subtitle preference to set.
    async fn update_preference(&self, preference: SubtitlePreference);

    /// Selects a subtitle from the available list or returns a default if none match the preference.
    ///
    /// # Arguments
    ///
    /// * `subtitles` - Available subtitle options to choose from.
    ///
    /// # Returns
    ///
    /// The selected subtitle information, or a default if none match the preference.
    async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo;

    /// Resets the current selected subtitle information.
    async fn reset(&self);

    /// Cleans up stored subtitle files.
    async fn cleanup(&self);
}

/// The subtitle manager manages subtitles for media item playbacks.
#[derive(Debug, Clone)]
pub struct DefaultSubtitleManager {
    inner: Arc<InnerSubtitleManager>,
}

impl DefaultSubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    pub async fn new(settings: ApplicationConfig) -> Self {
        let inner = Arc::new(InnerSubtitleManager::new(settings).await);

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start().await;
        });

        Self { inner }
    }
}

impl Callback<SubtitleEvent> for DefaultSubtitleManager {
    fn subscribe(&self) -> Subscription<SubtitleEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<SubtitleEvent>) {
        self.inner.callbacks.subscribe_with(subscriber);
    }
}

#[async_trait]
impl SubtitleManager for DefaultSubtitleManager {
    async fn preference(&self) -> SubtitlePreference {
        self.inner.preference().await
    }

    async fn preference_async(&self) -> SubtitlePreference {
        self.inner.preference().await
    }

    async fn update_preference(&self, preference: SubtitlePreference) {
        self.inner.update_preference(preference).await
    }

    async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        self.inner.select_or_default(subtitles).await
    }

    async fn reset(&self) {
        self.inner.reset().await
    }

    async fn cleanup(&self) {
        self.inner.cleanup().await
    }
}

impl Drop for DefaultSubtitleManager {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerSubtitleManager {
    /// The known info of the selected subtitle if applicable.
    preference: Arc<Mutex<SubtitlePreference>>,
    /// Callbacks for handling subtitle events.
    callbacks: MultiThreadedCallback<SubtitleEvent>,
    /// Application settings.
    settings: ApplicationConfig,
    cancellation_token: CancellationToken,
}

impl InnerSubtitleManager {
    /// Creates a new `SubtitleManager` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the manager.
    async fn new(settings: ApplicationConfig) -> Self {
        let preference = Self::default_preference(&settings).await;
        Self {
            preference: Arc::new(Mutex::new(preference)),
            callbacks: MultiThreadedCallback::new(),
            settings,
            cancellation_token: Default::default(),
        }
    }

    async fn start(&self) {
        self.cancellation_token.cancelled().await;
        self.on_drop().await;
    }

    /// Find the subtitle for the default configured subtitle language.
    /// This uses the [SubtitleSettings::default_subtitle] setting.
    async fn find_for_default_subtitle_language(
        &self,
        subtitles: &[SubtitleInfo],
    ) -> Option<SubtitleInfo> {
        let subtitle_language = self
            .settings
            .user_settings_ref(|e| e.subtitle().default_subtitle().clone())
            .await;

        subtitles
            .iter()
            .find(|e| e.language() == &subtitle_language)
            .map(|e| e.clone())
    }

    /// Find the subtitle for the interface language.
    /// This uses the [UiSettings::default_language] setting.
    async fn find_for_interface_language(
        &self,
        subtitles: &[SubtitleInfo],
    ) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings().await;
        let language = settings.ui().default_language();

        subtitles
            .iter()
            .find(|e| &e.language().code() == language)
            .map(|e| e.clone())
    }

    async fn preference(&self) -> SubtitlePreference {
        (*self.preference.lock().await).clone()
    }

    async fn update_preference(&self, preference: SubtitlePreference) {
        *self.preference.lock().await = preference;
    }

    async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        trace!("Selecting subtitle out of {:?}", subtitles);
        let mut subtitle: Option<SubtitleInfo> = None;
        let preference = self.preference().await;

        if let SubtitlePreference::Language(language) = preference {
            if let Some(subtitle_info) = subtitles.iter().find(|e| e.language() == &language) {
                subtitle = Some(subtitle_info.clone());
            } else {
                trace!("Subtitle preference language {} not found, using default subtitle language instead", language);
                subtitle = self.find_for_default_subtitle_language(subtitles).await;

                if subtitle.is_none() {
                    subtitle = self.find_for_interface_language(subtitles).await;
                }
            }
        }

        debug!("Selected subtitle {:?}", &subtitle);
        subtitle.unwrap_or(SubtitleInfo::none())
    }

    /// Reset the player to its default state for the next media playback.
    async fn reset(&self) {
        let preference = Self::default_preference(&self.settings).await;
        self.update_preference(preference).await;
        info!("Subtitle has been reset for next media playback")
    }

    /// Clean up the subtitle directory by removing all files.
    async fn cleanup(&self) {
        let path = self
            .settings
            .user_settings_ref(|e| e.subtitle_settings.directory())
            .await;
        let absolute_path = path.to_str().expect("expected a valid path");

        debug!("Cleaning subtitle directory {}", absolute_path);
        if let Err(e) = Storage::clean_directory(path.as_path()) {
            error!("Failed to clean subtitle directory, {}", e);
        } else {
            info!("Subtitle directory {} has been cleaned", absolute_path);
        }
    }

    async fn on_drop(&self) {
        let auto_cleaning_enabled = self
            .settings
            .user_settings_ref(|e| e.subtitle().auto_cleaning_enabled)
            .await;
        if auto_cleaning_enabled {
            self.cleanup().await
        } else {
            trace!("Skipping subtitle directory cleaning")
        }
    }

    async fn default_preference(settings: &ApplicationConfig) -> SubtitlePreference {
        let preferred_language = settings
            .user_settings_ref(|e| e.subtitle_settings.default_subtitle.clone())
            .await;
        SubtitlePreference::Language(preferred_language)
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
    use crate::testing::copy_test_file;
    use crate::{assert_timeout, init_logger};

    use super::*;

    #[tokio::test]
    async fn test_update_preference_language() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Language(SubtitleLanguage::Dutch);
        let manager = DefaultSubtitleManager::new(settings).await;

        manager.update_preference(preference.clone()).await;

        let result = manager.preference().await;
        assert_eq!(preference, result)
    }

    #[tokio::test]
    async fn test_update_preference_disabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Disabled;
        let manager = DefaultSubtitleManager::new(settings).await;

        manager.update_preference(preference.clone()).await;

        let result = manager.preference().await;
        assert_eq!(preference, result)
    }

    #[tokio::test]
    async fn test_reset() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, false);
        let preference = SubtitlePreference::Language(SubtitleLanguage::Bulgarian);
        let manager = DefaultSubtitleManager::new(settings).await;

        manager.update_preference(preference.clone()).await;
        manager.reset().await;

        let result = manager.preference().await;
        assert_eq!(
            SubtitlePreference::Language(SubtitleLanguage::English),
            result
        )
    }

    #[tokio::test]
    async fn test_select_or_default_select_for_default_subtitle_language() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let manager = DefaultSubtitleManager::new(settings).await;
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("lorem")
            .language(SubtitleLanguage::English)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles).await;

        assert_eq!(subtitle_info, result)
    }

    #[tokio::test]
    async fn test_select_or_default_select_for_interface_language() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        settings
            .update_ui(UiSettings {
                default_language: "fr".to_string(),
                ui_scale: UiScale::new(1.0).unwrap(),
                start_screen: Category::Movies,
                maximized: false,
                native_window_enabled: false,
            })
            .await;
        let manager = DefaultSubtitleManager::new(settings).await;
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("ipsum")
            .language(SubtitleLanguage::French)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles).await;

        assert_eq!(subtitle_info, result)
    }

    #[tokio::test]
    async fn test_drop_cleanup_subtitles() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_settings(temp_path, true);
        let manager = DefaultSubtitleManager::new(settings).await;
        let filepath = copy_test_file(temp_path, "example.srt", None);

        drop(manager);

        assert_timeout!(
            Duration::from_millis(250),
            !PathBuf::from(filepath.as_str()).exists(),
            "expected the file to have been removed"
        );
        assert_eq!(
            true,
            PathBuf::from(temp_path).exists(),
            "expected the subtitle directory to not have been removed"
        );
    }

    fn default_settings(temp_path: &str, auto_cleaning_enabled: bool) -> ApplicationConfig {
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
            .build()
    }
}
