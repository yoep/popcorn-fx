use crate::app::{AppCommand, AppCommandSender, FXKeyEvent};
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuSection};
use crate::widget::{CheckboxWidget, InputWidget};
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, StatefulWidget, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListState, Widget};
use ratatui::Frame;
use std::fmt::Debug;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

trait Setting: Debug + Send {
    fn on_key_event(&mut self, key: FXKeyEvent);

    fn text(&'_ self) -> Text<'_>;

    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Debug)]
pub struct MenuSettings {
    items: Vec<Box<dyn Setting>>,
    active_menu: ActiveMenuSetting,
    state: Mutex<ListState>,
    menu_sender: UnboundedSender<MenuCommand>,
}

impl MenuSettings {
    pub fn new(app_sender: AppCommandSender, menu_sender: UnboundedSender<MenuCommand>) -> Self {
        Self {
            menu_sender,
            items: vec![
                Box::new(DhtSetting::new(app_sender.clone())),
                Box::new(TrackerSetting::new(app_sender.clone())),
                Box::new(StorageSetting::new(app_sender)),
            ],
            active_menu: ActiveMenuSetting::Overview,
            state: Mutex::new(ListState::default().with_selected(Some(0))),
        }
    }

    fn selected_index(&self) -> usize {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or(0)
    }

    fn selected(&self) -> &Box<dyn Setting> {
        let selected = self.selected_index();

        self.items
            .get(selected)
            .expect("expected a valid item to have been selected")
    }

    fn selected_mut(&mut self) -> &mut Box<dyn Setting> {
        let selected = self.selected_index();

        self.items
            .get_mut(selected)
            .expect("expected a valid item to have been selected")
    }
}

impl MenuSectionWidget for MenuSettings {
    fn preferred_width(&self) -> u16 {
        128
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        if self.active_menu == ActiveMenuSetting::Overview {
            match key.code() {
                KeyCode::Up => {
                    key.consume();
                    let selected = self.selected_index();
                    if let Ok(mut state) = self.state.lock() {
                        key.consume();
                        let offset = selected.saturating_sub(1);
                        state.select(Some(offset));
                    }
                }
                KeyCode::Down => {
                    key.consume();
                    let selected = self.selected_index().saturating_add(1);
                    if let Ok(mut state) = self.state.lock() {
                        if selected < self.items.len() - 1 {
                            state.select(Some(selected));
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    key.consume();
                    let _ = self
                        .menu_sender
                        .send(MenuCommand::SelectSection(MenuSection::Overview));
                }
                KeyCode::Enter => {
                    self.selected_mut().on_key_event(key);
                }
                _ => {}
            }
        } else {
            self.selected_mut().on_key_event(key);
        }
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.active_menu == ActiveMenuSetting::Overview {
            let items = self.items.iter().map(|e| e.text()).collect::<Vec<_>>();
            let menu_list = List::new(items)
                .block(Block::new().title("Settings").borders(Borders::ALL))
                .highlight_style(Style::new().bg(Color::DarkGray));

            let mut state = self.state.lock().expect("Mutex poisoned");
            StatefulWidget::render(menu_list, area, frame.buffer_mut(), &mut state);
        } else {
            self.selected().render(frame, area);
        }
    }
}

#[derive(Debug, PartialEq)]
enum ActiveMenuSetting {
    Overview,
    Storage,
}

#[derive(Debug)]
struct DhtSetting {
    checkbox: CheckboxWidget,
    app_sender: AppCommandSender,
}

impl DhtSetting {
    fn new(app_sender: AppCommandSender) -> Self {
        Self {
            checkbox: CheckboxWidget::new("DHT", true),
            app_sender,
        }
    }
}

impl Setting for DhtSetting {
    fn on_key_event(&mut self, key: FXKeyEvent) {
        if key.code() == KeyCode::Enter {
            self.checkbox.toggle();
            let _ = self
                .app_sender
                .send(AppCommand::DhtEnabled(self.checkbox.is_checked()));
        }
    }

    fn text(&'_ self) -> Text<'_> {
        Text::from(&self.checkbox)
    }

    fn render(&self, _: &mut Frame, _: Rect) {
        // no-op
    }
}

#[derive(Debug)]
struct TrackerSetting {
    checkbox: CheckboxWidget,
    app_sender: AppCommandSender,
}

impl TrackerSetting {
    fn new(app_sender: AppCommandSender) -> Self {
        Self {
            checkbox: CheckboxWidget::new("Tracker", true),
            app_sender,
        }
    }
}

impl Setting for TrackerSetting {
    fn on_key_event(&mut self, key: FXKeyEvent) {
        if key.code() == KeyCode::Enter {
            self.checkbox.toggle();
            let _ = self
                .app_sender
                .send(AppCommand::TrackerEnabled(self.checkbox.is_checked()));
        }
    }

    fn text(&'_ self) -> Text<'_> {
        Text::from(&self.checkbox)
    }

    fn render(&self, _: &mut Frame, _: Rect) {
        // no-op
    }
}

#[derive(Debug)]
struct StorageSetting {
    input: InputWidget,
    app_sender: AppCommandSender,
}

impl StorageSetting {
    fn new(app_sender: AppCommandSender) -> Self {
        Self {
            input: InputWidget::new(),
            app_sender,
        }
    }
}

impl Setting for StorageSetting {
    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        match key.code() {
            KeyCode::Esc => {
                key.consume();
                self.input.reset();
            }
            KeyCode::Backspace => {
                key.consume();
                self.input.backspace();
            }
            KeyCode::Enter => {
                key.consume();
                // TODO
            }
            KeyCode::Char(char) => {
                key.consume();
                self.input.insert(char);
            }
            KeyCode::Left => {
                key.consume();
                self.input.cursor_left();
            }
            KeyCode::Right => {
                key.consume();
                self.input.cursor_right();
            }
            _ => {}
        }
    }

    fn text(&'_ self) -> Text<'_> {
        self.input.as_str().into()
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let border = Block::new().title("Storage location").borders(Borders::ALL);

        self.input.render(frame, border.inner(area));
        border.render(area, frame.buffer_mut());
    }
}
