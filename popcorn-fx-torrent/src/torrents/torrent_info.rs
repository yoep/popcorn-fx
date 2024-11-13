use log::{trace, warn};
use popcorn_fx_core::core::torrents::magnet::Magnet;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::str::FromStr;
use url::Url;

use crate::torrents::errors::{Result, TorrentError};
use crate::torrents::info_hash::InfoHash;

const VALIDATION_ERR_MISSING_METADATA_FIELDS: &str = "info or info_hash must be set";

/// Represents a list of URLs, which can be single, multiple, or ignored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UrlList {
    Single(String),
    Multiple(Vec<String>),
    Ignore(Vec<Vec<String>>),
    Ignore2(Vec<i64>),
}

/// Represents a web seed, also known as a URL seed or HTTP seed.
/// It's essentially a URL with some state associated with it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebSeed {
    UrlSeed(String),
    HttpSeed(String),
}

/// Metadata for a file within a torrent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentFileMeta {
    /// Length of the file in bytes.
    pub length: u64,
    /// Path to the file.
    pub path: Vec<String>,
    /// The utf-8 representation path to the file
    #[serde(
        rename = "path.utf-8",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub path_utf8: Option<Vec<String>>,
    /// MD5 checksum of the file, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5sum: Option<String>,
    /// When present the characters each represent a file attribute. l = symlink, x = executable, h = hidden, p = padding file.
    /// See BEP47
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attr: Option<String>,
    /// Path of the symlink target relative to the torrent root directory.
    /// See BEP47
    #[serde(
        rename = "symlink path",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub symlink_path: Option<Vec<String>>,
    /// The SHA1 digest calculated over the contents of the file itself, without any additional padding. Can be used to aid file deduplication.
    /// See BEP47
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
}

/// Information about a torrent file, which can be a single file or multiple files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TorrentInfoFile {
    Single {
        /// Length of the file in bytes.
        length: u64,
        /// MD5 checksum of the file, if available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        md5sum: Option<String>,
        /// When present the characters each represent a file attribute. l = symlink, x = executable, h = hidden, p = padding file.
        /// See BEP47
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attr: Option<String>,
        /// Path of the symlink target relative to the torrent root directory.
        /// See BEP47
        #[serde(
            rename = "symlink path",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        symlink_path: Option<Vec<String>>,
        /// The SHA1 digest calculated over the contents of the file itself, without any additional padding. Can be used to aid file deduplication.
        /// See BEP47
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha1: Option<String>,
    },
    Multiple {
        /// List of files within the directory.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        files: Vec<TorrentFileMeta>,
    },
}

/// Metadata of a torrent, including pieces, piece length, file info, etc.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentMetadata {
    /// Length of each piece in bytes.
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    /// Pieces of the torrent.
    /// Can be empty when the torrent only supports v2, see BEP52.
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Vec::is_empty")]
    pub pieces: Vec<u8>,
    /// Name of the torrent.
    /// This either represents the name of the file or the name of the directory.
    pub name: String,
    /// Name of the torrent in UTF-8 format.
    #[serde(
        rename = "name.utf-8",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name_utf8: Option<String>,
    /// Flag indicating if the torrent is private, see BEP27.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// An integer value, set to 2 to indicate compatibility with the current revision of BEP52. Version 1 is not assigned to avoid confusion with BEP3.
    #[serde(
        rename = "meta version",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub meta_version: Option<u64>,
    /// Information about the torrent files.
    #[serde(flatten)]
    pub files: TorrentInfoFile,
}

impl TorrentMetadata {
    /// Converts the pieces of the torrent into a vector of SHA1 hashes.
    ///
    /// # Returns
    ///
    /// A vector containing the SHA1 hashes of each piece.
    pub fn sha1_pieces(&self) -> Vec<[u8; 20]> {
        self.pieces
            .as_slice()
            .chunks_exact(20)
            .map(|e| e.try_into().unwrap())
            .collect()
    }

    /// Converts the pieces of the torrent into a vector of SHA256 hashes.
    ///
    /// # Returns
    ///
    /// A vector containing the SHA256 hashes of each piece.
    pub fn sha256_pieces(&self) -> Vec<[u8; 64]> {
        self.pieces
            .as_slice()
            .chunks_exact(64)
            .map(|e| e.try_into().unwrap())
            .collect()
    }

