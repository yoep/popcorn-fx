use crate::app::{
    AppCommand, AppCommandSender, FXKeyEvent, APP_DEFAULT_STORAGE, DEFAULT_TORRENT_FLAGS,
};
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuSection};
use crate::widget::{CheckboxWidget, InputWidget};
use async_trait::async_trait;
use crossterm::event::KeyCode;
use popcorn_fx_torrent::torrent::TorrentFlags;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, StatefulWidget, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Widget};
use ratatui::Frame;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::vec;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// A trait that implements a simple setting option.
/// This option has no additional rendering for modifying the settings.
trait Setting: Debug + Send {
    /// Activate/trigger the setting option.
    /// This either updates the state of the option or opens the settings widget.
    fn activate(&mut self);

    /// Get the list item representation of the settings widget for the overview.
    fn item(&'_ self) -> ListItem<'_>;
}

/// A trait that implements a setting widget that renders additional content.
trait SettingWidget: Setting {
    /// Handle a received key event for the setting widget.
    fn on_key_event(&mut self, key: FXKeyEvent);

    /// Render the settings widget details.
    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Debug)]
enum SettingsMenuItem {
    Title(String),
    Option(Box<dyn Setting>),
    Widget(Box<dyn SettingWidget>),
}

impl SettingsMenuItem {
    fn activate(&mut self) {
        match self {
            SettingsMenuItem::Option(option) => option.activate(),
            SettingsMenuItem::Widget(widget) => widget.activate(),
            _ => {}
        }
    }

