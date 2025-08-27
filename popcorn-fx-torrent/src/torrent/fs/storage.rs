use crate::torrent::fs::{Error, Result};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use tokio::sync::RwLock;
use tokio::task::spawn_blocking;

/// The underlying torrent file storage handler.
/// It stores torrent data on the current file system.
#[derive(Debug)]
pub struct TorrentFileStorage {
    base_path: PathBuf,
    handles: RwLock<HashMap<PathBuf, Arc<fs::File>>>,
}

impl TorrentFileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            handles: Default::default(),
        }
    }

    /// Check if the given file exists within the storage.
    /// This doesn't check if the file contains any actual data.
    pub fn exists<P: AsRef<Path>>(&self, filepath: P) -> bool {
        let absolute_filepath = self.absolute_filepath(&filepath);
        self.is_valid_filepath(&absolute_filepath) && absolute_filepath.exists()
    }

    /// Get the path of this storage.
    /// This is the path under which all file operations are performed.
    pub fn path(&self) -> &Path {
        self.base_path.as_path()
    }

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
    pub async fn write(&self, filepath: &Path, offset: u64, bytes: &[u8]) -> Result<()> {
        let absolute_path = self.absolute_filepath(filepath);
        self.assert_valid_filepath(&absolute_path)?;
        let file = self.get_or_open_std(absolute_path).await?;

        let data = bytes.to_vec();
        spawn_blocking(move || -> Result<()> {
            let needed_len = offset.checked_add(data.len() as u64).ok_or_else(|| {
                Error::Io(io::Error::new(io::ErrorKind::Other, "file length overflow"))
            })?;
            let current_len = file.metadata()?.len();
            if current_len < needed_len {
                file.set_len(needed_len)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::FileExt;
                let mut written = 0;
                while written < data.len() {
                    let n = file.write_at(&data[written..], offset + written as u64)?;
                    if n == 0 {
                        return Err(Error::Io(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "unable to write data",
                        )));
                    }
                    written += n;
                }
                Ok(())
            }

            #[cfg(windows)]
            {
                use std::os::windows::fs::FileExt;
                let mut written = 0;
                while written < data.len() {
                    let n = file.seek_write(&data[written..], offset + written as u64)?;
                    if n == 0 {
                        return Err(Error::Io(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "unable to write data",
                        )));
                    }
                    written += n;
                }
                Ok(())
            }
        })
        .await
        .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?
    }

    /// Read the given range of bytes from the torrent filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to read from.
    /// * `range` - The range of bytes to read from the file.
    ///
    /// # Returns
    ///
    /// It returns the requested byte range.
    pub async fn read(&self, filepath: &Path, range: Range<usize>) -> Result<Vec<u8>> {
        let expected_bytes = range.len();
        let range = range.start as u64..range.end as u64;
        let (bytes_read, bytes) = self.internal_read(filepath, range).await?;

        if bytes_read != expected_bytes {
            Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("EOF reached at {}/{}", bytes_read, expected_bytes),
            )))
        } else {
            Ok(bytes)
        }
    }

    /// Reads a specified range of bytes from the torrent filepath,
    /// padding with `0` if the file is smaller than the requested range.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to read from.
    /// * `range` - The range of bytes to read from the file.
    ///
    /// # Returns
    ///
    /// It returns the requested byte range, padded with `0`.
    pub async fn read_with_padding(&self, filepath: &Path, range: Range<usize>) -> Result<Vec<u8>> {
        let range = range.start as u64..range.end as u64;
        let (_, bytes) = self.internal_read(filepath, range).await?;
        Ok(bytes)
    }

    /// Read all bytes from the torrent filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The relative filepath within this storage to read from
    ///
    /// # Returns
    ///
    /// Returns the bytes read from the file if successful
    pub async fn read_to_end(&self, filepath: &Path) -> Result<Vec<u8>> {
        let absolute_path = self.absolute_filepath(filepath);
        self.assert_valid_filepath(&absolute_path)?;
        Ok(tokio::fs::read(&absolute_path).await?)
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

    /// Assert if the given filepath is valid within the storage.
    fn assert_valid_filepath<P: AsRef<Path>>(&self, filepath: P) -> Result<()> {
        if !self.is_valid_filepath(&filepath) {
            return Err(Error::InvalidFilepath(filepath.as_ref().to_path_buf()));
        }

        Ok(())
    }

    async fn get_or_open_std(&self, absolute_path: PathBuf) -> Result<Arc<fs::File>> {
        if let Some(file) = self.handles.read().await.get(&absolute_path).cloned() {
            return Ok(file);
        }

        let file_path = absolute_path.clone();
        let parent = absolute_path.parent().map(Path::to_path_buf);
        let file = spawn_blocking(move || -> Result<fs::File> {
            if let Some(parent_dir) = parent.as_ref() {
                fs::create_dir_all(parent_dir)?;
            }

            let file = fs::OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .truncate(false)
                .open(&file_path)?;

            Ok(file)
        })
        .await
        .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))??;

        let file = Arc::new(file);
        self.handles
            .write()
            .await
            .insert(absolute_path.clone(), file.clone());
        Ok(file)
    }

    /// Try to read the byte range from the specified file.
    /// This method applies padding to the missing bytes.
    async fn internal_read(&self, filepath: &Path, range: Range<u64>) -> Result<(usize, Vec<u8>)> {
        let absolute_path = self.absolute_filepath(filepath);
        self.assert_valid_filepath(&absolute_path)?;
        let file = self.get_or_open_std(absolute_path).await?;

        let len = (range.end - range.start) as usize;
        spawn_blocking(move || -> Result<(usize, Vec<u8>)> {
            // pre-allocate the buffer
            let mut buffer = vec![0u8; len];
            let offset = range.start;
            // check if the offset is available within the file
            let file_len = file.metadata()?.len();
            if offset >= file_len {
                return Ok((0, buffer));
            }

            let bytes_to_read = ((file_len - offset) as usize).min(buffer.len());

            #[cfg(unix)]
            {
                use std::os::unix::fs::FileExt;
                let bytes_read = file.read_at(&mut buffer[..bytes_to_read], offset)?;
                Ok((bytes_read, buffer))
            }

            #[cfg(windows)]
            {
                use std::os::windows::fs::FileExt;
                let bytes_read = file.seek_read(&mut buffer[..bytes_to_read], offset)?;
                Ok((bytes_read, buffer))
            }
        })
        .await
        .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?
    }

    /// Get the canonicalized path for the given path.
    /// This function traverses the path components and resolves ".." and "." appropriately.
    /// It returns the resulting path.
    fn canonicalize_unchecked(path: &Path) -> PathBuf {
        let mut result = PathBuf::new();

        // Traverse the path components and resolve ".." and "." appropriately
        for component in path.components() {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::init_logger;

    use popcorn_fx_core::testing::copy_test_file;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_exists() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = TorrentFileStorage::new(temp_path);
        let filepath = Path::new("test.mp4");
        let absolute_filepath = storage.absolute_filepath(filepath);

        assert_eq!(
            false,
            storage.exists(filepath),
            "expected the file to not exist"
        );

        storage.get_or_open_std(absolute_filepath).await.unwrap();
        assert_eq!(true, storage.exists(filepath), "expected the file to exist");
    }

    #[test]
    fn test_is_valid_filepath() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = TorrentFileStorage::new(temp_path);

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
    async fn test_write_offset_larger_than_file() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let file = Path::new("test.mp4");
        let data = vec![1, 2, 3, 4, 5, 16, 88];
        let mut expected_result = vec![0; 128];
        expected_result.extend_from_slice(&data);
        let storage = TorrentFileStorage::new(temp_path);

        let result = storage.write(file, 128, &data).await;
        assert_eq!(Ok(()), result);

        let result = storage
            .read_to_end(file)
            .await
            .expect("Failed to read file");
        assert_eq!(135, result.len());
        assert_eq!(expected_result, result);
    }

    #[tokio::test]
    async fn test_read() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "debian-12.4.0-amd64-DVD-1.iso";
        copy_test_file(temp_path, "piece-1.iso", Some(filename));
        let storage = TorrentFileStorage::new(temp_path);

        let result = storage
            .read(filename.as_ref(), 0..30)
            .await
            .expect("expected the bytes to have been returned");
        assert_eq!(30, result.len());

        let result = storage.read(filename.as_ref(), 0..512000).await;
        let err_text = result
            .err()
            .expect("expected an error to have been returned")
            .to_string();
        assert_eq!(
            "an io error occurred, EOF reached at 262144/512000".to_string(),
            err_text
        );
    }

    #[tokio::test]
    async fn test_read_with_padding() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "debian-12.4.0-amd64-DVD-1.iso";
        copy_test_file(temp_path, "piece-1.iso", Some(filename));
        let storage = TorrentFileStorage::new(temp_path);

        let result = storage
            .read_with_padding(filename.as_ref(), 0..128)
            .await
            .expect("expected the bytes to have been returned");
        assert_eq!(128, result.len());

        let result = storage
            .read_with_padding(filename.as_ref(), 0..512000)
            .await
            .expect("expected the bytes to have been returned");
        assert_eq!(512000, result.len());
    }
}
