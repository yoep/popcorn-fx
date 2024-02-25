use log::error;

use crate::core::config::ApplicationConfig;

/// Retrieve the available uri's from the settings for the given provider name.
pub fn available_uris(config: &ApplicationConfig, provider_name: &str) -> Vec<String> {
    let settings = config.user_settings();
    let api_server = settings.server().api_server();
    let mut uris: Vec<String> = vec![];

    match api_server {
        None => {}
        Some(e) => uris.push(e.clone())
    }

    let properties = config.properties();
    match properties.provider(provider_name) {
        Ok(e) => {
            for uri in e.uris() {
                uris.push(uri.clone());
            }
        }
        Err(err) => error!("Failed to retrieve provider info, {}", err)
    };

    uris
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::core::config::{PopcornProperties, PopcornSettings, ProviderProperties, ServerSettings};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_available_uris_provider_available() {
        init_logger();
        let api_server = "http://lorem".to_string();
        let provider = "http://ipsum".to_string();
        let provider_name = "my-provider".to_string();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: HashMap::from([
                    (provider_name.clone(),
                     ProviderProperties {
                         uris: vec![provider.clone()],
                         genres: vec![],
                         sort_by: vec![],
                     })
                ]),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: Default::default(),
            })
            .settings(PopcornSettings {
                subtitle_settings: Default::default(),
                ui_settings: Default::default(),
                server_settings: ServerSettings {
                    api_server: Some(api_server.clone()),
                },
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
            })
            .build();
        let expected_result = vec![
            api_server,
            provider,
        ];

        let result = available_uris(&settings, provider_name.as_str());

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_available_uris_provider_not_available() {
        init_logger();
        let api_server = "https://www.google.com".to_string();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: HashMap::new(),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: Default::default(),
            })
            .settings(PopcornSettings {
                subtitle_settings: Default::default(),
                ui_settings: Default::default(),
                server_settings: ServerSettings {
                    api_server: Some(api_server.clone()),
                },
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
            })
            .build();
        let expected_result = vec![
            api_server,
        ];

        let result = available_uris(&settings, "lorem");

        assert_eq!(expected_result, result)
    }
}