    /// Get the total file size of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the total file size of the torrent in bytes.
    pub fn total_size(&self) -> usize {
        match &self.files {
            TorrentInfoFile::Single { length, .. } => length.clone() as usize,
            TorrentInfoFile::Multiple { files, .. } => {
                files.iter().map(|f| f.length as usize).sum()
            }
        }
    }

    /// Get the total number of files in the torrent.
    ///
    /// # Returns
    ///
    /// Returns the total number of files in the torrent.
    pub fn total_files(&self) -> usize {
        match &self.files {
            TorrentInfoFile::Single { .. } => 1,
            TorrentInfoFile::Multiple { files, .. } => files.len(),
        }
    }

    /// Get the files of the torrent.
    ///
    /// # Returns
    ///
    /// Returns an array of the files of the torrent.
    pub fn files(&self) -> Vec<TorrentFileMeta> {
        match &self.files {
            TorrentInfoFile::Single {
                length,
                md5sum,
                attr,
                symlink_path,
                sha1,
            } => vec![TorrentFileMeta {
                length: *length,
                md5sum: md5sum.clone(),
                path: vec![self.name.clone()],
                path_utf8: None,
                attr: attr.clone(),
                symlink_path: symlink_path.clone(),
                sha1: sha1.clone(),
            }],
            TorrentInfoFile::Multiple { files, .. } => files.clone(),
        }
    }

    /// Calculate the info hash of this metadata.
    ///
    /// # Returns
    ///
    /// Returns the calculated info hash, or returns an error when the info hash could not be calculated.
    pub fn info_hash(&self) -> Result<InfoHash> {
        let metadata_bytes = serde_bencode::to_bytes(&self)?;

        Ok(InfoHash::from(metadata_bytes))
    }
}

impl Debug for TorrentMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentMetadata")
            .field("pieces", &self.pieces.len())
            .field("piece_length", &self.piece_length)
            .field("private", &self.private)
            .field("files", &self.files)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct TorrentMetadataBuilder {
    pieces: Option<Vec<u8>>,
    piece_length: Option<u64>,
    name: Option<String>,
    name_utf8: Option<String>,
    private: Option<i64>,
    source: Option<String>,
    files: Option<TorrentInfoFile>,
}

impl TorrentMetadataBuilder {
    pub fn builder() -> TorrentMetadataBuilder {
        TorrentMetadataBuilder::default()
    }

    pub fn pieces(mut self, pieces: Vec<u8>) -> Self {
        self.pieces = Some(pieces);
        self
    }

    pub fn piece_length(mut self, piece_length: u64) -> Self {
        self.piece_length = Some(piece_length);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn name_utf8(mut self, name_utf8: String) -> Self {
        self.name_utf8 = Some(name_utf8);
        self
    }

    pub fn private(mut self, private: i64) -> Self {
        self.private = Some(private);
        self
    }

    pub fn source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn files(mut self, files: TorrentInfoFile) -> Self {
        self.files = Some(files);
        self
    }

    pub fn build(self) -> TorrentMetadata {
        TorrentMetadata {
            pieces: self.pieces.unwrap_or_default(),
            piece_length: self.piece_length.unwrap_or_default(),
            name: self.name.expect("expected name to be set"),
            name_utf8: self.name_utf8,
            private: self.private,
            source: self.source,
            meta_version: None,
            files: self.files.expect("expected files to be set"),
        }
    }
}

/// Detailed information from a .torrent file, akin to `add_torrent_params` in `libtorrent`.
///
/// This struct facilitates adding a new [crate::torrents::Torrent] to a session.
///
/// # Examples
///
/// ```
/// use std::convert::TryInto;
/// use crate::popcorn_fx_torrent::torrents::{TorrentInfo, TorrentError, Result};
///
/// fn parse_torrent_data(data: &[u8]) -> Result<TorrentInfo> {
///     let torrent_info: TorrentInfo = data.try_into()?;
///     Ok(torrent_info)
/// }
/// ```
///
/// # Remarks
///
/// Either the `info` field or `info_hash` field must be set for the struct to be valid.
/// If only the info-hash is specified, the torrent file will be downloaded from peers,
/// requiring support for the metadata extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentInfo {
    /// The display name of the torrent.
    #[serde(skip)]
    name: Option<String>,
    /// URL of the tracker for the torrent.
    pub announce: Option<String>,
    /// Metadata specific to the torrent, equivalent to `ti` field in `add_torrent_params`.
    pub info: Option<TorrentMetadata>,
    /// A dictionary of strings. For each file in the file tree that is larger than the piece size it contains one string value.
    #[serde(rename = "piece layers")]
    pub piece_layers: Option<HashMap<String, String>>,
    #[serde(rename = "magnet-uri")]
    pub magnet_uri: Option<String>,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    /// Optional creation date of the torrent.
    #[serde(rename = "creation date")]
    pub creation_date: Option<u64>,
    /// Optional comment associated with the torrent.
    pub comment: Option<String>,
    /// Optional creator of the torrent.
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    /// Optional encoding format of the torrent.
    pub encoding: Option<String>,
    /// Optional list of URLs associated with the torrent.
    #[serde(rename = "url-list")]
    pub url_list: Option<UrlList>,
    /// When metadata (.torrent file) isn't available, this uniquely identifies the torrent
    /// and validates the info-dict when received from the swarm.
    #[serde(skip)]
    pub info_hash: InfoHash,
    /// The web seeds list of the torrent.
    #[serde(default)]
    pub web_seeds: Vec<WebSeed>,
}

