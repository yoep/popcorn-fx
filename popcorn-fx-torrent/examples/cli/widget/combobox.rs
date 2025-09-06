use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListState, StatefulWidget, Widget};
use std::fmt::Display;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ComboboxWidget<T>
where
    T: Display,
{
    name: String,
    items: Vec<T>,
    state: Mutex<ListState>,
}

impl<T: Display> ComboboxWidget<T> {
    pub fn new<S: AsRef<str>>(name: S, items: Vec<T>, selected_value: T) -> Self {
        let state = ListState::default().with_selected(
            items
                .iter()
                .position(|e| e.to_string() == selected_value.to_string()),
        );

        Self {
            name: name.as_ref().to_string(),
            items,
            state: Mutex::new(state),
        }
    }

    /// Get the selected item.
    pub fn selected(&self) -> Option<&T> {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected().and_then(|i| self.items.get(i)))
    }

    pub fn previous(&mut self) {
        let selected = self.selected_index();
        if let Ok(mut state) = self.state.lock() {
            state.select(Some(selected.saturating_sub(1)));
        }
    }

    pub fn next(&mut self) {
        let selected = self.selected_index().saturating_add(1);
        if let Ok(mut state) = self.state.lock() {
            if selected < self.items.len() {
                state.select(Some(selected));
            }
        }
    }

    fn selected_index(&self) -> usize {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or_default()
    }
}

impl<T: Display> Widget for &ComboboxWidget<T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = self.items.iter().map(|e| e.to_string()).collect::<Vec<_>>();
        let list = List::new(items)
            .block(Block::new().title(self.name.as_str()).borders(Borders::ALL))
            .highlight_style(Style::new().bg(Color::DarkGray));

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(list, area, buf, &mut state);
        }
    }
}
