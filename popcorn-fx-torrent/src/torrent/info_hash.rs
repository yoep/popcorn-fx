use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use base32::Alphabet;
use hex::FromHex;
use log::{debug, error, trace, warn};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha1::{Digest, Sha1};
use sha2::Sha256;

use crate::torrent::errors::Result;
use crate::torrent::TorrentError;

pub const V1_HASH_IDENTIFIER: &str = "btih";
pub const V2_HASH_IDENTIFIER: &str = "btmh";

/// Represents the available protocol versions of the BitTorrent protocol.
pub const PROTOCOL_VERSIONS: [ProtocolVersion; 2] = [ProtocolVersion::V1, ProtocolVersion::V2];

/// Represent the v1 hash type of the BitTorrent protocol.
pub type Sha1Hash = [u8; 20];

/// Represent the v2 hash type of the BitTorrent protocol.
pub type Sha256Hash = [u8; 32];

/// Represents the unique identifier for a torrent's metadata.
/// It can contain both v1 (SHA1) and v2 (SHA256) hashes.
#[derive(Default, Clone, Eq, Hash)]
pub struct InfoHash {
    /// The v1 (SHA1) hash, if present.
    v1: Option<Sha1Hash>,
    /// The v2 (SHA256) hash, if present.
    v2: Option<Vec<u8>>,
}

impl InfoHash {
    /// Creates a new `InfoHashBuilder` for constructing an `InfoHash`.
    ///
    /// # Returns
    ///
    /// A new `InfoHashBuilder` instance.
    pub fn builder() -> InfoHashBuilder {
        InfoHashBuilder::default()
    }

    /// Checks if the v1 info hash is present.
    ///
    /// # Returns
    ///
    /// `true` if the v1 info hash is present, otherwise `false`.
    pub fn has_v1(&self) -> bool {
        self.v1.is_some()
    }

    /// Checks if the v2 info hash is present.
    ///
    /// # Returns
    ///
    /// `true` if the v2 info hash is present, otherwise `false`.
    pub fn has_v2(&self) -> bool {
        self.v2.is_some()
    }

    /// Retrieve the v1 info hash if present.
    /// The v1 hash is always exactly 20 bytes long.
    ///
    /// # Returns
    ///
    /// Returns the v1 info hash as a 20-byte array if present, otherwise `None`.
    pub fn hash_v1(&self) -> Option<Sha1Hash> {
        self.v1.as_ref().map(|e| e.clone())
    }

    /// Returns a reference to the v2 info hash if present.
    ///
    /// # Returns
    ///
    /// Returns a reference to the v2 info hash as a slice of bytes if present, otherwise `None`.
    pub fn hash_v2(&self) -> Option<&[u8]> {
        self.v2.as_ref().map(|e| e.as_slice())
    }

    /// Get the info hash as a 20-byte array.
    /// This will be the v1 hash if present, otherwise the shortened v2 hash to match 20 bytes.
    ///
    /// # Returns
    ///
    /// Returns the info hash as a 20-byte array.
    pub fn short_info_hash_bytes(&self) -> Sha1Hash {
        let mut info_hash_bytes = [0u8; 20];

        if let Some(hash) = self.hash_v1() {
            info_hash_bytes = hash;
        } else if let Some(bytes) = self.v2_as_short() {
            info_hash_bytes = bytes;
        } else {
            error!("Both v1 & v2 info hashes are missing, using empty 20-byte array as fallback");
        }

        info_hash_bytes
    }

    /// Get the v2 info hash as shortened 20-byte array.
    ///
    /// # Returns
    ///
    /// It returns the v2 hash as a 20-byte array.
    pub fn v2_as_short(&self) -> Option<Sha1Hash> {
        if let Some(hash) = self.hash_v2() {
            let mut info_hash_bytes = [0u8; 20];
            info_hash_bytes.copy_from_slice(&hash[..20]);
            return Some(info_hash_bytes);
        }

        None
    }

    /// Get the v1 hash as a hex encoded string.
    pub fn v1_as_str(&self) -> Option<String> {
        self.v1.as_ref().map(|e| hex::encode(e).to_uppercase())
    }

    /// Get the v2 hash as a hex encoded string.
    pub fn v2_as_str(&self) -> Option<String> {
        self.v2.as_ref().map(|e| hex::encode(e).to_uppercase())
    }

