use std::fmt::Debug;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use log::{debug, error, trace, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::core::{block_in_place, storage};
use crate::core::storage::StorageError;

/// The storage module is responsible for storing and retrieving files from the file system.
///
/// It uses the home directory for the main files of the application.
///
/// The `Storage` struct is thread-safe and can be safely shared across multiple threads.
#[derive(Debug, Clone)]
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    /// Creates and returns a new instance of `StorageOptions` for configuring storage operations.
    ///
    /// # Returns
    ///
    /// A new instance of `StorageOptions` with the base path set to the current `Storage` instance's base path.
    ///
    /// # Examples
    ///
    /// ```
    /// use popcorn_fx_core::core::storage::Storage;
    ///
    /// let storage = Storage::from("/path/to/storage");
    ///
    /// let options = storage.options();
    ///
    /// // Configure storage options...
    ///
    /// // Create a SerializerStorage for storing and retrieving serialized data
    /// let serializer = options.serializer("data.json");
    ///
    /// // Write data to the storage
    /// let data = vec![1, 2, 3];
    /// serializer.write(&data).expect("Failed to write data");
    ///
    /// // Read data from the storage
    /// let retrieved_data: Vec<u8> = serializer.read().expect("Failed to read data");
    ///
    /// println!("Retrieved data: {:?}", retrieved_data);
    /// ```
    ///
    /// This example demonstrates how to use the `options` method to create a new `StorageOptions` instance for configuring storage operations.
    /// The `options` method is called on an existing `Storage` instance, and the resulting `StorageOptions` instance can be used to customize the behavior of storage operations.
    /// In this example, a `SerializerStorage` is created using the `serializer` method of the `StorageOptions` instance.
    /// Data is then written to the storage using the `write` method of `SerializerStorage`, and subsequently read using the `read` method.
    /// The retrieved data is printed to the console.
    pub fn options(&self) -> StorageOptions {
        StorageOptions::new(self.base_path.clone())
    }

    /// Deletes a file at the specified filepath.
    ///
    /// # Arguments
    ///
    /// * `filepath` - The path to the file to be deleted.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use popcorn_fx_core::core::storage::Storage;
    ///
    /// let storage = Storage::from("/path/to/storage");
    ///
    /// // Delete a file named "data.json" in the storage
    /// let result = storage.delete_path("data.json");
    ///
    /// match result {
    ///     Ok(()) => {
    ///         println!("File deleted successfully");
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Failed to delete file: {}", err);
    ///     }
    /// }
    /// ```
    ///
    /// This example demonstrates how to use the `delete` method to delete a file within the storage.
    /// The method takes the filepath as an argument and returns a `Result` indicating the success or failure of the operation.
    pub fn delete_path<P: AsRef<Path>>(&self, filepath: P) -> storage::Result<()> {
        Self::delete(self.base_path.join(filepath))
    }

    /// Clean the given directory path.
    ///
    /// This method removes all files within the directory but does not delete the directory itself.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to clean.
    ///
    /// # Returns
    ///
    /// An empty `Result` indicating success, or a `StorageError` if the directory couldn't be cleaned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::storage::Storage;
    /// let storage = Storage::from("/path/to/directory");
    ///
    /// match storage.clean_directory("/path/to/directory") {
    ///     Ok(()) => println!("Directory cleaned successfully."),
    ///     Err(err) => eprintln!("Failed to clean directory: {}", err),
    /// }
    /// ```
    ///
    /// This example demonstrates how to use the `clean_directory` method to remove all files within a directory. If the operation is successful, a success message is printed; otherwise, an error message is printed.
    pub fn clean_directory(path: impl AsRef<Path>) -> storage::Result<()> {
        let path_value = path.as_ref().to_str().unwrap().to_string();
        // check if the directory exist before we try to clean it
        if !path.as_ref().exists() {
            return Err(StorageError::NotFound(path_value));
        }

        let dir_entry = fs::read_dir(path)
            .map_err(|e| StorageError::IO(path_value, e.to_string()))?;
        for entry in dir_entry {
            match entry {
                Ok(path) => Self::delete(path.path())?,
                Err(e) => warn!("Unable to read directory entry, {}", e)
            }
        }

        Ok(())
    }

    /// Delete the given path from the system.
    ///
    /// This path can either point to a file or directory. In the case of a directory, the whole directory, including its contents, will be removed.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to delete.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error of type [storage::Error].
    pub fn delete<P: AsRef<Path>>(path: P) -> storage::Result<()> {
        let path = path.as_ref();
        let absolute_path = path.to_str().unwrap();
        debug!("Deleting path {}", absolute_path);

        if path.is_file() {
            trace!("Deleting filepath {}", absolute_path);
            fs::remove_file(path)
                .map_err(|e| StorageError::IO(absolute_path.to_string(), e.to_string()))
        } else {
            trace!("Deleting directory {}", absolute_path);
            fs::remove_dir_all(path)
                .map_err(|e| StorageError::IO(absolute_path.to_string(), e.to_string()))
        }
    }
}

