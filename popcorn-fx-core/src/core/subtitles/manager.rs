use std::ops::DerefMut;
use std::sync::Arc;

use log::info;
use tokio::sync::Mutex;

use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;

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
}

impl SubtitleManager {
    /// The current preferred subtitle for [Media] item playbacks.
    ///
    /// It returns a reference of the preferred [SubtitleInfo] if present.
    pub fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
        let mutex = self.subtitle_info.blocking_lock();

        if mutex.is_some() {
            mutex.clone()
        } else {
            None
        }
    }

    /// The current preferred subtitle language for the [Media] item playback.
    ///
    /// It returns an owned instance of the preferred [SubtitleLanguage].
    pub fn preferred_language(&self) -> SubtitleLanguage {
        let arc = self.preferred_language.clone();
        let mutex = futures::executor::block_on(arc.lock());
        mutex.clone()
    }

    /// The configured custom subtitle filepath if one is present.
    ///
    /// It returns the subtitle filepath.
    pub fn custom_subtitle(&self) -> Option<String> {
        let mutex_file = self.custom_subtitle_file.blocking_lock();

        if mutex_file.is_some() {
            let filepath = mutex_file.as_ref().unwrap();
            Some(filepath.clone())
        } else {
            None
        }
    }

    /// Verify if the subtitle has been disabled by the user.
    pub fn is_disabled(&self) -> bool {
        *self.disabled_by_user.blocking_lock()
    }

    /// Update the [SubtitleInfo] for the next [Media] item playback.
    pub fn update_subtitle(&self, subtitle: SubtitleInfo) {
        let subtitle_text = subtitle.to_string();
        let language = subtitle.language().clone();

        self.update_subtitle_info(subtitle);
        self.update_language(language);
        self.update_disabled_state(false);
        info!("Subtitle has been updated to {}", subtitle_text);
    }

    /// Update the active subtitle to a custom selected subtitle file.
    ///
    /// * `subtitle_file`   - The custom subtitle filepath.
    pub fn update_custom_subtitle(&self, subtitle_file: &str) {
        let mut mutex = self.custom_subtitle_file.blocking_lock();
        let _ = mutex.insert(subtitle_file.to_string());

        self.update_language(SubtitleLanguage::Custom);
        self.update_disabled_state(false);
        info!("Subtitle custom file applied for {}", subtitle_file)
    }

    /// Disable the subtitle for the next video playback.
    /// This will make the `is_disabled()` fn return true.
    pub fn disable_subtitle(&self) {
        self.update_subtitle_info(SubtitleInfo::none());
        self.update_language(SubtitleLanguage::None);
        self.update_disabled_state(true);
        info!("Subtitle track has been disabled")
    }

    /// Reset the subtitle for the next [Media] item playback.
    pub fn reset(&self) {
        let mut mutex_language = self.preferred_language.blocking_lock();
        let mut mutex_file = self.custom_subtitle_file.blocking_lock();
        let mut mutex_subtitle = self.subtitle_info.blocking_lock();
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

    fn update_language(&self, preferred_language: SubtitleLanguage) {
        let arc = self.preferred_language.clone();
        let mut mutex = futures::executor::block_on(arc.lock());
        let value = mutex.deref_mut();
        let language_text = preferred_language.to_string();

        *value = preferred_language;
        info!("Subtitle language has been updated to {}", language_text);
    }

    fn update_subtitle_info(&self, subtitle: SubtitleInfo) {
        let mut mutex = self.subtitle_info.blocking_lock();
        let _ = mutex.insert(subtitle);
    }

    fn update_disabled_state(&self, new_state: bool) {
        let mut mutex = self.disabled_by_user.blocking_lock();
        let value = mutex.deref_mut();
        *value = new_state;
    }
}

impl Default for SubtitleManager {
    fn default() -> Self {
        Self {
            subtitle_info: Arc::new(Mutex::new(None)),
            preferred_language: Arc::new(Mutex::new(SubtitleLanguage::None)),
            custom_subtitle_file: Mutex::new(None),
            disabled_by_user: Mutex::new(false),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_update_subtitle() {
        init_logger();
        let subtitle = SubtitleInfo::new(
            "tt1111".to_string(),
            SubtitleLanguage::Croatian,
        );
        let manager = SubtitleManager::default();

        manager.disable_subtitle();
        manager.update_subtitle(subtitle.clone());
        let subtitle_result = manager.preferred_subtitle();
        let language_result = manager.preferred_language();

        assert_eq!(Some(subtitle), subtitle_result);
        assert_eq!(SubtitleLanguage::Croatian, language_result);
        assert_eq!(false, manager.is_disabled())
    }

    #[test]
    fn test_update_custom_subtitle() {
        init_logger();
        let filepath = "/home/lorem/ipsum.srt";
        let manager = SubtitleManager::default();

        manager.update_custom_subtitle(filepath);
        let result = manager.custom_subtitle();

        assert_eq!(Some(filepath.to_string()), result)
    }

    #[test]
    fn test_disable_subtitle() {
        init_logger();
        let manager = SubtitleManager::default();

        manager.disable_subtitle();
        let result = manager.is_disabled();

        assert!(result, "expected the subtitle to be disabled")
    }

    #[test]
    fn test_reset() {
        init_logger();
        let subtitle = SubtitleInfo::new(
            "tt121212".to_string(),
            SubtitleLanguage::Lithuanian,
        );
        let manager = SubtitleManager::default();

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_subtitle(subtitle);
        manager.disable_subtitle();
        manager.reset();

        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(None, manager.custom_subtitle());
        assert_eq!(false, manager.is_disabled(), "expected the subtitle to not be disabled")
    }
}