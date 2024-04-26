use std::str::FromStr;

use log::{trace, warn};
use thiserror::Error;
use url::Url;

pub type MagnetResult = Result<Magnet, MagnetError>;

/// Represents possible errors that can occur when parsing a magnet URI.
#[derive(Debug, Error)]
pub enum MagnetError {
    #[error("failed to parse magnet uri, {0}")]
    Parse(String),
    #[error("invalid magnet uri")]
    InvalidUri,
}

/// Represents a Magnet link.
#[derive(Debug, Clone, PartialEq)]
pub struct Magnet {
    pub exact_topic: String,
    pub display_name: Option<String>,
    pub exact_length: Option<u64>,
    pub address_tracker: Vec<String>,
    pub web_seed: Vec<String>,
    pub acceptable_source: Vec<String>,
    pub exact_source: Option<String>,
    pub keyword_topic: Option<String>,
    pub manifest_topic: Option<String>,
    pub select_only: Option<String>,
    pub peer: Option<String>,
}

impl Magnet {
    /// Gets the 'xt' (exact topic) value from the magnet link.
    pub fn xt(&self) -> &str {
        self.exact_topic.as_str()
    }

    /// Gets the 'dn' (display name) value from the magnet link, if present.
    pub fn dn(&self) -> Option<&str> {
        self.display_name.as_ref().map(|e| e.as_str())
    }

    /// Gets the 'xl' (exact length) value from the magnet link, if present.
    pub fn xl(&self) -> Option<u64> {
        self.exact_length.clone()
    }

    /// Gets the 'tr' (address tracker) values from the magnet link.
    pub fn tr(&self) -> &[String] {
        self.address_tracker.as_slice()
    }

    /// Gets the 'ws' (web seed) values from the magnet link.
    pub fn ws(&self) -> &[String] {
        self.web_seed.as_slice()
    }

    /// Gets the 'as' (acceptable source) values from the magnet link.
    pub fn as_(&self) -> &[String] {
        self.acceptable_source.as_slice()
    }

    /// Gets the 'xs' (exact source) value from the magnet link, if present.
    pub fn xs(&self) -> Option<&str> {
        self.exact_source.as_ref().map(|e| e.as_str())
    }

    /// Gets the 'kt' (keyword topic) value from the magnet link, if present.
    pub fn kt(&self) -> Option<&str> {
        self.keyword_topic.as_ref().map(|e| e.as_str())
    }

    /// Gets the 'mt' (manifest topic) value from the magnet link, if present.
    pub fn mt(&self) -> Option<&str> {
        self.manifest_topic.as_ref().map(|e| e.as_str())
    }

    /// Gets the 'so' (select only) value from the magnet link, if present.
    pub fn so(&self) -> Option<&str> {
        self.select_only.as_ref().map(|e| e.as_str())
    }

    /// Gets the 'x.pe' (peer) value from the magnet link, if present.
    pub fn x_pe(&self) -> Option<&str> {
        self.peer.as_ref().map(|e| e.as_str())
    }

    /// Parses a magnet URI and constructs a `Magnet` instance.
    pub fn from_str(uri: &str) -> MagnetResult {
        let uri = Url::parse(uri).map_err(|e| MagnetError::Parse(e.to_string()))?;
        let query = uri.query_pairs();
        let mut builder = MagnetBuilder::builder();

        for (key, value) in query {
            let key = key.to_lowercase();

            match key.as_str() {
                "xt" => {
                    builder.exact_topic(value);
                }
                "dn" => {
                    builder.display_name(value);
                }
                "xl" => {
                    builder.exact_length(u64::from_str(value.as_ref()).map_err(|_| {
                        trace!("Value {} is not a valid number", value);
                        MagnetError::Parse("xl is invalid".to_string())
                    })?);
                }
                "tr" => {
                    builder.address_tracker(value);
                }
                "ws" => {
                    builder.web_seed(value);
                }
                "as" => {
                    builder.acceptable_source(value);
                }
                "xs" => {
                    builder.exact_source(value);
                }
                "kt" => {
                    builder.keyword_topic(value);
                }
                "mt" => {
                    builder.manifest_topic(value);
                }
                "so" => {
                    builder.select_only(value);
                }
                "x.pe" => {
                    builder.peer(value);
                }
                _ => warn!("Unsupported magnet parameter {}", key),
            }
        }

        builder.build()
    }
}

/// A builder for constructing a `Magnet` struct.
#[derive(Debug, Clone, Default)]
pub struct MagnetBuilder {
    exact_topic: Option<String>,
    display_name: Option<String>,
    exact_length: Option<u64>,
    address_tracker: Vec<String>,
    web_seed: Vec<String>,
    acceptable_source: Vec<String>,
    exact_source: Option<String>,
    keyword_topic: Option<String>,
    manifest_topic: Option<String>,
    select_only: Option<String>,
    peer: Option<String>,
}

