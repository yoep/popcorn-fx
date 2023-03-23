use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::{from_c_owned, from_c_string, from_c_vec, into_c_owned, into_c_string, to_c_vec};
use popcorn_fx_core::core::subtitles::{SubtitleEvent, SubtitleFile};
use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo};

/// The C compatible [SubtitleInfo] representation.
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubtitleInfoC {
    /// The IMDB ID if known, this can be [ptr::null]
    pub imdb_id: *const c_char,
    pub language: SubtitleLanguage,
    pub files: *mut SubtitleFileC,
    pub len: i32,
}

impl SubtitleInfoC {
    pub fn empty() -> Self {
        Self {
            imdb_id: ptr::null(),
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
            Some(files) => to_c_vec(files.into_iter()
                .map(|e| SubtitleFileC::from(e.clone()))
                .collect())
        };

        Self {
            imdb_id: match value.imdb_id() {
                None => ptr::null(),
                Some(e) => into_c_string(e.clone())
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
            from_c_vec(value.files, value.len).iter()
                .map(SubtitleFile::from)
                .collect()
        } else {
            vec![]
        };

        SubtitleInfo::new_with_files(
            imdb_id,
            value.language.clone(),
            files,
        )
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
            from_c_vec(value.files, value.len).iter()
                .map(SubtitleFile::from)
                .collect()
        } else {
            vec![]
        };

        SubtitleInfo::new_with_files(
            imdb_id,
            value.language.clone(),
            files,
        )
    }
}

/// The C compatible [SubtitleEvent] representation
#[repr(C)]
#[derive(Debug)]
pub enum SubtitleEventC {
    SubtitleInfoChanged(*mut SubtitleInfoC),
    PreferredLanguageChanged(SubtitleLanguage),
}

impl From<SubtitleEvent> for SubtitleEventC {
    fn from(value: SubtitleEvent) -> Self {
        trace!("Converting SubtitleEvent to C for {:?}", value);
        match value {
            SubtitleEvent::SubtitleInfoChanged(info) => SubtitleEventC::SubtitleInfoChanged(info
                .map(|e| into_c_owned(SubtitleInfoC::from(e)))
                .or_else(|| Some(ptr::null_mut()))
                .unwrap()),
            SubtitleEvent::PreferredLanguageChanged(language) => SubtitleEventC::PreferredLanguageChanged(language),
        }
    }
}

/// The C compatible [SubtitleFile] representation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleFileC {
    pub file_id: i32,
    pub name: *const c_char,
    pub url: *const c_char,
    pub score: f32,
    pub downloads: i32,
    pub quality: *const i32,
}

impl From<SubtitleFile> for SubtitleFileC {
    fn from(value: SubtitleFile) -> Self {
        trace!("Converting SubtitleFile to C for {:?}", &value);
        Self {
            file_id: *value.file_id(),
            name: into_c_string(value.name().clone()),
            url: into_c_string(value.url().clone()),
            score: *value.score(),
            downloads: *value.downloads(),
            quality: match value.quality() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(*e)
            },
        }
    }
}

impl From<&SubtitleFileC> for SubtitleFile {
    fn from(value: &SubtitleFileC) -> Self {
        trace!("Converting SubtitleFile from C for {:?}", &value);
        let quality = if value.quality.is_null() {
            None
        } else {
            Some(unsafe { value.quality.read() })
        };
        let name = from_c_string(value.name);
        let url = from_c_string(value.url);

        SubtitleFile::new_with_quality(
            value.file_id,
            name,
            url,
            value.score,
            value.downloads,
            quality,
        )
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
        let (subtitles, len) = to_c_vec(value);

        Self {
            subtitles,
            len,
        }
    }
}

/// The subtitle matcher C compatible struct.
/// It contains the information which should be matched when selecting a subtitle file to load.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleMatcherC {
    /// The nullable name of the media item.
    name: *const c_char,
    /// The nullable quality of the media item.
    /// This can be represented as `720p` or `720`.
    quality: *const c_char,
}

impl SubtitleMatcherC {
    pub fn from(matcher: SubtitleMatcher) -> Self {
        Self {
            name: match matcher.name() {
                None => ptr::null(),
                Some(e) => into_c_string(e.to_string())
            },
            quality: match matcher.quality() {
                None => ptr::null(),
                Some(e) => into_c_string(e.to_string())
            },
        }
    }