    /// Try to parse the given hash bytes into an `InfoHash`.
    /// This should only be used by peer implementations or piece calculations,
    /// which want to validate an incoming handshake or piece data.
    ///
    /// # Returns
    ///
    /// Returns the info hash if the given bytes are a valid v1 or v2 info hash.
    pub fn try_from_bytes<T>(bytes: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        if bytes.as_ref().len() == 20 {
            let mut hash: Sha1Hash = [0; 20];
            hash.copy_from_slice(bytes.as_ref());
            return Ok(Self {
                v1: Some(hash),
                v2: None,
            });
        } else if bytes.as_ref().len() == 32 {
            let mut hash: Sha256Hash = [0; 32];
            hash.copy_from_slice(bytes.as_ref());
            return Ok(Self {
                v1: None,
                v2: Some(hash.to_vec()),
            });
        }

        Err(TorrentError::InvalidInfoHash(
            "invalid info hash length".to_string(),
        ))
    }

    /// Try to parse the info hash from multiple str slices.
    /// This can be used when an info hash might be hybrid and both contain a v1 & v2 hash.
    pub fn try_from_str_slice(values: &[&str]) -> Result<Self> {
        if values.is_empty() {
            return Err(TorrentError::InvalidTopic(
                "empty topic string slice".to_string(),
            ));
        }

        trace!("Parsing info hash from values {:?}", values);
        let mut v1_hash: Option<Sha1Hash> = None;
        let mut v2_hash: Option<Vec<u8>> = None;

        for value in values {
            let segments: Vec<&str> = value.split(':').collect();
            let info_hash = if segments.len() == 1 {
                Self::try_from_str_value(value).ok()
            } else {
                Self::try_from_str_segments(value, segments).ok()
            };

            if let Some(info_hash) = info_hash {
                v1_hash = info_hash.v1.or(v1_hash);
                v2_hash = info_hash.v2.or(v2_hash);
            }
        }

        if v1_hash.is_none() && v2_hash.is_none() {
            return Err(TorrentError::InvalidTopic(
                "none of the provided values are valid".to_string(),
            ));
        }

        Ok(Self {
            v1: v1_hash,
            v2: v2_hash,
        })
    }

    /// Create the info from the given v1 bencoded metadata bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bencoded metadata bytes.
    ///
    /// # Returns
    ///
    /// Returns the info hash for the metadata bytes.
    pub fn from_metadata_v1<T>(bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self::from_metadata(bytes, false)
    }

    /// Create the info from the given v2 bencoded metadata bytes.
    /// This will result in both a v1 & v2 hash being created.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bencoded metadata bytes.
    ///
    /// # Returns
    ///
    /// Returns the info hash for the metadata bytes.
    pub fn from_metadata_v2<T>(bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self::from_metadata(bytes, true)
    }

    fn from_metadata<T>(bytes: T, is_v2: bool) -> Self
    where
        T: AsRef<[u8]>,
    {
        let v1_hasher = Sha1::digest(bytes.as_ref());
        let mut v1_hash = [0; 20];
        let mut v2_hash = None;

        v1_hash.copy_from_slice(&v1_hasher[..20]);

        if is_v2 {
            let v2_hasher = Sha256::digest(bytes.as_ref());
            v2_hash = Some(v2_hasher.to_vec());
        }

        Self {
            v1: Some(v1_hash),
            v2: v2_hash,
        }
    }

    /// Try to parse the given v1 info hash into an `InfoHash`.
    /// The following formats are supported for the info hash:
    /// * SHA1 hex encoded
    /// * SHA1 base32 encoded
    /// * Raw SHA1 hash
    ///
    /// # Arguments
    ///
    /// * `value` - The v1 info hash string to parse.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `InfoHash` if successful, or a `TorrentError` if parsing fails.
    fn try_from_v1(value: &str) -> Result<Self> {
        trace!("Trying to parse info hash from v1 {}", value);
        let topic_length = value.len();
        let mut topic_bytes = value.as_bytes().to_vec();

        if topic_length == 40 {
            trace!("Parsing info hash value as v1 hex");
            topic_bytes = Vec::from_hex(value).map_err(|e| {
                debug!("Failed to parse v1 info hash hex, {}", e);
                TorrentError::InvalidInfoHash("invalid hex value".to_string())
            })?;
        } else if topic_length == 32 {
            trace!("Parsing info hash value as v1 base32");
            let decoded_topic = base32::decode(Alphabet::Z, value).ok_or(
                TorrentError::InvalidInfoHash("invalid base32 value".to_string()),
            )?;

            if decoded_topic.len() != 20 {
                return Err(TorrentError::InvalidInfoHash(
                    "expected 20 bytes".to_string(),
                ));
            }

            topic_bytes = decoded_topic;
        } else if topic_length != 20 {
            trace!("Parsing info hash value as v1 sha1");
            return Err(TorrentError::InvalidInfoHash(
                "expected 20 bytes".to_string(),
            ));
        }

        if topic_bytes.len() != 20 {
            return Err(TorrentError::InvalidInfoHash(format!(
                "expected 20 bytes, got {}",
                topic_bytes.len()
            )));
        }

        let mut hash = [0u8; 20];
        hash.copy_from_slice(&topic_bytes[..20]);
        Ok(Self {
            v1: Some(hash),
            v2: None,
        })
    }

