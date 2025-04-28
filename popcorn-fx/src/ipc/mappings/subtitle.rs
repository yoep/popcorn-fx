use crate::ipc::proto::subtitle::subtitle;
use crate::ipc::proto::subtitle::subtitle_preference::Preference;
use crate::ipc::{proto, Error, Result};
use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo};
use popcorn_fx_core::core::subtitles::{SubtitleError, SubtitleFile, SubtitlePreference};
use protobuf::MessageField;

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

impl TryFrom<&subtitle::Info> for SubtitleInfo {
    type Error = Error;

    fn try_from(value: &subtitle::Info) -> Result<Self> {
        let mut builder = Self::builder();
        let language = value
            .language
            .enum_value()
            .map(|e| SubtitleLanguage::from(&e))
            .map_err(|_| Error::UnsupportedEnum)?;

        if let Some(imdb_id) = value.imdb_id.as_ref() {
            builder.imdb_id(imdb_id);
        }

        Ok(builder
            .language(language)
            .files(value.files.iter().map(SubtitleFile::from).collect())
            .build())
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

impl From<&subtitle::info::File> for SubtitleFile {
    fn from(value: &subtitle::info::File) -> Self {
        let mut builder = SubtitleFile::builder();

        if let Some(quality) = value.quality.as_ref() {
            builder.quality(*quality);
        }

        builder
            .file_id(value.file_id)
            .name(value.name.as_str())
            .url(value.url.as_str())
            .score(value.score)
            .downloads(value.downloads)
            .build()
    }
}

impl From<&Subtitle> for proto::subtitle::Subtitle {
    fn from(value: &Subtitle) -> Self {
        Self {
            file_path: value.file().to_string(),
            info: value
                .info()
                .as_ref()
                .map(|e| subtitle::Info::from(*e))
                .into(),
            cues: vec![],
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitleCue> for subtitle::Cue {
    fn from(value: &SubtitleCue) -> Self {
        Self {
            id: value.id().clone(),
            start_time: *value.start_time(),
            end_time: *value.end_time(),
            lines: value
                .lines()
                .iter()
                .map(subtitle::cue::Line::from)
                .collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitleLine> for subtitle::cue::Line {
    fn from(value: &SubtitleLine) -> Self {
        Self {
            text: value
                .texts()
                .iter()
                .map(subtitle::cue::line::Text::from)
                .collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&StyledText> for subtitle::cue::line::Text {
    fn from(value: &StyledText) -> Self {
        Self {
            text: value.text().clone(),
            italic: *value.italic(),
            bold: *value.bold(),
            underline: *value.underline(),
            special_fields: Default::default(),
        }
    }
}

impl From<&SubtitlePreference> for proto::subtitle::SubtitlePreference {
    fn from(value: &SubtitlePreference) -> Self {
        let mut preference = Self::new();

        match value {
            SubtitlePreference::Language(language) => {
                preference.preference = Preference::LANGUAGE.into();
                preference.language = Some(subtitle::Language::from(language).into());
            }
            SubtitlePreference::Disabled => {
                preference.preference = Preference::DISABLED.into();
            }
        }

        preference
    }
}

impl TryFrom<&proto::subtitle::SubtitlePreference> for SubtitlePreference {
    type Error = Error;

    fn try_from(value: &proto::subtitle::SubtitlePreference) -> Result<Self> {
        let preference = value.preference.enum_value_or(Preference::DISABLED);
        let language = value
            .language
            .as_ref()
            .map(|e| e.enum_value_or(subtitle::Language::NONE))
            .map(|e| SubtitleLanguage::from(&e));

        match preference {
            Preference::LANGUAGE => Ok(Self::Language(language.ok_or(Error::MissingField)?)),
            Preference::DISABLED => Ok(Self::Disabled),
        }
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

impl From<&subtitle::Language> for SubtitleLanguage {
    fn from(value: &subtitle::Language) -> Self {
        match value {
            subtitle::Language::NONE => Self::None,
            subtitle::Language::CUSTOM => Self::Custom,
            subtitle::Language::ARABIC => Self::Arabic,
            subtitle::Language::BULGARIAN => Self::Bulgarian,
            subtitle::Language::BOSNIAN => Self::Bosnian,
            subtitle::Language::CZECH => Self::Czech,
            subtitle::Language::DANISH => Self::Danish,
            subtitle::Language::GERMAN => Self::German,
            subtitle::Language::MODERN_GREEK => Self::ModernGreek,
            subtitle::Language::ENGLISH => Self::English,
            subtitle::Language::SPANISH => Self::Spanish,
            subtitle::Language::ESTONIAN => Self::Estonian,
            subtitle::Language::BASQUE => Self::Basque,
            subtitle::Language::PERSIAN => Self::Persian,
            subtitle::Language::FINNISH => Self::Finnish,
            subtitle::Language::FRENCH => Self::French,
            subtitle::Language::HEBREW => Self::Hebrew,
            subtitle::Language::CROATIAN => Self::Croatian,
            subtitle::Language::HUNGARIAN => Self::Hungarian,
            subtitle::Language::INDONESIAN => Self::Indonesian,
            subtitle::Language::ITALIAN => Self::Italian,
            subtitle::Language::LITHUANIAN => Self::Lithuanian,
            subtitle::Language::DUTCH => Self::Dutch,
            subtitle::Language::NORWEGIAN => Self::Norwegian,
            subtitle::Language::POLISH => Self::Polish,
            subtitle::Language::PORTUGUESE => Self::Portuguese,
            subtitle::Language::PORTUGUESE_BRAZIL => Self::PortugueseBrazil,
            subtitle::Language::ROMANIAN => Self::Romanian,
            subtitle::Language::RUSSIAN => Self::Russian,
            subtitle::Language::SLOVENE => Self::Slovene,
            subtitle::Language::SERBIAN => Self::Serbian,
            subtitle::Language::SWEDISH => Self::Swedish,
            subtitle::Language::THAI => Self::Thai,
            subtitle::Language::TURKISH => Self::Turkish,
            subtitle::Language::UKRAINIAN => Self::Ukrainian,
            subtitle::Language::VIETNAMESE => Self::Vietnamese,
        }
    }
}

impl From<&subtitle::Matcher> for SubtitleMatcher {
    fn from(value: &subtitle::Matcher) -> Self {
        Self::from_string(Some(value.filename.clone()), value.quality.clone())
    }
}

impl From<&SubtitleError> for subtitle::Error {
    fn from(value: &SubtitleError) -> Self {
        let mut err = Self::new();

        match value {
            SubtitleError::InvalidUrl(url) => {
                err.type_ = subtitle::error::Type::INVALID_URL.into();
                err.invalid_url = MessageField::some(subtitle::error::InvalidUrl {
                    url: url.clone(),
                    special_fields: Default::default(),
                });
            }
            SubtitleError::DownloadFailed(filename, reason) => {
                err.type_ = subtitle::error::Type::DOWNLOAD_FAILED.into();
                err.download_failed = MessageField::some(subtitle::error::DownloadFailed {
                    filename: filename.clone(),
                    reason: reason.clone(),
                    special_fields: Default::default(),
                })
            }
            SubtitleError::SearchFailed(reason) => {
                err.type_ = subtitle::error::Type::SEARCH_FAILED.into();
                err.search_failed = MessageField::some(subtitle::error::SearchFailed {
                    reason: reason.clone(),
                    special_fields: Default::default(),
                });
            }
            _ => todo!(),
        }

        err
    }
}
