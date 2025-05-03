use std::collections::HashMap;

use chrono::serde::ts_milliseconds;
use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Local, Utc};
use derive_more::Display;
use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "trackers: {:?}", "self.trackers()")]
pub struct TrackingSettings {
    last_sync: Option<LastSync>,
    trackers: HashMap<String, Tracker>,
}

impl TrackingSettings {
    pub fn builder() -> TrackingSettingsBuilder {
        TrackingSettingsBuilder::builder()
    }

    pub fn last_sync(&self) -> Option<&LastSync> {
        self.last_sync.as_ref()
    }

    pub fn update_state(&mut self, state: MediaTrackingSyncState) {
        trace!("Updating last sync state to {}", state);
        self.last_sync = Some(LastSync {
            time: Local::now().with_timezone(&Utc),
            state,
        });
    }

    pub fn trackers(&self) -> Vec<String> {
        self.trackers.keys().map(|e| e.clone()).collect()
    }

    pub fn tracker(&self, name: &str) -> Option<Tracker> {
        self.trackers
            .iter()
            .find(|(key, _)| key.as_str() == name)
            .map(|(_, v)| v.clone())
    }

    pub fn update<S: Into<String>>(&mut self, name: S, tracker: Tracker) {
        self.trackers.insert(name.into(), tracker);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.trackers.remove(name).is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LastSync {
    #[serde(with = "ts_milliseconds")]
    pub time: DateTime<Utc>,
    pub state: MediaTrackingSyncState,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    pub access_token: String,
    #[serde(with = "ts_milliseconds_option")]
    pub expires_in: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaTrackingSyncState {
    #[display(fmt = "success")]
    Success = 0,
    #[display(fmt = "failed")]
    Failed = 1,
}

/// Builder for constructing `TrackingSettings` instances.
#[derive(Debug, Default)]
pub struct TrackingSettingsBuilder {
    last_sync: Option<LastSync>,
    trackers: HashMap<String, Tracker>,
}

impl TrackingSettingsBuilder {
    /// Creates a new `TrackingSettingsBuilder`.
    pub fn builder() -> Self {
        TrackingSettingsBuilder::default()
    }

    /// Sets the last sync for the builder.
    pub fn last_sync(mut self, last_sync: LastSync) -> Self {
        self.last_sync = Some(last_sync);
        self
    }

    /// Adds a new tracker to the builder.
    ///
    /// If a tracker with the same name already exists, it will be replaced by the new one.
    pub fn tracker<S: Into<String>>(mut self, name: S, tracker: Tracker) -> Self {
        self.trackers.insert(name.into(), tracker);
        self
    }

    /// Builds the `TrackingSettings` instance.
    pub fn build(self) -> TrackingSettings {
        TrackingSettings {
            last_sync: self.last_sync,
            trackers: self.trackers,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn test_update_state() {
        let mut settings = TrackingSettings {
            last_sync: None,
            trackers: vec![].into_iter().collect(),
        };

        settings.update_state(MediaTrackingSyncState::Success);

        let result = settings.last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Success, result.state);
        assert!(
            Local::now().with_timezone(&Utc) - result.time < TimeDelta::milliseconds(100),
            "expected the last sync time to have been filled in"
        );
    }

    #[test]
    fn test_trackers() {
        let expected_result = vec!["lorem", "ipsum"];
        let settings = TrackingSettings {
            last_sync: None,
            trackers: vec![
                ("lorem".to_string(), Tracker::default()),
                ("ipsum".to_string(), Tracker::default()),
            ]
            .into_iter()
            .collect(),
        };

        let result = settings.trackers();

        for e in expected_result {
            assert!(
                result.contains(&e.to_string()),
                "expected {} to have been present",
                e
            )
        }
    }

    #[test]
    fn test_tracker() {
        let name = "MyTracker";
        let tracker = Tracker {
            access_token: "".to_string(),
            expires_in: None,
            refresh_token: None,
            scopes: None,
        };
        let settings = TrackingSettings {
            last_sync: None,
            trackers: vec![(name.to_string(), tracker.clone())]
                .into_iter()
                .collect(),
        };

        let result = settings.tracker(name);

        assert_eq!(Some(tracker), result);
    }

    #[test]
    fn test_update() {
        let name = "SomeRandomTracker";
        let tracker = Tracker {
            access_token: "SomeRandomToken".to_string(),
            expires_in: None,
            refresh_token: None,
            scopes: None,
        };
        let mut settings = TrackingSettings {
            last_sync: None,
            trackers: Default::default(),
        };

        settings.update(name, tracker.clone());
        let result = settings.tracker(name);

        assert_eq!(Some(tracker), result);
    }

    #[test]
    fn test_remove() {
        let name = "FooBar";
        let mut settings = TrackingSettings {
            last_sync: None,
            trackers: vec![(name.to_string(), Tracker::default())]
                .into_iter()
                .collect(),
        };

        settings.remove(name);

        assert_eq!(0, settings.trackers.len());
    }

    #[test]
    fn test_builder() {
        let name = "MyTracker";
        let time = Local::now().with_timezone(&Utc);
        let tracker = Tracker {
            access_token: "84521".to_string(),
            expires_in: None,
            refresh_token: None,
            scopes: None,
        };
        let expected_result = TrackingSettings {
            last_sync: Some(LastSync {
                time,
                state: MediaTrackingSyncState::Success,
            }),
            trackers: vec![(name.to_string(), tracker.clone())]
                .into_iter()
                .collect(),
        };

        let result = TrackingSettings::builder()
            .last_sync(LastSync {
                time,
                state: MediaTrackingSyncState::Success,
            })
            .tracker(name, tracker)
            .build();

        assert_eq!(expected_result, result);
    }
}
