use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use log::{debug, trace};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::core::storage;
use crate::core::storage::StorageError;

const DEFAULT_APP_DIRECTORY: &str = ".popcorn-time";

/// The storage is responsible for storing & retrieving files from the file system.
/// It uses the home directory for the main files of the application.
#[derive(Debug)]
pub struct Storage {
    directory: PathBuf,
}

impl Storage {
    /// Create a storage from the default application directory.
    /// This directory is always located in the home dir of the user.
    /// The addition of [DEFAULT_APP_DIRECTORY] will be added to the home directory path.
    ///
    /// It will `panic` if no home directory is found for the current user.
    pub fn new() -> Self {
        let mut directory = home::home_dir().expect("expected a home dir to exist");
        directory.push(DEFAULT_APP_DIRECTORY);

        debug!("Using application storage path {:?}", directory);
        Self {
            directory,
        }
    }

    /// Create a storage from the given directory path.
    /// It will use the given directory path without any additions to it.
    ///
    /// This means that path `/opt/popcorn` will be used as `/opt/popcorn/settings.json`
    pub fn from_directory(directory: &str) -> Self {
        Self {
            directory: PathBuf::from(directory),
        }
    }

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
                        Err(StorageError::CorruptRead(filename.to_string(), e.to_string()))
                    }
                }
            }
            Err(e) => {
                trace!("Application file {} does not exist, {}", filename, e);
                Err(StorageError::FileNotFound(filename.to_string()))
            }
        }
    }

    /// Write the value to the given storage filename.
    ///
    /// It returns an error when the file failed to be written.
    pub async fn write<T: Serialize>(&self, filename: &str, value: &T) -> storage::Result<()> {
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

    async fn write_to<T: Serialize>(file: &mut tokio::fs::File, value: &T, path_string: &String) -> storage::Result<()> {
        trace!("Serializing the data to write");
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
            Err(e) => Err(StorageError::CorruptWrite(e.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::{PopcornSettings, UiSettings};
    use crate::testing::{init_logger, read_temp_dir_file, test_resource_directory};

    use super::*;

    #[test]
    fn test_from_directory_should_use_given_path() {
        let resource_directory = test_resource_directory();
        let expected_result = PathBuf::from(resource_directory.to_str().expect("expected the testing directory to be valid"));

        let storage = Storage::from_directory(resource_directory.to_str().expect("expected path to be valid"));

        assert_eq!(expected_result, storage.directory)
    }

    #[test]
    fn test_read_settings() {
        init_logger();
        let resource_directory = test_resource_directory();
        let path = PathBuf::from(resource_directory.to_str().expect("expected the testing directory to be valid"));
        let storage = Storage {
            directory: path
        };

        let result = storage.read::<PopcornSettings>("simple-settings.json");

        assert!(result.is_ok(), "Expected the storage reading to have succeeded")
    }

    #[tokio::test]
    async fn test_write() {
        init_logger();
        let filename = "test";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path();
        let storage = Storage {
            directory: temp_path.clone(),
        };
        let settings = UiSettings::default();
        let expected_result = "{\"default_language\":\"en\",\"ui_scale\":{\"value\":1.0},\"start_screen\":\"MOVIES\",\"maximized\":false,\"native_window_enabled\":false}".to_string();

        let result = storage.write(filename.clone(), &settings).await;
        assert!(result.is_ok(), "expected no error to have occurred");
        let contents = read_temp_dir_file(temp_path, filename);

        assert_eq!(expected_result, contents)
    }
}