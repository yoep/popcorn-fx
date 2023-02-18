extern crate derive_more;

use std::cmp::Ordering;
use std::path::PathBuf;

use derive_more::Display;
use itertools::Itertools;
use log::{info, trace, warn};
use regex::Regex;

use crate::core::subtitles;
use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::error::{SubtitleError, SubtitleParseError};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::SubtitleFile;

const SRT_EXTENSION: &str = "srt";
const VTT_EXTENSION: &str = "vtt";
const NORMALIZATION_PATTERN: &str = "[\\.\\[\\]\\(\\)_-]";

const SUBTITLE_TYPES: [SubtitleType; 2] = [
    SubtitleType::Srt,
    SubtitleType::Vtt
];

#[repr(i32)]
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum SubtitleType {
    Srt = 0,
    Vtt = 1,
}

impl SubtitleType {
    /// Retrieve the subtitle type based on the given extension.
    /// It can return an error when no type could be found.
    pub fn from_extension(extension: &String) -> Result<SubtitleType, SubtitleParseError> {
        for subtitle in SUBTITLE_TYPES {
            if extension == &subtitle.extension() {
                return Ok(subtitle);
            }
        }

        Err(SubtitleParseError::ExtensionNotSupported(extension.clone()))
    }

    pub fn from_ordinal(ordinal: usize) -> Self {
        SUBTITLE_TYPES[ordinal].clone()
    }

    /// The file extension for this subtitle type.
    pub fn extension(&self) -> String {
        match self {
            SubtitleType::Srt => SRT_EXTENSION.to_string(),
            SubtitleType::Vtt => VTT_EXTENSION.to_string()
        }
    }

    /// Retrieve the content type of the subtitle type.
    /// This represents a valid HTTP content type.
    pub fn content_type(&self) -> &str {
        match self {
            SubtitleType::Srt => "text/srt",
            SubtitleType::Vtt => "text/vtt"
        }
    }
}

/// The subtitle info contains information about available subtitles for a certain [Media].
/// This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
#[derive(Debug, Clone, Display)]
#[display(fmt = "imdb_id: {:?}, language: {}", imdb_id, language)]
pub struct SubtitleInfo {
    imdb_id: Option<String>,
    language: SubtitleLanguage,
    files: Option<Vec<SubtitleFile>>,
    normalize_regex: Regex,
}

impl SubtitleInfo {
    /// The special _none_ subtitle instance.
    pub fn none() -> Self {
        Self {
            imdb_id: None,
            language: SubtitleLanguage::None,
            files: None,
            normalize_regex: Regex::new(NORMALIZATION_PATTERN).unwrap(),
        }
    }

    /// The special _custom_ subtitle instance.
    pub fn custom() -> Self {
        Self {
            imdb_id: None,
            language: SubtitleLanguage::Custom,
            files: None,
            normalize_regex: Regex::new(NORMALIZATION_PATTERN).unwrap(),
        }
    }

    /// Create a new subtitle info without any files.
    pub fn new(imdb_id: String, language: SubtitleLanguage) -> Self {
        Self {
            imdb_id: Some(imdb_id),
            language,
            files: None,
            normalize_regex: Regex::new(NORMALIZATION_PATTERN).unwrap(),
        }
    }

    /// Create a new subtitle info with subtitle files.
    pub fn new_with_files(imdb_id: Option<String>, language: SubtitleLanguage, files: Vec<SubtitleFile>) -> Self {
        Self {
            imdb_id,
            language,
            files: Some(files),
            normalize_regex: Regex::new(NORMALIZATION_PATTERN).unwrap(),
        }
    }

    /// Verify if the subtitle info is a special type
    /// such as [SubtitleInfo::none()] or [SubtitleInfo::custom()]
    pub fn is_special(&self) -> bool {
        self.is_none() || self.is_custom()
    }

    pub fn imdb_id(&self) -> Option<&String> {
        match &self.imdb_id {
            None => None,
            Some(e) => Some(e)
        }
    }

    pub fn language(&self) -> &SubtitleLanguage {
        &self.language
    }

