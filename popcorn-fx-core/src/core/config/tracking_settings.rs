use std::collections::HashMap;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "trackers: {:?}", "self.trackers()")]
pub struct TrackingSettings {
    pub trackers: HashMap<String, Tracker>,
}

impl TrackingSettings {
    pub fn trackers(&self) -> Vec<String> {
        self.trackers.keys()
            .map(|e| e.clone())
            .collect()
    }

    pub fn tracker(&self, name: &str) -> Option<Tracker> {
        self.trackers.iter()
            .find(|(key, _)| key.as_str() == name)
            .map(|(_, v)| v.clone())
    }
    
    pub fn remove(&mut self, name: &str) {
        self.trackers.remove(name);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trackers() {
        let expected_result = vec!["lorem", "ipsum"];
        let settings = TrackingSettings {
            trackers: vec![
                ("lorem".to_string(), Tracker::default()),
                ("ipsum".to_string(), Tracker::default()),
            ].into_iter().collect(),
        };

        let result = settings.trackers();

        for e  in expected_result {
            assert!(result.contains(&e.to_string()), "expected {} to have been present", e)
        }
    }
}