impl MagnetBuilder {
    /// Creates a new `MagnetBuilder` instance.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the exact topic for the magnet link.
    pub fn exact_topic<S>(&mut self, exact_topic: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.exact_topic = Some(exact_topic.into());
        self
    }

    /// Sets the display name for the magnet link.
    pub fn display_name<S>(&mut self, display_name: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.display_name = Some(display_name.into());
        self
    }

    /// Sets the exact length for the magnet link.
    pub fn exact_length(&mut self, exact_length: u64) -> &mut Self {
        self.exact_length = Some(exact_length);
        self
    }

    /// Adds an address tracker to the magnet link.
    pub fn address_tracker<S>(&mut self, address_tracker: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.address_tracker.push(address_tracker.into());
        self
    }

    /// Adds a web seed to the magnet link.
    pub fn web_seed<S>(&mut self, web_seed: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.web_seed.push(web_seed.into());
        self
    }

    /// Adds an acceptable source to the magnet link.
    pub fn acceptable_source<S>(&mut self, acceptable_source: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.acceptable_source.push(acceptable_source.into());
        self
    }

    /// Sets the exact source for the magnet link.
    pub fn exact_source<S>(&mut self, exact_source: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.exact_source = Some(exact_source.into());
        self
    }

    /// Sets the keyword topic for the magnet link.
    pub fn keyword_topic<S>(&mut self, keyword_topic: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.keyword_topic = Some(keyword_topic.into());
        self
    }

    /// Sets the manifest topic for the magnet link.
    pub fn manifest_topic<S>(&mut self, manifest_topic: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.manifest_topic = Some(manifest_topic.into());
        self
    }

    /// Sets the select only for the magnet link.
    pub fn select_only<S>(&mut self, select_only: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.select_only = Some(select_only.into());
        self
    }

    /// Sets the peer for the magnet link.
    pub fn peer<S>(&mut self, peer: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.peer = Some(peer.into());
        self
    }

    /// Builds a `Magnet` instance from the builder's configuration.
    ///
    /// # Returns
    ///
    /// - `Ok(Magnet)`: A `Magnet` instance with the specified configuration.
    /// - `Err(MagnetError::InvalidUri)`: If the exact topic is not set, indicating an invalid magnet link.
    pub fn build(self) -> MagnetResult {
        if let Some(exact_topic) = self.exact_topic {
            Ok(Magnet {
                exact_topic,
                display_name: self.display_name,
                exact_length: self.exact_length,
                address_tracker: self.address_tracker,
                web_seed: self.web_seed,
                acceptable_source: self.acceptable_source,
                exact_source: self.exact_source,
                keyword_topic: self.keyword_topic,
                manifest_topic: self.manifest_topic,
                select_only: self.select_only,
                peer: self.peer,
            })
        } else {
            Err(MagnetError::InvalidUri)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_dn() {
        init_logger();
        let display_name = "Example File Name";
        let magnet = Magnet {
            exact_topic: "urn:btih:6b0cd35c4a6b724".to_string(),
            display_name: Some(display_name.to_string()),
            exact_length: Some(8455000),
            address_tracker: vec!["http://tracker.example.com:12345/announce".to_string()],
            web_seed: vec![],
            acceptable_source: vec![],
            exact_source: None,
            keyword_topic: None,
            manifest_topic: None,
            select_only: None,
            peer: None,
        };

        let result = magnet.dn();

        assert_eq!(Some(display_name), result)
    }

    #[test]
    fn test_from_str() {
        init_logger();
        let xt = "urn:btih:6b0cd35c4a6b7240b93d1e159f8c82b841d83a7a";
        let magnet_uri = format!("magnet:?xt={}&dn=Example%20File%20Name&tr=http%3A%2F%2Ftracker.example.com%3A12345%2Fannounce&xl=1234567890&sf=Example%20Folder", xt);
        let expected_result = Magnet {
            exact_topic: xt.to_string(),
            display_name: Some("Example File Name".to_string()),
            exact_length: Some(1234567890),
            address_tracker: vec!["http://tracker.example.com:12345/announce".to_string()],
            web_seed: vec![],
            acceptable_source: vec![],
            exact_source: None,
            keyword_topic: None,
            manifest_topic: None,
            select_only: None,
            peer: None,
        };

        let result = Magnet::from_str(magnet_uri.as_str());

        if let Ok(magnet) = result {
            assert_eq!(expected_result, magnet);
        } else {
            assert!(
                false,
                "expected a magnet to have been returned, got {:?} instead",
                result
            );
        }
    }
}
