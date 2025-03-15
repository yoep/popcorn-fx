use crate::torrent::errors::{Result, TorrentError};
use crate::torrent::info_hash::InfoHash;
use crate::torrent::{errors, Magnet, Sha1Hash, Sha256Hash};
use bitmask_enum::bitmask;
use log::{debug, warn};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

/// Represents a list of URLs, which can be single, multiple, or ignored.
pub type UrlList = Vec<String>;

/// Represents the piece layers as a dictionary of strings.
pub type PieceLayers = HashMap<String, String>;

/// The file attributes of a torrent file.
/// See BEP47 for more info.
#[bitmask(u8)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum FileAttributeFlags {
    Symlink = 0b0001,
    Executable = 0b0010,
    Hidden = 0b0100,
    PaddingFile = 0b1000,
}

impl Default for FileAttributeFlags {
    fn default() -> Self {
        FileAttributeFlags::none()
    }
}

impl Display for FileAttributeFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        if self.contains(FileAttributeFlags::Symlink) {
            flags.push('l');
        }
        if self.contains(FileAttributeFlags::Executable) {
            flags.push('x');
        }
        if self.contains(FileAttributeFlags::Hidden) {
            flags.push('h');
        }
        if self.contains(FileAttributeFlags::PaddingFile) {
            flags.push('p');
        }

        write!(f, "{}", flags.into_iter().collect::<String>())
    }
}

impl FromStr for FileAttributeFlags {
    type Err = TorrentError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let mut flags = FileAttributeFlags::none();

        for ch in value.to_lowercase().chars() {
            flags = match ch {
                'l' => flags | FileAttributeFlags::Symlink,
                'x' => flags | FileAttributeFlags::Executable,
                'h' => flags | FileAttributeFlags::Hidden,
                'p' => flags | FileAttributeFlags::PaddingFile,
                _ => {
                    return Err(TorrentError::TorrentParse(
                        "Invalid character in file attributes".to_string(),
                    ))
                }
            };
        }

        Ok(flags)
    }
}

impl Serialize for FileAttributeFlags {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for FileAttributeFlags {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserializer::deserialize_any(deserializer, FileAttributeFlagVisitor {})
    }
}

struct FileAttributeFlagVisitor;

impl<'de> Visitor<'de> for FileAttributeFlagVisitor {
    type Value = FileAttributeFlags;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a string")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        FileAttributeFlags::from_str(v).map_err(|e| Error::custom(e))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        FileAttributeFlags::from_str(String::from_utf8_lossy(v).as_ref())
            .map_err(|e| Error::custom(e))
    }
}

/// The file info metadata information of a file within a torrent.
/// This information is specific to a single file inside the [TorrentMetadataInfo].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentFileInfo {
    /// Length of the file in bytes.
    pub length: u64,
    /// Path of the file within the torrent.
    /// This is never present in a single torrent file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<String>>,
    /// The utf-8 representation path to the file.
    /// This is never present in a single torrent file.
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
    pub attr: Option<FileAttributeFlags>,
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

impl TorrentFileInfo {
    /// Get the path to the file within the torrent.
    /// If the file information belongs to a [TorrentFiles::Single], the returned path will be empty.
    pub fn path(&self) -> PathBuf {
        let empty_vec = Vec::with_capacity(0);
        let segments = self
            .path_utf8
            .as_ref()
            .unwrap_or_else(|| self.path.as_ref().unwrap_or(&empty_vec));

        let mut path = PathBuf::new();
        for segment in segments {
            path.push(segment);
        }

        path
    }

    /// Get the segments of the path to the torrent file.
    /// If the file information belongs to a [TorrentFiles::Single], the returned path will be empty.
    ///
    /// # Returns
    ///
    /// Returns either the utf8 representation of the path or the normal path.
    pub fn path_segments(&self) -> Vec<String> {
        self.path_utf8
            .as_ref()
            .map_or_else(|| self.path.clone(), |e| Some(e.clone()))
            .unwrap_or(Vec::new())
    }

