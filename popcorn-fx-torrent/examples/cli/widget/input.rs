use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;
use std::ops::RangeBounds;
use std::string::Drain;

#[derive(Debug)]
pub struct InputWidget {
    text: String,
    cursor: usize,
    wrap: bool,
}

impl InputWidget {
    /// Create a new text input widget.
    pub fn new() -> Self {
        Self::new_with_opts("", false)
    }

    /// Create a new text input widget with the given options.
    pub fn new_with_opts<S: AsRef<str>>(text: S, wrap: bool) -> Self {
        Self {
            text: text.as_ref().to_string(),
            cursor: 0,
            wrap,
        }
    }

    /// Extracts a string slice containing the entire input.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    /// Append the given string slice to the existing text within the input widget.
    pub fn append(&mut self, text: &str) {
        self.text += text;
        self.cursor = text.len();
    }

    /// Inserts a character at the current cursor position.
    pub fn insert(&mut self, ch: char) {
        let position = self.text.len().min(self.cursor);
        self.text.insert(position, ch);
        self.cursor += 1;
    }

    /// Removes the specified range from the string in bulk, returning all removed characters as an iterator.
    /// The returned iterator keeps a mutable borrow on the string to optimize its implementation.
    pub fn drain<R>(&mut self, range: R) -> Drain<'_>
    where
        R: RangeBounds<usize>,
    {
        self.text.drain(range)
    }

    /// Remove the input character at the current cursor position.
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let before = self.text.chars().take(self.cursor - 1);
            let after = self.text.chars().skip(self.cursor);
            self.text = before.chain(after).collect();
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    /// Set if the text should be wrapped.
    pub fn wrap_text(&mut self, wrap: bool) -> &mut Self {
        self.wrap = wrap;
        self
    }

    /// Reset the input widget.
    pub fn reset(&mut self) {
        self.text = String::new();
        self.cursor = 0;
    }

    /// Move the cursor to the left within the input.
    pub fn cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Move the cursor to the right within the input.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    /// Render the input widget in the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // render the input
        let mut input = Paragraph::new(self.text.as_str());

        if self.wrap {
            input = input.wrap(Wrap { trim: false });
        }

        frame.render_widget(input, area);

        // render the cursor
        let area_width = area.width;
        let cursor = self.cursor as u16;
        frame.set_cursor_position(Position::new(
            area.x + (cursor % area_width),
            area.y + (cursor / area_width),
        ));
    }
}