    /// Try to parse the given v2 info hash into an `InfoHash`.
    /// The v2 hash must be SHA256 and prefixed with "1220".
    ///
    /// # Arguments
    ///
    /// * `value` - The v2 info hash string to parse.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `InfoHash` if successful, or a `TorrentError` if parsing fails.
    fn try_from_v2(value: &str) -> Result<Self> {
        trace!("Trying to parse info hash from v2 {}", value);
        if !value.starts_with("1220") {
            return Err(TorrentError::InvalidInfoHash(
                "invalid sha256 prefix".to_string(),
            ));
        }

        let value = &value.as_bytes()[4..];

        if value.len() != 64 {
            return Err(TorrentError::InvalidInfoHash(
                "expected sha256 to be 64 bytes".to_string(),
            ));
        }

        trace!("Parsing info hash value as v2 sha256 hex encoded");
        let info_hash = hex::decode(value).map_err(|e| {
            warn!("Failed to parse info hash hex, {}", e);
            TorrentError::InvalidInfoHash("invalid sha256 hex".to_string())
        })?;

        Ok(Self {
            v1: None,
            v2: Some(info_hash),
        })
    }

    /// Try to parse the given value into an `InfoHash`.
    /// It tries to find the info hash format without any identifiers.
    fn try_from_str_value(value: &str) -> Result<Self> {
        match Self::try_from_v1(value) {
            Ok(info_hash) => Ok(info_hash),
            Err(_) => match Self::try_from_v2(value) {
                Ok(info_hash) => Ok(info_hash),
                Err(_) => Err(TorrentError::InvalidTopic(value.to_string())),
            },
        }
    }

    /// Try to parse the given value into an `InfoHash`.
    /// It tries to find the info hash format based on identifiers from within the segments.
    fn try_from_str_segments(value: &str, segments: Vec<&str>) -> Result<Self> {
        let info_hash_version_identifier = segments
            .get(1)
            .ok_or(TorrentError::InvalidTopic(value.to_string()))?;
        let info_hash_value = segments
            .get(2)
            .ok_or(TorrentError::InvalidTopic(value.to_string()))?;

        if *info_hash_version_identifier == V1_HASH_IDENTIFIER {
            Self::try_from_v1(info_hash_value)
        } else if *info_hash_version_identifier == V2_HASH_IDENTIFIER {
            Self::try_from_v2(info_hash_value)
        } else {
            warn!("Unable to identify info hash version for xt {}", value);
            Err(TorrentError::InvalidTopic(value.to_string()))
        }
    }
}

impl Serialize for InfoHash {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.short_info_hash_bytes();
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for InfoHash {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(InfoHashVisitor {})
    }
}

impl PartialEq for InfoHash {
    fn eq(&self, other: &Self) -> bool {
        match (&self.v1, &other.v1) {
            (Some(v1_self), Some(v1_other)) if v1_self != v1_other => return false,
            _ => {}
        }

        match (&self.v2, &other.v2) {
            (Some(v2_self), Some(v2_other)) if v2_self != v2_other => return false,
            _ => {}
        }

        true
    }
}

impl Debug for InfoHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfoHash")
            .field("v1", &self.v1_as_str())
            .field("v2", &self.v2_as_str())
            .finish()
    }
}

impl Display for InfoHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let hash = self
            .v1_as_str()
            .unwrap_or_else(|| self.v2_as_str().unwrap_or("INVALID INFO HASH".to_string()));

        write!(f, "{}", hash)
    }
}

impl FromStr for InfoHash {
    type Err = TorrentError;

    /// Parses an `InfoHash` from a string representation.
    /// The expected format is `xt:<version>:<info_hash>` where `<version>` identifies the hash version or the `<info_hash>` hex value without any additions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::str::FromStr;
    /// use popcorn_fx_torrent::torrent::InfoHash;
    ///
    /// // parse from a v1 info hash
    /// let xt_v1 = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
    /// let info_hash = InfoHash::from_str(xt_v1).unwrap();
    ///
    /// // parse from an unidentified info hash
    /// let hash = "EADAF0EFEA39406914414D359E0EA16416409BD7";
    /// let info_hash = InfoHash::from_str(hash).unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `xt` - The string to parse, containing the info hash.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `InfoHash` if parsing is successful, or a `TorrentError` if it fails.
    fn from_str(value: &str) -> Result<Self> {
        trace!("Parsing info hash from value {}", value);
        let segments: Vec<&str> = value.split(':').collect();

        if segments.len() == 1 {
            Self::try_from_str_value(value)
        } else {
            Self::try_from_str_segments(value, segments)
        }
    }
}

