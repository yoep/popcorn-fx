use std::os::raw::c_char;

use popcorn_fx_core::core::torrents::collection::MagnetInfo;
use popcorn_fx_core::into_c_vec;

use crate::into_c_string;

/// The collection of stored magnets.
/// It contains the C compatible information for [std::ffi].
#[repr(C)]
#[derive(Debug)]
pub struct TorrentCollectionSet {
    /// The array of magnets
    pub magnets: *mut MagnetInfoC,
    /// The length of the array
    pub len: i32,
}

impl From<Vec<MagnetInfo>> for TorrentCollectionSet {
    fn from(value: Vec<MagnetInfo>) -> Self {
        let (magnets, len) = into_c_vec(value.into_iter()
            .map(MagnetInfoC::from)
            .collect());

        Self {
            magnets,
            len,
        }
    }
}

/// The C compatible struct for [MagnetInfo].
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MagnetInfoC {
    /// The name of the magnet
    pub name: *mut c_char,
    /// The magnet uri to the torrent
    pub magnet_uri: *mut c_char,
}

impl From<MagnetInfo> for MagnetInfoC {
    fn from(value: MagnetInfo) -> Self {
        Self {
            name: into_c_string(value.name),
            magnet_uri: into_c_string(value.magnet_uri),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{from_c_string, from_c_vec};

    use super::*;

    #[test]
    fn test_torrent_collection_set_from() {
        let name = "LoremIpsumMagnet";
        let magnet_uri = "magnet:?MyUri";
        let infos = vec![MagnetInfo {
            name: name.to_string(),
            magnet_uri: magnet_uri.to_string(),
        }];

        let set = TorrentCollectionSet::from(infos.clone());
        assert_eq!(1, set.len);
        let magnet = from_c_vec(set.magnets, set.len);
        let result = magnet
            .get(0)
            .unwrap();

        assert_eq!(name.to_string(), from_c_string(result.name));
        assert_eq!(magnet_uri.to_string(), from_c_string(result.magnet_uri));
    }

    #[test]
    fn test_magnet_info_c_from() {
        let name = "MyMagnet";
        let uri = "magnet:?MyMagnetUri";
        let info = MagnetInfo {
            name: name.to_string(),
            magnet_uri: uri.to_string(),
        };

        let result = MagnetInfoC::from(info.clone());

        assert_eq!(name.to_string(), from_c_string(result.name));
        assert_eq!(uri.to_string(), from_c_string(result.magnet_uri));
    }
}