use std::os::raw::c_char;

use log::trace;

use crate::{from_c_owned, from_c_string, from_c_vec, into_c_owned, to_c_string, to_c_vec};
use crate::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};

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
    pub imdb_id: *const c_char,
    pub language: SubtitleLanguage,
    subtitle_info: *mut SubtitleInfo,
}

impl SubtitleInfoC {
    pub fn empty() -> Self {
        Self {
            imdb_id: to_c_string(String::new()),
            language: SubtitleLanguage::None,
            subtitle_info: std::ptr::null_mut(),
        }
    }

    pub fn from(info: SubtitleInfo) -> Self {
        trace!("Converting subtitle info to C for {}", &info);
        Self {
            imdb_id: match info.imdb_id() {
                None => to_c_string(String::new()),
                Some(e) => to_c_string(e.clone())
            },
            language: info.language().clone(),
            subtitle_info: into_c_owned(info),
        }
    }

    pub fn to_subtitle(self) -> SubtitleInfo {
        from_c_owned(self.subtitle_info)
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubtitleMatcherC {
    name: *const c_char,
    quality: i32,
}

impl SubtitleMatcherC {
    pub fn from(matcher: SubtitleMatcher) -> Self {
        let empty_name = "".to_string();

        Self {
            name: to_c_string(matcher.name().or_else(|| Some(&empty_name)).unwrap().clone()),
            quality: matcher.quality()
                .map(|e| e.clone())
                .or_else(|| Some(-1))
                .unwrap(),
        }
    }

    pub fn to_matcher(&self) -> SubtitleMatcher {
        let name: Option<String>;
        let quality: Option<String>;

        if self.name.is_null() {
            name = None;
        } else {
            name = Some(from_c_string(self.name))
        }

        if self.quality == -1 {
            quality = None
        } else {
            quality = Some(self.quality.to_string())
        }

        SubtitleMatcher::new(name, quality)
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
            .map(|e| SubtitleCueC::from(e))
            .collect());

        Self {
            file: to_c_string(subtitle.file().clone()),
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
        let info = self.info.clone().to_subtitle();
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
            id: to_c_string(cue.id().clone()),
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
            text: to_c_string(text.text().clone()),
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