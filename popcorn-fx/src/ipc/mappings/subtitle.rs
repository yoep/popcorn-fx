use crate::ipc::proto::subtitle::subtitle;
use crate::ipc::proto::subtitle::subtitle_preference::Preference;
use crate::ipc::{proto, Error, Result};
use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
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
            cues: value.cues().iter().map(subtitle::Cue::from).collect(),
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
            SubtitleError::IO(_) => {
                err.type_ = subtitle::error::Type::IO.into();
            }
            SubtitleError::ParseFileError(_, _) => {
                err.type_ = subtitle::error::Type::PARSE_FILE.into();
            }
            SubtitleError::ParseUrlError(_) => {
                err.type_ = subtitle::error::Type::PARSE_URL.into();
            }
            SubtitleError::ConversionFailed(subtitle_type, reason) => {
                err.type_ = subtitle::error::Type::CONVERSION.into();
                err.conversion_failed = MessageField::some(subtitle::error::ConversionFailed {
                    type_: subtitle::Type::from(subtitle_type).into(),
                    reason: reason.clone(),
                    special_fields: Default::default(),
                })
            }
            SubtitleError::TypeNotSupported(subtitle_type) => {
                err.type_ = subtitle::error::Type::UNSUPPORTED_TYPE.into();
                err.unsupported_type = MessageField::some(subtitle::error::UnsupportedType {
                    type_: subtitle::Type::from(subtitle_type).into(),
                    special_fields: Default::default(),
                })
            }
            SubtitleError::NoFilesFound => {
                err.type_ = subtitle::error::Type::NO_FILES_FOUND.into();
            }
            SubtitleError::InvalidFile(_, _) => {
                err.type_ = subtitle::error::Type::INVALID_FILE.into();
            }
        }

        err
    }
}

