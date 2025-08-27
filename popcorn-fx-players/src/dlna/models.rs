use std::collections::HashMap;
use std::str::FromStr;

use popcorn_fx_core::core::players::PlayerState;

use crate::dlna::{DlnaError, Result};

const DLNA_FIELD_SPEED: &str = "CurrentSpeed";
const DLNA_FIELD_STATE: &str = "CurrentTransportState";
const DLNA_FIELD_STATUS: &str = "CurrentTransportStatus";

/// Represents an event received from UPnP.
#[derive(Debug, PartialEq)]
pub enum UpnpEvent {
    /// An event containing time information.
    Time(PositionInfo),
    /// An event containing transport information.
    State(TransportInfo),
}

/// Represents position information received from UPnP.
#[derive(Debug, PartialEq)]
pub struct PositionInfo {
    /// The URI of the track.
    pub track_uri: String,
    /// The absolute count.
    pub abs_count: i32,
    /// The relative count.
    pub rel_count: i32,
    /// The relative time.
    pub rel_time: String,
    /// The track number.
    pub track: u32,
    /// The metadata of the track.
    pub track_metadata: String,
    /// The duration of the track.
    pub track_duration: String,
}

impl From<HashMap<String, String>> for PositionInfo {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            track_uri: map.get("TrackURI").cloned().unwrap_or_default(),
            abs_count: map
                .get("AbsCount")
                .map(|e| e.parse().unwrap_or(-1))
                .unwrap_or(-1),
            rel_count: map
                .get("RelCount")
                .map(|e| e.parse().unwrap_or(-1))
                .unwrap_or(-1),
            rel_time: map.get("RelTime").cloned().unwrap_or_default(),
            track: map.get("Track").unwrap().parse().unwrap(),
            track_metadata: map.get("TrackMetaData").cloned().unwrap_or_default(),
            track_duration: map.get("TrackDuration").cloned().unwrap_or_default(),
        }
    }
}

/// Represents transport information received from UPnP.
#[derive(Debug, PartialEq)]
pub struct TransportInfo {
    /// The current speed.
    pub current_speed: i32,
    /// The current transport state.
    pub current_transport_state: UpnpState,
    /// The current transport status.
    pub current_transport_status: String,
}

impl TryFrom<HashMap<String, String>> for TransportInfo {
    type Error = DlnaError;

    fn try_from(map: HashMap<String, String>) -> Result<Self> {
        let current_speed = map
            .get(DLNA_FIELD_SPEED)
            .ok_or(DlnaError::Device(format!(
                "missing field {}",
                DLNA_FIELD_SPEED
            )))?
            .parse()
            .map_err(|e| {
                DlnaError::Device(format!("device returned invalid speed value, {}", e))
            })?;
        let current_transport_state = map
            .get(DLNA_FIELD_STATE)
            .ok_or(DlnaError::Device(format!(
                "missing field {}",
                DLNA_FIELD_STATE
            )))?
            .parse()?;
        let current_transport_status = map.get(DLNA_FIELD_STATUS).cloned().unwrap_or_default();

        Ok(Self {
            current_speed,
            current_transport_state,
            current_transport_status,
        })
    }
}

/// Represents the state of a UPnP instance.
#[derive(Debug, PartialEq)]
pub enum UpnpState {
    /// The UPnP instance is stopped.
    Stopped,
    /// The UPnP instance is playing.
    Playing,
    /// The UPnP instance is in paused playback.
    PausedPlayback,
    /// The UPnP instance is transitioning.
    Transitioning,
    /// There is no media present in the UPnP instance.
    NoMediaPresent,
    /// The UPnP instance is in a custom state.
    Custom,
}

impl FromStr for UpnpState {
    type Err = DlnaError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "STOPPED" => Ok(UpnpState::Stopped),
            "PLAYING" => Ok(UpnpState::Playing),
            "PAUSED_PLAYBACK" => Ok(UpnpState::PausedPlayback),
            "TRANSITIONING" => Ok(UpnpState::Transitioning),
            "LG_TRANSITIONING" => Ok(UpnpState::Transitioning),
            "NO_MEDIA_PRESENT" => Ok(UpnpState::NoMediaPresent),
            _ => Err(DlnaError::InvalidTransportState(s.to_string())),
        }
    }
}

