use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;

const DEFAULT_API_SERVER: fn() -> Option<String> = || None;

/// The api server preferences of the user for the application.
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "api_server: {:?}", api_server)]
pub struct ServerSettings {
    /// The api server to use
    #[serde(default = "DEFAULT_API_SERVER")]
    pub api_server: Option<String>,
}

impl ServerSettings {
    /// The configured API server to use for all [crate::core::media::Media] providers.
    pub fn api_server(&self) -> Option<&String> {
        match &self.api_server {
            None => None,
            Some(e) => Some(e)
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            api_server: DEFAULT_API_SERVER(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_server_settings_default() {
        let expected_result = ServerSettings {
            api_server: DEFAULT_API_SERVER(),
        };

        let result = ServerSettings::default();

        assert_eq!(expected_result, result)
    }
}