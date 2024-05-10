extern crate derive_more;

use std::cmp::Ordering;
use std::path::PathBuf;

use derive_more::Display;
use itertools::Itertools;
use log::{debug, info, trace, warn};
use regex::Regex;

use crate::core::subtitles;
use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::error::{SubtitleError, SubtitleParseError};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::SubtitleFile;

const SRT_EXTENSION: &str = "srt";
const VTT_EXTENSION: &str = "vtt";
const NORMALIZATION_PATTERN: &str = "[\\.\\[\\]\\(\\)_\\-+]";

const SUBTITLE_TYPES: [SubtitleType; 2] = [SubtitleType::Srt, SubtitleType::Vtt];

/// The type of a subtitle, indicating its format.
#[repr(i32)]
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum SubtitleType {
    /// SubRip subtitle format.
    Srt = 0,
    /// WebVTT subtitle format.
    Vtt = 1,
}

impl SubtitleType {
    /// Retrieve the subtitle type based on the given file extension.
    ///
    /// # Arguments
    ///
    /// * `extension` - The file extension.
    ///
    /// # Returns
    ///
    /// The corresponding `SubtitleType` if found, or an error if the extension is not supported.
    pub fn from_extension(extension: &String) -> Result<SubtitleType, SubtitleParseError> {
        for subtitle in SUBTITLE_TYPES {
            if extension == &subtitle.extension() {
                return Ok(subtitle);
            }
        }

        Err(SubtitleParseError::ExtensionNotSupported(extension.clone()))
    }

    /// Retrieve the subtitle type from its ordinal value.
    ///
    /// # Arguments
    ///
    /// * `ordinal` - The ordinal value.
    ///
    /// # Returns
    ///
    /// The corresponding `SubtitleType`.
    pub fn from_ordinal(ordinal: usize) -> Self {
        SUBTITLE_TYPES[ordinal].clone()
    }

    /// Get the file extension for this subtitle type.
    ///
    /// # Returns
    ///
    /// The file extension as a string.
    pub fn extension(&self) -> String {
        match self {
            SubtitleType::Srt => SRT_EXTENSION.to_string(),
            SubtitleType::Vtt => VTT_EXTENSION.to_string(),
        }
    }

    /// Retrieve the content type of the subtitle type.
    ///
    /// # Returns
    ///
    /// The content type as a string, representing a valid HTTP content type.
    pub fn content_type(&self) -> &str {
        match self {
            SubtitleType::Srt => "text/srt",
            SubtitleType::Vtt => "text/vtt",
        }
    }
}

/// The subtitle info contains information about available subtitles for a certain [Media].
/// This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
///
/// # Examples
///
/// ```rust
/// use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
/// use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
/// use popcorn_fx_core::core::subtitles::SubtitleFile;
///
/// // Create a new subtitle info instance using the builder pattern
/// let subtitle_info = SubtitleInfo::builder()
///     .imdb_id("tt1234567")
///     .language(SubtitleLanguage::English)
///     .files(vec![
///         SubtitleFile::builder()
///             .file_id(1)
///             .name("example_subtitle.srt")
///             .url("https://example.com/subtitle.srt")
///             .score(0.9)
///             .downloads(1000)
///             .build()
///     ])
///     .build();
/// ```
#[derive(Debug, Clone, Display)]
#[display(fmt = "imdb_id: {:?}, language: {}", imdb_id, language)]
pub struct SubtitleInfo {
    /// The IMDB ID of the subtitle title.
    imdb_id: Option<String>,
    /// The language of the subtitle.
    language: SubtitleLanguage,
    /// The list of available subtitle files.
    files: Option<Vec<SubtitleFile>>,
    /// Regex for normalization.
    normalize_regex: Regex,
}

impl SubtitleInfo {
    /// Creates a new instance of `SubtitleInfoBuilder`.
    pub fn builder() -> SubtitleInfoBuilder {
        SubtitleInfoBuilder::builder()
    }

    /// The special _none_ subtitle instance.
    pub fn none() -> Self {
        Self::builder().language(SubtitleLanguage::None).build()
    }

    /// The special _custom_ subtitle instance.
    pub fn custom() -> Self {
        Self::builder().language(SubtitleLanguage::Custom).build()
    }

    /// Verify if the subtitle info is a special type
    /// such as [SubtitleInfo::none()] or [SubtitleInfo::custom()]
    pub fn is_special(&self) -> bool {
        self.is_none() || self.is_custom()
    }

    /// Retrieves the IMDb ID of the subtitle.
    pub fn imdb_id(&self) -> Option<&String> {
        match &self.imdb_id {
            None => None,
            Some(e) => Some(e),
        }
    }

    /// Retrieves the language of the subtitle.
    pub fn language(&self) -> &SubtitleLanguage {
        &self.language
    }

