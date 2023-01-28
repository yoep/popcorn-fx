use std::ops::DerefMut;
use std::sync::Arc;

use log::info;
use tokio::sync::Mutex;

use crate::core::subtitles::language::SubtitleLanguage;

/// The subtitle manager manages the subtitle for the [Media] item playbacks.
#[derive(Debug)]
pub struct SubtitleManager {
    preferred_language: Arc<Mutex<SubtitleLanguage>>,
    custom_subtitle_file: Arc<Mutex<Option<String>>>,
}

impl SubtitleManager {
    /// The current preferred subtitle language for [Media] item playbacks.
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
        }else {
            None
        }
    }

    /// Update the active subtitle which will be used for the next [Media] playback.
    pub fn update_language(&self, preferred_language: SubtitleLanguage) {
        let arc = self.preferred_language.clone();
        let mut mutex = futures::executor::block_on(arc.lock());
        let value = mutex.deref_mut();

        *value = preferred_language;
        info!("Subtitle preferred language updated to {}", mutex)
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
        let mut mutex_language = futures::executor::block_on(arc.lock());
        let mut mutex_file = futures::executor::block_on(arc_file.lock());
        let value = mutex_language.deref_mut();

        *value = SubtitleLanguage::None;
        if mutex_file.is_some() {
            let _ = mutex_file.take();
        }

        info!("Subtitle has been reset for next media playback")
    }
}

impl Default for SubtitleManager {
    fn default() -> Self {
        Self {
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
    fn test_update_language() {
        init_logger();
        let language = SubtitleLanguage::English;
        let manager = SubtitleManager::default();

        manager.update_language(language.clone());
        let result = manager.preferred_language();

        assert_eq!(language, result)
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
        let language = SubtitleLanguage::Croatian;
        let manager = SubtitleManager::default();

        manager.update_custom_subtitle("my-subtitle.srt");
        manager.update_language(language.clone());
        manager.reset();

        assert_eq!(SubtitleLanguage::None, manager.preferred_language());
        assert_eq!(None, manager.custom_subtitle())
    }
}