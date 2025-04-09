use crate::ipc::proto;
use crate::ipc::proto::subtitle::{subtitle, subtitle_preference};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use popcorn_fx_core::core::subtitles::{SubtitleFile, SubtitlePreference};

impl From<&SubtitleInfo> for subtitle::Info {
    fn from(value: &SubtitleInfo) -> Self {
        Self {
            imdb_id: value.imdb_id().map(|e| e.clone()),
            language: subtitle::Language::from(value.language()).into(),
            files: value
                .files()
                .map(|files| {
                    files
                        .iter()
                        .map(|file| subtitle::info::File::from(file))
                        .collect()
                })
                .unwrap_or(Vec::with_capacity(0)),
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitleFile> for subtitle::info::File {
    fn from(value: &SubtitleFile) -> Self {
        Self {
            file_id: *value.file_id(),
            name: value.name().to_string(),
            url: value.url().to_string(),
            score: *value.score(),
            downloads: *value.downloads(),
            quality: value.quality().map(|e| *e),
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitlePreference> for proto::subtitle::SubtitlePreference {
    fn from(value: &SubtitlePreference) -> Self {
        let mut preference = Self::new();

        match value {
            SubtitlePreference::Language(language) => {
                preference.preference = subtitle_preference::Preference::LANGUAGE.into();
                preference.language = Some(subtitle::Language::from(language).into());
            }
            SubtitlePreference::Disabled => {
                preference.preference = subtitle_preference::Preference::DISABLED.into();
            }
        }

        preference
    }
}

impl From<&SubtitleLanguage> for subtitle::Language {
    fn from(value: &SubtitleLanguage) -> Self {
        match value {
            SubtitleLanguage::None => Self::NONE,
            SubtitleLanguage::Custom => Self::CUSTOM,
            SubtitleLanguage::Arabic => Self::ARABIC,
            SubtitleLanguage::Bulgarian => Self::BULGARIAN,
            SubtitleLanguage::Bosnian => Self::BOSNIAN,
            SubtitleLanguage::Czech => Self::CZECH,
            SubtitleLanguage::Danish => Self::DANISH,
            SubtitleLanguage::German => Self::GERMAN,
            SubtitleLanguage::ModernGreek => Self::MODERN_GREEK,
            SubtitleLanguage::English => Self::ENGLISH,
            SubtitleLanguage::Spanish => Self::SPANISH,
            SubtitleLanguage::Estonian => Self::ESTONIAN,
            SubtitleLanguage::Basque => Self::BASQUE,
            SubtitleLanguage::Persian => Self::PERSIAN,
            SubtitleLanguage::Finnish => Self::FINNISH,
            SubtitleLanguage::French => Self::FRENCH,
            SubtitleLanguage::Hebrew => Self::HEBREW,
            SubtitleLanguage::Croatian => Self::CROATIAN,
            SubtitleLanguage::Hungarian => Self::HUNGARIAN,
            SubtitleLanguage::Indonesian => Self::INDONESIAN,
            SubtitleLanguage::Italian => Self::ITALIAN,
            SubtitleLanguage::Lithuanian => Self::LITHUANIAN,
            SubtitleLanguage::Dutch => Self::DUTCH,
            SubtitleLanguage::Norwegian => Self::NORWEGIAN,
            SubtitleLanguage::Polish => Self::POLISH,
            SubtitleLanguage::Portuguese => Self::PORTUGUESE,
            SubtitleLanguage::PortugueseBrazil => Self::PORTUGUESE_BRAZIL,
            SubtitleLanguage::Romanian => Self::ROMANIAN,
            SubtitleLanguage::Russian => Self::RUSSIAN,
            SubtitleLanguage::Slovene => Self::SLOVENE,
            SubtitleLanguage::Serbian => Self::SERBIAN,
            SubtitleLanguage::Swedish => Self::SWEDISH,
            SubtitleLanguage::Thai => Self::THAI,
            SubtitleLanguage::Turkish => Self::TURKISH,
            SubtitleLanguage::Ukrainian => Self::UKRAINIAN,
            SubtitleLanguage::Vietnamese => Self::VIETNAMESE,
        }
    }
}