    /// Check if the file is a padding file (see BEP47).
    pub fn is_padding_file(&self) -> bool {
        self.attr
            .as_ref()
            .map(|e| e.contains(FileAttributeFlags::PaddingFile))
            .unwrap_or(false)
    }
}

/// The torrent files information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TorrentFiles {
    Single {
        #[serde(flatten)]
        file: TorrentFileInfo,
    },
    Multiple {
        /// List of files within the directory.
        files: Vec<TorrentFileInfo>,
    },
    FileTree {
        /// A tree of dictionaries where dictionary keys represent UTF-8 encoded path elements.
        /// See BEP52 for more info
        #[serde(rename = "file tree")]
        file_tree: FileTree,
    },
}

/// The file tree of a torrent.
/// These are the merkle roots of a v2 torrent.
#[derive(Debug, Clone, PartialEq)]
pub enum FileTree {
    File(FileNode),
    Dir(HashMap<String, FileTree>),
}

impl FileTree {
    /// Get the total size/length of all files in the tree.
    pub fn len(&self) -> usize {
        match self {
            FileTree::File(node) => node.length as usize,
            FileTree::Dir(nodes) => nodes.iter().map(|(_, value)| value.len()).sum(),
        }
    }

    /// Get the total amount of files in the file tree.
    pub fn total_files(&self) -> usize {
        match self {
            FileTree::File(_) => 1,
            FileTree::Dir(nodes) => nodes.iter().map(|(_, value)| value.total_files()).sum(),
        }
    }

    /// Get the files information of the file tree.
    pub fn files(&self) -> Vec<TorrentFileInfo> {
        self.create_files(Vec::new())
    }

    fn create_files(&self, path: Vec<String>) -> Vec<TorrentFileInfo> {
        match self {
            FileTree::Dir(nodes) => nodes
                .iter()
                .flat_map(|(key, value)| {
                    let mut new_path = path.clone();
                    new_path.push(key.clone());
                    value.create_files(new_path)
                })
                .collect(),
            FileTree::File(node) => {
                let path = if !path.is_empty() { Some(path) } else { None };

                vec![TorrentFileInfo {
                    length: node.length,
                    path: path.clone(),
                    path_utf8: path,
                    md5sum: None,
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                }]
            }
        }
    }
}

impl Serialize for FileTree {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_file_tree::serialize(&self, serializer)
    }
}

impl<'de> Deserialize<'de> for FileTree {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_file_tree::deserialize(deserializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileNode {
    /// Length of the file in bytes.
    pub length: u64,
    /// For non-empty files this is the root hash of a merkle tree with a branching factor of 2, constructed from 16KiB blocks of the file.
    #[serde(
        rename = "pieces root",
        default,
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub pieces_root: Option<Vec<u8>>,
}

/// Metadata of a torrent, including pieces, piece length, file info, etc.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentMetadataInfo {
    /// Length of each piece in bytes.
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    /// Pieces bytes of the torrent.
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
    /// An integer value, set to 2 to indicate compatibility with the current revision of BEP52.
    /// Version 1 is not assigned to avoid confusion with BEP3.
    #[serde(
        rename = "meta version",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub meta_version: Option<u64>,
    /// Information about the torrent files.
    #[serde(flatten)]
    pub files: TorrentFiles,
}

impl TorrentMetadataInfo {
    /// Get the name of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the utf8 representation of the name if available, else the name field.
    pub fn name(&self) -> String {
        self.name_utf8.clone().unwrap_or(self.name.clone())
    }

