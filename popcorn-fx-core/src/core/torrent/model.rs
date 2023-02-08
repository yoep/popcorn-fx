use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

use mockall::automock;

/// The torrent describes the meta-info of a shared file
/// that can be queried over the network.
#[automock]
pub trait Torrent: Display + Debug + Send + Sync {
    /// The absolute path to this torrent file.
    fn file(&self) -> PathBuf;
    
    /// Verify if the given bytes are available for this [Torrent].
    /// 
    /// It returns true when the bytes are available, else false.
    fn has_bytes(&self, bytes: &[u64]) -> bool;
    
    /// Prioritize the given bytes to be downloaded.
    fn prioritize_bytes(&self, bytes: &[u64]);
}

impl Display for MockTorrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTorrent")
    }
}