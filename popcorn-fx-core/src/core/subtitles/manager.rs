use std::ops::DerefMut;
use std::sync::Arc;

use log::info;
use tokio::sync::Mutex;

use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::model::SubtitleInfo;

/// The subtitle manager manages the subtitle for the [Media] item playbacks.
#[derive(Debug)]
pub struct SubtitleManager {
    subtitle_info: Arc<Mutex<Option<SubtitleInfo>>>,
    preferred_language: Arc<Mutex<SubtitleLanguage>>,
    custom_subtitle_file: Arc<Mutex<Option<String>>>,
}

impl SubtitleManager {
    /// The current preferred subtitle for [Media] item playbacks.
    ///
    /// It returns a reference of the preferred [SubtitleInfo] if present.
    pub fn preferred_subtitle(&self) -> Option<SubtitleInfo> {
        let arc = self.subtitle_info.clone();
        let mutex = futures::executor::block_on(arc.lock());

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
        let arc_file = self.custom_subtitle_file.clone();
        let mutex_file = futures::executor::block_on(arc_file.lock());

        if mutex_file.is_some() {
            let filepath = mutex_file.as_ref().unwrap();
            Some(filepath.clone())
        } else {
            None
        }
    }

    /// Update the [SubtitleInfo] for the next [Media] item playback.
    pub fn update_subtitle(&self, subtitle: SubtitleInfo) {
        let arc = self.subtitle_info.clone();
        let mut mutex = futures::executor::block_on(arc.lock());
        let subtitle_text = subtitle.to_string();
        let language = subtitle.language().clone();

        let _ = mutex.insert(subtitle);
        self.update_language(language);
        info!("Subtitle has been updated to {}", subtitle_text);
    }

    /// Update the active subtitle to a custom selected subtitle file.
    ///
    /// * `subtitle_file`   - The custom subtitle filepath.
    pub fn update_custom_subtitle(&self, subtitle_file: &str) {
        let arc_language = self.preferred_language.clone();
        let arc_file = self.custom_subtitle_file.clone();
        let mut mutex_language = futures::executor::block_on(arc_language.lock());
        let mut mutex_file = futures::executor::block_on(arc_file.lock());
        let value_language = mutex_language.deref_mut();

        *value_language = SubtitleLanguage::Custom;
        let _ = mutex_file.insert(subtitle_file.to_string());
        info!("Subtitle custom file applied for {}", subtitle_file)
    }

    /// Reset the subtitle for the next [Media] item playback.
    pub fn reset(&self) {
        let arc = self.preferred_language.clone();
        let arc_file = self.custom_subtitle_file.clone();
        let arc_subtitle = self.subtitle_info.clone();
        let mut mutex_language = futures::executor::block_on(arc.lock());
        let mut mutex_file = futures::executor::block_on(arc_file.lock());
        let mut mutex_subtitle = futures::executor::block_on(arc_subtitle.lock());
        let value = mutex_language.deref_mut();

        *value = SubtitleLanguage::None;
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
}

impl Default for SubtitleManager {
    fn default() -> Self {
        Self {
            subtitle_info: Arc::new(Mutex::new(None)),
            preferred_language: Arc::new(Mutex::new(SubtitleLanguage::None)),
            custom_subtitle_file: Arc::new(Mutex::new(None)),
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

        manager.update_subtitle(subtitle.clone());
        let subtitle_result = manager.preferred_subtitle();
        let language_result = manager.preferred_language();

        assert_eq!(Some(subtitle), subtitle_result);
        assert_eq!(SubtitleLanguage::Croatian, language_result)
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
    fn test_reset() {
        init_logger();
        let subtitle = SubtitleInfo::new(
            "tt121212".to_string(),
            SubtitleLanguage::Lithuanian,
        );
        let manager = SubtitleManager::default();

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_subtitle(subtitle);
        manager.reset();

        assert_eq!(None, manager.preferred_subtitle());
        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(None, manager.custom_subtitle())
    }
}