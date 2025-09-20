use crate::torrent::storage::{Metrics, Result};
use crate::torrent::torrent_pools::FilePool;
use crate::torrent::{InfoHash, PieceIndex, Sha1Hash, Sha256Hash};
use async_trait::async_trait;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait Storage: Debug + Send + Sync {
    /// Read the torrent data from the given piece into the buffer.
    /// The [Storage] keeps reading piece(s) data until the buffer is filled,
    /// or the no more data is available.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to write the bytes into.
    /// * `piece` - The piece index to read.
    /// * `offset` - The offset from the piece to start reading from.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes read from the storage.
    async fn read(&self, buffer: &mut [u8], piece: &PieceIndex, offset: usize) -> Result<usize>;

    /// Write the piece data to the storage for the given bytes.
    /// The given bytes should be verified against the hash before calling this fn.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to write to the storage.
    /// * `piece` - The piece index to write to.
    /// * `offset` - The offset within the piece to start writing to.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written to the storage.
    async fn write(&self, data: &[u8], piece: &PieceIndex, offset: usize) -> Result<usize>;

    /// Calculate the hash for the given piece stored in the storage.
    async fn hash_v1(&self, piece: &PieceIndex) -> Result<Sha1Hash>;

    /// Calculate the hash for the given piece stored in the storage.
    async fn hash_v2(&self, piece: &PieceIndex) -> Result<Sha256Hash>;

    /// Move the storage to the new location path.
    async fn move_storage(&self, new_path: &Path) -> Result<()>;

    /// Get the metrics of the storage.
    fn metrics(&self) -> &Metrics;
}

/// The storage parameters for initializing a new [Storage] instance.
#[derive(Debug, Clone)]
pub struct StorageParams {
    pub info_hash: InfoHash,
    pub path: PathBuf,
    pub files: FilePool,
}
