use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo};
use popcorn_fx_core::core::subtitles::{SubtitleEvent, SubtitleFile, SubtitlePreference};
use popcorn_fx_core::{
    from_c_owned, from_c_string, from_c_vec, from_c_vec_owned, into_c_owned, into_c_string,
    into_c_vec,
};

/// The C compatible [SubtitleInfo] representation.
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubtitleInfoC {
    /// The IMDB ID if known, this can be [ptr::null]
    pub imdb_id: *mut c_char,
    pub language: SubtitleLanguage,
    pub files: *mut SubtitleFileC,
    pub len: i32,
}

impl SubtitleInfoC {
    pub fn empty() -> Self {
        Self {
            imdb_id: ptr::null_mut(),
            language: SubtitleLanguage::None,
            files: ptr::null_mut(),
            len: 0,
        }
    }
}

impl From<SubtitleInfo> for SubtitleInfoC {
    fn from(value: SubtitleInfo) -> Self {
        trace!("Converting subtitle info to C for {}", &value);
        let (files, len) = match value.files() {
            None => (ptr::null_mut(), 0),
            Some(files) => into_c_vec(
                files
                    .into_iter()
                    .map(|e| SubtitleFileC::from(e.clone()))
                    .collect(),
            ),
        };

        Self {
            imdb_id: match value.imdb_id() {
                None => ptr::null_mut(),
                Some(e) => into_c_string(e.clone()),
            },
            language: value.language().clone(),
            files,
            len,
        }
    }
}

impl From<&SubtitleInfoC> for SubtitleInfo {
    fn from(value: &SubtitleInfoC) -> Self {
        trace!("Converting SubtitleInfo from C for {:?}", value);
        let imdb_id = if !value.imdb_id.is_null() {
            Some(from_c_string(value.imdb_id))
        } else {
            None
        };
        let files = if !value.files.is_null() && value.len > 0 {
            from_c_vec(value.files, value.len)
                .iter()
                .map(SubtitleFile::from)
                .collect()
        } else {
            vec![]
        };

        let mut builder = SubtitleInfo::builder()
            .language(value.language.clone())
            .files(files);

        if let Some(e) = imdb_id {
            builder = builder.imdb_id(e);
        }

        builder.build()
    }
}

impl From<SubtitleInfoC> for SubtitleInfo {
    fn from(value: SubtitleInfoC) -> Self {
        trace!("Converting SubtitleInfo from C for {:?}", value);
        let imdb_id = if !value.imdb_id.is_null() {
            Some(from_c_string(value.imdb_id))
        } else {
            None
        };
        let files = if !value.files.is_null() && value.len > 0 {
            from_c_vec(value.files, value.len)
                .iter()
                .map(SubtitleFile::from)
                .collect()
        } else {
            vec![]
        };

        let mut builder = SubtitleInfo::builder()
            .language(value.language.clone())
            .files(files);

        if let Some(e) = imdb_id {
            builder = builder.imdb_id(e);
        }

        builder.build()
    }
}

impl Drop for SubtitleInfoC {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // if !self.imdb_id.is_null() {
        //     let _ = from_c_string_owned(self.imdb_id);
        // }

        // if !self.files.is_null() {
        //     let _ = from_c_vec_owned(self.files, self.len);
        // }
    }
}

/// The C compatible [SubtitleEvent] representation
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum SubtitleEventC {
    PreferenceChanged(SubtitlePreference),
}

impl From<SubtitleEvent> for SubtitleEventC {
    fn from(value: SubtitleEvent) -> Self {
        trace!("Converting SubtitleEvent to C for {:?}", value);
        match value {
            SubtitleEvent::PreferenceChanged(preference) => {
                SubtitleEventC::PreferenceChanged(preference)
            }
        }
    }
}

/// The C compatible [SubtitleFile] representation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleFileC {
    pub file_id: i32,
    pub name: *mut c_char,
    pub url: *mut c_char,
    pub score: f32,
    pub downloads: i32,
    pub quality: *const i32,
}

