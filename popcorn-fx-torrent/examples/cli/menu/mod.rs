use crate::app::{AppCommand, AppCommandSender, FXKeyEvent, FXWidget};
use crate::app_logger::AppLogger;
use crate::menu::add_torrent::MenuAddTorrent;
use crate::menu::logging::MenuLogging;
use crate::menu::overview::MenuOverview;
use crate::menu::settings::MenuSettings;
use crate::menu::widget::MenuSectionWidget;
use async_trait::async_trait;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

mod add_torrent;
mod logging;
mod overview;
mod settings;
mod widget;

const LOG_LIMIT: usize = 100;

#[derive(Debug)]
pub struct MenuWidget {
    sections: HashMap<MenuSection, Box<dyn MenuSectionWidget>>,
    active_section: MenuSection,
    logs: Vec<String>,
    app_sender: AppCommandSender,
    menu_receiver: UnboundedReceiver<MenuCommand>,
    logger: AppLogger,
}

impl MenuWidget {
    pub fn new(app_sender: AppCommandSender, logger: AppLogger) -> Self {
        let (menu_sender, menu_receiver) = unbounded_channel();

        Self {
            sections: vec![
                (
                    MenuSection::Overview,
                    Box::new(MenuOverview::new(app_sender.clone(), menu_sender.clone()))
                        as Box<dyn MenuSectionWidget>,
                ),
                (
                    MenuSection::AddTorrent,
                    Box::new(MenuAddTorrent::new(menu_sender.clone())),
                ),
                (
                    MenuSection::Settings,
                    Box::new(MenuSettings::new(app_sender.clone(), menu_sender.clone())),
                ),
                (
                    MenuSection::Logging,
                    Box::new(MenuLogging::new(logger.clone(), menu_sender)),
                ),
            ]
            .into_iter()
            .collect(),
            active_section: MenuSection::Overview,
            logs: vec![],
            app_sender,
            menu_receiver,
            logger,
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
    fn name(&self) -> &str {
        "Menu"
    }

    async fn tick(&mut self) {
        while let Ok(command) = self.menu_receiver.try_recv() {
            self.handle_command(command);
        }

        while let Some(entry) = self.logger.next() {
            self.log(entry.text);
        }

        for (_, section) in &mut self.sections {
            section.tick().await;
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
pub enum MenuCommand {
    SelectSection(MenuSection),
    AddTorrentUri(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MenuSection {
    Overview,
    AddTorrent,
    Settings,
    Logging,
}
