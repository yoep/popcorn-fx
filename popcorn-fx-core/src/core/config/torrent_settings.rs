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

/// The torrent user's settings for the application.
#[derive(Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
#[display(fmt = "directory: {:?}", directory)]
pub struct TorrentSettings {
    /// The path to the torrent directory
    #[serde(default = "DEFAULT_DIRECTORY")]
    directory: PathBuf,
    /// Indicates if the torrent directory should be cleaned
    #[serde(default = "DEFAULT_AUTO_CLEANING")]
    auto_cleaning_enabled: bool,
}

impl TorrentSettings {
    pub fn new(directory: &str, auto_cleaning_enabled: bool) -> Self {
        Self {
            directory: PathBuf::from(directory),
            auto_cleaning_enabled,
        }
    }
    
    /// The torrent directory to store the downloaded files
    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }
}

impl Default for TorrentSettings {
    fn default() -> Self {
        TorrentSettings {
            directory: DEFAULT_DIRECTORY(),
            auto_cleaning_enabled: DEFAULT_AUTO_CLEANING(),
        }
    }
}