impl From<SubtitleFile> for SubtitleFileC {
    fn from(value: SubtitleFile) -> Self {
        trace!("Converting SubtitleFile to C for {:?}", &value);
        Self {
            file_id: *value.file_id(),
            name: into_c_string(value.name().to_string()),
            url: into_c_string(value.url().clone()),
            score: *value.score(),
            downloads: *value.downloads(),
            quality: match value.quality() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(*e),
            },
        }
    }
}

impl From<&SubtitleFileC> for SubtitleFile {
    fn from(value: &SubtitleFileC) -> Self {
        trace!("Converting SubtitleFile from C for {:?}", &value);
        let name = from_c_string(value.name);
        let url = from_c_string(value.url);

        let mut builder = Self::builder()
            .file_id(value.file_id)
            .name(name)
            .url(url)
            .score(value.score)
            .downloads(value.downloads);

        if !value.quality.is_null() {
            builder = builder.quality(unsafe { value.quality.read() });
        }

        builder.build()
    }
}

impl Drop for SubtitleFileC {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // if !self.name.is_null() {
        //     let _ = from_c_string_owned(self.name);
        // }
        // if !self.url.is_null() {
        //     let _ = from_c_string_owned(self.url);
        // }
    }
}

/// The C array of available [SubtitleInfo].
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleInfoSet {
    /// The available subtitle array
    pub subtitles: *mut SubtitleInfoC,
    /// The length of the array
    pub len: i32,
}

impl From<Vec<SubtitleInfoC>> for SubtitleInfoSet {
    fn from(value: Vec<SubtitleInfoC>) -> Self {
        let (subtitles, len) = into_c_vec(value);

        Self { subtitles, len }
    }
}

impl Drop for SubtitleInfoSet {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // let _ = from_c_vec(self.subtitles, self.len);
        // from_c_vec_owned(self.subtitles, self.len);
    }
}

/// The subtitle matcher C compatible struct.
/// It contains the information which should be matched when selecting a subtitle file to load.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleMatcherC {
    /// The nullable name of the media item.
    name: *mut c_char,
    /// The nullable quality of the media item.
    /// This can be represented as `720p` or `720`.
    quality: *mut c_char,
}

impl SubtitleMatcherC {
    pub fn from(matcher: SubtitleMatcher) -> Self {
        Self {
            name: match matcher.name() {
                None => ptr::null_mut(),
                Some(e) => into_c_string(e.to_string()),
            },
            quality: match matcher.quality() {
                None => ptr::null_mut(),
                Some(e) => into_c_string(e.to_string()),
            },
        }
    }
}

impl Drop for SubtitleMatcherC {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // let _ = from_c_string_owned(self.name);
        // let _ = from_c_string_owned(self.quality);
    }
}

impl From<&SubtitleMatcherC> for SubtitleMatcher {
    fn from(value: &SubtitleMatcherC) -> Self {
        trace!("Converting matcher from C for {:?}", value);
        let name = if value.name.is_null() {
            None
        } else {
            Some(from_c_string(value.name))
        };
        let quality = if value.quality.is_null() {
            None
        } else {
            Some(from_c_string(value.quality))
        };

        SubtitleMatcher::from_string(name, quality)
    }
}

/// The parsed subtitle representation for C.
/// It contains the data of a subtitle file that can be displayed.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleC {
    /// The filepath that has been parsed
    pub file: *mut c_char,
    /// The info of the parsed subtitle if available, else [ptr::null_mut]
    pub info: *mut SubtitleInfoC,
    /// The parsed cues from the subtitle file
    pub cues: *mut SubtitleCueC,
    /// The total number of cue elements
    pub len: i32,
}

