use log::debug;

use crate::popcorn::fx::subtitle::model::Subtitle;

/// Listener of the subtitle service which get notified when the context changes.
pub trait SubtitleServiceListener {
    /// Invoked when the subtitle is changed.
    fn on_subtitle_changed(&self, subtitle: Option<Subtitle>);
}

pub struct SubtitleService {
    active_subtitle: Option<Subtitle>,
    listeners: Vec<Box<dyn SubtitleServiceListener>>,
}

impl SubtitleService {
    pub fn new() -> SubtitleService {
        return SubtitleService {
            active_subtitle: None,
            listeners: Vec::new(),
        };
    }

    /// Retrieve the active subtitle of the application.
    /// It returns the subtitle if one is present, else None.
    pub fn active_subtitle(&self) -> Option<&Subtitle> {
        return match &self.active_subtitle {
            Some(subtitle) => Some(&subtitle),
            None => None,
        };
    }

    /// Update the current active subtitle.
    /// The service will manage the ownership of the `subtitle`.
    pub fn update_subtitle(&mut self, subtitle: Subtitle) {
        debug!("Updating active subtitle to {}", subtitle);
        self.active_subtitle = Option::Some(subtitle);
    }

    /// Add a listener to the service which is notified when the subtitle changes.
    pub fn add_listener(&mut self, listener: Box<dyn SubtitleServiceListener>) {
        self.listeners.push(listener);
    }
}