impl From<&SubtitleType> for subtitle::Type {
    fn from(value: &SubtitleType) -> Self {
        match value {
            SubtitleType::Srt => Self::SRT,
            SubtitleType::Vtt => Self::VTT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_language_proto_from() {
        assert_eq!(
            subtitle::Language::NONE,
            subtitle::Language::from(&SubtitleLanguage::None)
        );
        assert_eq!(
            subtitle::Language::CUSTOM,
            subtitle::Language::from(&SubtitleLanguage::Custom)
        );
        assert_eq!(
            subtitle::Language::ARABIC,
            subtitle::Language::from(&SubtitleLanguage::Arabic)
        );
        assert_eq!(
            subtitle::Language::BULGARIAN,
            subtitle::Language::from(&SubtitleLanguage::Bulgarian)
        );
        assert_eq!(
            subtitle::Language::BOSNIAN,
            subtitle::Language::from(&SubtitleLanguage::Bosnian)
        );
        assert_eq!(
            subtitle::Language::CZECH,
            subtitle::Language::from(&SubtitleLanguage::Czech)
        );
        assert_eq!(
            subtitle::Language::DANISH,
            subtitle::Language::from(&SubtitleLanguage::Danish)
        );
        assert_eq!(
            subtitle::Language::GERMAN,
            subtitle::Language::from(&SubtitleLanguage::German)
        );
        assert_eq!(
            subtitle::Language::MODERN_GREEK,
            subtitle::Language::from(&SubtitleLanguage::ModernGreek)
        );
        assert_eq!(
            subtitle::Language::ENGLISH,
            subtitle::Language::from(&SubtitleLanguage::English)
        );
        assert_eq!(
            subtitle::Language::SPANISH,
            subtitle::Language::from(&SubtitleLanguage::Spanish)
        );
        assert_eq!(
            subtitle::Language::ESTONIAN,
            subtitle::Language::from(&SubtitleLanguage::Estonian)
        );
        assert_eq!(
            subtitle::Language::BASQUE,
            subtitle::Language::from(&SubtitleLanguage::Basque)
        );
        assert_eq!(
            subtitle::Language::PERSIAN,
            subtitle::Language::from(&SubtitleLanguage::Persian)
        );
        assert_eq!(
            subtitle::Language::FINNISH,
            subtitle::Language::from(&SubtitleLanguage::Finnish)
        );
        assert_eq!(
            subtitle::Language::FRENCH,
            subtitle::Language::from(&SubtitleLanguage::French)
        );
        assert_eq!(
            subtitle::Language::HEBREW,
            subtitle::Language::from(&SubtitleLanguage::Hebrew)
        );
        assert_eq!(
            subtitle::Language::CROATIAN,
            subtitle::Language::from(&SubtitleLanguage::Croatian)
        );
        assert_eq!(
            subtitle::Language::HUNGARIAN,
            subtitle::Language::from(&SubtitleLanguage::Hungarian)
        );
        assert_eq!(
            subtitle::Language::INDONESIAN,
            subtitle::Language::from(&SubtitleLanguage::Indonesian)
        );
        assert_eq!(
            subtitle::Language::ITALIAN,
            subtitle::Language::from(&SubtitleLanguage::Italian)
        );
        assert_eq!(
            subtitle::Language::LITHUANIAN,
            subtitle::Language::from(&SubtitleLanguage::Lithuanian)
        );
        assert_eq!(
            subtitle::Language::DUTCH,
            subtitle::Language::from(&SubtitleLanguage::Dutch)
        );
        assert_eq!(
            subtitle::Language::NORWEGIAN,
            subtitle::Language::from(&SubtitleLanguage::Norwegian)
        );
        assert_eq!(
            subtitle::Language::POLISH,
            subtitle::Language::from(&SubtitleLanguage::Polish)
        );
        assert_eq!(
            subtitle::Language::PORTUGUESE,
            subtitle::Language::from(&SubtitleLanguage::Portuguese)
        );
        assert_eq!(
            subtitle::Language::PORTUGUESE_BRAZIL,
            subtitle::Language::from(&SubtitleLanguage::PortugueseBrazil)
        );
        assert_eq!(
            subtitle::Language::ROMANIAN,
            subtitle::Language::from(&SubtitleLanguage::Romanian)
        );
        assert_eq!(
            subtitle::Language::RUSSIAN,
            subtitle::Language::from(&SubtitleLanguage::Russian)
        );
        assert_eq!(
            subtitle::Language::SLOVENE,
            subtitle::Language::from(&SubtitleLanguage::Slovene)
        );
        assert_eq!(
            subtitle::Language::SERBIAN,
            subtitle::Language::from(&SubtitleLanguage::Serbian)
        );
        assert_eq!(
            subtitle::Language::SWEDISH,
            subtitle::Language::from(&SubtitleLanguage::Swedish)
        );
        assert_eq!(
            subtitle::Language::THAI,
            subtitle::Language::from(&SubtitleLanguage::Thai)
        );
        assert_eq!(
            subtitle::Language::TURKISH,
            subtitle::Language::from(&SubtitleLanguage::Turkish)
        );
        assert_eq!(
            subtitle::Language::UKRAINIAN,
            subtitle::Language::from(&SubtitleLanguage::Ukrainian)
        );
        assert_eq!(
            subtitle::Language::VIETNAMESE,
            subtitle::Language::from(&SubtitleLanguage::Vietnamese)
        );
    }

    #[test]
    fn test_subtitle_language_from() {
        assert_eq!(
            SubtitleLanguage::None,
            SubtitleLanguage::from(&subtitle::Language::NONE)
        );
        assert_eq!(
            SubtitleLanguage::Custom,
            SubtitleLanguage::from(&subtitle::Language::CUSTOM)
        );
        assert_eq!(
            SubtitleLanguage::Arabic,
            SubtitleLanguage::from(&subtitle::Language::ARABIC)
        );
        assert_eq!(
            SubtitleLanguage::Bulgarian,
            SubtitleLanguage::from(&subtitle::Language::BULGARIAN)
        );
        assert_eq!(
            SubtitleLanguage::Bosnian,
            SubtitleLanguage::from(&subtitle::Language::BOSNIAN)
        );
        assert_eq!(
            SubtitleLanguage::Czech,
            SubtitleLanguage::from(&subtitle::Language::CZECH)
        );
        assert_eq!(
            SubtitleLanguage::Danish,
            SubtitleLanguage::from(&subtitle::Language::DANISH)
        );
        assert_eq!(
            SubtitleLanguage::German,
            SubtitleLanguage::from(&subtitle::Language::GERMAN)
        );
        assert_eq!(
            SubtitleLanguage::ModernGreek,
            SubtitleLanguage::from(&subtitle::Language::MODERN_GREEK)
        );
        assert_eq!(
            SubtitleLanguage::English,
            SubtitleLanguage::from(&subtitle::Language::ENGLISH)
        );
        assert_eq!(
            SubtitleLanguage::Spanish,
            SubtitleLanguage::from(&subtitle::Language::SPANISH)
        );
        assert_eq!(
            SubtitleLanguage::Estonian,
            SubtitleLanguage::from(&subtitle::Language::ESTONIAN)
        );
        assert_eq!(
            SubtitleLanguage::Basque,
            SubtitleLanguage::from(&subtitle::Language::BASQUE)
        );
        assert_eq!(
            SubtitleLanguage::Persian,
            SubtitleLanguage::from(&subtitle::Language::PERSIAN)
        );
        assert_eq!(
            SubtitleLanguage::Finnish,
            SubtitleLanguage::from(&subtitle::Language::FINNISH)
        );
        assert_eq!(
            SubtitleLanguage::French,
            SubtitleLanguage::from(&subtitle::Language::FRENCH)
        );
        assert_eq!(
            SubtitleLanguage::Hebrew,
            SubtitleLanguage::from(&subtitle::Language::HEBREW)
        );
        assert_eq!(
            SubtitleLanguage::Croatian,
            SubtitleLanguage::from(&subtitle::Language::CROATIAN)
        );
        assert_eq!(
            SubtitleLanguage::Hungarian,
            SubtitleLanguage::from(&subtitle::Language::HUNGARIAN)
        );
        assert_eq!(
            SubtitleLanguage::Indonesian,
            SubtitleLanguage::from(&subtitle::Language::INDONESIAN)
        );
        assert_eq!(
            SubtitleLanguage::Italian,
            SubtitleLanguage::from(&subtitle::Language::ITALIAN)
        );
        assert_eq!(
            SubtitleLanguage::Lithuanian,
            SubtitleLanguage::from(&subtitle::Language::LITHUANIAN)
        );
        assert_eq!(
            SubtitleLanguage::Dutch,
            SubtitleLanguage::from(&subtitle::Language::DUTCH)
        );
        assert_eq!(
            SubtitleLanguage::Norwegian,
            SubtitleLanguage::from(&subtitle::Language::NORWEGIAN)
        );
        assert_eq!(
            SubtitleLanguage::Polish,
            SubtitleLanguage::from(&subtitle::Language::POLISH)
        );
        assert_eq!(
            SubtitleLanguage::Portuguese,
            SubtitleLanguage::from(&subtitle::Language::PORTUGUESE)
        );
        assert_eq!(
            SubtitleLanguage::PortugueseBrazil,
            SubtitleLanguage::from(&subtitle::Language::PORTUGUESE_BRAZIL)
        );
        assert_eq!(
            SubtitleLanguage::Romanian,
            SubtitleLanguage::from(&subtitle::Language::ROMANIAN)
        );
        assert_eq!(
            SubtitleLanguage::Russian,
            SubtitleLanguage::from(&subtitle::Language::RUSSIAN)
        );
        assert_eq!(
            SubtitleLanguage::Slovene,
            SubtitleLanguage::from(&subtitle::Language::SLOVENE)
        );
        assert_eq!(
            SubtitleLanguage::Serbian,
            SubtitleLanguage::from(&subtitle::Language::SERBIAN)
        );
        assert_eq!(
            SubtitleLanguage::Swedish,
            SubtitleLanguage::from(&subtitle::Language::SWEDISH)
        );
        assert_eq!(
            SubtitleLanguage::Thai,
            SubtitleLanguage::from(&subtitle::Language::THAI)
        );
        assert_eq!(
            SubtitleLanguage::Turkish,
            SubtitleLanguage::from(&subtitle::Language::TURKISH)
        );
        assert_eq!(
            SubtitleLanguage::Ukrainian,
            SubtitleLanguage::from(&subtitle::Language::UKRAINIAN)
        );
        assert_eq!(
            SubtitleLanguage::Vietnamese,
            SubtitleLanguage::from(&subtitle::Language::VIETNAMESE)
        );
    }

    #[test]
    fn test_subtitle_error_from_invalid_url() {
        let url = "https://invalid-url.com";
        let err = SubtitleError::InvalidUrl(url.to_string());
        let expected_result = subtitle::Error {
            type_: subtitle::error::Type::INVALID_URL.into(),
            invalid_url: MessageField::some(subtitle::error::InvalidUrl {
                url: url.to_string(),
                special_fields: Default::default(),
            }),
            search_failed: Default::default(),
            download_failed: Default::default(),
            conversion_failed: Default::default(),
            unsupported_type: Default::default(),
            special_fields: Default::default(),
        };

        let result = subtitle::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_subtitle_error_from_download_failed() {
        let filename = "SomeSubtitle.srt";
        let reason = "Some failure reason";
        let err = SubtitleError::DownloadFailed(filename.to_string(), reason.to_string());
        let expected_result = subtitle::Error {
            type_: subtitle::error::Type::DOWNLOAD_FAILED.into(),
            invalid_url: Default::default(),
            search_failed: Default::default(),
            download_failed: MessageField::some(subtitle::error::DownloadFailed {
                filename: filename.to_string(),
                reason: reason.to_string(),
                special_fields: Default::default(),
            }),
            conversion_failed: Default::default(),
            unsupported_type: Default::default(),
            special_fields: Default::default(),
        };

        let result = subtitle::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_subtitle_error_from_conversion_failed() {
        let reason = "Some failure reason";
        let err = SubtitleError::ConversionFailed(SubtitleType::Srt, reason.to_string());
        let expected_result = subtitle::Error {
            type_: subtitle::error::Type::CONVERSION.into(),
            invalid_url: Default::default(),
            search_failed: Default::default(),
            download_failed: Default::default(),
            conversion_failed: MessageField::some(subtitle::error::ConversionFailed {
                type_: subtitle::Type::SRT.into(),
                reason: reason.to_string(),
                special_fields: Default::default(),
            }),
            unsupported_type: Default::default(),
            special_fields: Default::default(),
        };

        let result = subtitle::Error::from(&err);

        assert_eq!(expected_result, result);
    }
}
