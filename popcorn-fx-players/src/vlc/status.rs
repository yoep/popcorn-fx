use crate::vlc::{Result, VlcError};
use popcorn_fx_core::core::players::PlayerState;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::str::FromStr;

const VLC_STATE_VARIANTS: [&str; 3] = ["paused", "playing", "stopped"];

/// Represents the state of a VLC player.
#[derive(Debug, Clone, PartialEq)]
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

impl FromStr for VlcState {
    type Err = VlcError;

    fn from_str(value: &str) -> Result<Self> {
        let normalized_value = value.trim().to_lowercase();

        match normalized_value.as_str() {
            "paused" => Ok(VlcState::Paused),
            "playing" => Ok(VlcState::Playing),
            "stopped" => Ok(VlcState::Stopped),
            _ => Err(VlcError::Parsing(format!(
                "invalid vlc state value {}",
                value
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for VlcState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VlcStateVisitor)
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

struct VlcStateVisitor;

impl<'de> Visitor<'de> for VlcStateVisitor {
    type Value = VlcState;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected text representing a VLC state")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        VlcState::from_str(value).map_err(|e| Error::unknown_variant(value, &VLC_STATE_VARIANTS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_vlc_state() {
        assert_eq!(PlayerState::Paused, PlayerState::from(VlcState::Paused));
        assert_eq!(PlayerState::Playing, PlayerState::from(VlcState::Playing));
        assert_eq!(PlayerState::Stopped, PlayerState::from(VlcState::Stopped));
    }

    #[test]
    fn test_vlc_state_from_str() {
        assert_eq!(Ok(VlcState::Paused), VlcState::from_str("Paused"));
        assert_eq!(Ok(VlcState::Playing), VlcState::from_str("Playing"));
        assert_eq!(Ok(VlcState::Stopped), VlcState::from_str("Stopped"));
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

        let result: VlcStatus = serde_xml_rs::from_str(response)
            .expect("expected the vlc response to have been parsed");

        assert_eq!(expected_result, result)
    }
}