    pub fn files(&self) -> Option<&Vec<SubtitleFile>> {
        match &self.files {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Verify if the subtitle info is the [SubtitleInfo::none()] type.
    pub fn is_none(&self) -> bool {
        self.language == SubtitleLanguage::None
    }

    /// Verify if the subtitle info the [SubtitleInfo::custom()] type.
    pub fn is_custom(&self) -> bool {
        self.language == SubtitleLanguage::Custom
    }

    /// retrieve the best matching file from this [SubtitleInfo] based on the given data.
    pub fn best_matching_file(&self, matcher: &SubtitleMatcher) -> subtitles::Result<SubtitleFile> {
        let name = matcher.name();
        let mut files = self.filter_and_sort_by_quality(matcher.quality())?;
        trace!("Searching matching subtitle for name: {:?}, quality: {:?} within files: {:?}", &name, &matcher.quality(), &files);

        // verify if a name is present to match
        // this will try to find a file matching the name in a normalized way
        if let Some(name) = name {
            trace!("Searching subtitle file based on filename {}", name);
            files = self.filter_by_filename(name, files);

            return match files.into_iter().next() {
                None => {
                    warn!("No subtitle file found matching {}, using best matching item instead", name);
                    match self.files().unwrap().iter()
                        .sorted()
                        .next() {
                        None => Err(SubtitleError::NoFilesFound),
                        Some(e) => Ok(e.clone())
                    }
                }
                Some(e) => Ok(e)
            };
        }

        match files.into_iter().next() {
            None => Err(SubtitleError::NoFilesFound),
            Some(e) => {
                info!("Next playback will use subtitle file {:?}", &e);
                Ok(e)
            }
        }
    }

    fn filter_and_sort_by_quality(&self, quality: Option<&i32>) -> subtitles::Result<Vec<SubtitleFile>> {
        trace!("Initial filter of subtitles files for quality {:?} for {:?}", &quality, &self.files);
        match &self.files {
            None => Err(SubtitleError::NoFilesFound),
            Some(files) => {
                match quality {
                    None => Ok(files.clone()),
                    Some(quality) => {
                        Ok(files.iter()
                            .filter(|e| Self::matches_quality(quality, e))
                            .cloned()
                            .sorted()
                            .collect())
                    }
                }
            }
        }
    }

    fn filter_by_filename(&self, name: &str, files: Vec<SubtitleFile>) -> Vec<SubtitleFile> {
        let normalized_filename = self.normalize(name);
        files.into_iter()
            .filter(|e| self.normalize(e.name()) == normalized_filename)
            .collect()
    }

    fn normalize(&self, name: &str) -> String {
        let path = PathBuf::from(name);

        match path.file_stem() {
            None => {
                warn!("Unable to normalize {}, invalid basename", name);
                name.to_lowercase()
            }
            Some(e) => {
                let name = e.to_str()
                    .expect("expected a valid str")
                    .to_lowercase();

                self.normalize_regex.replace_all(name.as_str(), "").to_string()
            }
        }
    }

    fn matches_quality(quality: &i32, file: &&SubtitleFile) -> bool {
        match file.quality() {
            None => true,
            Some(e) => e == quality
        }
    }
}

impl PartialEq for SubtitleInfo {
    fn eq(&self, other: &Self) -> bool {
        self.imdb_id == other.imdb_id && self.language == other.language && self.files.iter()
            .all(|file| {
                other.files.iter().any(|e| {
                    e == file
                })
            })
    }
}

impl Eq for SubtitleInfo {}

impl PartialOrd for SubtitleInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.language().partial_cmp(other.language())
    }
}

impl Ord for SubtitleInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("expected a ordering for SubtitleInfo")
    }
}

/// The parsed [SubtitleInfo] which has downloaded and parsed the .srt file.
#[derive(Debug, Clone, Display)]
#[display(fmt = "file: {:?}, info: {:?}, total cues: {}", file, info, "cues.len()")]
pub struct Subtitle {
    /// The parsed cues within the subtitle file.
    cues: Vec<SubtitleCue>,
    /// The original subtitle info that was used to create this subtitle.
    info: Option<SubtitleInfo>,
    /// The subtitle file path which was used to parse the subtitle file.
    file: String,
}

impl Subtitle {
    pub fn new(cues: Vec<SubtitleCue>, info: Option<SubtitleInfo>, file: String) -> Self {
        Self {
            cues,
            info,
            file,
        }
    }

    pub fn cues(&self) -> &Vec<SubtitleCue> {
        &self.cues
    }

    pub fn info(&self) -> Option<&SubtitleInfo> {
        match &self.info {
            Some(e) => Some(e),
            None => None
        }
    }

    pub fn file(&self) -> &String {
        &self.file
    }
}