impl From<Subtitle> for SubtitleC {
    fn from(value: Subtitle) -> Self {
        trace!("Converting subtitle to C for {}", value);
        let (cues_ptr, number_of_cues) =
            into_c_vec(value.cues().iter().map(SubtitleCueC::from).collect());
        let info = match value.info() {
            None => ptr::null_mut(),
            Some(e) => into_c_owned(SubtitleInfoC::from(e.clone())),
        };

        Self {
            file: into_c_string(value.file()),
            info,
            cues: cues_ptr,
            len: number_of_cues,
        }
    }
}

impl From<SubtitleC> for Subtitle {
    fn from(value: SubtitleC) -> Self {
        trace!("Converting Subtitle from C for {:?}", value);
        let cues = from_c_vec(value.cues, value.len)
            .into_iter()
            .map(|e| e.to_cue())
            .collect();
        let info = if !value.info.is_null() {
            Some(SubtitleInfo::from(from_c_owned(value.info)))
        } else {
            None
        };

        Subtitle::new(cues, info, from_c_string(value.file))
    }
}

impl Drop for SubtitleC {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // if !self.info.is_null() {
        //     let info = from_c_owned(self.info);
        //     drop(info);
        // }

        drop(from_c_vec_owned(self.cues, self.len));
    }
}

/// Represents a cue in a subtitle track in a C-compatible format.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleCueC {
    /// A pointer to a null-terminated C string representing the cue identifier.
    pub id: *mut c_char,
    /// The start time of the cue in milliseconds.
    pub start_time: u64,
    /// The end time of the cue in milliseconds.
    pub end_time: u64,
    /// A pointer to an array of subtitle lines.
    pub lines: *mut SubtitleLineC,
    /// The number of lines in the cue.
    pub number_of_lines: i32,
}

impl SubtitleCueC {
    pub fn from(cue: &SubtitleCue) -> Self {
        trace!("Converting cue to C for {}", cue);
        let (lines, number_of_lines) =
            into_c_vec(cue.lines().iter().map(|e| SubtitleLineC::from(e)).collect());

        Self {
            id: into_c_string(cue.id().clone()),
            start_time: cue.start_time().clone(),
            end_time: cue.end_time().clone(),
            lines,
            number_of_lines,
        }
    }