impl From<&str> for Storage {
    fn from(value: &str) -> Self {
        Self {
            base_path: PathBuf::from(value),
        }
    }
}

impl From<&PathBuf> for Storage {
    fn from(value: &PathBuf) -> Self {
        Self {
            base_path: value.clone(),
        }
    }
}

/// Options for configuring storage behavior.
#[derive(Debug)]
pub struct StorageOptions {
    path: PathBuf,
    create: bool,
    make_dirs: bool,
}

impl StorageOptions {
    /// Creates and returns a new instance of `StorageOptions` with the initial path.
    ///
    /// # Arguments
    ///
    /// * `initial_path` - The initial path to set.
    ///
    /// # Returns
    ///
    /// A new `StorageOptions` instance.
    fn new<P: AsRef<Path>>(initial_path: P) -> Self {
        Self {
            path: PathBuf::from(initial_path.as_ref()),
            create: false,
            make_dirs: false,
        }
    }

    /// Appends a directory to the storage path.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory name to append to the storage path.
    pub fn directory(mut self, directory: &str) -> Self {
        self.path = self.path.join(directory);
        self
    }

    /// Sets whether the storage directory should be created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `create` - A boolean indicating whether the storage directory should be created if it doesn't exist.
    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    /// Sets whether the storage parent directories of the file should be created.
    ///
    /// # Arguments
    ///
    /// * `make_dirs` - A boolean indicating if parent directories should be created if they don't exist.
    pub fn make_dirs(mut self, make_dirs: bool) -> Self {
        self.make_dirs = make_dirs;
        self
    }

    /// Checks if the storage directory exists.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the storage directory exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Creates a `Serializer` storage instance with the provided filename.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename for the `SerializerStorage`.
    pub fn serializer<F: AsRef<str>>(self, filename: F) -> SerializerStorage {
        SerializerStorage {
            base: BaseStorage {
                path: self.path.join(filename.as_ref()),
                create: self.create,
                make_dirs: self.make_dirs,
            }
        }
    }

    /// Creates a `Binary` storage instance with the provided filename.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename for the `BinaryStorage`.
    pub fn binary<F: AsRef<str>>(self, filename: F) -> BinaryStorage {
        BinaryStorage {
            base: BaseStorage {
                path: self.path.join(filename.as_ref()),
                create: self.create,
                make_dirs: self.make_dirs,
            },
        }
    }
}

/// Base storage information for a file.
#[derive(Debug)]
struct BaseStorage {
    path: PathBuf,
    create: bool,
    make_dirs: bool,
}

impl BaseStorage {
    /// Checks if the file exists.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Returns the absolute path of the file as a string.
    ///
    /// # Returns
    ///
    /// The absolute path of the file as a string.
    pub fn absolute_path(&self) -> &str {
        self.path.to_str().unwrap()
    }

    /// Returns the path of the file as a `Path` reference.
    ///
    /// # Returns
    ///
    /// The path of the file as a `Path` reference.
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Opens the file in read mode.
    ///
    /// # Returns
    ///
    /// A `Result` containing the opened `File` if successful, or a `StorageError` if the file couldn't be opened.
    pub fn read_open(&self) -> storage::Result<File> {
        trace!("Opening storage file {}", self.absolute_path());
        OpenOptions::new()
            .read(true)
            .create(self.create)
            .open(self.path.as_path())
            .map_err(|e| {
                let absolute_path = self.absolute_path();
                trace!("File {} couldn't be opened, {}", absolute_path, e);

                if e.kind() == ErrorKind::NotFound {
                    StorageError::NotFound(absolute_path.to_string())
                } else {
                    StorageError::ReadingFailed(absolute_path.to_string(), e.to_string())
                }
            })
    }

