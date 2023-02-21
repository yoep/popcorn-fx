use std::os::raw::c_char;

use log::trace;

use crate::{into_c_string, to_c_vec};
use crate::core::config::PopcornProperties;

/// The C compatible properties of the application.
#[repr(C)]
#[derive(Debug)]
pub struct PopcornPropertiesC {
    /// The version of the application
    pub version: *const c_char,
    /// The update channel to retrieve updates from
    pub update_channel: *const c_char,
    /// The array of available provider properties
    pub provider_properties: *mut ProviderPropertiesC,
    /// The length of the provider properties array
    pub provider_properties_len: i32,
}

impl From<&PopcornProperties> for PopcornPropertiesC {
    fn from(value: &PopcornProperties) -> Self {
        trace!("Converting PopcornProperties to C for {:?}", value);
        let (provider_properties, provider_properties_len) = to_c_vec(value.providers.iter()
            .map(|(key, v)| {
                let (genres, genres_len) = to_c_vec(v.genres().iter()
                    .map(|e| into_c_string(e.clone()))
                    .collect());
                let (sort_by, sort_by_len) = to_c_vec(v.sort_by().iter()
                    .map(|e| into_c_string(e.clone()))
                    .collect());

                ProviderPropertiesC {
                    name: into_c_string(key.clone()),
                    genres,
                    genres_len,
                    sort_by,
                    sort_by_len,
                }
            })
            .collect());

        Self {
            version: into_c_string(value.version().to_string()),
            update_channel: into_c_string(value.update_channel().to_string()),
            provider_properties,
            provider_properties_len,
        }
    }
}

/// The C compatible media provider properties.
#[repr(C)]
#[derive(Debug)]
pub struct ProviderPropertiesC {
    /// The name of the provider.
    pub name: *const c_char,
    /// The array of available genres for the provider.
    pub genres: *mut *const c_char,
    /// The length of the genres array.
    pub genres_len: i32,
    /// The array of available sorting options for the provider.
    pub sort_by: *mut *const c_char,
    /// The length of the sorting options array.
    pub sort_by_len: i32
}

#[cfg(test)]
mod test {
    use crate::from_c_string;

    use super::*;

    #[test]
    fn test_properties_from() {
        let version = "1.0.0";
        let update_channel = "https://my-update-channel.com";
        let properties = PopcornProperties {
            version: version.to_string(),
            update_channel: update_channel.to_string(),
            providers: Default::default(),
            subtitle: Default::default(),
        };

        let result = PopcornPropertiesC::from(&properties);

        assert_eq!(version.to_string(), from_c_string(result.version));
        assert_eq!(update_channel.to_string(), from_c_string(result.update_channel));
    }
}