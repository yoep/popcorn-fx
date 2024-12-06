use crate::torrent::fs::{Error, Result};
use async_trait::async_trait;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const CACHE_CLEANUP_AFTER: Duration = Duration::from_secs(3);

/// Trait for handling the torrent file storage.
/// A [TorrentFileStorage] should always be specific to a single [Torrent].
///
/// ## Remarks
///
/// All [Path] references within this trait are relative to the [TorrentFileStorage] base path.
#[async_trait]
pub trait TorrentFileStorage: Debug + Send + Sync {
    /// Check if the given file exists within the storage.
    /// This doesn't check if the file contains any actual data.
    fn exists(&self, filepath: &Path) -> bool;

    /// Get the path of this storage.
    /// This is the path under which all file operations are performed.
    fn path(&self) -> &Path;

    /// Try to create/open the file.
    /// It returns an error if the specified location couldn't be accessed.
    async fn open(&self, filepath: &Path) -> Result<()>;

    /// Write the given data chunk to the given torrent filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to write to.
    /// * `offset` - The offset to start from within the file.
    /// * `bytes` - The data to write to the file.
    ///
    /// # Returns
    ///
    /// Returns an error when the write operation failed.
    async fn write(&self, filepath: &Path, offset: usize, bytes: &[u8]) -> Result<()>;

    /// Read the given range of bytes from the torrent filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to read from.
    /// * `range` - The range of bytes to read from the file.
    ///
    /// # Returns
    ///
    /// Returns the bytes read from the file if successful.
    async fn read(&self, filepath: &Path, range: Range<usize>) -> Result<Vec<u8>>;

    /// Read all bytes from the torrent filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to read from
    ///
    /// # Returns
    ///
    /// Returns the bytes read from the file if successful
    async fn read_to_end(&self, filepath: &Path) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct DefaultTorrentFileStorage {
    base_path: PathBuf,
}

impl DefaultTorrentFileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the absolute filepath for the given filepath within the storage.
    fn absolute_filepath<P: AsRef<Path>>(&self, filepath: P) -> PathBuf {
        self.base_path.join(filepath.as_ref())
    }

    /// Check if the given filepath is valid within the storage.
    /// If the target filepath leaves the storage path, it returns false.
    fn is_valid_filepath<P: AsRef<Path>>(&self, filepath: P) -> bool {
        let base = Self::canonicalize_unchecked(&self.base_path);
        let target = Self::canonicalize_unchecked(filepath.as_ref());

        target.starts_with(&base)
    }

    async fn internal_open<P: AsRef<Path>>(
        &self,
        filepath: P,
        writeable: bool,
    ) -> Result<tokio::fs::File> {
        let absolute_path = self.absolute_filepath(filepath);

        // check if the given filepath is valid before trying to open it
        // if it's leaving the storage path, it's invalid
        if !self.is_valid_filepath(&absolute_path) {
            return Err(Error::InvalidFilepath(absolute_path));
        }

        // make sure that the parent directories exists
        if writeable {
            let parent_directory = absolute_path.parent().unwrap_or(self.base_path.as_path());
            tokio::fs::create_dir_all(parent_directory)
                .await
                .map_err(|e| Error::Io(e.to_string()))?;
        }

        Ok(tokio::fs::OpenOptions::new()
            .create(writeable)
            .read(true)
            .write(writeable)
            .truncate(false)
            .open(absolute_path.clone())
            .await?)
    }

    /// Get the canonicalized path for the given path.
    /// This function traverses the path components and resolves ".." and "." appropriately.
    /// It returns the resulting path.
    fn canonicalize_unchecked(path: &Path) -> PathBuf {
        let components = path.components();
        let mut result = PathBuf::new();

        // Traverse the path components and resolve ".." and "." appropriately
        for component in components {
            match component {
                // Ignore "." (current directory)
                std::path::Component::CurDir => {}
                // Remove the last component for ".." (parent directory)
                std::path::Component::ParentDir => {
                    result.pop();
                }
                // Add other components as normal
                std::path::Component::Normal(part) => {
                    result.push(part);
                }
                // Handle other components, like RootDir, if necessary
                _ => {}
            }
        }

        result
    }
}

#[async_trait]
impl TorrentFileStorage for DefaultTorrentFileStorage {
    fn exists(&self, filepath: &Path) -> bool {
        let absolute_filepath = self.absolute_filepath(filepath);

        self.is_valid_filepath(&absolute_filepath) && absolute_filepath.exists()
    }

    fn path(&self) -> &Path {
        self.base_path.as_path()
    }

    async fn open(&self, filepath: &Path) -> Result<()> {
        let _ = self.internal_open(filepath, true).await?;
        Ok(())
    }

    async fn write(&self, filepath: &Path, offset: usize, bytes: &[u8]) -> Result<()> {
        let mut file = self.internal_open(filepath, true).await?;
        let file_size = file.metadata().await?.len();

        // check if the offset is out of bounds
        if offset > file_size as usize {
            // write empty bytes to fill the gap
            file.seek(std::io::SeekFrom::Start(file_size)).await?;
            file.write_all(&vec![0; offset - file_size as usize])
                .await?
        } else {
            file.seek(std::io::SeekFrom::Start(offset as u64)).await?;
        }

        file.write_all(bytes).await?;
        Ok(())
    }

    async fn read(&self, filepath: &Path, range: Range<usize>) -> Result<Vec<u8>> {
        let mut file = self.internal_open(filepath, false).await?;
        let mut buffer = vec![0u8; range.len()];

        file.seek(std::io::SeekFrom::Start(range.start as u64))
            .await?;
        file.read_exact(&mut buffer).await?;

        Ok(buffer.to_vec())
    }

    async fn read_to_end(&self, filepath: &Path) -> Result<Vec<u8>> {
        let mut file = self.internal_open(filepath, false).await?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_exists() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = DefaultTorrentFileStorage::new(temp_path);
        let filepath = Path::new("test.mp4");

        assert_eq!(
            false,
            storage.exists(filepath),
            "expected the file to not exist"
        );

        storage.open(filepath).await.unwrap();
        assert_eq!(true, storage.exists(filepath), "expected the file to exist");
    }

    #[test]
    fn test_is_valid_filepath() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = DefaultTorrentFileStorage::new(temp_path);

        assert_eq!(
            true,
            storage.is_valid_filepath(storage.absolute_filepath("test")),
            "expected the filepath to be valid"
        );
        assert_eq!(
            false,
            storage.is_valid_filepath(storage.absolute_filepath("../test")),
            "expected the filepath to be invalid"
        );
    }

    #[tokio::test]
    async fn test_open() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = DefaultTorrentFileStorage::new(temp_path);
        let filepath = Path::new("test.mp4");

        let result = storage.open(filepath).await;

        assert_eq!(Ok(()), result);
    }

    #[tokio::test]
    async fn test_write_offset_larger_than_file() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let file = Path::new("test.mp4");
        let data = vec![1, 2, 3, 4, 5, 16, 88];
        let mut expected_result = vec![0; 128];
        expected_result.extend_from_slice(&data);
        let storage = DefaultTorrentFileStorage::new(temp_path);

        let result = storage.write(file, 128, &data).await;
        assert_eq!(Ok(()), result);

        let result = storage
            .read_to_end(file)
            .await
            .expect("Failed to read file");
        assert_eq!(135, result.len());
        assert_eq!(expected_result, result);
    }
}