impl From<&UpnpState> for PlayerState {
    fn from(state: &UpnpState) -> Self {
        match state {
            UpnpState::Stopped => PlayerState::Stopped,
            UpnpState::Playing => PlayerState::Playing,
            UpnpState::PausedPlayback => PlayerState::Paused,
            UpnpState::Transitioning => PlayerState::Buffering,
            UpnpState::NoMediaPresent => PlayerState::Stopped,
            UpnpState::Custom => PlayerState::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_info_from_hashmap() {
        let track_uri = "MyTrackUri";
        let rel_time = "00:01:25";
        let track_metadata = "MyTrackMetaData";
        let track_duration = "00:30:00";
        let map: HashMap<String, String> = vec![
            ("TrackURI".to_string(), track_uri.to_string()),
            ("AbsCount".to_string(), "100".to_string()),
            ("RelCount".to_string(), "2".to_string()),
            ("RelTime".to_string(), rel_time.to_string()),
            ("Track".to_string(), "1".to_string()),
            ("TrackMetaData".to_string(), track_metadata.to_string()),
            ("TrackDuration".to_string(), track_duration.to_string()),
        ]
        .into_iter()
        .collect();
        let expected_result = PositionInfo {
            track_uri: track_uri.to_string(),
            abs_count: 100,
            rel_count: 2,
            rel_time: rel_time.to_string(),
            track: 1,
            track_metadata: track_metadata.to_string(),
            track_duration: track_duration.to_string(),
        };

        let result = PositionInfo::from(map);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_transport_info_from_valid_hashmap() {
        let status = "OK";
        let map: HashMap<String, String> = vec![
            ("CurrentSpeed".to_string(), "1".to_string()),
            ("CurrentTransportState".to_string(), "PLAYING".to_string()),
            ("CurrentTransportStatus".to_string(), status.to_string()),
        ]
        .into_iter()
        .collect();
        let expected_result = TransportInfo {
            current_speed: 1,
            current_transport_state: UpnpState::Playing,
            current_transport_status: status.to_string(),
        };

        let result = TransportInfo::try_from(map).expect("expected the info to have been mapped");

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_transport_info_from_invalid_hashmap() {
        let map: HashMap<String, String> = vec![
            ("CurrentSpeed".to_string(), "FooBar".to_string()),
            ("CurrentTransportState".to_string(), "PLAYING".to_string()),
            ("CurrentTransportStatus".to_string(), "Lorem".to_string()),
        ]
        .into_iter()
        .collect();

        let result = TransportInfo::try_from(map);
        assert!(
            result.is_err(),
            "expected an error to have been returned, got {:?} instead",
            result
        );

        if let Err(DlnaError::Device(msg)) = &result {
            assert!(
                msg.starts_with("device returned invalid speed value"),
                "expected invalid speed message, but got \"{:?}\" instead",
                msg
            );
        } else {
            assert!(
                false,
                "expected DlnaError::Device, but got {:?} instead",
                result
            );
        }
    }

    #[test]
    fn test_upnp_state_from_str() {
        let result = UpnpState::from_str("STOPPED").unwrap();
        assert_eq!(UpnpState::Stopped, result);

        let result = UpnpState::from_str("PLAYING").unwrap();
        assert_eq!(UpnpState::Playing, result);

        let result = UpnpState::from_str("PAUSED_PLAYBACK").unwrap();
        assert_eq!(UpnpState::PausedPlayback, result);

        let result = UpnpState::from_str("TRANSITIONING").unwrap();
        assert_eq!(UpnpState::Transitioning, result);

        let result = UpnpState::from_str("NO_MEDIA_PRESENT").unwrap();
        assert_eq!(UpnpState::NoMediaPresent, result);
    }

    #[test]
    fn test_player_state_from_upnp_state() {
        let result = PlayerState::from(&UpnpState::Stopped);
        assert_eq!(PlayerState::Stopped, result);

        let result = PlayerState::from(&UpnpState::Playing);
        assert_eq!(PlayerState::Playing, result);

        let result = PlayerState::from(&UpnpState::PausedPlayback);
        assert_eq!(PlayerState::Paused, result);

        let result = PlayerState::from(&UpnpState::Transitioning);
        assert_eq!(PlayerState::Buffering, result);

        let result = PlayerState::from(&UpnpState::NoMediaPresent);
        assert_eq!(PlayerState::Stopped, result);

        let result = PlayerState::from(&UpnpState::Custom);
        assert_eq!(PlayerState::Error, result);
    }
}
