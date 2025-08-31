use crate::app::{AppCommand, FXKeyEvent, FXWidget};
use crate::app_logger::LogEntry;
use async_trait::async_trait;
use crossterm::event::KeyCode;
use derive_more::Display;
use fx_handle::Handle;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph, StatefulWidget, Widget, Wrap};
use ratatui::Frame;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const LOG_LIMIT: usize = 100;

trait MenuSectionWidget: Debug + Send {
    /// Get the preferred width of the menu section.
    fn preferred_width(&self) -> u16;

    /// Handle the specified key event within the menu section.
    fn on_key_event(&mut self, key: FXKeyEvent);

    /// Handle a paste event within this widget.
    fn on_paste_event(&mut self, text: String);

    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Debug)]
pub struct MenuWidget {
    handle: Handle,
    sections: HashMap<MenuSection, Box<dyn MenuSectionWidget>>,
    active_section: MenuSection,
    logs: Vec<String>,
    app_sender: UnboundedSender<AppCommand>,
    menu_receiver: UnboundedReceiver<MenuCommand>,
    log_receiver: UnboundedReceiver<LogEntry>,
}

impl MenuWidget {
    pub fn new(
        app_sender: UnboundedSender<AppCommand>,
        log_receiver: UnboundedReceiver<LogEntry>,
    ) -> Self {
        let (menu_sender, menu_receiver) = unbounded_channel();

        Self {
            handle: Default::default(),
            sections: vec![
                (
                    MenuSection::Main,
                    Box::new(MainMenuSection::new(
                        app_sender.clone(),
                        menu_sender.clone(),
                    )) as Box<dyn MenuSectionWidget>,
                ),
                (
                    MenuSection::AddTorrent,
                    Box::new(MenuAddTorrent::new(menu_sender)),
                ),
            ]
            .into_iter()
            .collect(),
            active_section: MenuSection::Main,
            logs: vec![],
            app_sender,
            menu_receiver,
            log_receiver,
        }
    }

    fn log(&mut self, log: String) {
        self.logs.push(log);
        if self.logs.len() > LOG_LIMIT {
            self.logs.remove(0);
        }
    }

    fn handle_command(&mut self, command: MenuCommand) {
        match command {
            MenuCommand::SelectSection(section) => {
                self.active_section = section;
            }
            MenuCommand::AddTorrentUri(uri) => self.add_torrent_uri(uri),
        }
    }

    fn add_torrent_uri(&self, uri: String) {
        let _ = self.app_sender.send(AppCommand::AddTorrentUri(uri));
    }

    fn active_section(&self) -> &Box<dyn MenuSectionWidget> {
        self.sections
            .get(&self.active_section)
            .expect("active section not found")
    }

    fn active_section_mut(&mut self) -> &mut Box<dyn MenuSectionWidget> {
        self.sections
            .get_mut(&self.active_section)
            .expect("active section not found")
    }

    fn validate_torrent_uri(uri: &str) -> bool {
        uri.starts_with("magnet:?")
            || PathBuf::from_str(uri)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
                .and_then(|e| {
                    if e.exists() {
                        Ok(())
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "torrent file not found",
                        ))
                    }
                })
                .is_ok()
    }

    fn render_logs(&self, frame: &mut Frame, log_area: Rect) {
        let block = Block::bordered().title(" Logs ");
        let inner_height = block.inner(log_area).height as usize;
        let log_len = self.logs.len().min(inner_height);
        let start_index = if self.logs.len() < inner_height {
            0
        } else {
            self.logs.len() - log_len
        };

        Paragraph::new(
            self.logs[start_index..]
                .iter()
                .map(|l| Line::from(l.clone()))
                .collect::<Vec<_>>(),
        )
        .block(block)
        .render(log_area, frame.buffer_mut());
    }
}

#[async_trait]
impl FXWidget for MenuWidget {
    fn handle(&self) -> Handle {
        self.handle
    }

    fn name(&self) -> &str {
        "Menu"
    }

    async fn tick(&mut self) {
        while let Ok(command) = self.menu_receiver.try_recv() {
            self.handle_command(command);
        }

        while let Ok(entry) = self.log_receiver.try_recv() {
            self.log(entry.text);
        }
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        self.active_section_mut().on_key_event(key);
    }

