use serde::Deserialize;

use popcorn_fx_core::core::players::PlayerState;

/// Represents the state of a VLC player.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VlcState {
    /// Represents the paused state of a VLC player.
    Paused,
    /// Represents the playing state of a VLC player.
    Playing,
    /// Represents the stopped state of a VLC player.
    Stopped,
}

impl From<VlcState> for PlayerState {
    fn from(value: VlcState) -> Self {
        match value {
            VlcState::Paused => PlayerState::Paused,
            VlcState::Playing => PlayerState::Playing,
            VlcState::Stopped => PlayerState::Stopped,
        }
    }
}

/// Represents the status of a VLC player.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename = "root")]
pub struct VlcStatus {
    /// The current time of the media being played.
    pub time: u64,
    /// The total length of the media being played.
    pub length: u64,
    /// The volume level indication of the VLC player between 0-256 (muted-max).
    pub volume: u32,
    /// The state of the VLC player.
    pub state: VlcState,
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::from_str;

    use super::*;

    #[test]
    fn test_from_vlc_state() {
        assert_eq!(PlayerState::Paused, PlayerState::from(VlcState::Paused));
        assert_eq!(PlayerState::Playing, PlayerState::from(VlcState::Playing));
        assert_eq!(PlayerState::Stopped, PlayerState::from(VlcState::Stopped));
    }

    #[test]
    fn test_deserialize() {
        let response = r#"<?xml version="1.0" encoding="utf-8" standalone="yes" ?>
<root>
    <time>200</time>
    <length>56000</length>
    <state>paused</state>
    <volume>256</volume>
</root>
"#;
        let expected_result = VlcStatus {
            time: 200,
            length: 56000,
            volume: 256,
            state: VlcState::Paused,
        };

        let result: VlcStatus = from_str(response).expect("expected the vlc response to have been parsed");

        assert_eq!(expected_result, result)
    }
}