    pub fn write_open(&self) -> storage::Result<File> {
        self.create_parent_directories_if_needed()?;

        trace!("Opening storage file {}", self.absolute_path());
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path.as_path())
            .map_err(|e| {
                let absolute_path = self.absolute_path();
                trace!("File {} couldn't be opened, {}", absolute_path, e);
                StorageError::WritingFailed(absolute_path.to_string(), e.to_string())
            })
    }

    pub async fn write_open_async(&self) -> storage::Result<tokio::fs::File> {
        self.create_parent_directories_if_needed()?;

        trace!("Opening storage file {}", self.absolute_path());
        tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path.as_path())
            .await
            .map_err(|e| {
                let absolute_path = self.absolute_path();
                trace!("File {} couldn't be opened, {}", absolute_path, e);
                StorageError::WritingFailed(absolute_path.to_string(), e.to_string())
            })
    }

    fn create_parent_directories_if_needed(&self) -> storage::Result<()> {
        if self.make_dirs {
            let parent = self.path.parent().expect("expected a parent directory to have been present for the file");
            let parent_absolute_path = parent.to_str().unwrap();
            trace!("Creating parent directories {}", parent_absolute_path);
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("Failed to create parent directories, {}", e);
                return Err(StorageError::IO(parent_absolute_path.to_string(), e.to_string()));
            }
        }

        Ok(())
    }
}

/// Storage for serializing and deserializing data.
#[derive(Debug)]
pub struct SerializerStorage {
    base: BaseStorage,
}

impl SerializerStorage {
    /// Checks if the storage file exists.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the storage file exists.
    pub fn exists(&self) -> bool {
        self.base.exists()
    }

    /// Reads the stored data from the storage file.
    ///
    /// # Returns
    ///
    /// The deserialized data if successful, or a `StorageError` if reading failed.
    ///
    /// # Generic Parameters
    ///
    /// * `T` - The type to deserialize the stored data into.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::storage::SerializerStorage;
    ///
    /// let storage = SerializerStorage::from_path("/path/to/storage.json");
    ///
    /// match storage.read::<Vec<u8>>() {
    ///     Ok(data) => println!("Data: {:?}", data),
    ///     Err(err) => eprintln!("Failed to read data: {}", err),
    /// }
    /// ```
    ///
    /// This example demonstrates how to use the `read` method to deserialize and read the stored data from the storage file. If the operation is successful, the deserialized data is printed; otherwise, an error message is printed.
    pub fn read<T>(self) -> storage::Result<T>
        where T: Serialize + DeserializeOwned {
        let mut file = self.base.read_open()?;

        trace!("Application file {:?} exists", &self.base.absolute_path());
        let mut data = String::new();
        file.read_to_string(&mut data).expect("unable to read file data");

        match serde_json::from_str::<T>(data.as_str()) {
            Ok(e) => {
                debug!("File {} has been loaded", self.base.absolute_path());
                Ok(e)
            }
            Err(e) => {
                debug!("File {} is invalid, {}", self.base.absolute_path(), &e);
                Err(StorageError::ReadingFailed(self.base.absolute_path().to_string(), e.to_string()))
            }
        }
    }

    /// Writes the given value to the storage file.
    ///
    /// The data will be stored under the storage file with the given `filename`.
    ///
    /// This method blocks the current thread until the write operation completes.
    /// Use `write_async` instead if you don't want to block the current thread.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to write to the storage file.
    ///
    /// # Returns
    ///
    /// The path of the storage file if successful, or a `StorageError` if writing failed.
    ///
    /// # Generic Parameters
    ///
    /// * `T` - The type of the value to write.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::storage::SerializerStorage;
    ///
    /// let storage = SerializerStorage::from_path("/path/to/storage.json");
    ///
    /// let data = vec![1, 2, 3];
    ///
    /// match storage.write(&data) {
    ///     Ok(path) => println!("Data written to: {:?}", path),
    ///     Err(err) => eprintln!("Failed to write data: {}", err),
    /// }
    /// ```
    ///
    /// This example demonstrates how to use the `write` method to serialize and write data to the storage file. If the operation is successful, the path of the storage file is printed; otherwise, an error message is printed.
    pub fn write<T>(self, value: &T) -> storage::Result<PathBuf>
        where T: Serialize + DeserializeOwned {
        block_in_place(async {
            self.write_async(value).await
        })
    }

