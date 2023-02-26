use derive_more::Display;
use serde::{Deserialize, Serialize};

const DEFAULT_QUALITY: fn() -> Option<Quality> = || None;
const DEFAULT_FULLSCREEN: fn() -> bool = || true;
const DEFAULT_AUTO_PLAY_NEXT_EPISODE: fn() -> bool = || true;

/// The preferences for the video playbacks
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "quality: {:?}", quality)]
pub struct PlaybackSettings {
    /// The default playback quality
    #[serde(default = "DEFAULT_QUALITY")]
    pub quality: Option<Quality>,
    /// Indicates if the playback should always start in fullscreen mode
    #[serde(default = "DEFAULT_FULLSCREEN")]
    pub fullscreen: bool,
    /// Indicates if the next episode should be started automatically
    #[serde(default = "DEFAULT_AUTO_PLAY_NEXT_EPISODE")]
    pub auto_play_next_episode_enabled: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            quality: DEFAULT_QUALITY(),
            fullscreen: DEFAULT_FULLSCREEN(),
            auto_play_next_episode_enabled: DEFAULT_AUTO_PLAY_NEXT_EPISODE(),
        }
    }
}

/// The playback quality defined in a resolution size
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Quality {
    P480,
    P720,
    P1080,
    P2160,
}

impl Quality {
    pub fn resolution(&self) -> u32 {
        match self {
            Quality::P480 => 480,
            Quality::P720 => 720,
            Quality::P1080 => 1080,
            Quality::P2160 => 2160,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_playback_settings_default() {
        let expected_result = PlaybackSettings {
            quality: DEFAULT_QUALITY(),
            fullscreen: DEFAULT_FULLSCREEN(),
            auto_play_next_episode_enabled: DEFAULT_AUTO_PLAY_NEXT_EPISODE(),
        };

        let result = PlaybackSettings::default();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_quality_resolution() {
        assert_eq!(480, Quality::P480.resolution());
        assert_eq!(720, Quality::P720.resolution());
        assert_eq!(1080, Quality::P1080.resolution());
        assert_eq!(2160, Quality::P2160.resolution());
    }
}