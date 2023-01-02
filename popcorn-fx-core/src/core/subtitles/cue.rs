use std::cmp::Ordering;

use derive_more::Display;

/// A parsed subtitle cue line from a subtitle file.
#[derive(Debug, Display, Clone, Eq, PartialEq)]
#[display(fmt = "id: {}, start_time: {}, end_time: {}, lines: {:?}", id, start_time, end_time, lines)]
pub struct SubtitleCue {
    id: String,
    start_time: u64,
    end_time: u64,
    lines: Vec<SubtitleLine>,
}

impl SubtitleCue {
    pub fn new(id: String, start_time: u64, end_time: u64, lines: Vec<SubtitleLine>) -> Self {
        Self {
            id,
            start_time,
            end_time,
            lines,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn start_time(&self) -> &u64 {
        &self.start_time
    }

    pub fn end_time(&self) -> &u64 {
        &self.end_time
    }

    pub fn lines(&self) -> &Vec<SubtitleLine> {
        &self.lines
    }
}

impl PartialOrd<Self> for SubtitleCue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time.partial_cmp(other.start_time())
    }
}

impl Ord for SubtitleCue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_time.cmp(other.start_time())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubtitleCueBuilder {
    id: String,
    start_time: u64,
    end_time: u64,
    lines: Vec<SubtitleLine>,
}

impl SubtitleCueBuilder {
    pub fn new() -> Self {
        Self {
            id: "".to_string(),
            start_time: 0,
            end_time: 0,
            lines: vec![],
        }
    }

    pub fn build(&self) -> SubtitleCue {
        SubtitleCue::new(self.id.clone(), self.start_time.clone(), self.end_time.clone(), self.lines.clone())
    }

    pub fn id(&mut self, id: String) -> &mut Self {
        self.id = id;
        self
    }

    pub fn start_time(&mut self, start_time: u64) -> &mut Self {
        self.start_time = start_time;
        self
    }

    pub fn end_time(&mut self, end_time: u64) -> &mut Self {
        self.end_time = end_time;
        self
    }

    pub fn add_line(&mut self, line: SubtitleLine) -> &mut Self {
        self.lines.push(line);
        self
    }
}

/// The subtitle line which is a new line within a subtitle
#[derive(Debug, Clone, Eq, PartialEq, Display)]
#[display(fmt = "texts: {:?}", texts)]
pub struct SubtitleLine {
    texts: Vec<StyledText>,
}

impl SubtitleLine {
    pub fn new(texts: Vec<StyledText>) -> Self {
        Self {
            texts
        }
    }

    pub fn texts(&self) -> &Vec<StyledText> {
        &self.texts
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StyledText {
    text: String,
    italic: bool,
    bold: bool,
    underline: bool,
}

impl StyledText {
    pub fn new(text: String, italic: bool, bold: bool, underline: bool) -> Self {
        Self {
            text,
            italic,
            bold,
            underline,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn italic(&self) -> &bool {
        &self.italic
    }

    pub fn bold(&self) -> &bool {
        &self.bold
    }

    pub fn underline(&self) -> &bool {
        &self.underline
    }
}