impl TorrentInfo {
    pub fn builder() -> TorrentInfoBuilder {
        TorrentInfoBuilder::builder()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|e| e.as_str())
    }

    pub fn update_name<T: Into<String>>(&mut self, name: T) {
        self.name = Some(name.into());
    }

    /// Validate the metadata contained within this struct.
    ///
    /// # Returns
    ///
    /// Returns nothing when the metadata is valid, else the validation error.
    pub fn validate(&self) -> Result<()> {
        trace!("Validating {:?}", self);
        // if self.info.is_none() && self.info_hash.is_none() {
        //     return Err(TorrentError::InvalidMetadata(
        //         VALIDATION_ERR_MISSING_METADATA_FIELDS.to_string(),
        //     ));
        // }

        Ok(())
    }

    /// Get the trackers in a tiered order for this torrent.
    /// It makes use of a [BTreeMap] to sort the trackers by tier index.
    ///
    /// The key is the index of the tier and the value is the list of trackers in that tier.
    ///
    /// # Returns
    ///
    /// Returns an ordered list, by tier, of trackers for this torrent.
    pub fn tiered_trackers(&self) -> BTreeMap<u8, Vec<Url>> {
        let mut tiered_trackers = BTreeMap::<u8, Vec<Url>>::new();
        let mut tier = 0u8;

        // add the announce tracker info to the tiered trackers if present
        if let Some(announce) = self.announce.as_ref() {
            match Url::parse(announce) {
                Ok(url) => {
                    tiered_trackers.insert(tier, vec![url]);
                }
                Err(e) => warn!("Failed to parse announce url: {}", e),
            }
        }

        // loop over the announce list and add the trackers to the tiered trackers
        // based on the order they appear in the announce list
        if let Some(announce_list) = self.announce_list.as_ref() {
            for trackers in announce_list {
                for tracker in trackers {
                    match Url::parse(tracker) {
                        Ok(url) => {
                            tiered_trackers
                                .entry(tier)
                                .or_insert_with(Vec::new)
                                .push(url);
                        }
                        Err(e) => warn!("Failed to parse announce url: {}", e),
                    }
                }

                tier += 1;
            }
        }

        tiered_trackers
    }

    /// Calculate the info hash from the metadata of the torrent.
    /// This can only be done when the metadata is available.
    ///
    /// # Returns
    ///
    /// Returns the calculated info hash of the metadata.
    pub fn calculate_info_hash(&self) -> Result<InfoHash> {
        match &self.info {
            Some(metadata) => metadata.info_hash(),
            None => Err(TorrentError::InvalidMetadata(
                "info dictionary is empty".to_string(),
            )),
        }
    }
}

impl TryFrom<&[u8]> for TorrentInfo {
    type Error = TorrentError;

    /// Attempts to parse torrent metadata from the given bytes.
    /// The bytes should be encoded in the `bencode` format.
    ///
    /// # Arguments
    ///
    /// * `value` - Byte slice containing the torrent metadata.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed TorrentInfo if successful,
    /// or a TorrentError if parsing fails.
    fn try_from(value: &[u8]) -> Result<Self> {
        let mut torrent = serde_bencode::from_bytes::<Self>(value)
            .map_err(|e| TorrentError::TorrentParse(e.to_string()))?;
        // calculate the info hash from the info dict
        let info_bytes = serde_bencode::to_bytes(&torrent.info)
            .map_err(|e| TorrentError::TorrentParse(e.to_string()))?;

        torrent.info_hash = InfoHash::from(info_bytes);
        Ok(torrent)
    }
}