    fn item(&self) -> ListItem<'_> {
        match self {
            SettingsMenuItem::Title(title) => {
                ListItem::new(vec![Line::from(vec![Span::from(title).bold()])])
            }
            SettingsMenuItem::Option(option) => option.item(),
            SettingsMenuItem::Widget(widget) => widget.item(),
        }
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        match self {
            SettingsMenuItem::Widget(widget) => widget.on_key_event(key),
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        match self {
            SettingsMenuItem::Widget(widget) => widget.render(frame, area),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct MenuSettings {
    items: Vec<SettingsMenuItem>,
    state: Mutex<ListState>,
    is_subitem_active: AtomicBool,
    close_receiver: UnboundedReceiver<()>,
    menu_sender: UnboundedSender<MenuCommand>,
}

impl MenuSettings {
    pub fn new(app_sender: AppCommandSender, menu_sender: UnboundedSender<MenuCommand>) -> Self {
        let (close_sender, close_receiver) = unbounded_channel();
        let torrent_flags = DEFAULT_TORRENT_FLAGS();

        Self {
            items: vec![
                SettingsMenuItem::Title("Peer discovery".to_string()),
                SettingsMenuItem::Option(Box::new(ToggleSetting::new(
                    "DHT",
                    true,
                    |enabled, sender| {
                        let _ = sender.send(AppCommand::DhtEnabled(enabled));
                    },
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleSetting::new(
                    "Tracker",
                    true,
                    |enabled, sender| {
                        let _ = sender.send(AppCommand::TrackerEnabled(enabled));
                    },
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Title("Peer connections".to_string()),
                SettingsMenuItem::Option(Box::new(ToggleSetting::new(
                    "TCP",
                    true,
                    |enabled, sender| {
                        let _ = sender.send(AppCommand::TcpPeerEnabled(enabled));
                    },
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleSetting::new(
                    "uTP",
                    true,
                    |enabled, sender| {
                        let _ = sender.send(AppCommand::UtpPeerEnabled(enabled));
                    },
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Title("Storage location".to_string()),
                SettingsMenuItem::Widget(Box::new(StorageSetting::new(
                    app_sender.clone(),
                    close_sender,
                ))),
                SettingsMenuItem::Title("Torrent options".to_string()),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Seed mode",
                    TorrentFlags::SeedMode,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Upload mode",
                    TorrentFlags::UploadMode,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Download mode",
                    TorrentFlags::DownloadMode,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Share mode",
                    TorrentFlags::ShareMode,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Apply IP filter",
                    TorrentFlags::ApplyIpFilter,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Paused",
                    TorrentFlags::Paused,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Metadata",
                    TorrentFlags::Metadata,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Sequential download",
                    TorrentFlags::SequentialDownload,
                    torrent_flags,
                    app_sender.clone(),
                ))),
                SettingsMenuItem::Option(Box::new(ToggleFlagSetting::new(
                    "Stop when ready",
                    TorrentFlags::StopWhenReady,
                    torrent_flags,
                    app_sender.clone(),
                ))),
            ],
            is_subitem_active: AtomicBool::new(false),
            close_receiver,
            state: Mutex::new(ListState::default().with_selected(Some(1))),
            menu_sender,
        }
    }

    fn select_index(&self, index: usize) {
        if let Ok(mut state) = self.state.lock() {
            state.select(Some(index));
        }
    }

    fn selected_index(&self) -> usize {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or(0)
    }

    fn selected_mut(&mut self) -> &mut SettingsMenuItem {
        let selected = self.selected_index();

        self.items
            .get_mut(selected)
            .expect("expected a valid item to have been selected")
    }

    /// Select the next widget item within the menu list.
    fn next_item(&self) {
        let current_index = self.selected_index().saturating_add(1);
        for index in current_index..self.items.len() {
            let item = &self.items[index];
            if let SettingsMenuItem::Title(_) = item {
                continue;
            }

            self.select_index(index);
            return;
        }
    }

    /// Select the previous widget item within the menu list.
    fn previous_item(&self) {
        let current_index = self.selected_index();
        for index in (0..current_index).into_iter().rev() {
            let item = &self.items[index];
            if let SettingsMenuItem::Title(_) = item {
                continue;
            }

            self.select_index(index);
            return;
        }
    }

    fn handle_menu_key_event(&mut self, mut key: FXKeyEvent) {
        match key.key_code() {
            KeyCode::Up => {
                key.consume();
                self.previous_item();
            }
            KeyCode::Down => {
                key.consume();
                self.next_item();
            }
            KeyCode::Esc | KeyCode::Backspace => {
                key.consume();
                let _ = self
                    .menu_sender
                    .send(MenuCommand::SelectSection(MenuSection::Overview));
            }
            KeyCode::Enter => {
                key.consume();
                let item = self.selected_mut();
                item.activate();
                if let SettingsMenuItem::Widget(_) = item {
                    self.is_subitem_active.store(true, Ordering::Relaxed);
                }
            }
            _ => {}
        }
    }

    fn render_menu_overview(&self, frame: &mut Frame, area: Rect) {
        let items = self.items.iter().map(|e| e.item()).collect::<Vec<_>>();
        let menu_list = List::new(items)
            .block(Block::new().title("Settings").borders(Borders::ALL))
            .highlight_style(Style::new().bg(Color::DarkGray));

        let mut state = self.state.lock().expect("Mutex poisoned");
        StatefulWidget::render(menu_list, area, frame.buffer_mut(), &mut state);
    }
}

#[async_trait]
impl MenuSectionWidget for MenuSettings {
    fn preferred_width(&self) -> u16 {
        128
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        if self.is_subitem_active.load(Ordering::Relaxed) {
            self.selected_mut().on_key_event(key);
        } else {
            self.handle_menu_key_event(key);
        }
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.is_subitem_active.load(Ordering::Relaxed) {
            let index = self.selected_index();
            self.items.get(index).map(|e| e.render(frame, area));
        } else {
            self.render_menu_overview(frame, area);
        }
    }

    async fn tick(&mut self) {
        if let Ok(_) = self.close_receiver.try_recv() {
            self.is_subitem_active.store(false, Ordering::Relaxed);
        }
    }
}

type CheckboxAction = dyn Fn(bool, &AppCommandSender) + Send + Sync + 'static;

struct ToggleSetting {
    checkbox: CheckboxWidget,
    app_sender: AppCommandSender,
    on_trigger: Box<CheckboxAction>,
}

impl ToggleSetting {
    fn new<F>(name: &str, default: bool, action: F, app_sender: AppCommandSender) -> Self
    where
        F: Fn(bool, &AppCommandSender) + Send + Sync + 'static,
    {
        Self {
            checkbox: CheckboxWidget::new(name, default),
            app_sender,
            on_trigger: Box::new(action),
        }
    }
}

impl Setting for ToggleSetting {
    fn activate(&mut self) {
        self.checkbox.toggle();
        (self.on_trigger)(self.checkbox.is_checked(), &self.app_sender);
    }

    fn item(&'_ self) -> ListItem<'_> {
        Text::from(&self.checkbox).into()
    }
}

impl Debug for ToggleSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleSetting")
            .field("checkbox", &self.checkbox)
            .field("app_sender", &self.app_sender)
            .finish()
    }
}

#[derive(Debug)]
struct ToggleFlagSetting {
    base: ToggleSetting,
}

impl ToggleFlagSetting {
    fn new(
        name: &str,
        flag: TorrentFlags,
        active_flags: TorrentFlags,
        app_sender: AppCommandSender,
    ) -> Self {
        Self {
            base: ToggleSetting::new(
                name,
                active_flags.contains(flag),
                move |enabled, app_sender| {
                    if enabled {
                        let _ = app_sender.send(AppCommand::AddTorrentFlags(flag));
                    } else {
                        let _ = app_sender.send(AppCommand::RemoveTorrentFlags(flag));
                    }
                },
                app_sender,
            ),
        }
    }
}

impl Setting for ToggleFlagSetting {
    fn activate(&mut self) {
        self.base.activate();
    }

    fn item(&'_ self) -> ListItem<'_> {
        self.base.item()
    }
}

#[derive(Debug)]
struct StorageSetting {
    input: InputWidget,
    app_sender: AppCommandSender,
    close_sender: UnboundedSender<()>,
}

impl StorageSetting {
    fn new(app_sender: AppCommandSender, close_sender: UnboundedSender<()>) -> Self {
        Self {
            input: InputWidget::new_with_opts(APP_DEFAULT_STORAGE, true),
            app_sender,
            close_sender,
        }
    }
}

impl Setting for StorageSetting {
    fn activate(&mut self) {
        // no-op
    }

    fn item(&'_ self) -> ListItem<'_> {
        Line::from(vec![self.input.as_str().into()]).into()
    }
}

impl SettingWidget for StorageSetting {
    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        match key.key_code() {
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
                let new_location = PathBuf::from(self.input.as_str());
                let _ = self.app_sender.send(AppCommand::Storage(new_location));
                let _ = self.close_sender.send(());
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

    fn render(&self, frame: &mut Frame, area: Rect) {
        let border = Block::new().title("Storage location").borders(Borders::ALL);

        self.input.render(frame, border.inner(area));
        border.render(area, frame.buffer_mut());
    }
}
