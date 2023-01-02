extern crate derive_more;

use std::cmp::Ordering;

use derive_more::Display;
use itertools::Itertools;
use log::{trace, warn};
use regex::Regex;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::errors::{SubtitleError, SubtitleParseError};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;

const SRT_EXTENSION: &'static str = "srt";
const VTT_EXTENSION: &'static str = "vtt";
const QUALITY_PATTERN: &'static str = "([0-9]{3,4})p";

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
                return Ok(subtitle.clone());
            }
        }

        Err(SubtitleParseError::ExtensionNotSupported(extension.clone()))
    }

    /// The file extension for this subtitle type.
    pub fn extension(&self) -> String {
        match self {
            SubtitleType::Srt => SRT_EXTENSION.to_string(),
            SubtitleType::Vtt => VTT_EXTENSION.to_string()
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
}

impl SubtitleInfo {
    /// The special _none_ subtitle instance.
    pub fn none() -> Self {
        Self {
            imdb_id: None,
            language: SubtitleLanguage::None,
            files: None,
        }
    }

    /// The special _custom_ subtitle instance.
    pub fn custom() -> Self {
        Self {
            imdb_id: None,
            language: SubtitleLanguage::Custom,
            files: None,
        }
    }

    /// Create a new subtitle info without any files.
    pub fn new(imdb_id: String, language: SubtitleLanguage) -> Self {
        Self {
            imdb_id: Some(imdb_id),
            language,
            files: None,
        }
    }

    /// Create a new subtitle info with subtitle files.
    pub fn new_with_files(imdb_id: String, language: SubtitleLanguage, files: Vec<SubtitleFile>) -> Self {
        Self {
            imdb_id: Some(imdb_id),
            language,
            files: Some(files),
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
    pub fn best_matching_file(&self, matcher: &SubtitleMatcher) -> Result<SubtitleFile, SubtitleError> {
        let name = matcher.name();

        // verify if a name is present to match
        // this will try to find a file matching the name in a normalized way
        if name.is_some() {
            let file_by_name = self.find_by_filename(name.unwrap());

            match file_by_name {
                Ok(e) => match e {
                    None => {}
                    Some(file) => return Ok(file.clone())
                },
                Err(err) => warn!("{}", err)
            }
        }

        match matcher.quality() {
            Some(quality) => {
                match self.find_by_quality(quality) {
                    Ok(e) => Ok(e.clone()),
                    Err(err) => Err(err)
                }
            }
            None => match self.find_by_best_score() {
                Ok(e) => Ok(e.clone()),
                Err(err) => Err(err)
            }
        }
    }

    fn find_by_filename(&self, name: &String) -> Result<Option<&SubtitleFile>, SubtitleError> {
        match &self.files {
            None => Err(SubtitleError::NoFilesFound()),
            Some(files) => Ok(Self::find_within_files_by_name(name, files))
        }
    }

    fn find_by_quality(&self, quality: &i32) -> Result<&SubtitleFile, SubtitleError> {
        match &self.files {
            None => Err(SubtitleError::NoFilesFound()),
            Some(files) => Ok(files.iter()
                .filter(|e| e.quality().is_none() || e.quality().unwrap() == quality)
                .sorted()
                .next()
                .unwrap())
        }
    }

    fn find_by_best_score(&self) -> Result<&SubtitleFile, SubtitleError> {
        match &self.files {
            None => Err(SubtitleError::NoFilesFound()),
            Some(files) => Ok(files.iter()
                .sorted()
                .next()
                .unwrap())
        }
    }

    fn find_within_files_by_name<'a, 'b>(name: &'a String, files: &'b Vec<SubtitleFile>) -> Option<&'b SubtitleFile> {
        match files.iter()
            .filter(|e| Self::normalize(e.name()) == Self::normalize(name))
            .sorted()
            .next() {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn normalize(name: &String) -> String {
        name
            .to_lowercase()
            .replace("[\\[\\]\\(\\)_-\\.]", "")
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

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SubtitleFile {
    file_id: i32,
    name: String,
    url: String,
    score: f32,
    downloads: i32,
    quality: Option<i32>,
}

impl SubtitleFile {
    /// Create a new subtitle file instance.
    /// The quality is automatically parsed from the `name`.
    pub fn new(file_id: i32, name: String, url: String, score: f32, downloads: i32) -> Self {
        let quality = Self::try_parse_subtitle_quality(&name);
        trace!("Parsed subtitle quality {:?} from \"{}\"", &quality, &name);

        Self {
            file_id,
            name,
            url,
            score,
            downloads,
            quality,
        }
    }

    /// Create a new subtitle file instance with the given quality.
    pub fn new_with_quality(file_id: i32, quality: i32, name: String, url: String, score: f32, downloads: i32) -> Self {
        Self {
            file_id,
            name,
            url,
            score,
            downloads,
            quality: Some(quality),
        }
    }

    pub fn file_id(&self) -> &i32 {
        &self.file_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn score(&self) -> &f32 {
        &self.score
    }

    pub fn downloads(&self) -> &i32 {
        &self.downloads
    }

    pub fn quality(&self) -> Option<&i32> {
        match &self.quality {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Try to parse the quality for the subtitle file based on the filename.
    fn try_parse_subtitle_quality(name: &String) -> Option<i32> {
        let regex = Regex::new(QUALITY_PATTERN).unwrap();
        regex.captures(name.as_str())
            .map(|e| e.get(1).unwrap())
            .map(|e| String::from(e.as_str()))
            .map(|e| e.parse::<i32>().unwrap())
    }
}

impl Eq for SubtitleFile {}

impl Ord for SubtitleFile {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.score() > other.score() ||
            (self.score() == other.score() && self.downloads() > other.downloads()) {
            return Ordering::Less;
        }

        if self.score() == other.score() && self.downloads() == other.downloads() {
            return Ordering::Equal;
        }

        return Ordering::Greater;
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
    file: Option<String>,
}

impl Subtitle {
    pub fn new(cues: Vec<SubtitleCue>, info: Option<SubtitleInfo>, file: Option<String>) -> Self {
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

    pub fn file(&self) -> Option<&String> {
        match &self.file {
            Some(e) => Some(e),
            None => None
        }
    }
}

impl PartialEq for Subtitle {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && self.file == other.file
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[cfg(test)]
mod test {
    use crate::test::init_logger;

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
}