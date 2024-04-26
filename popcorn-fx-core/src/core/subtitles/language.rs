use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// The available languages
const LANGUAGES: [SubtitleLanguage; 36] = [
    SubtitleLanguage::None,
    SubtitleLanguage::Custom,
    SubtitleLanguage::Arabic,
    SubtitleLanguage::Bulgarian,
    SubtitleLanguage::Bosnian,
    SubtitleLanguage::Czech,
    SubtitleLanguage::Danish,
    SubtitleLanguage::German,
    SubtitleLanguage::ModernGreek,
    SubtitleLanguage::English,
    SubtitleLanguage::Spanish,
    SubtitleLanguage::Estonian,
    SubtitleLanguage::Basque,
    SubtitleLanguage::Persian,
    SubtitleLanguage::Finnish,
    SubtitleLanguage::French,
    SubtitleLanguage::Hebrew,
    SubtitleLanguage::Croatian,
    SubtitleLanguage::Hungarian,
    SubtitleLanguage::Indonesian,
    SubtitleLanguage::Italian,
    SubtitleLanguage::Lithuanian,
    SubtitleLanguage::Dutch,
    SubtitleLanguage::Norwegian,
    SubtitleLanguage::Polish,
    SubtitleLanguage::Portuguese,
    SubtitleLanguage::PortugueseBrazil,
    SubtitleLanguage::Romanian,
    SubtitleLanguage::Russian,
    SubtitleLanguage::Slovene,
    SubtitleLanguage::Serbian,
    SubtitleLanguage::Swedish,
    SubtitleLanguage::Thai,
    SubtitleLanguage::Turkish,
    SubtitleLanguage::Ukrainian,
    SubtitleLanguage::Vietnamese,
];

/// The supported subtitle languages.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubtitleLanguage {
    None = 0,
    Custom = 1,
    Arabic = 2,
    Bulgarian = 3,
    Bosnian = 4,
    Czech = 5,
    Danish = 6,
    German = 7,
    ModernGreek = 8,
    English = 9,
    Spanish = 10,
    Estonian = 11,
    Basque = 12,
    Persian = 13,
    Finnish = 14,
    French = 15,
    Hebrew = 16,
    Croatian = 17,
    Hungarian = 18,
    Indonesian = 19,
    Italian = 20,
    Lithuanian = 21,
    Dutch = 22,
    Norwegian = 23,
    Polish = 24,
    Portuguese = 25,
    PortugueseBrazil = 26,
    Romanian = 27,
    Russian = 28,
    Slovene = 29,
    Serbian = 30,
    Swedish = 31,
    Thai = 32,
    Turkish = 33,
    Ukrainian = 34,
    Vietnamese = 35,
}

impl SubtitleLanguage {
    /// Get the [SubtitleLanguage] for the given code.
    pub fn from_code(code: String) -> Option<Self> {
        LANGUAGES.iter().find(|e| e.code() == code).cloned()
    }

    /// The subtitle language identifier code.
    pub fn code(&self) -> String {
        match *self {
            SubtitleLanguage::None => "none".to_string(),
            SubtitleLanguage::Custom => "custom".to_string(),
            SubtitleLanguage::Arabic => "ar".to_string(),
            SubtitleLanguage::Bulgarian => "bg".to_string(),
            SubtitleLanguage::Bosnian => "bs".to_string(),
            SubtitleLanguage::Czech => "cs".to_string(),
            SubtitleLanguage::Danish => "da".to_string(),
            SubtitleLanguage::German => "de".to_string(),
            SubtitleLanguage::ModernGreek => "el".to_string(),
            SubtitleLanguage::English => "en".to_string(),
            SubtitleLanguage::Spanish => "es".to_string(),
            SubtitleLanguage::Estonian => "et".to_string(),
            SubtitleLanguage::Basque => "eu".to_string(),
            SubtitleLanguage::Persian => "fa".to_string(),
            SubtitleLanguage::Finnish => "fi".to_string(),
            SubtitleLanguage::French => "fr".to_string(),
            SubtitleLanguage::Hebrew => "he".to_string(),
            SubtitleLanguage::Croatian => "hr".to_string(),
            SubtitleLanguage::Hungarian => "hu".to_string(),
            SubtitleLanguage::Indonesian => "id".to_string(),
            SubtitleLanguage::Italian => "it".to_string(),
            SubtitleLanguage::Lithuanian => "lt".to_string(),
            SubtitleLanguage::Dutch => "nl".to_string(),
            SubtitleLanguage::Norwegian => "no".to_string(),
            SubtitleLanguage::Polish => "pl".to_string(),
            SubtitleLanguage::Portuguese => "pt".to_string(),
            SubtitleLanguage::PortugueseBrazil => "pt-br".to_string(),
            SubtitleLanguage::Romanian => "ro".to_string(),
            SubtitleLanguage::Russian => "ru".to_string(),
            SubtitleLanguage::Slovene => "sl".to_string(),
            SubtitleLanguage::Serbian => "sr".to_string(),
            SubtitleLanguage::Swedish => "sv".to_string(),
            SubtitleLanguage::Thai => "th".to_string(),
            SubtitleLanguage::Turkish => "tr".to_string(),
            SubtitleLanguage::Ukrainian => "uk".to_string(),
            SubtitleLanguage::Vietnamese => "vi".to_string(),
        }
    }

