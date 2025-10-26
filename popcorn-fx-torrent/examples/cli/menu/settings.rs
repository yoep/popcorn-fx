use crate::app::{AppCommand, AppCommandSender, FXKeyEvent, APP_DEFAULT_STORAGE};
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuSection};
use crate::widget::{CheckboxWidget, InputWidget};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, StatefulWidget, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Widget};
use ratatui::Frame;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Mutex;
use std::vec;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

trait Setting: Debug + Send {
    fn menu_setting(&self) -> MenuSettingType;

    /// Handle a received key event for the setting widget.
    fn on_key_event(&mut self, key: FXKeyEvent);

    /// Get the list item representation of the settings widget for the overview.
    fn item(&'_ self) -> ListItem<'_>;

    /// Render the individual separate widget.
    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Debug)]
pub struct MenuSettings {
    items: Vec<Box<dyn Setting>>,
    active_menu: MenuSettingType,
    state: Mutex<ListState>,
    menu_sender: UnboundedSender<MenuCommand>,
    setting_receiver: UnboundedReceiver<MenuSettingCommand>,
}

impl MenuSettings {
    pub fn new(app_sender: AppCommandSender, menu_sender: UnboundedSender<MenuCommand>) -> Self {
        let (setting_sender, setting_receiver) = unbounded_channel();

        Self {
            menu_sender,
            items: vec![
                Box::new(DhtSetting::new(app_sender.clone())),
                Box::new(TrackerSetting::new(app_sender.clone())),
                Box::new(StorageSetting::new(app_sender, setting_sender)),
                Box::new(FlagsSetting::new()),
            ],
            active_menu: MenuSettingType::Overview,
            setting_receiver,
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

    fn selected_mut(&mut self) -> &mut Box<dyn Setting> {
        let selected = self.selected_index();

        self.items
            .get_mut(selected)
            .expect("expected a valid item to have been selected")
    }

    async fn handle_command(&mut self, command: MenuSettingCommand) {
        match command {
            MenuSettingCommand::SwitchMenu(active_setting) => {
                self.active_menu = active_setting;
            }
        }
    }
}

#[async_trait]
impl MenuSectionWidget for MenuSettings {
    fn preferred_width(&self) -> u16 {
        128
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        if self.active_menu == MenuSettingType::Overview {
            match key.code() {
                KeyCode::Up => {
                    key.consume();
                    let selected = self.selected_index().saturating_sub(1);
                    if let Ok(mut state) = self.state.lock() {
                        key.consume();
                        state.select(Some(selected));
                    }
                }
                KeyCode::Down => {
                    key.consume();
                    let selected = self.selected_index().saturating_add(1);
                    if let Ok(mut state) = self.state.lock() {
                        if selected <= self.items.len() - 1 {
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
        if self.active_menu == MenuSettingType::Overview {
            let items = self.items.iter().map(|e| e.item()).collect::<Vec<_>>();
            let menu_list = List::new(items)
                .block(Block::new().title("Settings").borders(Borders::ALL))
                .highlight_style(Style::new().bg(Color::DarkGray));

            let mut state = self.state.lock().expect("Mutex poisoned");
            StatefulWidget::render(menu_list, area, frame.buffer_mut(), &mut state);
        } else {
            if let Some(item) = self
                .items
                .iter()
                .find(|e| e.menu_setting() == self.active_menu)
            {
                item.render(frame, area);
            }
        }
    }

    async fn tick(&mut self) {
        while let Ok(command) = self.setting_receiver.try_recv() {
            self.handle_command(command).await;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MenuSettingType {
    Overview,
    Dht,
    Tracker,
    Storage,
    Flags,
}

#[derive(Debug, PartialEq)]
enum MenuSettingCommand {
    SwitchMenu(MenuSettingType),
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
    fn menu_setting(&self) -> MenuSettingType {
        MenuSettingType::Dht
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        if key.code() == KeyCode::Enter {
            self.checkbox.toggle();
            let _ = self
                .app_sender
                .send(AppCommand::DhtEnabled(self.checkbox.is_checked()));
        }
    }

    fn item(&'_ self) -> ListItem<'_> {
        Text::from(&self.checkbox).into()
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
    fn menu_setting(&self) -> MenuSettingType {
        MenuSettingType::Tracker
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        if key.code() == KeyCode::Enter {
            self.checkbox.toggle();
            let _ = self
                .app_sender
                .send(AppCommand::TrackerEnabled(self.checkbox.is_checked()));
        }
    }

    fn item(&'_ self) -> ListItem<'_> {
        Text::from(&self.checkbox).into()
    }

    fn render(&self, _: &mut Frame, _: Rect) {
        // no-op
    }
}

#[derive(Debug)]
struct StorageSetting {
    input: InputWidget,
    is_active: bool,
    app_sender: AppCommandSender,
    setting_sender: UnboundedSender<MenuSettingCommand>,
}

impl StorageSetting {
    fn new(
        app_sender: AppCommandSender,
        setting_sender: UnboundedSender<MenuSettingCommand>,
    ) -> Self {
        Self {
            input: InputWidget::new_with_opts(APP_DEFAULT_STORAGE, true),
            is_active: false,
            app_sender,
            setting_sender,
        }
    }
}

impl Setting for StorageSetting {
    fn menu_setting(&self) -> MenuSettingType {
        MenuSettingType::Storage
    }

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
                if !self.is_active {
                    let _ = self
                        .setting_sender
                        .send(MenuSettingCommand::SwitchMenu(MenuSettingType::Storage));
                } else {
                    let new_location = PathBuf::from(self.input.as_str());
                    let _ = self.app_sender.send(AppCommand::Storage(new_location));
                    let _ = self
                        .setting_sender
                        .send(MenuSettingCommand::SwitchMenu(MenuSettingType::Overview));
                }

                self.is_active = !self.is_active;
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

    fn item(&'_ self) -> ListItem<'_> {
        ListItem::new(vec![
            Line::from(vec![Span::from("Storage").bold()]),
            Line::from(vec![self.input.as_str().into()]),
        ])
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let border = Block::new().title("Storage location").borders(Borders::ALL);

        self.input.render(frame, border.inner(area));
        border.render(area, frame.buffer_mut());
    }
}

#[derive(Debug)]
struct FlagsSetting {}

impl FlagsSetting {
    fn new() -> Self {
        Self {}
    }
}

impl Setting for FlagsSetting {
    fn menu_setting(&self) -> MenuSettingType {
        MenuSettingType::Flags
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {}

    fn item(&'_ self) -> ListItem<'_> {
        ListItem::new(vec![
            Line::from(vec![Span::from("Torrent options").bold()]),
            Line::from(vec![Span::from("TODO").bold()]),
        ])
    }

    fn render(&self, frame: &mut Frame, area: Rect) {}
}
