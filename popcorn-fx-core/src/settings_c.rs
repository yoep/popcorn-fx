use std::os::raw::c_char;

use log::trace;

use crate::{from_c_string, into_c_string};
use crate::core::config::{ApplicationConfigEvent, DecorationType, PopcornSettings, SubtitleFamily, SubtitleSettings};
use crate::core::subtitles::language::SubtitleLanguage;

/// The C callback for the setting events.
pub type ApplicationConfigCallbackC = extern "C" fn(ApplicationConfigEventC);

/// The C compatible application events.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ApplicationConfigEventC {
    /// Indicates that the settings have been changed
    SettingsLoaded,
    /// Indicates that the subtitle settings have been changed
    SubtitleSettingsChanged(SubtitleSettingsC),
}

impl From<ApplicationConfigEvent> for ApplicationConfigEventC {
    fn from(value: ApplicationConfigEvent) -> Self {
        match value {
            ApplicationConfigEvent::SettingsLoaded => ApplicationConfigEventC::SettingsLoaded,
            ApplicationConfigEvent::SubtitleSettingsChanged(settings) => ApplicationConfigEventC::SubtitleSettingsChanged(SubtitleSettingsC::from(&settings))
        }
    }
}

/// The C compatible application settings.
#[repr(C)]
#[derive(Debug)]
pub struct PopcornSettingsC {
    /// The subtitle settings of the application
    pub subtitle_settings: SubtitleSettingsC,
}

impl From<&PopcornSettings> for PopcornSettingsC {
    fn from(value: &PopcornSettings) -> Self {
        trace!("Converting PopcornSettings to C for {:?}", value);
        Self {
            subtitle_settings: SubtitleSettingsC::from(value.subtitle()),
        }
    }
}

/// The C compatible subtitle settings.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct SubtitleSettingsC {
    /// The directory path for storing subtitles
    pub directory: *const c_char,
    /// Indicates if the subtitle directory will be cleaned
    /// when the application is closed
    pub auto_cleaning: bool,
    /// The default selected subtitle language
    pub default_subtitle: SubtitleLanguage,
    /// The subtitle font to use
    pub font_family: SubtitleFamily,
    /// The subtitle font size to use
    pub font_size: u32,
    /// The subtitle rendering decoration type
    pub decoration: DecorationType,
    /// Indicates if the subtitle should be rendered in a bold font
    pub bold: bool
}

impl From<&SubtitleSettings> for SubtitleSettingsC {
    fn from(value: &SubtitleSettings) -> Self {
        Self {
            directory: into_c_string(value.directory.clone()),
            auto_cleaning: value.auto_cleaning_enabled,
            default_subtitle: value.default_subtitle,
            font_family: value.font_family,
            font_size: value.font_size,
            decoration: value.decoration,
            bold: value.bold,
        }
    }
}

impl From<SubtitleSettingsC> for SubtitleSettings {
    fn from(value: SubtitleSettingsC) -> Self {
        Self {
            directory: from_c_string(value.directory),
            auto_cleaning_enabled: value.auto_cleaning,
            default_subtitle: value.default_subtitle,
            font_family: value.font_family,
            font_size: value.font_size,
            decoration: value.decoration,
            bold: value.bold,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::SubtitleFamily;
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::from_c_string;

    use super::*;

    #[test]
    fn test_from_application_settings_event() {
        let loaded_event = ApplicationConfigEvent::SettingsLoaded;

        let loaded_result = ApplicationConfigEventC::from(loaded_event);

        assert_eq!(ApplicationConfigEventC::SettingsLoaded, loaded_result);
    }

    #[test]
    fn test_from_subtitle_settings() {
        let directory = "/var/lorem/ipsum";
        let font_family = SubtitleFamily::Verdana;
        let subtitle_language = SubtitleLanguage::French;
        let settings = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: false,
            default_subtitle: subtitle_language.clone(),
            font_family: font_family.clone(),
            font_size: 28,
            decoration: DecorationType::Outline,
            bold: true,
        };

        let result = SubtitleSettingsC::from(&settings);

        assert_eq!(directory.to_string(), from_c_string(result.directory));
        assert_eq!(false, result.auto_cleaning);
        assert_eq!(subtitle_language, result.default_subtitle);
        assert_eq!(font_family, result.font_family);
        assert_eq!(28, result.font_size);
        assert_eq!(DecorationType::Outline, result.decoration);
        assert_eq!(true, result.bold);
    }

    #[test]
    fn test_from_subtitle_settings_c() {
        let directory = "/var/lorem/ipsum/dolor";
        let font_size = 32;
        let settings = SubtitleSettingsC {
            directory: into_c_string(directory.to_string()),
            auto_cleaning: true,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::ComicSans,
            font_size,
            decoration: DecorationType::OpaqueBackground,
            bold: true,
        };
        let expected_result = SubtitleSettings {
            directory: directory.to_string(),
            auto_cleaning_enabled: true,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::ComicSans,
            font_size,
            decoration: DecorationType::OpaqueBackground,
            bold: true,
        };

        let result = SubtitleSettings::from(settings);

        assert_eq!(expected_result, result)
    }
}