    /// Converts the pieces of the torrent into a vector of SHA1 hashes.
    ///
    /// # Returns
    ///
    /// A vector containing the SHA1 hashes of each piece.
    pub fn sha1_pieces(&self) -> Vec<Sha1Hash> {
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
    pub fn sha256_pieces(&self) -> Vec<Sha256Hash> {
        self.pieces
            .as_slice()
            .chunks_exact(32)
            .map(|e| e.try_into().unwrap())
            .collect()
    }

    /// Get the total file length/size of the torrent.
    ///
    /// # Returns
    ///
    /// Returns the total file size of the torrent in bytes.
    pub fn len(&self) -> usize {
        match &self.files {
            TorrentFiles::Single { file } => file.length.clone() as usize,
            TorrentFiles::Multiple { files, .. } => files.iter().map(|f| f.length as usize).sum(),
            TorrentFiles::FileTree { file_tree } => file_tree.len(),
        }
    }

    /// Get the total number of files in the torrent.
    ///
    /// # Returns
    ///
    /// Returns the total number of files in the torrent.
    pub fn total_files(&self) -> usize {
        match &self.files {
            TorrentFiles::Single { .. } => 1,
            TorrentFiles::Multiple { files } => files.len(),
            TorrentFiles::FileTree { file_tree } => file_tree.total_files(),
        }
    }

    /// Get the files of the torrent.
    ///
    /// # Returns
    ///
    /// Returns an array of the files of the torrent.
    pub fn files(&self) -> Vec<TorrentFileInfo> {
        match &self.files {
            TorrentFiles::Single { file } => vec![file.clone()],
            TorrentFiles::Multiple { files } => files.clone(),
            TorrentFiles::FileTree { file_tree } => file_tree.files(),
        }
    }

    /// Calculate the info hash of this metadata.
    ///
    /// # Returns
    ///
    /// Returns the calculated info hash, or returns an error when the info hash could not be calculated.
    pub fn info_hash(&self) -> Result<InfoHash> {
        let metadata_bytes = serde_bencode::to_bytes(&self)?;
        let is_v2 = self.meta_version.filter(|e| *e == 2).is_some();

        if is_v2 {
            Ok(InfoHash::from_metadata_v2(metadata_bytes))
        } else {
            Ok(InfoHash::from_metadata_v1(metadata_bytes))
        }
    }

    /// Get the path of the given file within the torrent.
    /// This will calculate the torrent path based on the file path segments.
    ///
    /// # Returns
    ///
    /// It returns the torrent filepath of the given file.
    pub fn path(&self, file: &TorrentFileInfo) -> PathBuf {
        let mut path = PathBuf::new().join(self.name());
        for path_section in file.path_segments() {
            path = path.join(path_section);
        }
        path
    }
}

impl Debug for TorrentMetadataInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentMetadata")
            .field("piece_length", &self.piece_length)
            .field("pieces", &self.pieces.len())
            .field("name", &self.name)
            .field("name_utf8", &self.name_utf8)
            .field("private", &self.private)
            .field("source", &self.source)
            .field("meta_version", &self.meta_version)
            .field("files", &self.files)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct TorrentMetadataInfoBuilder {
    pieces: Option<Vec<u8>>,
    piece_length: Option<u64>,
    name: Option<String>,
    name_utf8: Option<String>,
    private: Option<i64>,
    source: Option<String>,
    files: Option<TorrentFiles>,
}

impl TorrentMetadataInfoBuilder {
    pub fn builder() -> TorrentMetadataInfoBuilder {
        TorrentMetadataInfoBuilder::default()
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

    pub fn files(mut self, files: TorrentFiles) -> Self {
        self.files = Some(files);
        self
    }

    pub fn build(self) -> Result<TorrentMetadataInfo> {
        let name = self.name.ok_or(TorrentError::InvalidMetadata(
            "name must be set".to_string(),
        ))?;
        let files = self.files.ok_or(TorrentError::InvalidMetadata(
            "files must be set".to_string(),
        ))?;

        Ok(TorrentMetadataInfo {
            pieces: self.pieces.unwrap_or_default(),
            piece_length: self.piece_length.unwrap_or_default(),
            name,
            name_utf8: self.name_utf8,
            private: self.private,
            source: self.source,
            meta_version: None,
            files,
        })
    }
}

/// Detailed information from a .torrent file, akin to `add_torrent_params` in `libtorrent`.
///
/// This struct facilitates adding a new [crate::torrent::Torrent] to a session.
///
/// # Examples
///
/// ```
/// use std::convert::TryInto;
/// use crate::popcorn_fx_torrent::torrent::{TorrentMetadata, TorrentError, MagnetResult};
///
/// fn parse_torrent_data(data: &[u8]) -> MagnetResult<TorrentMetadata> {
///     let torrent_info: TorrentMetadata = data.try_into()?;
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
pub struct TorrentMetadata {
    /// The display name of the torrent.
    #[serde(skip)]
    name: Option<String>,
    /// URL of the tracker for the torrent.
    pub announce: Option<String>,
    /// Metadata specific to the torrent, equivalent to `ti` field in `add_torrent_params`.
    pub info: Option<TorrentMetadataInfo>,
    /// The size of the info [TorrentMetadataInfo] in bytes.
    #[serde(skip)]
    pub info_byte_size: Option<usize>,
    /// A dictionary of strings. For each file in the file tree that is larger than the piece size it contains one string value.
    /// See BEP52.
    #[serde(rename = "piece layers", default, with = "serde_piece_layers")]
    pub piece_layers: Option<PieceLayers>,
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
}

impl TorrentMetadata {
    /// Creates a new `TorrentInfoBuilder` instance.
    pub fn builder() -> TorrentMetadataBuilder {
        TorrentMetadataBuilder::builder()
    }

