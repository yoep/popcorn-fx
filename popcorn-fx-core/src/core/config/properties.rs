use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::string::ToString;

use derive_more::Display;
use log::{debug, trace, warn};
use serde::Deserialize;

use crate::core::config::{ConfigError, ProviderProperties};

const DEFAULT_SUBTITLE_URL: fn() -> String = || "https://api.opensubtitles.com/api/v1".to_string();
const DEFAULT_TVDB_URL: fn() -> String = || "https://thetvdb.com/series/lorem/episodes".to_string();
const DEFAULT_USER_AGENT: fn() -> String = || "Popcorn Time v1".to_string();
const DEFAULT_API_TOKEN: fn() -> String = || "mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo".to_string();
const DEFAULT_UPDATE_CHANNEL: fn() -> String = || "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string();
const DEFAULT_PROVIDERS: fn() -> HashMap<String, ProviderProperties> = || {
    let mut map: HashMap<String, ProviderProperties> = HashMap::new();
    map.insert("movies".to_string(), ProviderProperties {
        uris: vec![
            "https://popcorn-time.ga".to_string(),
            "https://movies-v2.api-fetch.am".to_string(),
            "https://movies-v2.api-fetch.website".to_string(),
            "https://movies-v2.api-fetch.sh".to_string()],
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
            "science-fiction".to_string(),
            "short".to_string(),
            "suspense".to_string(),
            "thriller".to_string(),
            "war".to_string(),
            "western".to_string()],
        sort_by: vec![
            "trending".to_string(),
            "popularity".to_string(),
            "last added".to_string(),
            "year".to_string(),
            "title".to_string(),
            "rating".to_string(),
        ],
    });
    map.insert("series".to_string(), ProviderProperties {
        uris: vec![
            "https://popcorn-time.ga".to_string(),
            "https://tv-v2.api-fetch.am".to_string(),
            "https://tv-v2.api-fetch.website".to_string(),
            "https://tv-v2.api-fetch.sh".to_string()],
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
    });
    map.insert("favorites".to_string(), ProviderProperties {
        uris: vec![],
        genres: vec![
            "all".to_string(),
            "movies".to_string(),
            "tv".to_string(),
        ],
        sort_by: vec![
            "watched".to_string(),
            "year".to_string(),
            "title".to_string(),
            "rating".to_string(),
        ],
    });
    map
};

const DEFAULT_CONFIG_FILENAME: &str = "application";
const CONFIG_EXTENSIONS: [&str; 2] = [
    "yml",
    "yaml"
];

/// In-between wrapper for serde to support the backwards compatible mapping
#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "popcorn: {:?}", popcorn)]
struct PropertiesWrapper {
    #[serde(default)]
    pub popcorn: PopcornProperties,
}

/// The immutable properties of the application.
#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "update_channel: {}, subtitle: {:?}", update_channel, subtitle)]
pub struct PopcornProperties {
    #[serde(default = "DEFAULT_UPDATE_CHANNEL")]
    pub update_channel: String,
    #[serde(default = "DEFAULT_PROVIDERS")]
    pub providers: HashMap<String, ProviderProperties>,
    #[serde(default)]
    pub subtitle: SubtitleProperties,
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
                e.read_to_string(&mut data).expect("Unable to read the config file");
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
    pub fn provider(&self, name: &str) -> crate::core::config::Result<&ProviderProperties> {
        self.providers.get(&name.to_string())
            .ok_or(ConfigError::UnknownProvider(name.to_string()))
    }

    /// Retrieve the default provider properties.
    pub fn default_providers() -> HashMap<String, ProviderProperties> {
        DEFAULT_PROVIDERS()
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
                Err(_) => trace!("Config file location {} doesn't exist", &path)
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
                warn!("Failed to parse properties, using defaults instead, {}", err);
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
            update_channel: DEFAULT_UPDATE_CHANNEL(),
            providers: DEFAULT_PROVIDERS(),
            subtitle: SubtitleProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SubtitleProperties {
    #[serde(default = "DEFAULT_SUBTITLE_URL")]
    pub url: String,
    #[serde(alias = "user-agent")]
    #[serde(alias = "userAgent")]
    #[serde(default = "DEFAULT_USER_AGENT")]
    pub user_agent: String,
    #[serde(alias = "api-token")]
    #[serde(alias = "apiToken")]
    #[serde(default = "DEFAULT_API_TOKEN")]
    pub api_token: String,
}

impl SubtitleProperties {
    /// Retrieve the subtitle base url to retrieve the subtitle info from.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Retrieve the user agent which needs to be used within the connection url.
    pub fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }

    /// Retrieve the api token to use while querying the subtitle provider.
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

#[cfg(test)]
mod test {
    use std::path::MAIN_SEPARATOR;

    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_config_file_path() {
        init_logger();
        let filename = "lorem";
        let extension = "csv";
        let expected_result = format!("{}{}{}.{}", env::current_dir().unwrap().to_str().unwrap(), MAIN_SEPARATOR, filename, extension);

        let result = PopcornProperties::config_file_path(filename, extension);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_filename_when_not_found_should_return_defaults() {
        init_logger();
        let expected_result = PopcornProperties {
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            subtitle: SubtitleProperties {
                url: String::from("https://api.opensubtitles.com/api/v1"),
                user_agent: String::from("Popcorn Time v1"),
                api_token: String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo"),
            },
        };

        let result = PopcornProperties::new_auto();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_str_should_return_parsed_data() {
        init_logger();
        let config_value = "
popcorn:
  subtitle:
    url: http://my-url
    user-agent: lorem
    api-token: ipsum";
        let expected_result = PopcornProperties {
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            subtitle: SubtitleProperties {
                url: String::from("http://my-url"),
                user_agent: "lorem".to_string(),
                api_token: "ipsum".to_string(),
            },
        };

        let result = PopcornProperties::from(config_value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_str_when_partial_fields_are_present_should_complete_with_defaults() {
        init_logger();
        let config_value = r#"
popcorn:
  subtitle:
    user-agent: lorem"#;
        let expected_result = PopcornProperties {
            update_channel: "https://raw.githubusercontent.com/yoep/popcorn-fx/master/".to_string(),
            providers: PopcornProperties::default_providers(),
            subtitle: SubtitleProperties {
                url: String::from("https://api.opensubtitles.com/api/v1"),
                user_agent: String::from("lorem"),
                api_token: String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo"),
            },
        };

        let result = PopcornProperties::from(config_value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_provider_unknown_name() {
        init_logger();
        let provider = "lorem ipsum";
        let properties = PopcornProperties::default();

        let result = properties.provider(provider)
            .err()
            .expect("expected an error");

        if let ConfigError::UnknownProvider(name) = result {
            assert_eq!(provider.to_string(), name)
        } else {
            assert!(false, "expected ConfigError::UnknownProvider")
        }
    }
}