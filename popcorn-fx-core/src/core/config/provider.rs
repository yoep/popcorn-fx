use derive_more::Display;
use serde::Deserialize;

/// The [crate::core::media::MediaIdentifier] provider properties which can be used to query a [crate::core::media::providers::MediaProvider].
#[derive(Debug, Display, Clone, PartialEq, Deserialize)]
#[display("uris: {:?}, genres: {:?}, sort_by: {:?}", uris, genres, sort_by)]
pub struct ProviderProperties {
    /// The provider uri's to use
    pub uris: Vec<String>,
    /// The provider supported genres
    /// For more info, see https://popcornofficial.docs.apiary.io/#reference/genres/page?console=1
    pub genres: Vec<String>,
    /// The provider sorting options
    pub sort_by: Vec<String>,
}

impl ProviderProperties {
    /// The array slice of available uri's for the provider.
    pub fn uris(&self) -> &[String] {
        &self.uris
    }

    /// The array slice of available genres for the provider.
    pub fn genres(&self) -> &[String] {
        &self.genres[..]
    }

    /// The array slice of the available sorting options for the provider.
    pub fn sort_by(&self) -> &[String] {
        &self.sort_by[..]
    }
}

/// The [crate::core::media::MediaIdentifier] enhancer properties which can be used by any enhancer.
#[derive(Debug, Display, Clone, PartialEq, Deserialize)]
#[display("uri: {}", uri)]
pub struct EnhancerProperties {
    /// The enhancer uri to use for retrieving additional information
    pub uri: String,
}