    /// Get the display name of the torrent if known.
    pub fn name(&self) -> Option<&str> {
        self.name
            .as_ref()
            .filter(|e| !e.is_empty())
            .map(|e| e.as_str())
    }

    pub fn update_name<T: Into<String>>(&mut self, name: T) {
        self.name = Some(name.into());
    }

    /// Get the metadata version of the torrent if known.
    ///
    /// If the protocol version is v1, it will return `1`.
    /// If the protocol version is v2, it will return `2`.
    ///
    /// # Returns
    ///
    /// Returns the metadata version of the torrent info if known, else [None].
    pub fn metadata_version(&self) -> Option<u64> {
        self.info.as_ref().map(|e| e.meta_version.unwrap_or(1))
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

    /// Get the trackers that don't contain any order for this torrent.
    ///
    /// # Returns
    ///
    /// Returns the trackers from the `announce` field for this torrent.
    pub fn trackers(&self) -> Vec<Url> {
        self.announce
            .as_ref()
            .iter()
            .filter_map(|url| Url::parse(url).ok())
            .collect()
    }

    /// Get the total number of pieces which are in the torrent.
    /// This can only be calculated if the metadata is known and will return [None] when not it's available.
    ///
    /// # Returns
    ///
    /// Returns the total number of pieces for the torrent.
    pub fn total_pieces(&self) -> Option<usize> {
        self.info
            .as_ref()
            .map(|metadata| {
                let file_size = metadata.len();
                let piece_length = metadata.piece_length as usize;
                let num_pieces = (file_size + piece_length - 1) / piece_length;

                let expected_pieces = if self.info_hash.has_v2() {
                    metadata.sha256_pieces().len()
                } else {
                    metadata.sha1_pieces().len()
                };

                if expected_pieces == num_pieces {
                    Some(num_pieces)
                } else {
                    debug!(
                        "Unable to determine pieces, expected {} but got {} instead",
                        expected_pieces, num_pieces
                    );
                    None
                }
            })
            .unwrap_or(None)
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

impl TryFrom<&[u8]> for TorrentMetadata {
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
        let mut torrent_info = serde_bencode::from_bytes::<Self>(value)
            .map_err(|e| TorrentError::TorrentParse(e.to_string()))?;
        // retrieve the metadata version from the metadata info, default to version 1 if unknown
        let metadata_version = torrent_info.metadata_version().unwrap_or(1);
        // calculate the info hash from the info dict
        let info_bytes = serde_bencode::to_bytes(&torrent_info.info)
            .map_err(|e| TorrentError::TorrentParse(e.to_string()))?;
        let info_len = info_bytes.len();
        // calculate the info hash based on the metadata version
        let info_hash = if metadata_version == 2 {
            InfoHash::from_metadata_v2(info_bytes)
        } else {
            InfoHash::from_metadata_v1(info_bytes)
        };

        torrent_info.info_hash = info_hash;
        torrent_info.info_byte_size = Some(info_len);
        Ok(torrent_info)
    }
}

impl TryFrom<Magnet> for TorrentMetadata {
    type Error = TorrentError;

    fn try_from(value: Magnet) -> Result<Self> {
        let mut builder = TorrentMetadataBuilder::builder();

        // extract the display name
        if let Some(name) = value.display_name.as_ref() {
            builder = builder.name(name);
        }
        // extract the trackers
        for tracker in value.trackers() {
            builder = builder.tracker(tracker);
        }
        // extract webseeds
        let webseeds: UrlList = value.ws().into_iter().map(|e| e.clone()).collect();
        if !webseeds.is_empty() {
            builder = builder.url_list(webseeds);
        }
        // extract the info hash
        builder = builder.info_hash(InfoHash::try_from_str_slice(value.xt().as_slice())?);

        Ok(builder.build())
    }
}

impl TryFrom<&TorrentMetadata> for Magnet {
    type Error = errors::MagnetError;

    fn try_from(value: &TorrentMetadata) -> errors::MagnetResult<Self> {
        if let Some(uri) = value.magnet_uri.as_ref() {
            Magnet::from_str(uri)
        } else {
            let mut builder = Magnet::builder();
            let trackers = value
                .announce_list
                .iter()
                .flat_map(|e| (*e).clone())
                .flat_map(|e| e)
                .collect();

            builder
                .exact_topic(value.info_hash.to_string())
                .address_trackers(trackers);

            if let Some(name) = value.name() {
                builder.display_name(name);
            }
            if let Some(web_seeds) = value.url_list.as_ref() {
                builder.web_seeds(web_seeds.clone());
            }
            if let Some(metadata) = value.info.as_ref() {
                if let Some(source) = metadata.source.as_ref() {
                    builder.exact_source(source);
                }
            }

            builder.build()
        }
    }
}

/// A builder for constructing a `TorrentInfo` struct with optional fields.
///
/// The `TorrentInfoBuilder` allows for the creation of a `TorrentInfo` instance with flexible configuration of its fields.
#[derive(Debug, Default)]
pub struct TorrentMetadataBuilder {
    name: Option<String>,
    announce: Option<String>,
    info: Option<TorrentMetadataInfo>,
    announce_list: Option<Vec<Vec<String>>>,
    creation_date: Option<u64>,
    comment: Option<String>,
    created_by: Option<String>,
    encoding: Option<String>,
    url_list: Option<UrlList>,
    info_hash: Option<InfoHash>,
    piece_layers: Option<HashMap<String, String>>,
}

impl TorrentMetadataBuilder {
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
    pub fn info(mut self, info: TorrentMetadataInfo) -> Self {
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
    pub fn build(self) -> TorrentMetadata {
        TorrentMetadata {
            name: self.name,
            announce: self.announce,
            info: self.info,
            info_byte_size: None,
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
        }
    }
}

#[derive(Debug)]
struct FileTreeVisitor;

impl<'de> Visitor<'de> for FileTreeVisitor {
    type Value = FileTree;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a valid bencoded file tree")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut file_tree = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == "" {
                let file_node = map.next_value::<FileNode>()?;
                file_tree = Some(FileTree::File(file_node));
            } else {
                let tree = map.next_value::<FileTree>()?;
                if let FileTree::Dir(file_tree_dir) =
                    file_tree.get_or_insert(FileTree::Dir(Default::default()))
                {
                    file_tree_dir.insert(key, tree);
                } else {
                    return Err(Error::custom(
                        "unexpected FileTree value, expected FileTree::Dir",
                    ));
                }
            }
        }

        match file_tree {
            Some(e) => Ok(e),
            None => Err(Error::custom("file tree map is empty")),
        }
    }
}

pub mod serde_file_tree {
    use super::*;
    use serde::ser::SerializeMap;

