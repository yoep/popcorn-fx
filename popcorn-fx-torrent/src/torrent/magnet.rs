use crate::torrent::{MagnetError, MagnetResult};
use itertools::Itertools;
use log::{trace, warn};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

/// The scheme identifier of a magnet
const MAGNET_SCHEME: &str = "magnet";

/// Represents a Magnet link.
#[derive(Debug, Clone, PartialEq)]
pub struct Magnet {
    pub exact_topics: Vec<String>,
    pub display_name: Option<String>,
    pub exact_length: Option<u64>,
    pub tracker_addresses: Vec<String>,
    pub web_seeds: Vec<String>,
    pub acceptable_source: Vec<String>,
    pub exact_source: Option<String>,
    pub keyword_topic: Option<String>,
    pub manifest_topic: Option<String>,
    pub select_only: Option<String>,
    pub peer: Option<String>,
}

impl Magnet {
    /// Get a builder instance for a magnet.
    pub fn builder() -> MagnetBuilder {
        MagnetBuilder::builder()
    }

    /// Gets the 'xt' (exact topic) value from the magnet link.
    pub fn xt(&self) -> Vec<&str> {
        self.exact_topics.iter().map(|e| e.as_str()).collect()
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
        self.tracker_addresses.as_slice()
    }

    /// Gets the 'tr' tracker values from the magnet link.
    pub fn trackers(&self) -> &[String] {
        self.tr()
    }

    /// Gets the 'ws' (web seed) values from the magnet link.
    pub fn ws(&self) -> &[String] {
        self.web_seeds.as_slice()
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

    /// Retrieve the select only indexes from the magnet link, if present.
    pub fn select_only(&self) -> MagnetResult<Option<Vec<u32>>> {
        if let Some(so) = self.so() {
            let mut indexes = Vec::new();
            let sections = so.split(",");

            for section in sections {
                if let Some((start, end)) = section.split_once("-") {
                    let start = start
                        .parse::<u32>()
                        .map_err(|e| MagnetError::InvalidValue(e.to_string()))?;
                    let end = end
                        .parse::<u32>()
                        .map_err(|e| MagnetError::InvalidValue(e.to_string()))?;
                    for i in start..=end {
                        indexes.push(i);
                    }
                } else {
                    indexes.push(
                        section
                            .parse::<u32>()
                            .map_err(|e| MagnetError::InvalidValue(e.to_string()))?,
                    );
                }
            }

            return Ok(Some(indexes));
        }

        Ok(None)
    }

    /// Gets the 'x.pe' (peer) value from the magnet link, if present.
    pub fn x_pe(&self) -> Option<&str> {
        self.peer.as_ref().map(|e| e.as_str())
    }

    /// Get the magnet as an url.
    pub fn as_url(&self) -> Url {
        let topics = self
            .exact_topics
            .iter()
            .map(|topic| format!("xt={}", topic))
            .join("&");
        let mut url = Url::from_str(format!("{}:?{}", MAGNET_SCHEME, topics).as_str())
            .expect("expected the magnet scheme to be valid");
        let mut params = url.query_pairs_mut();

        if let Some(dn) = self.display_name.as_ref() {
            params.append_pair("dn", dn.as_str());
        }
        if let Some(xl) = self.exact_length.as_ref() {
            params.append_pair("xl", xl.to_string().as_str());
        }
        for tr in self.tracker_addresses.iter() {
            params.append_pair("tr", tr.as_str());
        }
        for ws in self.web_seeds.iter() {
            params.append_pair("ws", ws.as_str());
        }
        for mas in self.acceptable_source.iter() {
            params.append_pair("as", mas.as_str());
        }
        if let Some(xs) = self.exact_source.as_ref() {
            params.append_pair("xs", xs.as_str());
        }
        if let Some(kt) = self.keyword_topic.as_ref() {
            params.append_pair("kt", kt.as_str());
        }
        if let Some(mt) = self.manifest_topic.as_ref() {
            params.append_pair("mt", mt.as_str());
        }
        if let Some(so) = self.select_only.as_ref() {
            params.append_pair("so", so.as_str());
        }
        if let Some(xpe) = self.peer.as_ref() {
            params.append_pair("x.pe", xpe.as_str());
        }

        params.finish();
        drop(params);

        url
    }

    /// Check if the given uri contains an encoded `&` as `&amp`.
    fn contains_encoded_ampersand(uri: &str) -> bool {
        uri.contains("&amp;")
    }
}

impl FromStr for Magnet {
    type Err = MagnetError;