/// A builder for constructing an `InfoHash` struct with optional v1 and v2 hashes.
///
/// This builder allows setting the v1 and v2 hashes individually and then constructing
/// an `InfoHash` with those values. It is useful for creating `InfoHash` instances when
/// you have specific hash values to include.
///
/// # Examples
///
/// ```rust
/// use popcorn_fx_torrent::torrent::InfoHash;
///
/// let info_hash = InfoHash::builder()
///     .v1(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20])
///     .v2(vec![21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64])
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct InfoHashBuilder {
    v1: Option<Vec<u8>>,
    v2: Option<Vec<u8>>,
}

impl InfoHashBuilder {
    /// Creates a new `InfoHashBuilder` instance with default settings.
    ///
    /// # Returns
    ///
    /// A new `InfoHashBuilder` instance.
    pub fn builder() -> InfoHashBuilder {
        InfoHashBuilder::default()
    }

    /// Sets the v1 hash in the builder.
    ///
    /// # Arguments
    ///
    /// * `v1` - A `Vec<u8>` containing the v1 hash.
    ///
    /// # Returns
    ///
    /// The updated `InfoHashBuilder` instance with the v1 hash set.
    pub fn v1<T: Into<Vec<u8>>>(mut self, v1: T) -> Self {
        self.v1 = Some(v1.into());
        self
    }

    /// Sets the v2 hash in the builder.
    ///
    /// # Arguments
    ///
    /// * `v2` - A `Vec<u8>` containing the v2 hash.
    ///
    /// # Returns
    ///
    /// The updated `InfoHashBuilder` instance with the v2 hash set.
    pub fn v2<T: Into<Vec<u8>>>(mut self, v2: T) -> Self {
        self.v2 = Some(v2.into());
        self
    }

    /// Constructs and returns an `InfoHash` based on the configured builder settings.
    ///
    /// This method validates the v1 hash to ensure it is exactly 20 bytes long, as required.
    /// If the v1 hash does not meet this requirement, it returns an error.
    ///
    /// # Returns
    ///
    /// * `Ok(InfoHash)` - An `InfoHash` instance containing the v1 and/or v2 hashes set in the builder, if all validations pass.
    /// * `Err(TorrentError)` - A `TorrentError` if the v1 hash is not 20 bytes long.
    pub fn build(self) -> Result<InfoHash> {
        if let Some(v1) = self.v1.as_ref() {
            if v1.len() != 20 {
                return Err(TorrentError::InvalidInfoHash(
                    "v1 hash must be 20 bytes".to_string(),
                ));
            }
        }

        Ok(InfoHash {
            v1: self.v1.map(|e| {
                let mut v1_hash = [0; 20];
                v1_hash.copy_from_slice(e.as_slice());
                v1_hash
            }),
            v2: self.v2,
        })
    }
}

/// BitTorrent version enumerator
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolVersion {
    /// The original BitTorrent version, using SHA-1 hashes
    V1,
    /// Version 2 of the BitTorrent protocol, using SHA-256 hashes
    V2,
}

#[derive(Debug)]
struct InfoHashVisitor;

impl<'de> Visitor<'de> for InfoHashVisitor {
    type Value = InfoHash;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected byte array of length 20")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        InfoHash::try_from_bytes(value).map_err(|e| serde::de::Error::custom(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use crate::torrent::{Magnet, TorrentMetadata};
    use hex_literal::hex;
    use popcorn_fx_core::testing::read_test_file_to_bytes;

