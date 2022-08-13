use std::env;
use std::fs::File;
use std::io::Read;
use std::string::ToString;

use derive_more::Display;
use log::{debug, trace, warn};
use serde::Deserialize;

const DEFAULT_URL: fn() -> String = || { "https://api.opensubtitles.com/api/v1".to_string() };
const DEFAULT_USER_AGENT: fn() -> String = || { "Popcorn Time v1".to_string() };
const DEFAULT_API_TOKEN: fn() -> String = || { "mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo".to_string() };

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

#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "subtitle: {:?}", subtitle)]
pub struct PopcornProperties {
    #[serde(default)]
    subtitle: SubtitleProperties,
}

impl PopcornProperties {
    /// Create a new [PopcornProperties] with the given properties.
    pub fn new(subtitle: SubtitleProperties) -> Self {
        Self {
            subtitle
        }
    }

    /// Create a new [PopcornProperties] which will look for the [DEFAULT_CONFIG_FILENAME] config file.
    /// It will parse the config file if found, else uses the defaults instead.
    pub fn new_auto() -> Self {
        Self::from_filename(DEFAULT_CONFIG_FILENAME)
    }

    /// Create [PopcornProperties] based on the defaults configured.
    /// This function won't search or load any config files.
    pub fn default() -> Self {
        Self::from_str("")
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
            .unwrap();

        Self::from_str(config_value.as_str())
    }

    pub fn from_str(config_data_value: &str) -> Self {
        trace!("Parsing config data {}", config_data_value);
        let data: PropertiesWrapper = match serde_yaml::from_str(config_data_value) {
            Ok(properties) => properties,
            Err(err) => {
                warn!("Failed to parse config, {}, using defaults instead", err);
                serde_yaml::from_str(String::new().as_str()).unwrap()
            }
        };

        debug!("Parsed config data {:?}", &data);
        data.popcorn
    }

    pub fn subtitle(&self) -> &SubtitleProperties {
        &self.subtitle
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

impl Default for PopcornProperties {
    fn default() -> Self {
        Self {
            subtitle: SubtitleProperties::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SubtitleProperties {
    #[serde(default = "DEFAULT_URL")]
    url: String,
    #[serde(alias = "user-agent")]
    #[serde(alias = "userAgent")]
    #[serde(default = "DEFAULT_USER_AGENT")]
    user_agent: String,
    #[serde(alias = "api-token")]
    #[serde(alias = "apiToken")]
    #[serde(default = "DEFAULT_API_TOKEN")]
    api_token: String,
}

impl Default for SubtitleProperties {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL(),
            user_agent: DEFAULT_USER_AGENT(),
            api_token: DEFAULT_API_TOKEN(),
        }
    }
}

impl SubtitleProperties {
    /// Create new [SubtitleProperties] for the subtitle static properties.
    /// * `url`         - The base url to use for retrieving subtitle info.
    /// * `user_agent`  - The user agent to communicate to the subtitle url host.
    /// * `api_token`   - The API token to use for authentication.
    pub fn new(url: String, user_agent: String, api_token: String) -> Self {
        Self {
            url,
            user_agent,
            api_token,
        }
    }

    /// Retrieve the subtitle base url to retrieve the subtitle info from.
    pub fn url(&self) -> &String {
        &self.url
    }

    /// Retrieve the user agent which needs to be used within the connection url.
    pub fn user_agent(&self) -> &String {
        &self.user_agent
    }

    pub fn api_token(&self) -> &String {
        &self.api_token
    }
}

#[cfg(test)]
mod test {
    use std::path::MAIN_SEPARATOR;

    use crate::test::init_logger;

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
        let expected_result = PopcornProperties::new(SubtitleProperties::new(String::from("https://api.opensubtitles.com/api/v1"), String::from("Popcorn Time v1"), String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo")));

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
        let expected_result = PopcornProperties::new(SubtitleProperties::new(String::from("http://my-url"), String::from("lorem"), String::from("ipsum")));

        let result = PopcornProperties::from_str(config_value);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_str_when_partial_fields_are_present_should_complete_with_defaults() {
        init_logger();
        let config_value = "
popcorn:
  subtitle:
    user-agent: lorem";
        let expected_result = PopcornProperties::new(SubtitleProperties::new(String::from("https://api.opensubtitles.com/api/v1"), String::from("lorem"), String::from("mjU10F1qmFwv3JHPodNt9T4O4SeQFhCo")));

        let result = PopcornProperties::from_str(config_value);

        assert_eq!(expected_result, result)
    }
}