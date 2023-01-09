use std::sync::Arc;

use log::error;

use crate::core::config::Application;

/// Retrieve the available uri's from the settings for the given provider name.
pub fn available_uris(settings: &Arc<Application>, provider_name: &str) -> Vec<String> {
    let api_server = settings.settings().server().api_server();
    let mut uris: Vec<String> = vec![];

    match api_server {
        None => {}
        Some(e) => uris.push(e.clone())
    }

    match settings.properties().provider(provider_name.to_string()) {
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

    use crate::core::config::{PopcornProperties, PopcornSettings, ProviderProperties, ServerSettings, SubtitleProperties, SubtitleSettings, UiSettings};
    use crate::test::init_logger;

    use super::*;

    #[test]
    fn test_available_uris_provider_available() {
        init_logger();
        let api_server = "http://lorem".to_string();
        let provider = "http://ipsum".to_string();
        let provider_name = "my-provider".to_string();
        let settings = Arc::new(Application::new(
            PopcornProperties::new_with_providers(SubtitleProperties::default(), HashMap::from([
                (provider_name.clone(), ProviderProperties::new(
                    vec![provider.clone()],
                    vec![],
                    vec![],
                ))
            ])),
            PopcornSettings::new(SubtitleSettings::default(), UiSettings::default(),
                                 ServerSettings::new(api_server.clone())),
        ));
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
        let settings = Arc::new(Application::new(
            PopcornProperties::new_with_providers(SubtitleProperties::default(), HashMap::new()),
            PopcornSettings::new(SubtitleSettings::default(), UiSettings::default(),
                                 ServerSettings::new(api_server.clone())),
        ));
        let expected_result = vec![
            api_server,
        ];

        let result = available_uris(&settings, "lorem");

        assert_eq!(expected_result, result)
    }
}