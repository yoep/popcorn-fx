use crate::app::{AppCommand, FXKeyEvent};
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuItem, MenuSection};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListState, StatefulWidget};
use ratatui::Frame;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct MenuOverview {
    items: Vec<MenuItem>,
    state: Mutex<ListState>,
    menu_sender: UnboundedSender<MenuCommand>,
    app_sender: UnboundedSender<AppCommand>,
}

impl MenuOverview {
    pub fn new(
        app_sender: UnboundedSender<AppCommand>,
        menu_sender: UnboundedSender<MenuCommand>,
    ) -> Self {
        Self {
            items: MenuItem::all(),
            state: Mutex::new(ListState::default().with_selected(Some(0))),
            menu_sender,
            app_sender,
        }
    }

    fn select_menu_item(&self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        match self.items[index] {
            MenuItem::AddTorrent => {
                let _ = self
                    .menu_sender
                    .send(MenuCommand::SelectSection(MenuSection::AddTorrent));
            }
            MenuItem::Settings => {
                let _ = self
                    .menu_sender
                    .send(MenuCommand::SelectSection(MenuSection::Settings));
            }
            MenuItem::Quit => {
                let _ = self.app_sender.send(AppCommand::Quit);
            }
        }
    }
}

impl MenuSectionWidget for MenuOverview {
    fn preferred_width(&self) -> u16 {
        20
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        match key.code() {
            KeyCode::Up => {
                if let Ok(mut state) = self.state.lock() {
                    key.consume();
                    let offset = state.selected().unwrap_or(0).saturating_sub(1);
                    state.select(Some(offset));
                }
            }
            KeyCode::Down => {
                if let Ok(mut state) = self.state.lock() {
                    key.consume();
                    let mut offset = state.selected().unwrap_or(0).saturating_add(1);
                    if offset > self.items.len() {
                        offset = self.items.len() - 1;
                    }

                    state.select(Some(offset));
                }
            }
            KeyCode::Enter => {
                if let Ok(state) = self.state.lock() {
                    key.consume();
                    let menu_index = state.selected().unwrap_or(0);
                    self.select_menu_item(menu_index);
                }
            }
            KeyCode::Char(char) => {
                if let Some(menu_index) = char.to_digit(10).map(|e| e as usize) {
                    if menu_index < self.items.len() {
                        key.consume();
                        self.select_menu_item(menu_index);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        Widget::render(self, area, frame.buffer_mut());
    }
}

impl Widget for &MenuOverview {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{}. {}", i + 1, item))
            .collect::<Vec<_>>();
        let menu_list = List::new(items)
            .block(Block::new().title("Options").borders(Borders::ALL))
            .highlight_style(Style::new().bg(Color::DarkGray));

        let mut state = self.state.lock().expect("Mutex poisoned");
        StatefulWidget::render(menu_list, area, buf, &mut state);
    }
}