impl TryFrom<Magnet> for TorrentInfo {
    type Error = TorrentError;

    fn try_from(value: Magnet) -> Result<Self> {
        let mut builder = TorrentInfoBuilder::builder();

        // extract the display name
        if let Some(name) = value.display_name.as_ref() {
            builder = builder.name(name);
        }
        // extract the trackers
        for tracker in value.trackers() {
            builder = builder.tracker(tracker);
        }
        // extract the info hash
        builder = builder.info_hash(InfoHash::from_str(value.xt())?);

        Ok(builder.build())
    }
}

/// A builder for constructing a `TorrentInfo` struct with optional fields.
///
/// The `TorrentInfoBuilder` allows for the creation of a `TorrentInfo` instance with flexible configuration of its fields.
#[derive(Debug, Default)]
pub struct TorrentInfoBuilder {
    name: Option<String>,
    announce: Option<String>,
    info: Option<TorrentMetadata>,
    announce_list: Option<Vec<Vec<String>>>,
    creation_date: Option<u64>,
    comment: Option<String>,
    created_by: Option<String>,
    encoding: Option<String>,
    url_list: Option<UrlList>,
    info_hash: Option<InfoHash>,
    piece_layers: Option<HashMap<String, String>>,
}

impl TorrentInfoBuilder {
    /// Creates a new `TorrentInfoBuilder` instance.
    ///
    /// # Returns
    ///
    /// A new instance of `TorrentInfoBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the name of the torrent.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the torrent.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the announce URL for the torrent.
    ///
    /// # Arguments
    ///
    /// * `announce` - The URL of the announce tracker.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn announce<T: Into<String>>(mut self, announce: T) -> Self {
        self.announce = Some(announce.into());
        self
    }

    /// Sets the metadata for the torrent.
    ///
    /// # Arguments
    ///
    /// * `info` - The `TorrentMetadata` for the torrent.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn info(mut self, info: TorrentMetadata) -> Self {
        self.info = Some(info);
        self
    }

    /// Sets the list of announce URLs for the torrent.
    ///
    /// # Arguments
    ///
    /// * `announce_list` - A list of lists of announce URLs.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn announce_list(mut self, announce_list: Vec<Vec<String>>) -> Self {
        self.announce_list = Some(announce_list);
        self
    }

    /// Adds a single tracker URL to the announce list.
    ///
    /// # Arguments
    ///
    /// * `tracker_uri` - The URL of the tracker to add.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn tracker<T: Into<String>>(mut self, tracker_uri: T) -> Self {
        let mut announce_list = self.announce_list.unwrap_or_else(Vec::new);
        announce_list.push(vec![tracker_uri.into()]);
        self.announce_list = Some(announce_list);
        self
    }

    /// Sets the creation date of the torrent.
    ///
    /// # Arguments
    ///
    /// * `creation_date` - The creation date as a Unix timestamp.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn creation_date(mut self, creation_date: u64) -> Self {
        self.creation_date = Some(creation_date);
        self
    }

    /// Sets a comment for the torrent.
    ///
    /// # Arguments
    ///
    /// * `comment` - A comment associated with the torrent.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn comment<T: Into<String>>(mut self, comment: T) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Sets the creator of the torrent.
    ///
    /// # Arguments
    ///
    /// * `created_by` - The name of the creator.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn created_by(mut self, created_by: String) -> Self {
        self.created_by = Some(created_by);
        self
    }

    /// Sets the encoding format of the torrent.
    ///
    /// # Arguments
    ///
    /// * `encoding` - The encoding format.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn encoding(mut self, encoding: String) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Sets the list of URLs associated with the torrent.
    ///
    /// # Arguments
    ///
    /// * `url_list` - The `UrlList` containing associated URLs.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn url_list(mut self, url_list: UrlList) -> Self {
        self.url_list = Some(url_list);
        self
    }

    /// Sets the info hash for the torrent.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The `InfoHash` for the torrent.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn info_hash(mut self, info_hash: InfoHash) -> Self {
        self.info_hash = Some(info_hash);
        self
    }

    /// Sets the piece layers for the torrent.
    ///
    /// # Arguments
    ///
    /// * `piece_layers` - A `HashMap` of piece layers.
    ///
    /// # Returns
    ///
    /// The updated `TorrentInfoBuilder` instance.
    pub fn piece_layers(mut self, piece_layers: HashMap<String, String>) -> Self {
        self.piece_layers = Some(piece_layers);
        self
    }

    /// Constructs and returns a `TorrentInfo` instance based on the configured builder settings.
    ///
    /// # Returns
    ///
    /// A `TorrentInfo` instance containing the fields set in the builder.
    /// Panics if the `info_hash` field is not set.
    pub fn build(self) -> TorrentInfo {
        TorrentInfo {
            name: self.name,
            announce: self.announce,
            info: self.info,
            piece_layers: self.piece_layers,
            magnet_uri: None,
            announce_list: self.announce_list,
            creation_date: self.creation_date,
            comment: self.comment,
            created_by: self.created_by,
            encoding: self.encoding,
            url_list: self.url_list,
            info_hash: self
                .info_hash
                .expect("expected the info hash to be present"),
            web_seeds: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use log::trace;
    use std::str::FromStr;

    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_torrent_info_validate() {
        init_logger();

        let info = TorrentInfoBuilder::builder()
            .info_hash(
                InfoHash::from_str(
                    "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
                )
                .unwrap(),
            )
            .build();
        let result = info.validate();
        // assert_eq!(
        //     Err(TorrentError::InvalidMetadata(
        //         VALIDATION_ERR_MISSING_METADATA_FIELDS.to_string()
        //     )),
        //     result
        // );

        let info = TorrentInfoBuilder::builder()
            .info(TorrentMetadata {
                pieces: vec![],
                piece_length: 20,
                private: None,
                name: "FooBarFile".to_string(),
                name_utf8: None,
                files: TorrentInfoFile::Single {
                    length: 145600,
                    md5sum: None,
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                },
                source: None,
                meta_version: None,
            })
            .info_hash(
                InfoHash::from_str(
                    "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
                )
                .unwrap(),
            )
            .build();
        let result = info.validate();
        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_torrent_info_tiered_trackers() {
        init_logger();
        let announce = "udp://example.tracker.org:6969/announce";
        let info = TorrentInfoBuilder::builder()
            .announce(announce)
            .announce_list(vec![
                vec![
                    "http://foobar.tracker.org:6970/announce".to_string(),
                    "http://foobar.tracker.org:6971/announce".to_string(),
                ],
                vec!["udp://first_tier.tracker.org:6900/announce".to_string()],
            ])
            .info_hash(
                InfoHash::from_str(
                    "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
                )
                .unwrap(),
            )
            .build();
        let expected_result: BTreeMap<u8, Vec<Url>> = vec![
            (
                0u8,
                vec![
                    Url::parse(announce).unwrap(),
                    Url::parse("http://foobar.tracker.org:6970/announce").unwrap(),
                    Url::parse("http://foobar.tracker.org:6971/announce").unwrap(),
                ],
            ),
            (
                1u8,
                vec![Url::parse("udp://first_tier.tracker.org:6900/announce").unwrap()],
            ),
        ]
        .into_iter()
        .collect();

        let result = info.tiered_trackers();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_info_try_from_bytes() {
        init_logger();
        let announce = "http://bttracker.debian.org:6969/announce";
        let data = read_test_file_to_bytes("debian.torrent");

        let info = TorrentInfo::try_from(data.as_slice()).unwrap();

        trace!("{:?}", info);
        assert_eq!(announce, info.announce.expect("expected announce").as_str());
    }

    #[test]
    fn test_torrent_info_try_from_magnet() {
        init_logger();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();

        let result = TorrentInfo::try_from(magnet).unwrap();

        let info_hash = &result.info_hash;
        assert_eq!(
            "eadaf0efea39406914414d359e0ea16416409bd7",
            hex::encode(info_hash.hash_v1().unwrap())
        );
        assert_eq!(Some("debian-12.4.0-amd64-DVD-1.iso"), result.name());
    }

    #[test]
    fn test_torrent_info_create_info_hash() {
        init_logger();
        let torrent = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentInfo::try_from(torrent.as_slice()).unwrap();

        let result = info.calculate_info_hash().unwrap();

        assert_eq!(info.info_hash, result);
    }
}
