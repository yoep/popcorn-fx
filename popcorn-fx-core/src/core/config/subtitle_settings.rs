use std::path::PathBuf;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::core::config::DEFAULT_HOME_DIRECTORY;
use crate::core::subtitles::language::SubtitleLanguage;

const DEFAULT_SUBTITLE_DIRECTORY_NAME: &str = "subtitles";
const DEFAULT_DIRECTORY: fn() -> String = || {
    home::home_dir()
        .map(|e| e
            .join(DEFAULT_HOME_DIRECTORY)
            .join(DEFAULT_SUBTITLE_DIRECTORY_NAME))
        .map(|e| e.into_os_string().into_string().unwrap())
        .expect("Home directory should exist")
};
const DEFAULT_AUTO_CLEANING: fn() -> bool = || true;
const DEFAULT_SUBTITLE_LANGUAGE: fn() -> SubtitleLanguage = || SubtitleLanguage::None;
const DEFAULT_SUBTITLE_FAMILY: fn() -> SubtitleFamily = || SubtitleFamily::Arial;

#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "directory: {}, auto_cleaning_enabled: {}, default_subtitle: {}", directory, auto_cleaning_enabled, default_subtitle)]
pub struct SubtitleSettings {
    #[serde(default = "DEFAULT_DIRECTORY")]
    directory: String,
    #[serde(default = "DEFAULT_AUTO_CLEANING")]
    auto_cleaning_enabled: bool,
    #[serde(default = "DEFAULT_SUBTITLE_LANGUAGE")]
    default_subtitle: SubtitleLanguage,
    #[serde(default = "DEFAULT_SUBTITLE_FAMILY")]
    font_family: SubtitleFamily,
}

impl SubtitleSettings {
    pub fn new(directory: String, auto_cleaning_enabled: bool, default_subtitle: SubtitleLanguage, font_family: SubtitleFamily) -> Self {
        Self { directory, auto_cleaning_enabled, default_subtitle, font_family }
    }

    pub fn directory(&self) -> PathBuf {
        PathBuf::from(&self.directory)
    }

    pub fn auto_cleaning_enabled(&self) -> &bool {
        &self.auto_cleaning_enabled
    }

    pub fn default_subtitle(&self) -> &SubtitleLanguage {
        &self.default_subtitle
    }
}

impl Default for SubtitleSettings {
    fn default() -> Self {
        Self {
            directory: DEFAULT_DIRECTORY(),
            auto_cleaning_enabled: DEFAULT_AUTO_CLEANING(),
            default_subtitle: DEFAULT_SUBTITLE_LANGUAGE(),
            font_family: DEFAULT_SUBTITLE_FAMILY(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubtitleFamily {
    Arial,
    ComicSans,
    Georgia,
    Tahoma,
    TrebuchetMs,
    Verdana,
}

impl SubtitleFamily {
    /// Retrieve the family name.
    pub fn family(&self) -> String {
        match self {
            SubtitleFamily::Arial => "Arial".to_string(),
            SubtitleFamily::ComicSans => "Comic Sans MS".to_string(),
            SubtitleFamily::Georgia => "Georgia".to_string(),
            SubtitleFamily::Tahoma => "Tahoma".to_string(),
            SubtitleFamily::TrebuchetMs => "Trebuchet MS".to_string(),
            SubtitleFamily::Verdana => "Verdana".to_string(),
        }
    }
}