    /// Writes the given value to the storage file asynchronously.
    ///
    /// The data will be stored under the storage file with the given `filename`.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to write to the storage file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the path of the storage file if successful, or a `StorageError` if writing failed.
    ///
    /// # Generic Parameters
    ///
    /// * `T` - The type of the value to write.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_core::core::storage::SerializerStorage;
    ///
    /// let mut rt = Runtime::new().expect("Failed to create Tokio runtime");
    ///
    /// let storage = SerializerStorage::from_path("/path/to/storage.json");
    ///
    /// let data = vec![1, 2, 3];
    ///
    /// let result = rt.block_on(async {
    ///     storage.write_async(&data).await
    /// });
    ///
    /// match result {
    ///     Ok(path) => println!("Data written to: {:?}", path),
    ///     Err(err) => eprintln!("Failed to write data: {}", err),
    /// }
    /// ```
    ///
    /// This example demonstrates how to use the `write_async` method to serialize and write data to the storage file asynchronously using the Tokio runtime. The `block_on` function is used to await the asynchronous operation and obtain the result. If the operation is successful, the path of the storage file is printed; otherwise, an error message is printed.
    pub async fn write_async<T>(self, value: &T) -> storage::Result<PathBuf>
        where T: Serialize + DeserializeOwned {
        let path_string = self.base.absolute_path();

        trace!("Opening storage file {}", path_string);
        let mut file = self.base.write_open_async().await?;
        self.write_to(&mut file, value).await
    }

    async fn write_to<T>(self, file: &mut tokio::fs::File, value: &T) -> storage::Result<PathBuf>
        where T: Serialize + DeserializeOwned {
        let display_path = self.base.absolute_path();

        trace!("Serializing storage data to {}", display_path);
        match serde_json::to_string(value) {
            Ok(e) => {
                trace!("Writing to storage {:?}, {}", &display_path, &e);
                file.write_all(e.as_bytes())
                    .await
                    .map_err(|e| StorageError::WritingFailed(display_path.to_string(), e.to_string()))?;
                debug!("Storage file {} has been saved", display_path);
                Ok(self.base.path.clone())
            }
            Err(e) => Err(StorageError::WritingFailed(display_path.to_string(), e.to_string()))
        }
    }
}

/// Binary storage for reading and writing binary data to files.
///
/// # Examples
///
/// ```no_run
/// use popcorn_fx_core::core::storage::Storage;
///
/// let storage = Storage::from("/path/to/storage");
///
/// // Create a BinaryStorage for reading and writing binary data
/// let binary_storage = storage.options().binary("data.bin");
///
/// // Write binary data to the storage
/// let data: Vec<u8> = vec![1, 2, 3];
/// binary_storage.write(&data).expect("Failed to write binary data");
///
/// // Read binary data from the storage
/// let retrieved_data = binary_storage.read().expect("Failed to read binary data");
///
/// println!("Retrieved data: {:?}", retrieved_data);
/// ```
///
/// This example demonstrates how to use the `BinaryStorage` struct to read and write binary data to files in storage.
#[derive(Debug)]
pub struct BinaryStorage {
    base: BaseStorage,
}

impl BinaryStorage {
    /// Checks if the file exists in the binary storage.
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    pub fn exists(&self) -> bool {
        self.base.exists()
    }

    /// Reads the binary data from the file in the binary storage.
    ///
    /// # Returns
    ///
    /// A `Result` containing the binary data if successful, or a `StorageError` if the read operation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::storage::Storage;
    ///
    /// let storage = Storage::from("/path/to/storage");
    ///
    /// let binary_storage = storage.options().binary("data.bin");
    ///
    /// let data = binary_storage.read().expect("Failed to read binary data");
    ///
    /// println!("Retrieved data: {:?}", data);
    /// ```
    ///
    /// This example demonstrates how to use the `read` method to read binary data from a file in the binary storage.
    /// The `read` method is called on a `BinaryStorage` instance, and it returns the binary data if the read operation is successful.
    pub fn read(self) -> storage::Result<Vec<u8>> {
        let mut buffer = vec![];
        let mut file = self.base.read_open()?;

        file.read_to_end(&mut buffer)
            .map_err(|e| {
                let absolute_path = self.base.absolute_path();
                error!("Failed to read file {}, {}", absolute_path, e);

                if e.kind() == ErrorKind::NotFound {
                    StorageError::NotFound(absolute_path.to_string())
                } else {
                    StorageError::ReadingFailed(absolute_path.to_string(), e.to_string())
                }
            })?;

        Ok(buffer)
    }

