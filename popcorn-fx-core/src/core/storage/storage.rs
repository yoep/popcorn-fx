use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

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
                Err(StorageError::FileNotFound(filename.to_string()))
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

    async fn write_to<T: Serialize + Debug>(file: &mut tokio::fs::File, value: &T, path_string: &String) -> storage::Result<()> {
        trace!("Serializing storage data {:?}", value);
        match serde_json::to_string(value) {
            Ok(e) => {
                trace!("Writing to storage {:?}, {}", &path_string, &e);
                match file.write_all(e.as_bytes()).await {
                    Ok(_) => {
                        debug!("Storage file {} has been saved", path_string);
                        Ok(())
                    }
                    Err(e) => Err(StorageError::WritingFailed(path_string.clone(), e.to_string()))
                }
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
        let temp_path = temp_dir.into_path();
        let storage = Storage {
            directory: temp_path.clone(),
        };
        let settings = UiSettings::default();
        let expected_result = "{\"default_language\":\"en\",\"ui_scale\":{\"value\":1.0},\"start_screen\":\"MOVIES\",\"maximized\":false,\"native_window_enabled\":false}".to_string();

        let result = storage.write(filename.clone(), &settings);
        assert!(result.is_ok(), "expected no error to have occurred");
        let contents = read_temp_dir_file(temp_path, filename);

        assert_eq!(expected_result, contents)
    }

    #[tokio::test]
    async fn test_write_async() {
        init_logger();
        let filename = "test";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path();
        let storage = Storage {
            directory: temp_path.clone(),
        };
        let settings = UiSettings::default();
        let expected_result = "{\"default_language\":\"en\",\"ui_scale\":{\"value\":1.0},\"start_screen\":\"MOVIES\",\"maximized\":false,\"native_window_enabled\":false}".to_string();

        let result = storage.write_async(filename.clone(), &settings).await;
        assert!(result.is_ok(), "expected no error to have occurred");
        let contents = read_temp_dir_file(temp_path, filename);

        assert_eq!(expected_result, contents)
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
}