    pub fn to_cue(&self) -> SubtitleCue {
        let id = from_c_string(self.id);
        let start_time = self.start_time.clone();
        let end_time = self.end_time.clone();
        let lines = from_c_vec(self.lines, self.number_of_lines);

        SubtitleCue::new(
            id,
            start_time,
            end_time,
            lines.iter().map(|e| e.to_line()).collect(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleLineC {
    pub texts: *mut StyledTextC,
    pub len: i32,
}

impl SubtitleLineC {
    pub fn from(line: &SubtitleLine) -> Self {
        trace!("Converting subtitle line to C for {}", line);
        let (texts, number_of_texts) =
            into_c_vec(line.texts().iter().map(|e| StyledTextC::from(e)).collect());

        Self {
            texts,
            len: number_of_texts,
        }
    }

    pub fn to_line(&self) -> SubtitleLine {
        let texts = from_c_vec(self.texts, self.len);

        SubtitleLine::new(texts.iter().map(|e| e.to_text()).collect())
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StyledTextC {
    pub text: *mut c_char,
    pub italic: bool,
    pub bold: bool,
    pub underline: bool,
}

impl StyledTextC {
    pub fn from(text: &StyledText) -> Self {
        Self {
            text: into_c_string(text.text().clone()),
            italic: text.italic().clone(),
            bold: text.bold().clone(),
            underline: text.underline().clone(),
        }
    }

    pub fn to_text(&self) -> StyledText {
        let italic = self.italic.clone();
        let bold = self.bold.clone();
        let underline = self.underline.clone();

        StyledText::new(from_c_string(self.text), italic, bold, underline)
    }
}

impl Drop for StyledTextC {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // let _ = from_c_string_owned(self.text);
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
    use popcorn_fx_core::init_logger;

    use super::*;

    #[test]
    fn test_subtitle_info_set_from() {
        init_logger!();
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt111000")
            .language(SubtitleLanguage::French)
            .build();
        let subtitles = vec![SubtitleInfoC::from(subtitle.clone())];

        let set = SubtitleInfoSet::from(subtitles);
        assert_eq!(1, set.len);

        let subtitles = from_c_vec(set.subtitles, set.len);
        let result = subtitles.get(0);
        assert_eq!(subtitle, SubtitleInfo::from(result.unwrap()))
    }

    #[test]
    fn test_subtitle_file_from() {
        init_logger!();
        let name = "lorem".to_string();
        let url = "ipsum".to_string();
        let subtitle_c = SubtitleFileC {
            file_id: 12,
            name: into_c_string(name.clone()),
            url: into_c_string(url.clone()),
            score: 7.3,
            downloads: 8754,
            quality: ptr::null_mut(),
        };

        let result = SubtitleFile::from(&subtitle_c);

        assert_eq!(&12, result.file_id());
        assert_eq!(&name, result.name());
        assert_eq!(&url, result.url());
        assert_eq!(&7.3, result.score());
        assert_eq!(&8754, result.downloads());
        assert_eq!(None, result.quality());
    }

    #[test]
    fn test_subtitle_info_with_files() {
        init_logger!();
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt22222233")
            .language(SubtitleLanguage::Italian)
            .files(vec![SubtitleFile::builder()
                .file_id(1)
                .name("lorem")
                .url("")
                .score(8.0)
                .downloads(1544)
                .build()])
            .build();

        let info_c = SubtitleInfoC::from(subtitle.clone());
        let result = SubtitleInfo::from(&info_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_subtitle_info_without_files() {
        init_logger!();
        let subtitle = SubtitleInfo::builder()
            .imdb_id("tt8788777")
            .language(SubtitleLanguage::Spanish)
            .build();

        let info_c = SubtitleInfoC::from(subtitle.clone());
        let result = SubtitleInfo::from(&info_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_subtitle_info_none() {
        init_logger!();
        let info = SubtitleInfo::none();

        let subtitle_info_c = SubtitleInfoC::from(info.clone());
        assert_eq!(ptr::null(), subtitle_info_c.imdb_id);

        let result = SubtitleInfo::from(subtitle_info_c);
        assert_eq!(info, result)
    }

    #[test]
    fn test_subtitle_from() {
        init_logger!();
        let subtitle = create_simple_subtitle();

        let subtitle_c = SubtitleC::from(subtitle.clone());
        let result = Subtitle::from(subtitle_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_from_subtitle_event() {
        init_logger!();
        let preference = SubtitlePreference::Language(SubtitleLanguage::None);
        let info_event = SubtitleEvent::PreferenceChanged(preference.clone());
        let expected_result = SubtitleEventC::PreferenceChanged(preference);

        let result = SubtitleEventC::from(info_event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_subtitle_matcher_from() {
        let name = "FooBar";
        let quality = "720p";
        let matcher = SubtitleMatcherC {
            name: into_c_string(name.to_string()),
            quality: into_c_string(quality.to_string()),
        };
        let expected_result =
            SubtitleMatcher::from_string(Some(name.to_string()), Some(quality.to_string()));

        let result = SubtitleMatcher::from(&matcher);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_drop_subtitle_file_c() {
        let subtitle = SubtitleFileC {
            file_id: 0,
            name: into_c_string("FooBar"),
            url: into_c_string("LoremIpsum"),
            score: 0.0,
            downloads: 0,
            quality: 720 as *const i32,
        };

        drop(subtitle);
    }

    fn create_simple_subtitle() -> Subtitle {
        Subtitle::new(
            vec![SubtitleCue::new(
                "01".to_string(),
                1200,
                2000,
                vec![SubtitleLine::new(vec![StyledText::new(
                    "lorem".to_string(),
                    false,
                    false,
                    false,
                )])],
            )],
            Some(
                SubtitleInfo::builder()
                    .imdb_id("tt00001")
                    .language(SubtitleLanguage::English)
                    .build(),
            ),
            "lorem.srt".to_string(),
        )
    }
}
