use crate::core::config::PopcornProperties;
use log::error;

/// Returns the available API uris for the given provider.
pub(crate) async fn available_uris(
    api_servers: &[String],
    properties: &PopcornProperties,
    provider_name: &str,
) -> Vec<String> {
    let mut uris: Vec<String> = api_servers.to_vec();
    match properties.provider(provider_name) {
        Ok(e) => {
            for uri in e.uris() {
                uris.push(uri.clone());
            }
        }
        Err(err) => error!("Failed to retrieve provider info, {}", err),
    };

    uris
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::config::{PopcornProperties, ProviderProperties};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_available_uris_provider_available() {
        let api_server = "http://lorem";
        let provider = "http://ipsum".to_string();
        let provider_name = "my-provider".to_string();
        let properties = PopcornProperties {
            loggers: Default::default(),
            update_channel: String::new(),
            providers: HashMap::from([(
                provider_name.clone(),
                ProviderProperties {
                    uris: vec![provider.clone()],
                    genres: vec![],
                    sort_by: vec![],
                },
            )]),
            enhancers: Default::default(),
            subtitle: Default::default(),
            tracking: Default::default(),
        };
        let expected_result = vec![api_server.to_string(), provider];

        let result = available_uris(
            &[api_server.to_string()],
            &properties,
            provider_name.as_str(),
        )
        .await;

        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_available_uris_provider_not_available() {
        let api_server = "https://www.google.com";
        let properties = PopcornProperties {
            loggers: Default::default(),
            update_channel: String::new(),
            providers: HashMap::new(),
            enhancers: Default::default(),
            subtitle: Default::default(),
            tracking: Default::default(),
        };
        let expected_result = vec![api_server.to_string()];

        let result = available_uris(&[api_server.to_string()], &properties, "lorem").await;

        assert_eq!(expected_result, result)
    }
}
