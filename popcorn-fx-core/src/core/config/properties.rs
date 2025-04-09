use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::string::ToString;

use derive_more::Display;
use log::{debug, trace, warn};
use serde::Deserialize;

use crate::core::config;
use crate::core::config::{ConfigError, EnhancerProperties, ProviderProperties};

const DEFAULT_SUBTITLE_URL: fn() -> String = || "https://api.opensubtitles.com/api/v1".to_string();
const DEFAULT_USER_AGENT: fn() -> String = || "Popcorn Time v1".to_string();
const DEFAULT_API_TOKEN: fn() -> String = || "mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo".to_string();
const DEFAULT_UPDATE_CHANNEL: fn() -> String =
    || "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string();
const DEFAULT_PROVIDERS: fn() -> HashMap<String, ProviderProperties> = || {
    vec![
        (
            "movies".to_string(),
            ProviderProperties {
                uris: vec![
                    "https://shows.cf/".to_string(),
                    "https://fusme.link".to_string(),
                    "https://jfper.link".to_string(),
                    "https://uxert.link".to_string(),
                ],
                genres: vec![
                    "all".to_string(),
                    "action".to_string(),
                    "adventure".to_string(),
                    "animation".to_string(),
                    "comedy".to_string(),
                    "crime".to_string(),
                    "disaster".to_string(),
                    "documentary".to_string(),
                    "drama".to_string(),
                    "family".to_string(),
                    "fantasy".to_string(),
                    "history".to_string(),
                    "holiday".to_string(),
                    "horror".to_string(),
                    "music".to_string(),
                    "mystery".to_string(),
                    "romance".to_string(),
                    "science fiction".to_string(),
                    "short".to_string(),
                    "suspense".to_string(),
                    "thriller".to_string(),
                    "war".to_string(),
                    "western".to_string(),
                ],
                sort_by: vec![
                    "trending".to_string(),
                    "popularity".to_string(),
                    "last added".to_string(),
                    "year".to_string(),
                    "title".to_string(),
                    "rating".to_string(),
                ],
            },
        ),
        (
            "series".to_string(),
            ProviderProperties {
                uris: vec![
                    "https://shows.cf/".to_string(),
                    "https://fusme.link".to_string(),
                    "https://jfper.link".to_string(),
                    "https://uxert.link".to_string(),
                ],
                genres: vec![
                    "all".to_string(),
                    "action".to_string(),
                    "adventure".to_string(),
                    "animation".to_string(),
                    "children".to_string(),
                    "comedy".to_string(),
                    "crime".to_string(),
                    "documentary".to_string(),
                    "drama".to_string(),
                    "family".to_string(),
                    "fantasy".to_string(),
                    "horror".to_string(),
                    "mini Series".to_string(),
                    "mystery".to_string(),
                    "news".to_string(),
                    "reality".to_string(),
                    "romance".to_string(),
                    "science-fiction".to_string(),
                    "soap".to_string(),
                    "special Interest".to_string(),
                    "sport".to_string(),
                    "suspense".to_string(),
                    "talk Show".to_string(),
                    "thriller".to_string(),
                    "western".to_string(),
                ],
                sort_by: vec![
                    "trending".to_string(),
                    "popularity".to_string(),
                    "updated".to_string(),
                    "year".to_string(),
                    "name".to_string(),
                    "rating".to_string(),
                ],
            },
        ),
        (
            "favorites".to_string(),
            ProviderProperties {
                uris: vec![],
                genres: vec!["all".to_string(), "movies".to_string(), "tv".to_string()],
                sort_by: vec![
                    "watched".to_string(),
                    "year".to_string(),
                    "title".to_string(),
                    "rating".to_string(),
                ],
            },
        ),
    ]
    .into_iter()
    .collect()
};
const DEFAULT_ENHANCERS: fn() -> HashMap<String, EnhancerProperties> = || {
    vec![(
        "tvdb".to_string(),
        EnhancerProperties {
            uri: "https://thetvdb.com/series/lorem/episodes".to_string(),
        },
    )]
    .into_iter()
    .collect()
};
const DEFAULT_LOGGERS: fn() -> HashMap<String, LoggingProperties> = || HashMap::new();
const DEFAULT_TRACKING: fn() -> HashMap<String, TrackingProperties> = || {
    vec![(
        "trakt".to_string(),
        TrackingProperties {
            uri: "https://api.trakt.tv".to_string(),
            client: TrackingClientProperties {
                client_id: "62a497cb224dc3d4c71a9da940fb9ef1b20ff8ab148c0ffb38b228e0a58ef246"
                    .to_string(),
                client_secret: "5dddda26c750b108990025e2d3a4fb4c0d348eb5c927c99622ca8edd5ca8c202"
                    .to_string(),
                user_authorization_uri: "https://trakt.tv/oauth/authorize".to_string(),
                access_token_uri: "https://api.trakt.tv/oauth/token".to_string(),
            },
        },
    )]
    .into_iter()
    .collect()
};

const DEFAULT_CONFIG_FILENAME: &str = "application";
const CONFIG_EXTENSIONS: [&str; 2] = ["yml", "yaml"];