impl PartialEq for Subtitle {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && self.file == other.file
    }
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_subtitle_info_partial_eq_when_subtitle_is_same_should_return_true() {
        let info1 = SubtitleInfo::new("12".to_string(), SubtitleLanguage::German);
        let info2 = SubtitleInfo::new("12".to_string(), SubtitleLanguage::German);

        let result = info1 == info2;

        assert_eq!(true, result)
    }

    #[test]
    fn test_subtitle_info_partial_eq_when_id_is_different_should_return_false() {
        let info1 = SubtitleInfo::new("12".to_string(), SubtitleLanguage::German);
        let info2 = SubtitleInfo::new("13".to_string(), SubtitleLanguage::German);

        let result = info1 == info2;

        assert_eq!(false, result)
    }

    #[test]
    fn test_subtitle_info_partial_eq_when_language_is_different_should_return_false() {
        let info1 = SubtitleInfo::new("12".to_string(), SubtitleLanguage::German);
        let info2 = SubtitleInfo::new("12".to_string(), SubtitleLanguage::Danish);

        let result = info1 == info2;

        assert_eq!(false, result)
    }

    #[test]
    fn test_subtitle_language_from_code_should_return_expected_result() {
        let code = "de".to_string();
        let expected_result = SubtitleLanguage::German;

        let result = SubtitleLanguage::from_code(code);

        assert_eq!(expected_result, result.unwrap())
    }

    #[test]
    fn test_subtitle_info_is_special_when_language_is_none_should_return_true() {
        let info = SubtitleInfo::none();

        let result = info.is_special();

        assert_eq!(true, result);
    }

    #[test]
    fn test_subtitle_info_is_special_when_language_is_custom_should_return_true() {
        let info = SubtitleInfo::custom();

        let result = info.is_special();

        assert_eq!(true, result);
    }

    #[test]
    fn test_subtitle_info_is_none_when_language_is_none_should_return_true() {
        let info = SubtitleInfo::none();

        let result = info.is_none();

        assert_eq!(true, result);
    }

    #[test]
    fn test_subtitle_info_is_custom_when_language_is_custom_should_return_true() {
        let info = SubtitleInfo::custom();

        let result = info.is_custom();

        assert_eq!(true, result);
    }

    #[test]
    fn test_subtitle_type_extension_srt() {
        let extension = "srt";

        let result = SubtitleType::from_extension(&extension.to_string());

        assert!(result.is_ok(), "Expected the extension to have been found");
        assert_eq!(SubtitleType::Srt, result.unwrap());
    }

    #[test]
    fn test_subtitle_type_extension_vtt() {
        let extension = "vtt";

        let result = SubtitleType::from_extension(&extension.to_string());

        assert!(result.is_ok(), "Expected the extension to have been found");
        assert_eq!(SubtitleType::Vtt, result.unwrap());
    }

    #[test]
    fn test_subtitle_type_when_extension_not_support_should_return_error() {
        let extension = "lorem".to_string();

        let result = SubtitleType::from_extension(&extension);

        assert!(result.is_err(), "Expected no extension to have been found");
        assert_eq!(result.err().unwrap(), SubtitleParseError::ExtensionNotSupported(extension.clone()))
    }

    #[test]
    fn test_subtitle_file_quality_present() {
        init_logger();
        let file_id = 49060;
        let name = "Frozen.2.2019.1080p.WEBRip.x264.AAC-[YTS.MX]".to_string();
        let expected_result = 1080;

        let result = SubtitleFile::new(file_id, name, String::new(), 8.0, 19546);

        assert!(result.quality().is_some(), "Expected a quality to have been found");
        assert_eq!(&expected_result, result.quality().unwrap())
    }

    #[test]
    fn test_subtitle_file_quality_not_present() {
        init_logger();
        let file_id = 49060;
        let name = "Frozen.II.2019.DVDScr.XVID.AC3.HQ.Hive-CM8".to_string();

        let result = SubtitleFile::new(file_id, name, String::new(), 8.0, 19546);

        assert!(result.quality().is_none(), "Expected no quality to have been found");
    }

    #[test]
    fn test_subtitle_file_order_should_return_item_with_highest_score_first() {
        let item1 = SubtitleFile::new(1, String::new(), String::new(), 7.0, 0);
        let item2 = SubtitleFile::new(2, String::new(), String::new(), 6.0, 0);
        let item3 = SubtitleFile::new(3, String::new(), String::new(), 8.0, 0);
        let mut items = vec![&item1, &item2, &item3];

        items.sort_by(|a, b| a.cmp(b));

        assert_eq!(vec![&item3, &item1, &item2], items)
    }

    #[test]
    fn test_subtitle_info_normalize() {
        let filename = "Lorem.S02E11[720p]AMZN.WEBRip.x264-GalaxyTV.mkv";
        let expected_result = "lorems02e11720pamznwebripx264galaxytv";
        let subtitle_info = SubtitleInfo::none();

        let result = subtitle_info.normalize(filename);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn subtitle_info_best_matching_file() {
        init_logger();
        let filename = "Lorem.S02E11.720p.AMZN.WEBRip.x264-GalaxyTV.mkv";
        let quality = Some(720);
        let expected_file = SubtitleFile::new_with_quality(
            102,
            "Lorem.S02E11.Ipsum.to.Dolor.DVDRip.Xvid-FoV.en.srt".to_string(),
            String::new(),
            9.0,
            44134,
            None,
        );
        let subtitle_info = SubtitleInfo::new_with_files(
            Some("tt100001010".to_string()),
            SubtitleLanguage::English,
            vec![
                SubtitleFile::new_with_quality(
                    100,
                    "Lorem S02 E11 Ipsum to Dolor 720p x264.srt".to_string(),
                    String::new(),
                    0.0,
                    6755,
                    Some(720),
                ),
                SubtitleFile::new_with_quality(
                    101,
                    "Lorem.M.D.S02E11.720p.WEB.DL.nHD.x264-NhaNc3-eng.srt".to_string(),
                    String::new(),
                    0.0,
                    4879,
                    Some(720),
                ),
                expected_file.clone(),
                SubtitleFile::new_with_quality(
                    103,
                    "Lorem MD Season 2 Episode 11 - Ipsum To Dolor-eng.srt".to_string(),
                    String::new(),
                    0.0,
                    5735,
                    None,
                ),
            ],
        );

        let result = subtitle_info.best_matching_file(&SubtitleMatcher::from_int(Some(filename.to_string()), quality))
            .expect("expected a file to be found");

        assert_eq!(expected_file, result)
    }
}