    pub fn serialize<S>(file_tree: &FileTree, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match file_tree {
            FileTree::File(node) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("", node)?;
                map.end()
            }
            FileTree::Dir(nodes) => {
                let mut map = serializer.serialize_map(Some(nodes.len()))?;
                for (key, value) in nodes {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<FileTree, D::Error>
    where
        D: Deserializer<'de>,
    {
        D::deserialize_map(deserializer, FileTreeVisitor {})
    }
}

#[derive(Debug)]
struct StringBytes(String);

impl<'de> Deserialize<'de> for StringBytes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StringBytesVisitor {})
    }
}

#[derive(Debug)]
struct StringBytesVisitor;

impl<'de> Visitor<'de> for StringBytesVisitor {
    type Value = StringBytes;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a string byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(StringBytes(String::from_utf8_lossy(v).to_string()))
    }
}

#[derive(Debug)]
struct PieceLayersVisitor;

impl<'de> Visitor<'de> for PieceLayersVisitor {
    type Value = PieceLayers;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a dictionary of bytes representing strings")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = HashMap::new();

        while let Some((key, value)) = map.next_entry::<StringBytes, StringBytes>()? {
            let key_str = key.0;
            let value_str = value.0;
            result.insert(key_str, value_str);
        }

        Ok(result)
    }
}

pub mod serde_piece_layers {
    use super::*;
    use serde::ser::SerializeMap;