    #[test]
    fn test_info_hash_from_metadata_v1() {
        init_logger!();
        let info_data = b"hello world";
        let mut expected_result: [u8; 20] = [0; 20];
        expected_result.copy_from_slice(hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed").as_ref());

        let result = InfoHash::from_metadata_v1(info_data);

        assert_eq!(Some(expected_result), result.hash_v1());
        assert_eq!(None, result.hash_v2());
    }

    #[test]
    fn test_info_hash_from_str() {
        init_logger!();
        let xt_v1 = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
        let result = InfoHash::from_str(xt_v1).unwrap();
        assert!(result.v1.is_some(), "expected a v1 hash");
        assert_eq!(
            "eadaf0efea39406914414d359e0ea16416409bd7",
            result.v1.map(|e| hex::encode(e)).unwrap()
        );

        let xt_v2 = "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
        let result = InfoHash::from_str(xt_v2).unwrap();
        assert!(result.v2.is_some(), "expected a v2 hash");
        assert_eq!(
            "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
            result.v2.map(|e| hex::encode(e)).unwrap()
        );
    }

    #[test]
    fn test_info_hash_from_str_display() {
        init_logger!();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let info_hash = torrent_info.info_hash;

        let value = info_hash.to_string();
        let result = InfoHash::from_str(&value).unwrap();

        assert_eq!(info_hash, result);
    }

    #[test]
    fn test_info_hash_short_info_hash_bytes() {
        init_logger!();
        let expected_result: [u8; 20] =
            Vec::from_hex("EADAF0EFEA39406914414D359E0EA16416409BD7".as_bytes())
                .expect("expected the hash string to be a valid hex")
                .try_into()
                .unwrap();
        let hash = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
        let info_hash = InfoHash::from_str(hash).unwrap();
        let result = info_hash.short_info_hash_bytes();
        assert_eq!(expected_result, result);

        let bytes = hex::decode("cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd")
            .unwrap();
        let mut expected_result = [0u8; 20];
        expected_result.copy_from_slice(&bytes[..20]);
        let hash = "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
        let info_hash = InfoHash::from_str(hash).unwrap();
        let result = info_hash.short_info_hash_bytes();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_info_hash_builder() {
        let result = InfoHashBuilder::default().v1(vec![0, 1, 2, 123]).build();
        assert_eq!(
            Err(TorrentError::InvalidInfoHash(
                "v1 hash must be 20 bytes".to_string()
            )),
            result
        );
    }

    #[test]
    fn test_info_hash_eq() {
        let info_hash = InfoHash {
            v1: Some([0; 20]),
            v2: Some(vec![1, 2, 3, 4, 5]),
        };

        let other = InfoHash {
            v1: Some([0; 20]),
            v2: None,
        };
        assert_eq!(info_hash, other, "expected the v2 hash to not be compared");

        let other = InfoHash {
            v1: None,
            v2: Some(vec![1, 2, 3, 4, 5]),
        };
        assert_eq!(info_hash, other, "expected the v1 hash to not be compared");
    }

    #[test]
    fn test_info_hash_short_info_hash_bytes_different_sources_same_hash() {
        init_logger!();
        let torrent = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info_file = TorrentMetadata::try_from(torrent.as_slice()).unwrap();
        let info_hash_file = torrent_info_file.info_hash;

        let magnet = Magnet::from_str("magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce").unwrap();
        let torrent_info_magnet = TorrentMetadata::try_from(magnet).unwrap();
        let info_hash_magnet = torrent_info_magnet.info_hash;

        let hash_bytes_file = info_hash_file.short_info_hash_bytes();
        let hash_bytes_magnet = info_hash_magnet.short_info_hash_bytes();

        assert_eq!(hash_bytes_file, hash_bytes_magnet);
    }

    #[test]
    fn test_info_hash_display() {
        let hash = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
        let info_hash = InfoHash::from_str(hash).unwrap();
        let result = info_hash.to_string();
        assert_eq!(
            "EADAF0EFEA39406914414D359E0EA16416409BD7", result,
            "expected the v1 hash to be displayed"
        );

        let hash = "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
        let info_hash = InfoHash::from_str(hash).unwrap();
        let result = info_hash.to_string();
        assert_eq!(
            "CDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCD", result,
            "expected the v2 hash to be displayed"
        );
    }

    #[test]
    fn test_info_hash_try_from_str_slice() {
        init_logger!();
        let magnet = Magnet::from_str("magnet:?xt=urn:btih:631a31dd0a46257d5078c0dee4e66e26f73e42ac&xt=urn:btmh:1220d8dd32ac93357c368556af3ac1d95c9d76bd0dff6fa9833ecdac3d53134efabb&dn=bittorrent-v1-v2-hybrid-test").unwrap();

        let result = InfoHash::try_from_str_slice(magnet.xt().as_slice()).unwrap();

        assert_ne!(None, result.v1, "expected the v1 hash to be present");
        assert_ne!(None, result.v2, "expected the v2 hash to be present");
    }

    #[test]
    fn test_info_hash_deserialize() {
        init_logger!();
        let hash = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
        let info_hash = InfoHash::from_str(hash).unwrap();
        let bytes = serde_bencode::to_bytes(&info_hash)
            .expect("expected the info hash to have been serialized");

        let result = serde_bencode::from_bytes(bytes.as_slice())
            .expect("Expected the info hash to have been deserialized");

        assert_eq!(info_hash, result);
    }
}