    /// Writes binary data to the file in the binary storage.
    ///
    /// # Arguments
    ///
    /// * `value` - The binary data to write.
    ///
    /// # Returns
    ///
    /// A `Result` containing the path of the file if successful, or a `StorageError` if the write operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use popcorn_fx_core::core::storage::Storage;
    ///
    /// let storage = Storage::from("/path/to/storage");
    ///
    /// let binary_storage = storage.options().binary("data.bin");
    ///
    /// let data: Vec<u8> = vec![1, 2, 3];
    /// binary_storage.write(&data).expect("Failed to write binary data");
    /// ```
    ///
    /// This example demonstrates how to use the `write` method to write binary data to a file in the binary storage.
    /// The `write` method is called on a `BinaryStorage` instance with the binary data to write as the argument.
    /// It returns the path of the file if the write operation is successful.
    pub fn write<V: AsRef<[u8]>>(self, value: V) -> storage::Result<PathBuf> {
        let mut file = self.base.write_open()?;

        debug!("Writing {} bytes to file {}", value.as_ref().len(), self.base.absolute_path());
        file.write_all(value.as_ref())
            .map_err(|e| {
                let absolute_path = self.base.absolute_path();
                error!("Failed to write to file {}, {}", absolute_path, e.to_string());
                StorageError::WritingFailed(absolute_path.to_string(), e.to_string())
            })?;

        Ok(self.base.path)
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    use crate::core::config::{PopcornSettings, SubtitleSettings, UiSettings};
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file_as_bytes, read_temp_dir_file_as_string, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_from_directory_should_use_given_path() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = PathBuf::from(temp_path);

        let storage = Storage::from(temp_path);

        assert_eq!(expected_result, storage.base_path)
    }

    #[test]
    fn test_read_settings() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let path = PathBuf::from(temp_path);
        let storage = Storage {
            base_path: path
        };

        let result = storage.options()
            .serializer("settings.json")
            .read::<PopcornSettings>();

        assert!(result.is_ok(), "Expected the storage reading to have succeeded")
    }

    #[test]
    fn test_write() {
        init_logger();
        let filename = "test";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };
        let settings = UiSettings::default();
        let expected_result = "{\"default_language\":\"en\",\"ui_scale\":{\"value\":1.0},\"start_screen\":\"MOVIES\",\"maximized\":false,\"native_window_enabled\":false}".to_string();

        let result = storage.options()
            .serializer(filename)
            .write(&settings);
        assert!(result.is_ok(), "expected no error to have occurred");
        let contents = read_temp_dir_file_as_string(&temp_dir, filename);

        assert_eq!(expected_result, contents)
    }

    #[test]
    fn test_write_async() {
        init_logger();
        let filename = "test.json";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };
        let settings = UiSettings::default();
        let runtime = Runtime::new().unwrap();

        let _ = runtime.block_on(storage.options()
            .serializer(filename)
            .write_async(&settings))
            .expect("expected no error to have been returned");
        let path = temp_dir.path().join(filename);

        assert!(path.exists(), "expected the storage {:?} exists", path);
    }

    #[test]
    fn test_write_invalid_storage() {
        init_logger();
        let storage = Storage {
            base_path: PathBuf::from("/invalid/file/path"),
        };
        let settings = SubtitleSettings::default();

        let result = storage.options()
            .serializer("my-random-filename.txt")
            .write(&settings);

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

    #[test]
    fn test_exists() {
        init_logger();
        let filename = "auto-resume.json";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };
        copy_test_file(temp_path, filename, None);

        assert_eq!(true, storage.options().serializer(filename).exists());
        assert_eq!(false, storage.options().serializer("lorem-ipsum.dolor").exists());
    }

    #[test]
    fn test_binary_storage_read() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "simple.jpg";
        let bytes = read_test_file_to_bytes("simple.jpg");
        copy_test_file(temp_path, "simple.jpg", None);
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };

        match storage.options()
            .binary(filename)
            .read() {
            Ok(result) => assert_eq!(bytes, result),
            Err(e) => assert!(false, "expected the read operation to succeed, {}", e),
        }
    }

    #[test]
    fn test_binary_storage_write() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "my-simple-test.jpg";
        let bytes = read_test_file_to_bytes("simple.jpg");
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };

        if let Err(e) = storage.options()
            .binary(filename)
            .write(&bytes) {
            assert!(false, "expected the write operation to succeed, {}", e)
        }
        let result = read_temp_dir_file_as_bytes(&temp_dir, filename);

        assert_eq!(bytes, result)
    }

    #[test]
    fn test_delete_path() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let path = copy_test_file(temp_path, "simple.jpg", None);
        let storage = Storage {
            base_path: PathBuf::from(temp_path),
        };

        assert_eq!(Ok(()), storage.delete_path(path))
    }

    #[test]
    fn test_delete() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filepath = copy_test_file(temp_path, "image.png", None);
        let directory_path = copy_test_file(temp_path, "image.png", Some("lorem/image.png"));

        Storage::delete(directory_path.as_str()).unwrap();
        assert_eq!(false, PathBuf::from(directory_path).exists(), "expected the directory to have been removed");

        Storage::delete(filepath.as_str()).unwrap();
        assert_eq!(false, PathBuf::from(filepath).exists(), "expected the file to have been removed");
    }
}