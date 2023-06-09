use std::path::PathBuf;

use derive_more::Display;
use directories::UserDirs;
use serde::{Deserialize, Serialize};

use crate::core::config::DEFAULT_HOME_DIRECTORY;

const DEFAULT_TORRENT_DIRECTORY_NAME: &str = "torrents";
const DEFAULT_DIRECTORY: fn() -> PathBuf = || UserDirs::new()
    .map(|e| PathBuf::from(e.home_dir()))
    .map(|e| e
        .join(DEFAULT_HOME_DIRECTORY)
        .join(DEFAULT_TORRENT_DIRECTORY_NAME))
    .expect("expected a home directory to exist");
const DEFAULT_CLEANING_MODE: fn() -> CleaningMode = || CleaningMode::OnShutdown;
const DEFAULT_CONNECTIONS_LIMIT: fn() -> u32 = || 300;
const DEFAULT_DOWNLOAD_RATE_LIMIT: fn() -> u32 = || 0;
const DEFAULT_UPLOAD_RATE_LIMIT: fn() -> u32 = || 0;

/// The torrent user's settings for the application.
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "directory: {:?}, cleaning mode: {}", directory, cleaning_mode)]
pub struct TorrentSettings {
    /// The path to the torrent directory
    #[serde(default = "DEFAULT_DIRECTORY")]
    pub directory: PathBuf,
    /// The cleaning mode for downloaded files.
    #[serde(default = "DEFAULT_CLEANING_MODE")]
    pub cleaning_mode: CleaningMode,
    /// The max number of connections
    #[serde(default = "DEFAULT_CONNECTIONS_LIMIT")]
    pub connections_limit: u32,
    /// The download rate limit, in bytes per second. A value of 0 means unlimited.
    #[serde(default = "DEFAULT_DOWNLOAD_RATE_LIMIT")]
    pub download_rate_limit: u32,
    /// The upload rate limit, in bytes per second. A value of 0 means unlimited.
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
            cleaning_mode: DEFAULT_CLEANING_MODE(),
            connections_limit: DEFAULT_CONNECTIONS_LIMIT(),
            download_rate_limit: DEFAULT_DOWNLOAD_RATE_LIMIT(),
            upload_rate_limit: DEFAULT_UPLOAD_RATE_LIMIT(),
        }
    }
}

/// The cleaning mode for downloaded files.
#[repr(i32)]
#[derive(Debug, Clone, Display, Serialize, Deserialize, PartialEq)]
pub enum CleaningMode {
    /// Cleaning is disabled.
    #[display(fmt = "Disabled")]
    Off = 0,
    /// Files are cleaned on application shutdown.
    #[display(fmt = "On application shutdown")]
    OnShutdown = 1,
    /// Files are cleaned when fully watched.
    #[display(fmt = "When fully watched")]
    Watched = 2,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let expected_result = TorrentSettings {
            directory: DEFAULT_DIRECTORY(),
            cleaning_mode: DEFAULT_AUTO_CLEANING(),
            connections_limit: DEFAULT_CONNECTIONS_LIMIT(),
            download_rate_limit: DEFAULT_DOWNLOAD_RATE_LIMIT(),
            upload_rate_limit: DEFAULT_UPLOAD_RATE_LIMIT(),
        };

        let result = TorrentSettings::default();

        assert_eq!(expected_result, result)
    }
}