    /// The native text to display for the language.
    pub fn native_name(&self) -> String {
        match *self {
            SubtitleLanguage::None => "Disabled".to_string(),
            SubtitleLanguage::Custom => "Custom".to_string(),
            SubtitleLanguage::Arabic => "العربية".to_string(),
            SubtitleLanguage::Bulgarian => "Български".to_string(),
            SubtitleLanguage::Bosnian => "Bosanski jezik".to_string(),
            SubtitleLanguage::Czech => "Český".to_string(),
            SubtitleLanguage::Danish => "Dansk".to_string(),
            SubtitleLanguage::German => "Deutsch".to_string(),
            SubtitleLanguage::ModernGreek => "Ελληνικά".to_string(),
            SubtitleLanguage::English => "English".to_string(),
            SubtitleLanguage::Spanish => "Español".to_string(),
            SubtitleLanguage::Estonian => "Eesti".to_string(),
            SubtitleLanguage::Basque => "Euskara".to_string(),
            SubtitleLanguage::Persian => "فارسی".to_string(),
            SubtitleLanguage::Finnish => "Suomi".to_string(),
            SubtitleLanguage::French => "Français".to_string(),
            SubtitleLanguage::Hebrew => "עברית".to_string(),
            SubtitleLanguage::Croatian => "Hrvatski".to_string(),
            SubtitleLanguage::Hungarian => "Magyar".to_string(),
            SubtitleLanguage::Indonesian => "Bahasa Indonesia".to_string(),
            SubtitleLanguage::Italian => "Italiano".to_string(),
            SubtitleLanguage::Lithuanian => "lietuvių kalba".to_string(),
            SubtitleLanguage::Dutch => "Nederlands".to_string(),
            SubtitleLanguage::Norwegian => "Norsk".to_string(),
            SubtitleLanguage::Polish => "Polski".to_string(),
            SubtitleLanguage::Portuguese => "Português".to_string(),
            SubtitleLanguage::PortugueseBrazil => "Português (Brasil)".to_string(),
            SubtitleLanguage::Romanian => "română".to_string(),
            SubtitleLanguage::Russian => "русский язык".to_string(),
            SubtitleLanguage::Slovene => "slovenščina".to_string(),
            SubtitleLanguage::Serbian => "српски језик".to_string(),
            SubtitleLanguage::Swedish => "svenska".to_string(),
            SubtitleLanguage::Thai => "ไทย".to_string(),
            SubtitleLanguage::Turkish => "Türkçe".to_string(),
            SubtitleLanguage::Ukrainian => "українська".to_string(),
            SubtitleLanguage::Vietnamese => "Tiếng Việt".to_string(),
        }
    }
}

impl From<i32> for SubtitleLanguage {
    fn from(value: i32) -> Self {
        LANGUAGES
            .iter()
            .find(|e| ((*e).clone() as i32) == value)
            .cloned()
            .unwrap_or_else(|| panic!("Ordinal {} is out of range for SubtitleLanguage", value))
    }
}

impl Display for SubtitleLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl PartialOrd for SubtitleLanguage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ordinal = self.clone() as i32;
        let other_ordinal = other.clone() as i32;

        ordinal.partial_cmp(&other_ordinal)
    }
}

impl Ord for SubtitleLanguage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("expected a Ordering for SubtitleLanguage")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subtitle_language_from_code_should_return_expected_language() {
        let code = "en".to_string();

        let result = SubtitleLanguage::from_code(code);

        assert_eq!(SubtitleLanguage::English, result.unwrap())
    }

    #[test]
    fn test_subtitle_language_from_code_when_code_is_unknown_should_return_none() {
        let code = "lorem".to_string();

        let result = SubtitleLanguage::from_code(code);

        assert_eq!(true, result.is_none())
    }

    #[test]
    fn test_ordering() {
        let language1 = SubtitleLanguage::None;
        let language2 = SubtitleLanguage::Custom;

        assert_eq!(Ordering::Greater, language2.cmp(&language1))
    }
}