    fn on_paste_event(&mut self, text: String) {
        self.active_section_mut().on_paste_event(text);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let section = self.active_section();
        let layout = Layout::horizontal([Length(section.preferred_width()), Fill(1)]);
        let [section_area, log_area] = layout.areas(area);

        // render the active section
        section.render(frame, section_area);

        // render logs
        self.render_logs(frame, log_area);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MenuCommand {
    SelectSection(MenuSection),
    AddTorrentUri(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MenuSection {
    Main,
    AddTorrent,
}

#[derive(Debug, Display, Clone, PartialEq)]
enum MenuItem {
    #[display(fmt = "Add torrent")]
    AddTorrent,
    #[display(fmt = "Settings")]
    Settings,
    #[display(fmt = "Quit")]
    Quit,
}

impl MenuItem {
    pub fn all() -> Vec<MenuItem> {
        vec![Self::AddTorrent, Self::Settings, Self::Quit]
    }
}

#[derive(Debug)]
struct MainMenuSection {
    items: Vec<MenuItem>,
    state: Mutex<ListState>,
    menu_sender: UnboundedSender<MenuCommand>,
    app_sender: UnboundedSender<AppCommand>,
}

impl MainMenuSection {
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
                // TODO
            }
            MenuItem::Quit => {
                let _ = self.app_sender.send(AppCommand::Quit);
            }
        }
    }
}

impl MenuSectionWidget for MainMenuSection {
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

    fn on_paste_event(&mut self, text: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        Widget::render(self, area, frame.buffer_mut());
    }
}

impl Widget for &MainMenuSection {
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

#[derive(Debug)]
struct MenuAddTorrent {
    text: String,
    cursor: usize,
    error: Option<String>,
    menu_sender: UnboundedSender<MenuCommand>,
}

impl MenuAddTorrent {
    pub fn new(menu_sender: UnboundedSender<MenuCommand>) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            error: None,
            menu_sender,
        }
    }

    fn add_torrent(&mut self) {
        if MenuWidget::validate_torrent_uri(self.text.as_str()) {
            self.error = None;
            let _ = self
                .menu_sender
                .send(MenuCommand::AddTorrentUri(self.text.drain(..).collect()));
            let _ = self
                .menu_sender
                .send(MenuCommand::SelectSection(MenuSection::Main));
        } else {
            self.error = Some("Torrent uri is invalid".to_string());
        }
    }

    fn reset(&mut self) {
        self.text = String::new();
        self.cursor = 0;
        self.error = None;
    }
}

impl MenuSectionWidget for MenuAddTorrent {
    fn preferred_width(&self) -> u16 {
        128
    }

    fn on_key_event(&mut self, mut key: FXKeyEvent) {
        match key.code() {
            KeyCode::Esc => {
                key.consume();
                self.reset();
                let _ = self
                    .menu_sender
                    .send(MenuCommand::SelectSection(MenuSection::Main));
            }
            KeyCode::Backspace => {
                key.consume();
                if self.cursor > 0 {
                    let before = self.text.chars().take(self.cursor - 1);
                    let after = self.text.chars().skip(self.cursor);
                    self.text = before.chain(after).collect();
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                key.consume();
                self.add_torrent();
            }
            KeyCode::Char(char) => {
                key.consume();
                let position = self.text.len().min(self.cursor);
                self.text.insert(position, char);
                self.cursor += 1;
            }
            KeyCode::Left => {
                key.consume();
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                key.consume();
                self.cursor = self.cursor.saturating_add(1);
            }
            _ => {}
        }
    }

    fn on_paste_event(&mut self, text: String) {
        self.text += &text;
        self.cursor = text.len();
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Fill(1), Length(1), Length(1)]);
        let [input_area, help_area, invalid_area] = layout.areas(area);

        // render the input area
        let block = Block::new().title("Torrent uri").borders(Borders::ALL);
        let input = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .block(block.clone());
        frame.render_widget(input, input_area);

        // render the cursor
        let inner = block.inner(input_area);
        let input_width = inner.width;
        let cursor = self.cursor as u16;
        frame.set_cursor_position(Position::new(
            inner.x + (cursor % input_width),
            inner.y + (cursor / input_width),
        ));

        // render the help info
        Text::from("Press Esc to return to the menu, Enter to add the new torrent")
            .style(Style::new().italic())
            .render(help_area, frame.buffer_mut());

        // render error
        if let Some(err) = self.error.as_ref() {
            Text::from(err.as_str())
                .style(Style::new().bold().fg(Color::Red))
                .render(invalid_area, frame.buffer_mut());
        }
    }
}
