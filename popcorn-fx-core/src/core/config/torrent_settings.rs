use std::path::PathBuf;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::core::config::DEFAULT_HOME_DIRECTORY;

const DEFAULT_TORRENT_DIRECTORY_NAME: &str = "torrents";
const DEFAULT_DIRECTORY: fn() -> PathBuf = || home::home_dir()
    .map(|e| e
        .join(DEFAULT_HOME_DIRECTORY)
        .join(DEFAULT_TORRENT_DIRECTORY_NAME))
    .expect("Home directory should exist");
const DEFAULT_AUTO_CLEANING: fn() -> bool = || true;
const DEFAULT_CONNECTIONS_LIMIT: fn() -> u32 = || 300;
const DEFAULT_DOWNLOAD_RATE_LIMIT: fn() -> u32 = || 0;
const DEFAULT_UPLOAD_RATE_LIMIT: fn() -> u32 = || 0;

/// The torrent user's settings for the application.
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "directory: {:?}", directory)]
pub struct TorrentSettings {
    /// The path to the torrent directory
    #[serde(default = "DEFAULT_DIRECTORY")]
    pub directory: PathBuf,
    /// Indicates if the torrent directory should be cleaned
    #[serde(default = "DEFAULT_AUTO_CLEANING")]
    pub auto_cleaning_enabled: bool,
    /// The max number of connections
    #[serde(default = "DEFAULT_CONNECTIONS_LIMIT")]
    pub connections_limit: u32,
    /// The download rate limit, 0 means unlimited
    #[serde(default = "DEFAULT_DOWNLOAD_RATE_LIMIT")]
    pub download_rate_limit: u32,
    /// The upload rate limit, 0 means unlimited
    #[serde(default = "DEFAULT_UPLOAD_RATE_LIMIT")]
    pub upload_rate_limit: u32,
}

impl TorrentSettings {
    /// The torrent directory to store the downloaded files
    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }
}

impl Default for TorrentSettings {
    fn default() -> Self {
        Self {
            directory: DEFAULT_DIRECTORY(),
            auto_cleaning_enabled: DEFAULT_AUTO_CLEANING(),
            connections_limit: DEFAULT_CONNECTIONS_LIMIT(),
            download_rate_limit: DEFAULT_DOWNLOAD_RATE_LIMIT(),
            upload_rate_limit: DEFAULT_UPLOAD_RATE_LIMIT(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let expected_result = TorrentSettings {
            directory: DEFAULT_DIRECTORY(),
            auto_cleaning_enabled: DEFAULT_AUTO_CLEANING(),
            connections_limit: DEFAULT_CONNECTIONS_LIMIT(),
            download_rate_limit: DEFAULT_DOWNLOAD_RATE_LIMIT(),
            upload_rate_limit: DEFAULT_UPLOAD_RATE_LIMIT(),
        };

        let result = TorrentSettings::default();

        assert_eq!(expected_result, result)
    }
}