    pub fn serialize<S>(
        value: &Option<PieceLayers>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => {
                let mut map = serializer.serialize_map(Some(value.len()))?;
                for (k, v) in value {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<PieceLayers>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Some(deserializer.deserialize_map(PieceLayersVisitor {})?).filter(|e| !e.is_empty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use std::str::FromStr;

    #[test]
    fn test_torrent_info_tiered_trackers() {
        init_logger!();
        let announce = "udp://example.tracker.org:6969/announce";
        let info = TorrentMetadataBuilder::builder()
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
    fn test_torrent_info_try_from_bytes_v1() {
        init_logger!();
        let announce = "http://bttracker.debian.org:6969/announce";
        let data = read_test_file_to_bytes("debian.torrent");
        let expected_name = "debian-11.6.0-amd64-netinst.iso";
        let expected_piece_length = 262144;
        let expected_files = TorrentFiles::Single {
            file: TorrentFileInfo {
                length: 406847488,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
        };

        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();

        assert_eq!(announce, info.announce.expect("expected announce").as_str());
        assert_ne!(
            None, info.info,
            "expected the metadata to have been present"
        );
        let metadata = info.info.unwrap();
        assert_eq!(
            expected_name, metadata.name,
            "expected the piece length to match"
        );
        assert_eq!(
            expected_piece_length, metadata.piece_length,
            "expected the piece length to match"
        );
        assert_eq!(expected_files, metadata.files);
    }

    #[test]
    fn test_torrent_info_try_from_bytes_v2() {
        init_logger!();
        let data = read_test_file_to_bytes("v2.torrent");
        let expected_name = "bittorrent-v2-test";
        let expected_piece_length = 4194304;
        let expected_total_files = 11;

        let info =
            TorrentMetadata::try_from(data.as_slice()).expect("expected the v2 torrent to parse");

        assert_ne!(
            None, info.info,
            "expected the metadata to have been present"
        );
        let metadata = info.info.unwrap();
        assert_eq!(
            expected_name, metadata.name,
            "expected the piece length to match"
        );
        assert_eq!(
            expected_piece_length, metadata.piece_length,
            "expected the piece length to match"
        );
        assert_eq!(expected_total_files, metadata.total_files());
    }

    #[test]
    fn test_torrent_info_files_v2() {
        init_logger!();
        let data = read_test_file_to_bytes("v2.torrent");

        let info =
            TorrentMetadata::try_from(data.as_slice()).expect("expected the v2 torrent to parse");
        assert_ne!(
            None, info.info,
            "expected the metadata to have been present"
        );

        let metadata = info.info.unwrap();
        let result = metadata.files();
        assert_eq!(11, result.len());
    }

    #[test]
    fn test_torrent_info_try_from_magnet() {
        init_logger!();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();

        let result = TorrentMetadata::try_from(magnet).unwrap();

        let info_hash = &result.info_hash;
        assert_eq!(
            "eadaf0efea39406914414d359e0ea16416409bd7",
            hex::encode(info_hash.hash_v1().unwrap())
        );
        assert_eq!(Some("debian-12.4.0-amd64-DVD-1.iso"), result.name());
    }

    #[test]
    fn test_torrent_info_create_info_hash() {
        init_logger!();
        let torrent = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentMetadata::try_from(torrent.as_slice()).unwrap();

        let result = info.calculate_info_hash().unwrap();

        assert_eq!(info.info_hash, result);
    }

    #[test]
    fn test_file_attribute_flags_from_str() {
        let result = FileAttributeFlags::from_str("x").unwrap();
        assert_eq!(FileAttributeFlags::Executable, result);

        let result = FileAttributeFlags::from_str("H").unwrap();
        assert_eq!(FileAttributeFlags::Hidden, result);

        let result = FileAttributeFlags::from_str("p").unwrap();
        assert_eq!(FileAttributeFlags::PaddingFile, result);

        let result = FileAttributeFlags::from_str("l").unwrap();
        assert_eq!(FileAttributeFlags::Symlink, result);
    }

    #[test]
    fn test_file_attribute_flags_deserialize() {
        let expected_result = FileAttributeFlags::Executable | FileAttributeFlags::PaddingFile;
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result: FileAttributeFlags = serde_bencode::from_bytes(bytes.as_ref()).unwrap();
        assert_eq!(expected_result, result);

        let expected_result = FileAttributeFlags::Symlink;
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result: FileAttributeFlags = serde_bencode::from_bytes(bytes.as_ref()).unwrap();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_file_path() {
        let expected_result = PathBuf::from("foo/bar");
        let file = TorrentFileInfo {
            length: 0,
            path: Some(vec!["foo".to_string(), "bar".to_string()]),
            path_utf8: None,
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let result = file.path();
        assert_eq!(expected_result, result);

        let expected_result = PathBuf::from("esta/dolor");
        let file = TorrentFileInfo {
            length: 0,
            path: Some(vec!["this".to_string(), "is invalid".to_string()]),
            path_utf8: Some(vec!["esta".to_string(), "dolor".to_string()]),
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let result = file.path();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_file_path_segments() {
        let expected_result = vec!["foo".to_string(), "bar".to_string()];
        let file = TorrentFileInfo {
            length: 0,
            path: Some(expected_result.clone()),
            path_utf8: None,
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let result = file.path_segments();
        assert_eq!(expected_result, result);

        let expected_result = vec!["esta".to_string(), "dolor".to_string()];
        let file = TorrentFileInfo {
            length: 0,
            path: Some(vec!["this".to_string(), "is invalid".to_string()]),
            path_utf8: Some(expected_result.clone()),
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let result = file.path_segments();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_metadata_info_path() {
        let file_info = TorrentFileInfo {
            length: 0,
            path: Some(vec!["TorrentFile.mp4".to_string()]),
            path_utf8: None,
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let metadata = TorrentMetadataInfo {
            piece_length: 0,
            pieces: vec![],
            name: "MyTorrentDir".to_string(),
            name_utf8: None,
            private: None,
            source: None,
            meta_version: None,
            files: TorrentFiles::Multiple {
                files: vec![file_info.clone()],
            },
        };
        let expected_result = PathBuf::from("MyTorrentDir/TorrentFile.mp4");

        let result = metadata.path(&file_info);

        assert_eq!(expected_result, result);
    }
}
