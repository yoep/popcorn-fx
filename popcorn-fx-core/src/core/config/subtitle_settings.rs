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
const DEFAULT_FONT_SIZE: fn() -> u32 = || 28;
const DEFAULT_DECORATION: fn() -> DecorationType = || DecorationType::Outline;
const DEFAULT_BOLD: fn() -> bool = || true;

/// The subtitle settings of the application.
/// These are the subtitle preferences of the user.
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "directory: {}, auto_cleaning_enabled: {}, default_subtitle: {}", directory, auto_cleaning_enabled, default_subtitle)]
pub struct SubtitleSettings {
    /// The subtitle directory where the subtitle files will be stored
    #[serde(default = "DEFAULT_DIRECTORY")]
    pub directory: String,
    /// Enable automatic cleanup of the subtitle directory
    /// This will clean the subtitles when the application instance is being disposed
    #[serde(default = "DEFAULT_AUTO_CLEANING")]
    pub auto_cleaning_enabled: bool,
    /// The default subtitle to select for media playbacks, if available
    #[serde(default = "DEFAULT_SUBTITLE_LANGUAGE")]
    pub default_subtitle: SubtitleLanguage,
    /// The font family to use for rendering subtitles
    #[serde(default = "DEFAULT_SUBTITLE_FAMILY")]
    pub font_family: SubtitleFamily,
    /// The size of the rendered subtitles
    #[serde(default = "DEFAULT_FONT_SIZE")]
    pub font_size: u32,
    /// The subtitle rendering type
    #[serde(default = "DEFAULT_DECORATION")]
    pub decoration: DecorationType,
    /// The subtitle should be rendered in a bold font
    #[serde(default = "DEFAULT_BOLD")]
    pub bold: bool,
}

impl SubtitleSettings {
    pub fn new(directory: Option<String>, auto_cleaning_enabled: Option<bool>,
               default_subtitle: Option<SubtitleLanguage>, font_family: Option<SubtitleFamily>,
               font_size: Option<u32>, decoration: Option<DecorationType>, bold: Option<bool>) -> Self {
        Self {
            directory: directory.or_else(|| Some(DEFAULT_DIRECTORY())).unwrap(),
            auto_cleaning_enabled: auto_cleaning_enabled.or_else(|| Some(DEFAULT_AUTO_CLEANING())).unwrap(),
            default_subtitle: default_subtitle.or_else(|| Some(DEFAULT_SUBTITLE_LANGUAGE())).unwrap(),
            font_family: font_family.or_else(|| Some(DEFAULT_SUBTITLE_FAMILY())).unwrap(),
            font_size: font_size.or_else(|| Some(DEFAULT_FONT_SIZE())).unwrap(),
            decoration: decoration.or_else(|| Some(DEFAULT_DECORATION())).unwrap(),
            bold: bold.or_else(|| Some(DEFAULT_BOLD())).unwrap(),
        }
    }

    /// The directory storing the subtitles
    pub fn directory(&self) -> PathBuf {
        PathBuf::from(&self.directory)
    }

    /// Indicates if the subtitles will be cleaned on closure of the application
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
            font_size: DEFAULT_FONT_SIZE(),
            decoration: DEFAULT_DECORATION(),
            bold: DEFAULT_BOLD(),
        }
    }
}

/// The supported subtitle fonts to use for rendering subtitles.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubtitleFamily {
    Arial = 0,
    ComicSans = 1,
    Georgia = 2,
    Tahoma = 3,
    TrebuchetMs = 4,
    Verdana = 5,
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

/// The decoration to apply to the subtitle during rendering.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecorationType {
    None = 0,
    Outline = 1,
    OpaqueBackground = 2,
    SeeThroughBackground = 3
}

#[cfg(test)]
mod test {
    use crate::core::config::{SubtitleFamily, SubtitleSettings};
    use crate::core::config::subtitle_settings::{DEFAULT_AUTO_CLEANING, DEFAULT_BOLD, DEFAULT_DECORATION, DEFAULT_FONT_SIZE, DEFAULT_SUBTITLE_FAMILY, DEFAULT_SUBTITLE_LANGUAGE};

    #[test]
    fn test_subtitle_new_use_defaults() {
        let directory = "/tmp/subtitles";
        let expected_result = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: DEFAULT_AUTO_CLEANING(),
            default_subtitle: DEFAULT_SUBTITLE_LANGUAGE(),
            font_family: DEFAULT_SUBTITLE_FAMILY(),
            font_size: DEFAULT_FONT_SIZE(),
            decoration: DEFAULT_DECORATION(),
            bold: DEFAULT_BOLD(),
        };

        let result = SubtitleSettings::new(
            Some(directory.to_string()),
            None,
            None,
            None,
            None,
            None,
            None
        );

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_family() {
        let tm = SubtitleFamily::TrebuchetMs.family();
        let verdana = SubtitleFamily::Verdana.family();

        assert_eq!("Trebuchet MS".to_string(), tm);
        assert_eq!("Verdana".to_string(), verdana);
    }
}
