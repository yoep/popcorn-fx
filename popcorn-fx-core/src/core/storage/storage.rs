use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use log::{debug, trace};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::core::{block_in_place, storage};
use crate::core::storage::StorageError;

/// The storage is responsible for storing & retrieving files from the file system.
/// It uses the home directory for the main files of the application.
#[derive(Debug, Clone)]
pub struct Storage {
    directory: PathBuf,
}

impl Storage {
    /// Read the contents from the given filename from within the app directory.
    ///
    /// It returns the deserialized struct on success, else the [StorageError] that occurred.
    pub fn read<T>(&self, filename: &str) -> storage::Result<T>
        where T: DeserializeOwned {
        let path = self.directory.clone()
            .join(filename);

        match File::open(&path) {
            Ok(mut file) => {
                trace!("Application file {:?} exists", &path);
                let mut data = String::new();
                file.read_to_string(&mut data).expect("unable to read file data");

                match serde_json::from_str::<T>(data.as_str()) {
                    Ok(e) => {
                        debug!("Application file {} loaded", filename);
                        Ok(e)
                    }
                    Err(e) => {
                        debug!("Application file {} is invalid, {}", filename, &e);
                        Err(StorageError::ReadingFailed(filename.to_string(), e.to_string()))
                    }
                }
            }
            Err(e) => {
                trace!("Application file {} does not exist, {}", filename, e);
                Err(StorageError::NotFound(filename.to_string()))
            }
        }
    }

    /// Write the given value to the storage.
    /// It will be stored under the storage with the given `filename`.
    ///
    /// This method will block the current thread until it completes.
    /// Use [Storage::write_async] instead if you don't want to block the current thread.
    pub fn write<T: Serialize + Debug>(&self, filename: &str, value: &T) -> storage::Result<()> {
        block_in_place(async {
            self.write_async(filename, value).await
        })
    }

    /// Write the given value to the storage.
    /// It will be stored under the storage with the given `filename`.
    pub async fn write_async<T: Serialize + Debug>(&self, filename: &str, value: &T) -> storage::Result<()> {
        let path = self.directory.clone()
            .join(filename);
        let path_string = path.to_str().expect("expected path to be valid").to_string();

        trace!("Opening storage file {:?}", path);
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path).await {
            Ok(mut file) => {
                Self::write_to(&mut file, value, &path_string).await
            }
            Err(e) => {
                Err(StorageError::WritingFailed(path_string, e.to_string()))
            }
        }
    }

    /// Clean the given directory path.
    /// This will not delete the directory itself, only the files within the directory.
    ///
    /// It returns a [StorageError] when the directory couldn't be cleaned.
    pub fn clean_directory(path: impl AsRef<Path>) -> storage::Result<()> {
        let path_value = path.as_ref().to_str().unwrap().to_string();
        // check if the directory exist before we try to clean it
        if !path.as_ref().exists() {
            return Err(StorageError::NotFound(path_value));
        }

        let dir_entry = fs::read_dir(path)
            .map_err(|e| StorageError::IO(path_value, e.to_string()))?;
        for file in dir_entry {
            let filepath = file.expect("expected path entry to be valid").path();

            // check if the path is an actual file
            if filepath.is_file() {
                trace!("Removing file {:?}", filepath);
                fs::remove_file(&filepath).map_err(|e| {
                    StorageError::IO(filepath.to_str().unwrap().to_string(), e.to_string())
                })?;
            } else {
                trace!("Removing directory {:?}", filepath);
                let filepath_value = filepath.to_str().unwrap().to_string();
                fs::remove_dir_all(filepath).map_err(|e| StorageError::IO(filepath_value, e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn write_to<T: Serialize + Debug>(file: &mut tokio::fs::File, value: &T, path_string: &String) -> storage::Result<()> {
        trace!("Serializing storage data {:?}", value);
        match serde_json::to_string(value) {
            Ok(e) => {
                trace!("Writing to storage {:?}, {}", &path_string, &e);
                file.write_all(e.as_bytes())
                    .await
                    .map_err(|e| StorageError::WritingFailed(path_string.clone(), e.to_string()))?;
                debug!("Storage file {} has been saved", path_string);
                Ok(())
            }
            Err(e) => Err(StorageError::WritingFailed(path_string.clone(), e.to_string()))
        }
    }
}

impl From<&str> for Storage {
    fn from(value: &str) -> Self {
        Self {
            directory: PathBuf::from(value),
        }
    }
}

impl From<&PathBuf> for Storage {
    fn from(value: &PathBuf) -> Self {
        Self {
            directory: value.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    use crate::core::config::{PopcornSettings, SubtitleSettings, UiSettings};
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file};

    use super::*;

    #[test]
    fn test_from_directory_should_use_given_path() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = PathBuf::from(temp_path);

        let storage = Storage::from(temp_path);

        assert_eq!(expected_result, storage.directory)
    }

    #[test]
    fn test_read_settings() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let path = PathBuf::from(temp_path);
        let storage = Storage {
            directory: path
        };

        let result = storage.read::<PopcornSettings>("settings.json");

        assert!(result.is_ok(), "Expected the storage reading to have succeeded")
    }

    #[test]
    fn test_write() {
        init_logger();
        let filename = "test";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage {
            directory: PathBuf::from(temp_path),
        };
        let settings = UiSettings::default();
        let expected_result = "{\"default_language\":\"en\",\"ui_scale\":{\"value\":1.0},\"start_screen\":\"MOVIES\",\"maximized\":false,\"native_window_enabled\":false}".to_string();

        let result = storage.write(filename.clone(), &settings);
        assert!(result.is_ok(), "expected no error to have occurred");
        let contents = read_temp_dir_file(&temp_dir, filename);

        assert_eq!(expected_result, contents)
    }

    #[test]
    fn test_write_async() {
        init_logger();
        let filename = "test.json";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage {
            directory: PathBuf::from(temp_path),
        };
        let settings = UiSettings::default();
        let runtime = Runtime::new().unwrap();

        let _ = runtime.block_on(storage.write_async(filename.clone(), &settings))
            .expect("expected no error to have been returned");
        let path = temp_dir.path().join(filename);

        assert!(path.exists(), "expected the storage {:?} exists", path);
    }

    #[test]
    fn test_write_invalid_storage() {
        init_logger();
        let storage = Storage {
            directory: PathBuf::from("/invalid/file/path"),
        };
        let settings = SubtitleSettings::default();

        let result = storage.write("my-random-filename.txt", &settings);

        assert_eq!(true, result.is_err(), "expected an error to be returned");
        match result.err().unwrap() {
            StorageError::WritingFailed(_, _) => {}
            _ => assert!(false, "expected StorageError::WritingFailed to be returned")
        }
    }

    #[test]
    fn test_clean_directory() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "auto-resume.json", None);

        Storage::clean_directory(Path::new(temp_path))
            .expect("expected the directory to be cleaned");

        assert_eq!(true, temp_dir.path().read_dir().unwrap().next().is_none())
    }

    #[test]
    fn test_clean_directory_non_existing_path() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        let result = Storage::clean_directory(PathBuf::from(temp_path).join("lorem"))
            .err().expect("expected an error to be returned");

        match result {
            StorageError::NotFound(_) => {}
            _ => assert!(false, "expected StorageError::NotFound")
        }
    }
}