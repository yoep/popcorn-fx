use std::fmt::{Display, Formatter};
use std::string::ToString;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::core::config::ConfigError;

const UI_SCALE_SUFFIX: &str = "%";
const DEFAULT_LANGUAGE: fn() -> String = || "en".to_string();
const DEFAULT_UI_SCALE: fn() -> UiScale = || UiScale::new(1f32).expect("Expected the ui scale to be valid");
const DEFAULT_START_SCREEN: fn() -> StartScreen = || StartScreen::Movies;
const DEFAULT_MAXIMIZED: fn() -> bool = || false;
const DEFAULT_NATIVE_WINDOW: fn() -> bool = || false;

#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "default_language: {}, ui_scale: {}", default_language, ui_scale)]
pub struct UiSettings {
    /// The default language of the application
    #[serde(default = "DEFAULT_LANGUAGE")]
    default_language: String,
    /// The ui scale of the application
    #[serde(default = "DEFAULT_UI_SCALE")]
    ui_scale: UiScale,
    /// The default start screen of the application
    #[serde(default = "DEFAULT_START_SCREEN")]
    start_screen: StartScreen,
    /// The indication if the UI was maximized the last time the application was closed
    #[serde(default = "DEFAULT_MAXIMIZED")]
    maximized: bool,
    /// The indication if the UI should use a native window rather than the borderless stage
    #[serde(default = "DEFAULT_NATIVE_WINDOW")]
    native_window_enabled: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            default_language: DEFAULT_LANGUAGE(),
            ui_scale: DEFAULT_UI_SCALE(),
            start_screen: DEFAULT_START_SCREEN(),
            maximized: DEFAULT_MAXIMIZED(),
            native_window_enabled: DEFAULT_NATIVE_WINDOW(),
        }
    }
}

impl UiSettings {
    pub fn new(default_language: String, ui_scale: UiScale, start_screen: StartScreen, maximized: bool, native_window_enabled: bool) -> Self {
        Self {
            default_language,
            ui_scale,
            start_screen,
            maximized,
            native_window_enabled,
        }
    }

    pub fn default_language(&self) -> &String {
        &self.default_language
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiScale {
    value: f32,
}

impl UiScale {
    pub fn new(value: f32) -> crate::core::config::Result<Self> {
        if value < 0f32 {
            return Err(ConfigError::InvalidValue(value.to_string(), "UiScale.value".to_string()));
        }

        Ok(Self {
            value
        })
    }
}

impl Display for UiScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_value: i32 = (self.value * 100f32) as i32;

        write!(f, "{}{}", display_value, UI_SCALE_SUFFIX)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StartScreen {
    Movies,
    Shows,
    Favorites,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ui_scale_display_text() {
        let scale = UiScale {
            value: 1.25f32
        };
        let expected_result = "125%".to_string();

        let result = scale.to_string();

        assert_eq!(expected_result, result)
    }
}