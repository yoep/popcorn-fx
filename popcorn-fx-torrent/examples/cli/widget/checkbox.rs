use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Span, Text};
use ratatui::widgets::Widget;

const CHECKBOX_FILLED: &str = "\u{25A0}";
const CHECKBOX_EMPTY: &str = "\u{25A1}";

#[derive(Debug)]
pub struct CheckboxWidget {
    name: String,
    checked: bool,
}

impl CheckboxWidget {
    pub fn new<S: AsRef<str>>(name: S, default: bool) -> Self {
        Self {
            name: name.as_ref().to_string(),
            checked: default,
        }
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    pub fn as_str(&self) -> String {
        let icon = if self.checked {
            CHECKBOX_FILLED
        } else {
            CHECKBOX_EMPTY
        };

        format!("{} {}", icon, self.name)
    }
}

impl Widget for &CheckboxWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Span::raw(self.as_str()).render(area, buf);
    }
}

impl<'a> From<&'a CheckboxWidget> for Text<'a> {
    fn from(value: &'a CheckboxWidget) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<&'a CheckboxWidget> for Span<'a> {
    fn from(value: &'a CheckboxWidget) -> Self {
        Self::from(value.as_str())
    }
}