    fn from_str(uri: &str) -> MagnetResult<Self> {
        let mut uri = uri.to_string();

        // replace any encoded ampersands
        if Self::contains_encoded_ampersand(uri.as_str()) {
            uri = uri.replace("&amp;", "&");
        }

        let uri = Url::parse(&uri).map_err(|e| MagnetError::Parse(e.to_string()))?;
        let scheme = uri.scheme();

        // verify if the given scheme is supported
        if scheme != MAGNET_SCHEME {
            return Err(MagnetError::UnsupportedScheme(scheme.to_string()));
        }

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

impl Display for Magnet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let url = self.as_url();
        write!(f, "{}", url)
    }
}

/// A builder for constructing a `Magnet` struct.
#[derive(Debug, Clone, Default)]
pub struct MagnetBuilder {
    exact_topics: Option<Vec<String>>,
    display_name: Option<String>,
    exact_length: Option<u64>,
    address_tracker: Option<Vec<String>>,
    web_seed: Option<Vec<String>>,
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

    /// Add the given exact topic to the magnet builder.
    pub fn exact_topic<S>(&mut self, exact_topic: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.exact_topics
            .get_or_insert(Vec::new())
            .push(exact_topic.into());
        self
    }

    /// Set the exact topics for the magnet builder.
    pub fn exact_topics(&mut self, exact_topics: Vec<String>) -> &mut Self {
        self.exact_topics
            .get_or_insert(Vec::new())
            .extend(exact_topics);
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

    /// Add a tracker address to the magnet link.
    pub fn address_tracker<S>(&mut self, address_tracker: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.address_tracker
            .get_or_insert(Vec::new())
            .push(address_tracker.into());
        self
    }

    /// Set the tracker addresses of the magnet link.
    pub fn address_trackers(&mut self, address_trackers: Vec<String>) -> &mut Self {
        self.address_tracker
            .get_or_insert(Vec::new())
            .extend(address_trackers);
        self
    }

    /// Add a web seed to the magnet link.
    pub fn web_seed<S>(&mut self, web_seed: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.web_seed
            .get_or_insert(Vec::new())
            .push(web_seed.into());
        self
    }

    /// Set the web seed addresses of the magnet link.
    pub fn web_seeds(&mut self, web_seeds: Vec<String>) -> &mut Self {
        self.web_seed.get_or_insert(Vec::new()).extend(web_seeds);
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
    pub fn build(self) -> MagnetResult<Magnet> {
        if let Some(exact_topic) = self.exact_topics {
            let tracker_addresses = self.address_tracker.unwrap_or(Vec::new());
            let web_seeds = self.web_seed.unwrap_or(Vec::new());

            Ok(Magnet {
                exact_topics: exact_topic,
                display_name: self.display_name,
                exact_length: self.exact_length,
                tracker_addresses,
                web_seeds,
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
    use super::*;
    use popcorn_fx_core::init_logger;

    #[test]
    fn test_magnet_from_str() {
        init_logger!();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let expected_result = create_expected_magnet_from_str();

        let result = Magnet::from_str(uri).unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_magnet_from_str_encoded_url() {
        init_logger!();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&amp;dn=debian-12.4.0-amd64-DVD-1.iso&amp;tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&amp;tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&amp;tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&amp;tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&amp;tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&amp;tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&amp;tr=udp%3A%2F%2Fexodus.desync.com%3A6969&amp;tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let expected_result = create_expected_magnet_from_str();

        let result = Magnet::from_str(uri).unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_magnet_from_str_invalid_scheme() {
        init_logger!();
        let uri = "custom:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA1641640007";

        let result = Magnet::from_str(uri);

        assert_eq!(
            Err(MagnetError::UnsupportedScheme("custom".to_string())),
            result
        );
    }

    #[test]
    fn test_magnet_xt() {
        init_logger!();
        let expected_result = vec!["urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7".to_string()];
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";

        let magnet = Magnet::from_str(uri).unwrap();

        let result = magnet.xt();
        assert_eq!(expected_result, result, "expected the exact topic to match");
    }

    #[test]
    fn test_magnet_dn() {
        init_logger!();
        let display_name = "Example File Name";
        let magnet = Magnet {
            exact_topics: vec!["urn:btih:6b0cd35c4a6b724".to_string()],
            display_name: Some(display_name.to_string()),
            exact_length: Some(8455000),
            tracker_addresses: vec!["http://tracker.example.com:12345/announce".to_string()],
            web_seeds: vec![],
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
    fn test_magnet_tr() {
        init_logger!();
        let expected_result = vec![
            "http://tracker1.example.com:12345/announce".to_string(),
            "http://tracker2.example.com:23456/announce".to_string(),
        ];
        let magnet = Magnet {
            exact_topics: vec!["urn:btih:6b0cd35c4a6b724".to_string()],
            display_name: None,
            exact_length: Some(8455000),
            tracker_addresses: expected_result.clone(),
            web_seeds: vec![],
            acceptable_source: vec![],
            exact_source: None,
            keyword_topic: None,
            manifest_topic: None,
            select_only: None,
            peer: None,
        };

        let result = magnet.tr();

        assert_eq!(expected_result.as_slice(), result);
        assert_eq!(expected_result.as_slice(), magnet.trackers());
    }

    #[test]
    fn test_magnet_select_only() {
        let expected_result: Vec<u32> = vec![0, 2, 4, 6, 7, 8];
        let magnet = Magnet {
            exact_topics: vec!["urn:btih:6b0cd35c4a6b724".to_string()],
            display_name: None,
            exact_length: None,
            tracker_addresses: vec![],
            web_seeds: vec![],
            acceptable_source: vec![],
            exact_source: None,
            keyword_topic: None,
            manifest_topic: None,
            select_only: Some("0,2,4,6-8".to_string()),
            peer: None,
        };

        let result = magnet
            .select_only()
            .expect("expected the so to be valid")
            .expect("expected the so to be present");

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_magnet_as_uri() {
        init_logger!();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let expected_result = Url::from_str(uri).unwrap();
        let magnet = Magnet::from_str(uri).unwrap();

        let result = magnet.as_url();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_magnet_to_string() {
        init_logger!();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();

        let result = magnet.to_string();

        assert_eq!(uri, result);
    }

    fn create_expected_magnet_from_str() -> Magnet {
        Magnet {
            exact_topics: vec!["urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7".to_string()],
            display_name: Some("debian-12.4.0-amd64-DVD-1.iso".to_string()),
            exact_length: None,
            tracker_addresses: vec![
                "udp://tracker.opentrackr.org:1337".to_string(),
                "udp://open.stealth.si:80/announce".to_string(),
                "udp://tracker.torrent.eu.org:451/announce".to_string(),
                "udp://tracker.bittor.pw:1337/announce".to_string(),
                "udp://public.popcorn-tracker.org:6969/announce".to_string(),
                "udp://tracker.dler.org:6969/announce".to_string(),
                "udp://exodus.desync.com:6969".to_string(),
                "udp://open.demonii.com:1337/announce".to_string(),
            ],
            web_seeds: vec![],
            acceptable_source: vec![],
            exact_source: None,
            keyword_topic: None,
            manifest_topic: None,
            select_only: None,
            peer: None,
        }
    }
}
