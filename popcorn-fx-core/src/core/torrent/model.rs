use std::fmt::Debug;
use std::path::PathBuf;

use mockall::automock;

/// The torrent describes the meta-info of a shared file
/// that can be queried over the network.
#[automock]
pub trait Torrent: Debug + Send + Sync {
    /// The absolute path to this torrent file.
    fn file(&self) -> PathBuf;
    
    /// Verify if the given byte is downloaded for this torrent.
    /// 
    /// It returns true when the byte is available, else false.
    fn has_byte(&self, byte: u64) -> bool;
    
    /// Prioritize the given byte to be downloaded.
    fn prioritize_byte(&self, byte: u64);
}