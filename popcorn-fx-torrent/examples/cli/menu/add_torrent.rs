use crate::app::FXKeyEvent;
use crate::menu::widget::MenuSectionWidget;
use crate::menu::{MenuCommand, MenuSection, MenuWidget};
use crate::widget::InputWidget;
use crossterm::event::KeyCode;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct MenuAddTorrent {
    input: InputWidget,
    error: Option<String>,
    menu_sender: UnboundedSender<MenuCommand>,
}

impl MenuAddTorrent {
    pub fn new(menu_sender: UnboundedSender<MenuCommand>) -> Self {
        Self {
            input: InputWidget::new_with_opts(true),
            error: None,
            menu_sender,
        }
    }

    fn add_torrent(&mut self) {
        if MenuWidget::validate_torrent_uri(self.input.as_str()) {
            let _ = self
                .menu_sender
                .send(MenuCommand::AddTorrentUri(self.input.drain(..).collect()));
            let _ = self
                .menu_sender
                .send(MenuCommand::SelectSection(MenuSection::Overview));
            self.reset();
        } else {
            self.error = Some("Torrent uri is invalid".to_string());
        }
    }

    fn reset(&mut self) {
        self.input.reset();
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
                    .send(MenuCommand::SelectSection(MenuSection::Overview));
            }
            KeyCode::Backspace => {
                key.consume();
                self.input.backspace();
            }
            KeyCode::Enter => {
                key.consume();
                self.add_torrent();
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

    fn on_paste_event(&mut self, text: String) {
        self.input.append(text.as_str());
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Fill(1), Length(1), Length(1)]);
        let [input_area, help_area, invalid_area] = layout.areas(area);

        // render the input area
        let block = Block::new().title("Torrent uri").borders(Borders::ALL);
        self.input.render(frame, block.inner(input_area));
        frame.render_widget(block, input_area);

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