    pub fn to_matcher(&self) -> SubtitleMatcher {
        trace!("Converting matcher from C for {:?}", self);
        let name = if self.name.is_null() {
            None
        } else {
            Some(from_c_string(self.name))
        };
        let quality = if self.quality.is_null() {
            None
        } else {
            Some(from_c_string(self.quality))
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
    pub file: *const c_char,
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
        let (cues_ptr, number_of_cues) = to_c_vec(value.cues().iter()
            .map(SubtitleCueC::from)
            .collect());
        let info = match value.info() {
            None => ptr::null_mut(),
            Some(e) => into_c_owned(SubtitleInfoC::from(e.clone()))
        };

        Self {
            file: into_c_string(value.file().clone()),
            info,
            cues: cues_ptr,
            len: number_of_cues,
        }
    }
}

impl From<SubtitleC> for Subtitle {
    fn from(value: SubtitleC) -> Self {
        trace!("Converting Subtitle from C for {:?}", value);
        let cues = from_c_vec(value.cues, value.len).into_iter()
            .map(|e| e.to_cue())
            .collect();
        let info = if !value.info.is_null() {
            Some(SubtitleInfo::from(from_c_owned(value.info)))
        } else {
            None
        };

        Subtitle::new(
            cues,
            info,
            from_c_string(value.file))
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleCueC {
    pub id: *const c_char,
    pub start_time: u64,
    pub end_time: u64,
    pub lines: *mut SubtitleLineC,
    pub number_of_lines: i32,
}

impl SubtitleCueC {
    pub fn from(cue: &SubtitleCue) -> Self {
        trace!("Converting cue to C for {}", cue);
        let (lines, number_of_lines) = to_c_vec(cue.lines().iter()
            .map(|e| SubtitleLineC::from(e))
            .collect());

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
            lines.iter()
                .map(|e| e.to_line())
                .collect())
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
        let (texts, number_of_texts) = to_c_vec(line.texts().iter()
            .map(|e| StyledTextC::from(e))
            .collect());

        Self {
            texts,
            len: number_of_texts,
        }
    }

    pub fn to_line(&self) -> SubtitleLine {
        let texts = from_c_vec(self.texts, self.len);

        SubtitleLine::new(texts.iter()
            .map(|e| e.to_text())
            .collect())
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct StyledTextC {
    pub text: *const c_char,
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

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_subtitle_info_set_from() {
        init_logger();
        let subtitle = SubtitleInfo::new(
            "tt111000".to_string(),
            SubtitleLanguage::French,
        );
        let subtitles = vec![SubtitleInfoC::from(subtitle.clone())];

        let set = SubtitleInfoSet::from(subtitles);
        assert_eq!(1, set.len);

        let subtitles = from_c_vec(set.subtitles, set.len);
        let result = subtitles.get(0);
        assert_eq!(subtitle, SubtitleInfo::from(result.unwrap()))
    }

    #[test]
    fn test_subtitle_file_from() {
        init_logger();
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
        init_logger();
        let subtitle = SubtitleInfo::new_with_files(
            Some("tt22222233".to_string()),
            SubtitleLanguage::Italian,
            vec![SubtitleFile::new(
                1,
                "lorem".to_string(),
                String::new(),
                8.0,
                1544,
            )],
        );

        let info_c = SubtitleInfoC::from(subtitle.clone());
        let result = SubtitleInfo::from(&info_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_subtitle_info_without_files() {
        init_logger();
        let subtitle = SubtitleInfo::new(
            "tt8788777".to_string(),
            SubtitleLanguage::Spanish,
        );

        let info_c = SubtitleInfoC::from(subtitle.clone());
        let result = SubtitleInfo::from(&info_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_subtitle_info_none() {
        init_logger();
        let info = SubtitleInfo::none();

        let subtitle_info_c = SubtitleInfoC::from(info.clone());
        assert_eq!(ptr::null(), subtitle_info_c.imdb_id);

        let result = SubtitleInfo::from(subtitle_info_c);
        assert_eq!(info, result)
    }

    #[test]
    fn test_subtitle_from() {
        init_logger();
        let subtitle = create_simple_subtitle();

        let subtitle_c = SubtitleC::from(subtitle.clone());
        let result = Subtitle::from(subtitle_c);

        assert_eq!(subtitle, result)
    }

    #[test]
    fn test_from_subtitle_event() {
        init_logger();
        let imdb_id = "tt122121";
        let info_none_event = SubtitleEvent::SubtitleInfoChanged(None);
        let subtitle_info = SubtitleInfo::new(imdb_id.to_string(), SubtitleLanguage::Finnish);
        let info_event = SubtitleEvent::SubtitleInfoChanged(Some(subtitle_info.clone()));

        let info_event_result = SubtitleEventC::from(info_event);

        match SubtitleEventC::from(info_none_event) {
            SubtitleEventC::SubtitleInfoChanged(info) => assert_eq!(ptr::null_mut(), info),
            _ => assert!(false, "expected SubtitleEventC::SubtitleInfoChanged"),
        }
        match info_event_result {
            SubtitleEventC::SubtitleInfoChanged(info) => {
                let subtitle_info_c = from_c_owned(info);

                assert_eq!(imdb_id.to_string(), from_c_string(subtitle_info_c.imdb_id));
                assert_eq!(SubtitleLanguage::Finnish, subtitle_info_c.language);
            },
            _ => assert!(false, "expected SubtitleEventC::SubtitleInfoChanged"),
        }
    }

    fn create_simple_subtitle() -> Subtitle {
        Subtitle::new(
            vec![SubtitleCue::new(
                "01".to_string(),
                1200,
                2000,
                vec![SubtitleLine::new(
                    vec![StyledText::new(
                        "lorem".to_string(),
                        false,
                        false,
                        false,
                    )]
                )],
            )],
            Some(SubtitleInfo::new("tt00001".to_string(), SubtitleLanguage::English)),
            "lorem.srt".to_string(),
        )
    }
}