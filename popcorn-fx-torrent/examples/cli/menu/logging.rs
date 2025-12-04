use crate::app::FXKeyEvent;
use crate::app_logger::AppLogger;
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuSection};
use crate::widget::ComboboxWidget;
use async_trait::async_trait;
use crossterm::event::KeyCode;
use log::Level;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, StatefulWidget, Style};
use ratatui::widgets::{Block, Cell, HighlightSpacing, Row, Table, TableState, Widget};
use ratatui::Frame;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct MenuLogging {
    logger: AppLogger,
    loggers: Vec<ComboboxWidget<Level>>,
    section: MenuLoggingSection,
    state: Mutex<TableState>,
    menu_sender: UnboundedSender<MenuCommand>,
}

impl MenuLogging {
    pub fn new(logger: AppLogger, menu_sender: UnboundedSender<MenuCommand>) -> Self {
        let loggers = logger
            .loggers()
            .into_iter()
            .map(|logger| ComboboxWidget::new(logger.name, Level::iter().collect(), logger.level))
            .collect();

        Self {
            logger,
            loggers,
            section: MenuLoggingSection::Overview,
            state: Mutex::new(TableState::default().with_selected(0)),
            menu_sender,
        }
    }

    fn selected(&self) -> usize {
        self.state
            .lock()
            .ok()
            .and_then(|e| e.selected())
            .unwrap_or_default()
    }

    fn on_key_event_overview(&mut self, mut key: FXKeyEvent) {
        match key.key_code() {
            KeyCode::Up => {
                key.consume();
                let selected = self.selected();
                if let Ok(mut state) = self.state.lock() {
                    state.select(Some(selected.saturating_sub(1)));
                }
            }
            KeyCode::Down => {
                key.consume();
                let selected = self.selected();
                if let Ok(mut state) = self.state.lock() {
                    let mut offset = selected.saturating_add(1);
                    if offset > self.loggers.len() {
                        offset = self.loggers.len() - 1;
                    }

                    state.select(Some(offset));
                }
            }
            KeyCode::Backspace | KeyCode::Esc => {
                key.consume();
                let _ = self
                    .menu_sender
                    .send(MenuCommand::SelectSection(MenuSection::Overview));
            }
            KeyCode::Enter => {
                key.consume();
                self.section = MenuLoggingSection::Logger;
            }
            _ => {}
        }
    }

    fn on_key_event_logger(&mut self, mut key: FXKeyEvent) {
        let selected = self.selected();
        let combo = if let Some(e) = self.loggers.get_mut(selected) {
            e
        } else {
            return;
        };

        match key.key_code() {
            KeyCode::Up => {
                key.consume();
                combo.previous();
            }
            KeyCode::Down => {
                key.consume();
                combo.next();
            }
            KeyCode::Backspace | KeyCode::Esc => {
                key.consume();
                self.section = MenuLoggingSection::Overview;
            }
            KeyCode::Enter => {
                key.consume();
                if let Some(logger) = self.logger.loggers().get(selected) {
                    let target = &logger.target;
                    self.logger
                        .update(target, combo.selected().unwrap_or(&Level::Info));
                }
                self.section = MenuLoggingSection::Overview;
            }
            _ => {}
        }
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let header = vec!["Logger", "Level"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().bg(Color::DarkGray).fg(Color::White));
        let rows = self
            .logger
            .loggers()
            .into_iter()
            .enumerate()
            .map(|(index, logger)| {
                let color = if index % 2 == 0 {
                    Color::Rgb(80, 80, 50)
                } else {
                    Color::Rgb(80, 80, 80)
                };

                Row::new(vec![logger.name, logger.level.to_string()]).style(Style::new().bg(color))
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows, [Fill(1), Length(6)])
            .header(header)
            .block(Block::bordered().title("Loggers"))
            .row_highlight_style(Style::new().bg(Color::LightYellow).fg(Color::DarkGray))
            .highlight_spacing(HighlightSpacing::Always);

        if let Ok(mut state) = self.state.lock() {
            StatefulWidget::render(table, area, frame.buffer_mut(), &mut state);
        }
    }
}

#[async_trait]
impl MenuSectionWidget for MenuLogging {
    fn preferred_width(&self) -> u16 {
        32
    }

    fn on_key_event(&mut self, key: FXKeyEvent) {
        if self.section == MenuLoggingSection::Overview {
            self.on_key_event_overview(key);
        } else {
            self.on_key_event_logger(key);
        }
    }

    fn on_paste_event(&mut self, _: String) {
        // no-op
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.section == MenuLoggingSection::Overview {
            self.render_overview(frame, area);
        } else {
            let selected = self
                .state
                .lock()
                .ok()
                .and_then(|e| e.selected())
                .unwrap_or_default();

            if let Some(combo) = self.loggers.get(selected) {
                combo.render(area, frame.buffer_mut());
            }
        }
    }

    async fn tick(&mut self) {
        // no-op
    }
}

#[derive(Debug, PartialEq)]
enum MenuLoggingSection {
    Overview,
    Logger,
}
