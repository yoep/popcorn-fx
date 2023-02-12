use std::os::raw::c_char;
use std::ptr;

use log::trace;

use crate::{from_c_owned, from_c_string, from_c_vec, into_c_owned, into_c_string, to_c_vec};
use crate::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::subtitles::SubtitleFile;

#[repr(C)]
#[derive(Debug)]
pub struct SubtitleInfoSet {
    subtitles: Box<[SubtitleInfoC]>,
}

impl SubtitleInfoSet {
    pub fn new(subtitles: Box<[SubtitleInfoC]>) -> Self {
        Self {
            subtitles
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleInfoC {
    imdb_id: *const c_char,
    language: SubtitleLanguage,
    files: *mut SubtitleFileC,
    len: i32,
}

impl SubtitleInfoC {
    pub fn empty() -> Self {
        Self {
            imdb_id: into_c_string(String::new()),
            language: SubtitleLanguage::None,
            files: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn from(info: SubtitleInfo) -> Self {
        trace!("Converting subtitle info to C for {}", &info);
        let (files, len) = match info.files() {
            None => (ptr::null_mut(), 0),
            Some(files) => to_c_vec(files.into_iter()
                .map(|e| SubtitleFileC::from(e.clone()))
                .collect())
        };

        Self {
            imdb_id: match info.imdb_id() {
                None => into_c_string(String::new()),
                Some(e) => into_c_string(e.clone())
            },
            language: info.language().clone(),
            files,
            len,
        }
    }
}

impl From<&SubtitleInfoC> for SubtitleInfo {
    fn from(value: &SubtitleInfoC) -> Self {
        trace!("Converting SubtitleInfo from C for {:?}", value);
        let files = if !value.files.is_null() {
            from_c_vec(value.files, value.len).into_iter()
                .map(SubtitleFile::from)
                .collect()
        } else {
            vec![]
        };

        SubtitleInfo::new_with_files(
            from_c_string(value.imdb_id.clone()),
            value.language.clone(),
            files,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleFileC {
    file_id: i32,
    name: *const c_char,
    url: *const c_char,
    score: f32,
    downloads: i32,
    quality: *mut i32,
}

impl SubtitleFileC {
    fn from(file: SubtitleFile) -> Self {
        Self {
            file_id: *file.file_id(),
            name: into_c_string(file.name().clone()),
            url: into_c_string(file.url().clone()),
            score: *file.score(),
            downloads: *file.downloads(),
            quality: match file.quality() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(*e)
            },
        }
    }
}

impl From<SubtitleFileC> for SubtitleFile {
    fn from(value: SubtitleFileC) -> Self {
        trace!("Converting SubtitleFile from C for {:?}", &value);
        let quality = if value.quality.is_null() {
            None
        } else {
            Some(from_c_owned(value.quality))
        };

        SubtitleFile::new_with_quality(
            value.file_id,
            from_c_string(value.name),
            from_c_string(value.url),
            value.score,
            value.downloads,
            quality,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VecSubtitleInfoC {
    pub subtitles: *mut SubtitleInfoC,
    pub len: i32,
    pub cap: i32,
}

impl VecSubtitleInfoC {
    pub fn new(subtitles: *mut SubtitleInfoC, len: i32, cap: i32) -> Self {
        Self {
            subtitles,
            len,
            cap,
        }
    }

    pub fn from(mut subtitles: Vec<SubtitleInfoC>) -> Self {
        Self {
            subtitles: subtitles.as_mut_ptr(),
            len: subtitles.len() as i32,
            cap: subtitles.capacity() as i32,
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleC {
    pub file: *const c_char,
    pub info: SubtitleInfoC,
    pub cues: *mut SubtitleCueC,
    pub number_of_cues: i32,
}

impl SubtitleC {
    pub fn from(subtitle: Subtitle) -> Self {
        trace!("Converting subtitle to C for {}", subtitle);
        let (cues_ptr, number_of_cues) = to_c_vec(subtitle.cues().iter()
            .map(SubtitleCueC::from)
            .collect());

        Self {
            file: into_c_string(subtitle.file().clone()),
            info: SubtitleInfoC::from(subtitle.info()
                .map(|e| e.clone())
                .or_else(|| Some(SubtitleInfo::none()))
                .unwrap()),
            cues: cues_ptr,
            number_of_cues,
        }
    }

    pub fn to_subtitle(&self) -> Subtitle {
        trace!("Converting subtitle from C for {:?}", self);
        let info = SubtitleInfo::from(&self.info);
        let cues = from_c_vec(self.cues, self.number_of_cues);

        Subtitle::new(
            cues.iter()
                .map(|e| e.to_cue())
                .collect(),
            Some(info),
            from_c_string(self.file))
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
    use crate::testing::init_logger;

    use super::*;

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

        let result = SubtitleFile::from(subtitle_c);

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
            "tt22222233".to_string(),
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
}