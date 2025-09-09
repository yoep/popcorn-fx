use crate::app::FXKeyEvent;
use async_trait::async_trait;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::Debug;

/// A widget which renders a topic/section of the menu.
#[async_trait]
pub trait MenuSectionWidget: Debug + Send {
    /// Get the preferred width of the menu section.
    fn preferred_width(&self) -> u16;

    /// Handle the specified key event within the menu section.
    fn on_key_event(&mut self, key: FXKeyEvent);

    /// Handle a paste event within this widget.
    fn on_paste_event(&mut self, text: String);

    fn render(&self, frame: &mut Frame, area: Rect);

    /// Execute a widget tick that allows to process events.
    async fn tick(&mut self);
}
