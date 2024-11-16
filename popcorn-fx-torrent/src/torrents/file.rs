use crate::torrents::{PiecePriority, TorrentFileInfo};
use std::path::PathBuf;

/// Alias name for the piece priority of a file.
pub type FilePriority = PiecePriority;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub offset: u64,
    pub length: u64,
    pub info: TorrentFileInfo,
    pub priority: FilePriority,
}