/// In-between wrapper for serde to support the backwards compatible mapping.
#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "popcorn: {:?}", popcorn)]
struct PropertiesWrapper {
    /// The properties under the "popcorn" field.
    #[serde(default)]
    pub popcorn: PopcornProperties,
}

/// The immutable properties of the application.
#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "update_channel: {}, subtitle: {:?}", update_channel, subtitle)]
pub struct PopcornProperties {
    /// Configuration for loggers.
    #[serde(default = "DEFAULT_LOGGERS")]
    pub loggers: HashMap<String, LoggingProperties>,
    /// The channel for updates.
    #[serde(alias = "update-channel")]
    #[serde(alias = "update_channel")]
    #[serde(default = "DEFAULT_UPDATE_CHANNEL")]
    pub update_channel: String,
    /// Configuration for providers.
    #[serde(default = "DEFAULT_PROVIDERS")]
    pub providers: HashMap<String, ProviderProperties>,
    /// Configuration for enhancers.
    /// Enhancer properties to enhance media items.
    #[serde(default = "DEFAULT_ENHANCERS")]
    pub enhancers: HashMap<String, EnhancerProperties>,
    /// Configuration for subtitles.
    #[serde(default)]
    pub subtitle: SubtitleProperties,
    /// Configuration for tracking.
    #[serde(default = "DEFAULT_TRACKING")]
    pub tracking: HashMap<String, TrackingProperties>,
}

impl PopcornProperties {
    /// Create a new [PopcornProperties] which will look for the [DEFAULT_CONFIG_FILENAME] config file.
    /// It will parse the config file if found, else uses the defaults instead.
    pub fn new_auto() -> Self {
        Self::from_filename(DEFAULT_CONFIG_FILENAME)
    }

    pub fn from_filename(filename: &str) -> Self {
        debug!("Searching for config file with name \"{}\"", filename);
        let config_value = Self::find_existing_file(filename)
            .map(|mut e| {
                let mut data = String::new();
                e.read_to_string(&mut data)
                    .expect("Unable to read the config file");
                data
            })
            .or_else(|| Some(String::new()))
            .expect("Properties should have been loaded");

        Self::from(config_value.as_str())
    }

    /// Retrieve the update channel to query and retrieve updates from.
    /// It returns the string slice of the configured channel.
    pub fn update_channel(&self) -> &str {
        self.update_channel.as_str()
    }

    pub fn subtitle(&self) -> &SubtitleProperties {
        &self.subtitle
    }

    /// Retrieve the provider properties for the given name.
    /// It returns the properties when found, else the [ConfigError].
    pub fn provider<S: AsRef<str>>(&self, name: S) -> config::Result<&ProviderProperties> {
        let name = name.as_ref().to_lowercase();
        self.providers
            .get(&name)
            .ok_or(ConfigError::UnknownProvider(name))
    }

    /// Retrieve the tracking provider properties for the given name.
    /// It returns the properties when found, else the [ConfigError].
    pub fn tracker(&self, name: &str) -> config::Result<&TrackingProperties> {
        let name = name.to_string();
        self.tracking
            .get(&name)
            .ok_or(ConfigError::UnknownTrackingProvider(name))
    }

    /// Retrieve the default provider properties.
    pub fn default_providers() -> HashMap<String, ProviderProperties> {
        DEFAULT_PROVIDERS()
    }

    /// Retrieve the default enhancer properties.
    pub fn default_enhancers() -> HashMap<String, EnhancerProperties> {
        DEFAULT_ENHANCERS()
    }

    pub fn default_trackings() -> HashMap<String, TrackingProperties> {
        DEFAULT_TRACKING()
    }

    fn find_existing_file(filename: &str) -> Option<File> {
        let mut result: Option<File> = None;

        for extension in CONFIG_EXTENSIONS {
            let path = Self::config_file_path(filename, extension);
            match File::open(&path) {
                Ok(file) => {
                    debug!("Found config file {}", &path);
                    result = Some(file);
                    break;
                }
                Err(_) => trace!("Config file location {} doesn't exist", &path),
            }
        }

        result
    }

    fn config_file_path(filename: &str, extension: &str) -> String {
        let mut directory = env::current_dir().unwrap();
        directory.push(format!("{}.{}", filename, extension));
        let path = directory.to_str();

        String::from(path.unwrap())
    }
}

impl From<&str> for PopcornProperties {
    /// Convert the given configuration `json` data into properties.
    /// If the given string slice is invalid, the defaults will be returned.
    fn from(json_value: &str) -> Self {
        trace!("Parsing configuration properties data {}", json_value);
        let data: PropertiesWrapper = match serde_yaml::from_str(json_value) {
            Ok(properties) => properties,
            Err(err) => {
                warn!(
                    "Failed to parse properties, using defaults instead, {}",
                    err
                );
                serde_yaml::from_str(String::new().as_str()).unwrap()
            }
        };

        debug!("Parsed configuration properties data {:?}", &data);
        data.popcorn
    }
}