    /// Retrieves the files associated with the subtitle.
    pub fn files(&self) -> Option<&Vec<SubtitleFile>> {
        match &self.files {
            None => None,
            Some(e) => Some(e),
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
        trace!(
            "Searching matching subtitle for name: {:?}, quality: {:?} within files: {:?}",
            &name,
            &matcher.quality(),
            &files
        );

        // verify if a name is present to match
        // this will try to find a file matching the name in a normalized way
        if let Some(name) = name {
            let name = self.normalize(name);
            debug!("Searching subtitle file based on filename {}", name);
            files = self.filter_by_filename(name.as_str(), files);

            return match files.into_iter().next() {
                None => {
                    warn!(
                        "No subtitle file found matching {}, using best matching item instead",
                        name
                    );
                    match self.files().unwrap().iter().sorted().next() {
                        None => Err(SubtitleError::NoFilesFound),
                        Some(e) => Ok(e.clone()),
                    }
                }
                Some(e) => Ok(e),
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

    fn filter_and_sort_by_quality(
        &self,
        quality: Option<&i32>,
    ) -> subtitles::Result<Vec<SubtitleFile>> {
        trace!(
            "Initial filter of subtitles files for quality {:?} for {:?}",
            &quality,
            &self.files
        );
        match &self.files {
            None => Err(SubtitleError::NoFilesFound),
            Some(files) => match quality {
                None => Ok(files.clone()),
                Some(quality) => Ok(files
                    .iter()
                    .filter(|e| Self::matches_quality(quality, e))
                    .cloned()
                    .sorted()
                    .collect()),
            },
        }
    }

    fn filter_by_filename<S: AsRef<str>>(
        &self,
        name: S,
        files: Vec<SubtitleFile>,
    ) -> Vec<SubtitleFile> {
        let name = name.as_ref().to_string();
        files
            .into_iter()
            .filter(|e| {
                let normalized_filename = self.normalize(e.name());
                trace!(
                    "Matching subtitle filename {} against expected filename {}",
                    normalized_filename,
                    name
                );
                normalized_filename == name
            })
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
                let name = e.to_str().expect("expected a valid str").to_lowercase();

                self.normalize_regex
                    .replace_all(name.as_str(), "")
                    .to_string()
            }
        }
    }

    fn matches_quality(quality: &i32, file: &&SubtitleFile) -> bool {
        match file.quality() {
            None => true,
            Some(e) => e == quality,
        }
    }
}

impl PartialEq for SubtitleInfo {
    fn eq(&self, other: &Self) -> bool {
        self.imdb_id == other.imdb_id
            && self.language == other.language
            && self
                .files
                .iter()
                .all(|file| other.files.iter().any(|e| e == file))
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
        self.partial_cmp(other)
            .expect("expected a ordering for SubtitleInfo")
    }
}

/// A builder for constructing a `SubtitleInfo` instance.
#[derive(Debug, Default)]
pub struct SubtitleInfoBuilder {
    imdb_id: Option<String>,
    language: Option<SubtitleLanguage>,
    files: Option<Vec<SubtitleFile>>,
}

impl SubtitleInfoBuilder {
    /// Creates a new instance of `SubtitleInfoBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the IMDb ID for the subtitle info.
    pub fn imdb_id<T: ToString>(mut self, imdb_id: T) -> Self {
        self.imdb_id = Some(imdb_id.to_string());
        self
    }

    /// Sets the language for the subtitle info.
    pub fn language(mut self, language: SubtitleLanguage) -> Self {
        self.language = Some(language);
        self
    }

    /// Sets the files for the subtitle info.
    pub fn files(mut self, files: Vec<SubtitleFile>) -> Self {
        self.files = Some(files);
        self
    }

    /// Builds the `SubtitleInfo` instance.
    ///
    /// # Panics
    ///
    /// This method will panic if the language is not set.
    pub fn build(self) -> SubtitleInfo {
        SubtitleInfo {
            imdb_id: self.imdb_id,
            language: self.language.expect("language is not set"),
            files: self.files,
            normalize_regex: Regex::new(NORMALIZATION_PATTERN).unwrap(),
        }
    }
}

/// The parsed [SubtitleInfo] which has downloaded and parsed the .srt file.
#[derive(Debug, Clone, Display)]
#[display(
    fmt = "file: {:?}, info: {:?}, total cues: {}",
    file,
    info,
    "cues.len()"
)]
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
        Self { cues, info, file }
    }

    pub fn cues(&self) -> &Vec<SubtitleCue> {
        &self.cues
    }

    pub fn info(&self) -> Option<&SubtitleInfo> {
        match &self.info {
            Some(e) => Some(e),
            None => None,
        }
    }

    pub fn file(&self) -> &str {
        self.file.as_str()
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
        let info1 = SubtitleInfo::builder()
            .imdb_id("12")
            .language(SubtitleLanguage::German)
            .build();
        let info2 = SubtitleInfo::builder()
            .imdb_id("12")
            .language(SubtitleLanguage::German)
            .build();

        let result = info1 == info2;

        assert_eq!(true, result)
    }

    #[test]
    fn test_subtitle_info_partial_eq_when_id_is_different_should_return_false() {
        let info1 = SubtitleInfo::builder()
            .imdb_id("12")
            .language(SubtitleLanguage::German)
            .build();
        let info2 = SubtitleInfo::builder()
            .imdb_id("13")
            .language(SubtitleLanguage::German)
            .build();

        let result = info1 == info2;

        assert_eq!(false, result)
    }

    #[test]
    fn test_subtitle_info_partial_eq_when_language_is_different_should_return_false() {
        let info1 = SubtitleInfo::builder()
            .imdb_id("12")
            .language(SubtitleLanguage::German)
            .build();
        let info2 = SubtitleInfo::builder()
            .imdb_id("12")
            .language(SubtitleLanguage::Danish)
            .build();

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
        assert_eq!(
            result.err().unwrap(),
            SubtitleParseError::ExtensionNotSupported(extension.clone())
        )
    }

    #[test]
    fn test_subtitle_file_quality_present() {
        init_logger();
        let file_id = 49060;
        let name = "Frozen.2.2019.1080p.WEBRip.x264.AAC-[YTS.MX]".to_string();
        let expected_result = 1080;

        let result = SubtitleFile::builder()
            .file_id(file_id)
            .name(name)
            .url("")
            .score(8.0)
            .downloads(19546)
            .build();

        assert!(
            result.quality().is_some(),
            "Expected a quality to have been found"
        );
        assert_eq!(&expected_result, result.quality().unwrap())
    }

    #[test]
    fn test_subtitle_file_quality_not_present() {
        init_logger();
        let file_id = 49060;
        let name = "Frozen.II.2019.DVDScr.XVID.AC3.HQ.Hive-CM8".to_string();

        let result = SubtitleFile::builder()
            .file_id(file_id)
            .name(name)
            .url("")
            .score(8.0)
            .downloads(19546)
            .build();

        assert!(
            result.quality().is_none(),
            "Expected no quality to have been found"
        );
    }

    #[test]
    fn test_subtitle_file_order_should_return_item_with_highest_score_first() {
        let item1 = SubtitleFile::builder()
            .file_id(1)
            .name("")
            .url("")
            .score(7.0)
            .downloads(0)
            .build();
        let item2 = SubtitleFile::builder()
            .file_id(2)
            .name("")
            .url("")
            .score(6.0)
            .downloads(0)
            .build();
        let item3 = SubtitleFile::builder()
            .file_id(3)
            .name("")
            .url("")
            .score(8.0)
            .downloads(0)
            .build();
        let mut items = vec![&item1, &item2, &item3];

        items.sort_by(|a, b| a.cmp(b));

        assert_eq!(vec![&item3, &item1, &item2], items)
    }

    #[test]
    fn test_subtitle_info_normalize() {
        let filename = "Lorem.S02E11+[720p]AMZN.WEBRip.x264-GalaxyTV.mkv";
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
        let expected_file = SubtitleFile::builder()
            .file_id(102)
            .name("Lorem.S02E11.Ipsum.to.Dolor.DVDRip.Xvid-FoV.en.srt")
            .url("")
            .score(9.0)
            .downloads(44134)
            .build();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("tt100001010")
            .language(SubtitleLanguage::English)
            .files(vec![
                SubtitleFile::builder()
                    .file_id(100)
                    .name("Lorem S02 E11 Ipsum to Dolor 720p x264.srt")
                    .url("")
                    .score(0.0)
                    .downloads(6755)
                    .quality(720)
                    .build(),
                SubtitleFile::builder()
                    .file_id(101)
                    .name("Lorem.M.D.S02E11.720p.WEB.DL.nHD.x264-NhaNc3-eng.srt")
                    .url("")
                    .score(0.0)
                    .downloads(4879)
                    .quality(720)
                    .build(),
                expected_file.clone(),
                SubtitleFile::builder()
                    .file_id(103)
                    .name("Lorem MD Season 2 Episode 11 - Ipsum To Dolor-eng.srt")
                    .url("")
                    .score(0.0)
                    .downloads(5735)
                    .build(),
            ])
            .build();

        let result = subtitle_info
            .best_matching_file(&SubtitleMatcher::from_int(
                Some(filename.to_string()),
                quality,
            ))
            .expect("expected a file to be found");

        assert_eq!(expected_file, result)
    }
}