impl Default for PopcornProperties {
    fn default() -> Self {
        Self {
            loggers: DEFAULT_LOGGERS(),
            update_channel: DEFAULT_UPDATE_CHANNEL(),
            providers: DEFAULT_PROVIDERS(),
            enhancers: DEFAULT_ENHANCERS(),
            subtitle: SubtitleProperties::default(),
            tracking: DEFAULT_TRACKING(),
        }
    }
}

/// Represents properties for subtitle provider configuration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SubtitleProperties {
    /// The URL for subtitle retrieval.
    #[serde(default = "DEFAULT_SUBTITLE_URL")]
    pub url: String,
    /// The user agent to be used in the connection URL.
    #[serde(alias = "user-agent")]
    #[serde(alias = "userAgent")]
    #[serde(default = "DEFAULT_USER_AGENT")]
    pub user_agent: String,
    /// The API token to use while querying the subtitle provider.
    #[serde(alias = "api-token")]
    #[serde(alias = "apiToken")]
    #[serde(default = "DEFAULT_API_TOKEN")]
    pub api_token: String,
}

impl SubtitleProperties {
    /// Retrieves the subtitle base URL for retrieving subtitle information.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Retrieves the user agent to be used within the connection URL.
    pub fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }

    /// Retrieves the API token to use while querying the subtitle provider.
    pub fn api_token(&self) -> &str {
        self.api_token.as_str()
    }
}

impl Default for SubtitleProperties {
    fn default() -> Self {
        Self {
            url: DEFAULT_SUBTITLE_URL(),
            user_agent: DEFAULT_USER_AGENT(),
            api_token: DEFAULT_API_TOKEN(),
        }
    }
}

/// Represents properties for logging configuration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoggingProperties {
    /// The logging level to apply.
    pub level: String,
}

/// Represents properties for tracking configuration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TrackingProperties {
    /// The URI for tracking.
    pub uri: String,
    /// Properties related to the tracking client.
    pub client: TrackingClientProperties,
}

impl TrackingProperties {
    /// Gets the URI for tracking.
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    /// Gets the properties related to the tracking client.
    pub fn client(&self) -> &TrackingClientProperties {
        &self.client
    }
}

/// Represents properties for the tracking client configuration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TrackingClientProperties {
    /// The client ID for tracking.
    pub client_id: String,
    /// The client secret for tracking.
    pub client_secret: String,
    /// The URI for user authorization.
    pub user_authorization_uri: String,
    /// The URI for accessing the access token.
    pub access_token_uri: String,
}

#[cfg(test)]
mod test {
    use std::path::MAIN_SEPARATOR;

    use crate::init_logger;

    use super::*;

    #[test]
    fn test_config_file_path() {
        init_logger!();
        let filename = "lorem";
        let extension = "csv";
        let expected_result = format!(
            "{}{}{}.{}",
            env::current_dir().unwrap().to_str().unwrap(),
            MAIN_SEPARATOR,
            filename,
            extension
        );

        let result = PopcornProperties::config_file_path(filename, extension);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_filename_when_not_found_should_return_defaults() {
        init_logger!();
        let expected_result = PopcornProperties {
            loggers: Default::default(),
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            enhancers: PopcornProperties::default_enhancers(),
            subtitle: SubtitleProperties {
                url: String::from("https://api.opensubtitles.com/api/v1"),
                user_agent: String::from("Popcorn Time v1"),
                api_token: String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo"),
            },
            tracking: PopcornProperties::default_trackings(),
        };

        let result = PopcornProperties::new_auto();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_str_should_return_parsed_data() {
        init_logger!();
        let config_value = "
popcorn:
  subtitle:
    url: http://my-url
    user-agent: lorem
    api-token: ipsum";
        let expected_result = PopcornProperties {
            loggers: Default::default(),
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            enhancers: PopcornProperties::default_enhancers(),
            subtitle: SubtitleProperties {
                url: String::from("http://my-url"),
                user_agent: "lorem".to_string(),
                api_token: "ipsum".to_string(),
            },
            tracking: PopcornProperties::default_trackings(),
        };

        let result = PopcornProperties::from(config_value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_str_when_partial_fields_are_present_should_complete_with_defaults() {
        init_logger!();
        let config_value = r#"
popcorn:
  subtitle:
    user-agent: lorem"#;
        let expected_result = PopcornProperties {
            loggers: Default::default(),
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            enhancers: PopcornProperties::default_enhancers(),
            subtitle: SubtitleProperties {
                url: String::from("https://api.opensubtitles.com/api/v1"),
                user_agent: String::from("lorem"),
                api_token: String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo"),
            },
            tracking: PopcornProperties::default_trackings(),
        };

        let result = PopcornProperties::from(config_value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_provider_unknown_name() {
        init_logger!();
        let provider = "lorem ipsum";
        let properties = PopcornProperties::default();

        let result = properties
            .provider(provider)
            .err()
            .expect("expected an error");

        if let ConfigError::UnknownProvider(name) = result {
            assert_eq!(provider.to_string(), name)
        } else {
            assert!(false, "expected ConfigError::UnknownProvider")
        }
    }
}
