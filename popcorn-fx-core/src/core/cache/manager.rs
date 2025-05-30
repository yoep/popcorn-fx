use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::cache;
use crate::core::cache::info::{CacheEntry, CacheInfo};
use crate::core::cache::strategies::{CacheFirstStrategy, CacheLastStrategy};
use crate::core::cache::{CacheError, CacheExecutionError, CacheParserError};
use crate::core::storage::{Storage, StorageError};
use chrono::Duration;
use log::{debug, error, trace, warn};
use ring::digest;
use ring::digest::digest;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const DIRECTORY: &str = "cache";
const FILENAME: &str = "cache.json";
const EXTENSION: &str = ".cache";

/// Specifies the type of caching behavior to use.
#[derive(Debug, Clone)]
pub enum CacheType {
    /// The cache will be used first, and the closure will only be invoked if the cache is not present.
    CacheFirst,
    /// The closure will be used first, and the cache will be used if the closure results in an `std::error::Error`.
    CacheLast,
}

/// Options for configuring caching behavior.
#[derive(Debug, Clone)]
pub struct CacheOptions {
    /// The type of caching behavior to use.
    pub cache_type: CacheType,
    /// The duration for which the cached data will be considered valid before expiring.
    pub expires_after: Duration,
}

/// The `CacheManager` is responsible for managing cache operations and providing a convenient API for working with caches.
///
/// It allows you to create, execute, and manage cache operations asynchronously. The `CacheManager` is thread-safe and can be safely shared across multiple threads.
#[derive(Debug, Clone)]
pub struct CacheManager {
    inner: Arc<InnerCacheManager>,
}

impl CacheManager {
    /// Creates a new `CacheManager` instance.
    ///
    /// # Arguments
    ///
    /// * `storage_path` - The storage path for cache operations.
    /// * `runtime` - The runtime used for executing asynchronous operations.
    ///
    /// # Returns
    ///
    /// A new `CacheManager` instance.
    pub fn new(storage_path: &str) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let instance = Self {
            inner: Arc::new(InnerCacheManager::new(storage_path, command_sender)),
        };

        let inner_main = instance.inner.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver).await;
        });

        instance.run_cleanup();
        instance
    }

    /// Returns a builder for creating a `CacheManager` instance with customized options.
    ///
    /// # Returns
    ///
    /// A `CacheManagerBuilder` instance.
    pub fn builder() -> CacheManagerBuilder {
        CacheManagerBuilder::default()
    }

    /// Starts a new cache operation which allows the usage of the cache managed by this manager.
    ///
    /// # Returns
    ///
    /// A `CacheOperation` instance that can be used to configure and execute cache operations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use chrono::Duration;
    /// use popcorn_fx_core::core::cache::{CacheManager, CacheManagerBuilder, CacheOptions, CacheType};
    ///
    /// let cache_manager = CacheManagerBuilder::default()
    ///     .storage_path("/path/to/cache")
    ///     .build();
    ///
    /// let data = cache_manager.operation()
    ///     .name("my_cache".to_string())
    ///     .key("my_key".to_string())
    ///     .options(CacheOptions {
    ///         cache_type: CacheType::CacheFirst,
    ///         expires_after: Duration::days(10),
    ///     })
    ///     .execute(|| {
    ///         // Perform cache operation here
    ///         Ok(vec![1, 2, 3])
    ///     });
    /// ```
    pub fn operation(&self) -> CacheOperation {
        CacheOperation::new(self.inner.clone())
    }

    /// Executes a cache operation asynchronously.
    ///
    /// This method allows you to execute a cache operation with the specified name, key, options, and operation.
    /// The operation is a closure that takes no arguments and returns a `Result<T, E>`, where `T` is the type of the cache operation result and `E` is the error type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name associated with the cache operation.
    /// * `key` - The key associated with the cache operation.
    /// * `options` - The cache options for the cache operation.
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    ///
    /// The result of the cache operation, wrapped in a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use chrono::Duration;
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_core::core::cache::{CacheManager, CacheManagerBuilder, CacheOptions, CacheType};
    ///
    /// let cache_manager = CacheManagerBuilder::default()
    ///     .storage_path("/path/to/cache")
    ///     .build();
    ///
    /// let options = CacheOptions {
    ///         cache_type: CacheType::CacheFirst,
    ///         expires_after: Duration::days(10),
    ///     };
    /// let result = cache_manager.execute("my_cache", "my_key", options, || {
    ///     // Perform cache operation here
    ///     Ok(vec![1, 2, 3])
    /// });
    ///
    /// match result {
    ///     Ok(data) => {
    ///         // Cache operation succeeded
    ///         println!("Cache operation result: {:?}", data);
    ///     }
    ///     Err(error) => {
    ///         // Cache operation failed
    ///         eprintln!("Cache operation failed: {:?}", error);
    ///     }
    /// }
    /// ```
    pub async fn execute<T, E, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]> + From<Vec<u8>>,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        self.inner.execute(name, key, options, operation).await
    }

    /// Executes a cache operation with a mapper function asynchronously.
    ///
    /// This method allows you to execute a cache operation with the specified name, key, options, mapper function, and operation. The operation is a closure that takes no arguments and returns a `Result<T, E>`, where `T` is the type of the cache operation result and `E` is the error type. The mapper function is a closure that takes the raw cache data as input and returns a mapped result.
    ///
    /// # Arguments
    ///
    /// * `name` - The name associated with the cache operation.
    /// * `key` - The key associated with the cache operation.
    /// * `options` - The cache options for the cache operation.
    /// * `mapper` - The mapper function to apply to the cache operation result.
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    ///
    /// The result of the cache operation, wrapped in a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use chrono::Duration;
    /// use popcorn_fx_core::core::cache::{CacheManager, CacheOptions, CacheType};
    ///
    /// async fn example(cache_manager: CacheManager) {
    ///     let options = CacheOptions {
    ///             cache_type: CacheType::CacheFirst,
    ///             expires_after: Duration::days(10),
    ///     };
    ///     let result = cache_manager.execute_with_mapper("my_cache", "my_key", options, |data| {
    ///         // Map the cache data to another type
    ///         Ok(String::from_utf8_lossy(&data).to_string())
    ///     }, || {
    ///         // Perform cache operation here
    ///         Ok("lorem ipsum".to_string())
    ///     }).await;
    ///
    ///     match result {
    ///         Ok(mapped_data) => {
    ///             // Cache operation and mapping succeeded
    ///             println!("Mapped cache operation result: {}", mapped_data);
    ///         }
    ///         Err(error) => {
    ///             // Cache operation or mapping failed
    ///             eprintln!("Cache operation failed: {:?}", error);
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn execute_with_mapper<T, E, M, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        mapper: M,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]> + From<Vec<u8>>,
        E: Error,
        M: FnOnce(Vec<u8>) -> Result<T, E>,
        O: Future<Output = Result<T, E>>,
    {
        self.inner
            .execute_with_mapper(name, key, options, mapper, operation)
            .await
    }

    /// Runs the cleanup task in a separate thread.
    /// This invocation doesn't wait for the task to complete.
    fn run_cleanup(&self) {
        self.inner.send_command(CacheManagerCommand::ExecuteCleanup);
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

/// A builder for creating a `CacheManager` instance with customizable options.
#[derive(Debug, Default)]
pub struct CacheManagerBuilder {
    storage_path: Option<String>,
}

impl CacheManagerBuilder {
    /// Sets the storage path for the cache manager.
    ///
    /// # Arguments
    ///
    /// * `path` - The storage path for cache operations.
    ///
    /// # Returns
    ///
    /// The updated `CacheManagerBuilder` instance.
    pub fn storage_path<P: AsRef<str>>(mut self, path: P) -> Self {
        self.storage_path = Some(path.as_ref().to_string());
        self
    }

    /// Builds and returns a new `CacheManager` instance.
    ///
    /// # Panics
    ///
    /// This method will panic if the storage path is not set.
    ///
    /// # Returns
    ///
    /// A new `CacheManager` instance.
    pub fn build(self) -> CacheManager {
        let storage_path = self.storage_path.expect("Storage path is required.");

        CacheManager::new(storage_path.as_str())
    }
}

#[derive(Debug, PartialEq)]
enum CacheManagerCommand {
    /// Execute a cache cleanup cycle
    ExecuteCleanup,
}

#[derive(Debug)]
struct InnerCacheManager {
    storage: Storage,
    cache_info: Mutex<CacheInfo>,
    command_sender: UnboundedSender<CacheManagerCommand>,
    cancellation_token: CancellationToken,
}

impl InnerCacheManager {
    fn new(storage_path: &str, command_sender: UnboundedSender<CacheManagerCommand>) -> Self {
        let storage_path = PathBuf::from(storage_path).join(DIRECTORY);
        let storage = Storage::from(&storage_path);
        let info = storage
            .options()
            .serializer(FILENAME)
            .read::<CacheInfo>()
            .map(|e| {
                debug!("Using existing cache information");
                e
            })
            .or_else(|e| {
                debug!("Creating cache info, reason: {}", e);
                Ok::<CacheInfo, StorageError>(CacheInfo::default())
            })
            .unwrap();

        Self {
            storage,
            cache_info: Mutex::new(info),
            command_sender,
            cancellation_token: Default::default(),
        }
    }

    async fn start(&self, mut command_receiver: UnboundedReceiver<CacheManagerCommand>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
            }
        }
        if let Err(e) = self.write_cache_info().await {
            warn!("Cache manager failed to write cache on shutdown, {}", e);
        }
        debug!("Cache manager main loop ended");
    }

    async fn handle_command(&self, command: CacheManagerCommand) {
        match command {
            CacheManagerCommand::ExecuteCleanup => self.execute_cleanup().await,
        }
    }

    async fn execute<T, E, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]> + From<Vec<u8>>,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        self.internal_execute(name, key, options, operation)
            .await
            .map(|e| T::from(e))
    }

    async fn execute_serializer<T, E, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: Serialize + DeserializeOwned,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        let operation = async move {
            match operation.await {
                Ok(e) => serde_json::to_string::<T>(&e)
                    .map(|e| e.as_bytes().to_vec())
                    .map_err(|e| CacheParserError::Parsing(e.to_string())),
                Err(e) => Err(CacheParserError::Operation(e)),
            }
        };
        let output_mapping: fn(Vec<u8>) -> Result<T, CacheParserError<E>> = |e: Vec<u8>| {
            serde_json::from_slice::<T>(e.as_slice())
                .map_err(|e| CacheParserError::Parsing(e.to_string()))
        };

        match self
            .internal_execute(name, key, options.clone(), operation)
            .await
        {
            Ok(e) => {
                debug!("Invoking cache mapper for cache {} entry {}", name, key);
                output_mapping(e).map_err(|e| Self::map_cache_parser_error(e))
            }
            Err(error) => {
                if let CacheExecutionError::Operation(_) = &error {
                    debug!("Operation of {} failed, trying to load cached data", name);
                    let mut options = options;
                    options.expires_after = Duration::days(90);
                    self.read(name, key, &options)
                        .await
                        .map_err(|e| {
                            return if let CacheError::NotFound(_) = e {
                                match error {
                                    CacheExecutionError::Operation(e) => {
                                        Self::map_cache_parser_error(e)
                                    }
                                    CacheExecutionError::Mapping(e) => {
                                        Self::map_cache_parser_error(e)
                                    }
                                    CacheExecutionError::Cache(inner) => {
                                        CacheExecutionError::Cache(inner)
                                    }
                                }
                            } else {
                                CacheExecutionError::Cache(e)
                            };
                        })
                        .and_then(|e| {
                            output_mapping(e).map_err(|err| {
                                CacheExecutionError::Cache(CacheError::Parsing(err.to_string()))
                            })
                        })
                } else {
                    match error {
                        CacheExecutionError::Operation(e) => Err(Self::map_cache_parser_error(e)),
                        CacheExecutionError::Mapping(e) => Err(Self::map_cache_parser_error(e)),
                        CacheExecutionError::Cache(inner) => Err(CacheExecutionError::Cache(inner)),
                    }
                }
            }
        }
    }

    async fn execute_with_mapper<T, E, M, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        output_mapping: M,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]>,
        E: Error,
        M: FnOnce(Vec<u8>) -> Result<T, E>,
        O: Future<Output = Result<T, E>>,
    {
        match self.internal_execute(name, key, options, operation).await {
            Ok(e) => {
                debug!("Invoking cache mapper for cache {} entry {}", name, key);
                output_mapping(e).map_err(|e| CacheExecutionError::Mapping(e))
            }
            Err(e) => Err(e),
        }
    }

    async fn internal_execute<T, E, O>(
        &self,
        name: &str,
        key: &str,
        options: CacheOptions,
        operation: O,
    ) -> Result<Vec<u8>, CacheExecutionError<E>>
    where
        T: AsRef<[u8]>,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        trace!("Executing cache operation for {} with key {}", name, key);
        let cache_entry = self.cache_entry(name, key, &options).await;

        if let Some(cache_entry) = cache_entry {
            debug!("Cache entry found for {}", cache_entry);
            let operation = async {
                self.execute_operation(name, key, &options, operation)
                    .await
                    .map(|e| e.as_ref().to_vec())
            };

            match options.cache_type {
                CacheType::CacheFirst => {
                    CacheFirstStrategy::execute(self.read_entry(cache_entry), operation).await
                }
                CacheType::CacheLast => {
                    CacheLastStrategy::execute(self.read_entry(cache_entry), operation).await
                }
            }
        } else {
            debug!("Cache entry not found for {} {}", name, key);
            self.execute_operation(name, key, &options, operation)
                .await
                .map(|e| e.as_ref().to_vec())
        }
    }

    async fn execute_operation<T, E, O>(
        &self,
        name: &str,
        key: &str,
        options: &CacheOptions,
        operation: O,
    ) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]>,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        trace!("Executing cache operation for cache {} entry {}", name, key);
        match operation.await {
            Ok(e) => {
                debug!(
                    "Cache operation of {} entry {} executed with success",
                    name, key
                );
                self.store(name, key, &options.expires_after, e.as_ref())
                    .await
                    .map_err(|e| CacheExecutionError::Cache(e))?;
                Ok(e)
            }
            Err(e) => Err(CacheExecutionError::Operation(e)),
        }
    }

    async fn cache_entry(
        &self,
        name: &str,
        key: &str,
        options: &CacheOptions,
    ) -> Option<CacheEntry> {
        let cache = self.cache_info.lock().await;
        let cache_entry = cache.info(name, key).filter(|entry| {
            trace!(
                "Filtering cache entry {:?} against options {:?}",
                entry,
                options
            );
            !entry.is_expired(&options.expires_after)
        });

        cache_entry
    }

    async fn read(
        &self,
        name: &str,
        key: &str,
        options: &CacheOptions,
    ) -> Result<Vec<u8>, CacheError> {
        let cache_entry = self.cache_entry(name, key, options).await;

        if let Some(cache_entry) = cache_entry {
            self.read_entry(cache_entry).await
        } else {
            debug!(
                "Unable to read cache entry {} with key {}, cache not found",
                name, key
            );
            Err(CacheError::NotFound(format!("Cache {} not found", name)))
        }
    }

    async fn read_entry(&self, cache: CacheEntry) -> Result<Vec<u8>, CacheError> {
        trace!("Trying to load cached entry {}", cache);
        self.storage
            .options()
            .make_dirs(false)
            .binary(cache.filename())
            .read()
            .map(|data| {
                debug!("Binary cached data of {} has been loaded", cache);
                data
            })
            .map_err(|e| {
                warn!("Failed to load cached entry {}, {}", cache, e);
                match e {
                    StorageError::NotFound(e) => CacheError::NotFound(e),
                    _ => CacheError::Io(e.to_string()),
                }
            })
    }

    async fn store(
        &self,
        name: &str,
        key: &str,
        expiration: &Duration,
        data: &[u8],
    ) -> cache::Result<()> {
        trace!(
            "Storing new cache {} entry {} with expiration {}",
            name,
            key,
            expiration
        );
        let filename = Self::generate_cache_filename(name, key);
        let path = self.write_cache_data(filename.as_str(), data).await?;
        self.create_cache_entry(name, key, path, expiration).await;
        self.write_cache_info().await?;

        Ok(())
    }

    async fn create_cache_entry(
        &self,
        name: &str,
        key: &str,
        path: PathBuf,
        expiration: &Duration,
    ) {
        trace!("Creating new cache {} entry {}", name, key);
        let mut info = self.cache_info.lock().await;

        info.add(
            name,
            CacheEntry::new(key, path.to_str().unwrap(), expiration),
        );
    }

    async fn write_cache_data(&self, filename: &str, data: &[u8]) -> cache::Result<PathBuf> {
        trace!("Writing cache data to {}", filename);
        let path = self
            .storage
            .options()
            .make_dirs(true)
            .binary(filename)
            .write(data)
            .map_err(|e| {
                error!("Failed to write cache {}, {}", filename, e);
                CacheError::Io(e.to_string())
            })?;

        Ok(path)
    }

    async fn write_cache_info(&self) -> cache::Result<()> {
        trace!("Saving cache information");
        let info = self.cache_info.lock().await;

        self.storage
            .options()
            .make_dirs(true)
            .serializer(FILENAME)
            .write_async(&*info)
            .await
            .map(|e| debug!("Cache info has been saved at {}", e.to_str().unwrap()))
            .map_err(|e| {
                warn!("Check information could not be stored, {}", e);
                CacheError::Io(e.to_string())
            })
    }

    async fn execute_cleanup(&self) {
        debug!("Checking for expired cache data");
        let mut cache = self.cache_info.lock().await;
        let expired_entries = cache.expired();

        for expired in expired_entries.into_iter() {
            match Storage::delete(expired.entry.path()) {
                Ok(_) => {
                    cache.remove(expired.name.as_str(), expired.entry.key());
                    debug!(
                        "Cache {} entry {} has been cleaned",
                        expired.name,
                        expired.entry.key()
                    )
                }
                Err(e) => {
                    if let StorageError::NotFound(e) = e {
                        debug!("Cache {} entry {} has been removed externally, removing entry from cache manager", expired.name, e);
                        cache.remove(expired.name.as_str(), expired.entry.key());
                    } else {
                        error!(
                            "Failed to delete cache file {}, {}",
                            expired.entry.absolute_path(),
                            e.to_string()
                        )
                    }
                }
            }
        }

        drop(cache);
        let _ = self.write_cache_info().await;
    }

    fn send_command(&self, command: CacheManagerCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Cache manager failed to send command, {}", e);
        }
    }

    fn generate_cache_filename(name: &str, key: &str) -> String {
        let filename = name.to_string() + key;
        trace!("Hashing filename {}", filename);

        digest(&digest::SHA256, filename.as_bytes())
            .as_ref()
            .to_vec()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
            + EXTENSION
    }

    fn map_cache_parser_error<E>(error: CacheParserError<E>) -> CacheExecutionError<E>
    where
        E: Error,
    {
        match error {
            CacheParserError::Operation(e) => CacheExecutionError::Operation(e),
            CacheParserError::Parsing(e) => CacheExecutionError::Cache(CacheError::Parsing(e)),
        }
    }
}

/// Represents a cache operation to be executed.
#[derive(Debug)]
pub struct CacheOperation {
    cache_manager: Arc<InnerCacheManager>,
    name: Option<String>,
    key: Option<String>,
    options: Option<CacheOptions>,
}

impl CacheOperation {
    /// Creates a new `CacheOperation` instance.
    ///
    /// # Arguments
    ///
    /// * `cache_manager` - The cache manager to use for executing cache operations.
    ///
    /// # Returns
    ///
    /// A new `CacheOperation` instance.
    fn new(cache_manager: Arc<InnerCacheManager>) -> Self {
        Self {
            cache_manager,
            name: None,
            key: None,
            options: None,
        }
    }

    /// Sets the name for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache operation.
    pub fn name<T: AsRef<str>>(mut self, name: T) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    /// Sets the key for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the cache operation.
    pub fn key<T: AsRef<str>>(mut self, key: T) -> Self {
        self.key = Some(key.as_ref().to_string());
        self
    }

    /// Sets the cache options for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `options` - The cache options for the cache operation.
    pub fn options(mut self, options: CacheOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Maps the result of the cache operation using the specified mapper function.
    ///
    /// # Arguments
    ///
    /// * `mapper` - The mapper function to apply to the cache operation result.
    ///
    /// # Returns
    ///
    /// A `MappedCacheOperation` instance with the specified mapper function.
    pub fn map<T, E, M>(self, mapper: M) -> MappedCacheOperation<T, E, M>
    where
        T: AsRef<[u8]>,
        E: Error,
        M: FnOnce(Vec<u8>) -> Result<T, E>,
    {
        MappedCacheOperation {
            inner: self,
            mapper,
        }
    }

    /// Serializes the data before storing it within the cache operation.
    ///
    /// # Returns
    ///
    /// A `SerializedCacheOperation` instance for further serialization operations.
    pub fn serializer(self) -> SerializedCacheOperation {
        SerializedCacheOperation { inner: self }
    }

    /// Executes the cache operation asynchronously.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    ///
    /// The result of the cache operation, wrapped in a `Result` indicating success or failure.
    ///
    /// # Panics
    ///
    /// This method will panic if the name, key, or options are missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use chrono::Duration;
    /// use popcorn_fx_core::core::cache::{CacheManager, CacheOptions, CacheType};
    ///
    /// let cache_manager = CacheManager::new("/path/to/cache");
    /// let result = cache_manager.operation()
    ///     .name("my_cache".to_string())
    ///     .key("my_key".to_string())
    ///     .options(CacheOptions {
    ///         cache_type: CacheType::CacheFirst,
    ///         expires_after: Duration::days(7),
    ///     })
    ///     .execute(|| {
    ///         // Perform cache operation here
    ///         Ok(vec![1, 2, 3])
    ///     });
    ///
    /// match result {
    ///     Ok(data) => {
    ///         // Cache operation succeeded
    ///         println!("Cache operation result: {:?}", data);
    ///     }
    ///     Err(error) => {
    ///         // Cache operation failed
    ///         eprintln!("Cache operation failed: {:?}", error);
    ///     }
    /// }
    /// ```
    pub async fn execute<T, E, O>(self, operation: O) -> Result<T, CacheExecutionError<E>>
    where
        T: AsRef<[u8]> + From<Vec<u8>>,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        let name = self.name.expect("Name is missing");
        let key = self.key.expect("Key is missing");
        let options = self.options.expect("Options are missing");

        self.cache_manager
            .execute(name.as_str(), key.as_str(), options, operation)
            .await
    }
}

/// Represents a mapped cache operation.
#[derive(Debug)]
pub struct MappedCacheOperation<T, E, M>
where
    T: AsRef<[u8]>,
    E: Error,
    M: FnOnce(Vec<u8>) -> Result<T, E>,
{
    inner: CacheOperation,
    mapper: M,
}

impl<T, E, M> MappedCacheOperation<T, E, M>
where
    T: AsRef<[u8]>,
    E: Error,
    M: FnOnce(Vec<u8>) -> Result<T, E>,
{
    /// Sets the name for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache operation.
    pub fn name<V: AsRef<str>>(mut self, name: V) -> Self {
        self.inner.name = Some(name.as_ref().to_string());
        self
    }

    /// Sets the key for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the cache operation.
    pub fn key<V: AsRef<str>>(mut self, key: V) -> Self {
        self.inner.key = Some(key.as_ref().to_string());
        self
    }

    /// Sets the cache options for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `options` - The cache options for the cache operation.
    pub fn options(mut self, options: CacheOptions) -> Self {
        self.inner.options = Some(options);
        self
    }

    /// Executes the mapped cache operation asynchronously.
    ///
    /// This method executes the mapped cache operation by combining the cache name, key, options,
    /// and the provided mapper function with the operation closure. It delegates the execution to
    /// the `execute_with_mapper` method of the `CacheManager`.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    ///
    /// The result of the mapped cache operation.
    ///
    /// # Panics
    ///
    /// This method panics if the cache name, key, or options are missing from the `inner`
    /// `CacheOperation` instance.
    pub async fn execute<O>(self, operation: O) -> Result<T, CacheExecutionError<E>>
    where
        O: Future<Output = Result<T, E>>,
    {
        let name = self.inner.name.expect("Name is missing");
        let key = self.inner.key.expect("Key is missing");
        let options = self.inner.options.expect("Options are missing");

        self.inner
            .cache_manager
            .execute_with_mapper(name.as_str(), key.as_str(), options, self.mapper, operation)
            .await
    }
}

/// Represents a cache operation specifically designed for serialization and deserialization.
#[derive(Debug)]
pub struct SerializedCacheOperation {
    inner: CacheOperation,
}

impl SerializedCacheOperation {
    /// Sets the name for the cache operation.
    ///
    /// # Arguments
    ///
    /// * name - The name of the cache operation.
    pub fn name<V: AsRef<str>>(mut self, name: V) -> Self {
        self.inner.name = Some(name.as_ref().to_string());
        self
    }

    /// Sets the key for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the cache operation.
    pub fn key<V: AsRef<str>>(mut self, key: V) -> Self {
        self.inner.key = Some(key.as_ref().to_string());
        self
    }

    /// Sets the cache options for the cache operation.
    ///
    /// # Arguments
    ///
    /// * `options` - The cache options for the cache operation.
    pub fn options(mut self, options: CacheOptions) -> Self {
        self.inner.options = Some(options);
        self
    }

    /// Executes the cache operation asynchronously.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute.
    ///
    /// # Returns
    ///
    /// The result of the cache operation.
    ///
    /// # Generic Parameters
    ///
    /// * `T` - The type of the value to be serialized or deserialized.
    /// * `E` - The type of the error that may occur during serialization or deserialization.
    /// * `O` - The type of the future representing the operation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use serde::{Serialize, Deserialize};
    /// use popcorn_fx_core::core::cache::{CacheExecutionError, CacheOptions, SerializedCacheOperation};
    ///
    /// async fn operation() -> Result<Vec<u8>, Box<dyn Error>> {
    ///     // Implementation code here...
    ///     # Ok(vec![])
    /// }
    ///
    /// let cache_operation = SerializedCacheOperation::new(cache_manager)
    ///     .name("my_cache")
    ///     .key("my_key")
    ///     .options(CacheOptions::default());
    ///
    /// let result: Result<Vec<u8>, CacheExecutionError<Box<dyn Error>>> = cache_operation.execute(operation);
    /// match result {
    ///     Ok(data) => {
    ///         // Process the obtained data...
    ///     }
    ///     Err(err) => {
    ///         // Handle the cache execution error...
    ///     }
    /// }
    /// ```
    pub async fn execute<T, E, O>(self, operation: O) -> Result<T, CacheExecutionError<E>>
    where
        T: Serialize + DeserializeOwned,
        E: Error,
        O: Future<Output = Result<T, E>>,
    {
        let name = self.inner.name.expect("Name is missing");
        let key = self.inner.key.expect("Key is missing");
        let options = self.inner.options.expect("Options are missing");

        self.inner
            .cache_manager
            .execute_serializer(name.as_str(), key.as_str(), options, operation)
            .await
    }
}

#[cfg(test)]
mod test {
    use crate::core::cache::CacheExecutionError;
    use crate::core::media::{MediaError, MovieOverview};
    use crate::testing::{copy_test_file, read_test_file_to_bytes};
    use crate::{assert_timeout, init_logger, recv_timeout};
    use tokio::time;

    use super::*;

    #[tokio::test]
    async fn test_execute_cache_not_present_and_operation_successful() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = CacheManagerBuilder::default()
            .storage_path(temp_path)
            .build();
        let expected_data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur suscipit ullamcorper eleifend. Nulla ac urna tellus. Nullam posuere ligula non consectetur rhoncus. Nam eleifend non elit nec accumsan.";
        let name = "test";
        let key = "lorem";

        let cloned_manager = cache_manager.clone();
        match cloned_manager
            .operation()
            .name(name)
            .key(key)
            .options(CacheOptions {
                cache_type: CacheType::CacheFirst,
                expires_after: Duration::hours(6),
            })
            .map(|e| String::from_utf8(e))
            .execute(async { Ok(expected_data.to_string()) })
            .await
        {
            Ok(data) => assert_eq!(expected_data.to_string(), data),
            Err(e) => assert!(false, "expected the cache execution to succeed, {}", e),
        };

        time::sleep(time::Duration::from_millis(50)).await;
        let cache_info: CacheInfo = cache_manager
            .inner
            .storage
            .options()
            .serializer(FILENAME)
            .read()
            .unwrap();
        let cache_entry = cache_info.info(name, key);
        assert!(
            cache_entry.is_some(),
            "expected the cache to contain the entry info"
        );
        let stored_data = cache_manager
            .inner
            .storage
            .options()
            .binary(cache_entry.unwrap().filename())
            .read()
            .map(|e| String::from_utf8(e))
            .unwrap()
            .unwrap();
        assert_eq!(expected_data, stored_data.as_str());
    }

    #[tokio::test]
    async fn test_execute_cache_not_present_and_operation_failed() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = CacheManagerBuilder::default()
            .storage_path(temp_path)
            .build();
        let name = "test";
        let key = "lorem";
        let expected_error = MediaError::ProviderRequestFailed("failed".to_string(), 503);

        let cloned_manager = cache_manager.clone();
        let cloned_error = expected_error.clone();
        let result: Result<Vec<u8>, CacheExecutionError<MediaError>> = cloned_manager
            .operation()
            .name(name)
            .key(key)
            .options(CacheOptions {
                cache_type: CacheType::CacheFirst,
                expires_after: Duration::hours(6),
            })
            .execute(async { Err(cloned_error) })
            .await;
        if let Err(e) = result {
            match e {
                CacheExecutionError::Operation(media_error) => {
                    assert_eq!(expected_error, media_error)
                }
                _ => assert!(
                    false,
                    "expected error CacheExecutionError::Operation, got {:?} instead",
                    e
                ),
            }
        } else {
            assert!(false, "expected an error to be returned")
        }
    }

    #[tokio::test]
    async fn test_execute_cache_is_present_and_type_is_cache_first() {
        init_logger!();
        let filename = "simple.jpg";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = CacheManagerBuilder::default()
            .storage_path(temp_path)
            .build();
        let name = "test";
        let key = "lorem";
        let expected_result = read_test_file_to_bytes(filename);
        let test_file_output = copy_test_file(temp_path, filename, Some("cache/simple.jpg"));

        let cloned_manager = cache_manager.clone();
        let data = async move {
            let mut cache_info = cloned_manager.inner.cache_info.lock().await;
            cache_info.add(
                name,
                CacheEntry::new(key, test_file_output.as_str(), &Duration::hours(6)),
            );
            drop(cache_info);

            let result: Result<Vec<u8>, CacheExecutionError<MediaError>> = cloned_manager
                .operation()
                .name(name)
                .key(key)
                .options(CacheOptions {
                    cache_type: CacheType::CacheFirst,
                    expires_after: Duration::hours(6),
                })
                .execute(async {
                    Err(MediaError::ProviderRequestFailed(
                        "this should not have been executed".to_string(),
                        500,
                    ))
                })
                .await;
            result
        }
        .await
        .unwrap();
        assert_eq!(expected_result, data);
    }

    #[tokio::test]
    async fn test_execute_cache_is_present_and_type_is_cache_last() {
        init_logger!();
        let filename = "simple.jpg";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = Arc::new(
            CacheManagerBuilder::default()
                .storage_path(temp_path)
                .build(),
        );
        let name = "test";
        let key = "lorem";
        let (tx, mut rx) = unbounded_channel();
        let expected_result = read_test_file_to_bytes(filename);
        let test_file_output = copy_test_file(temp_path, filename, Some("cache/simple.jpg"));

        let cloned_manager = cache_manager.clone();
        let data = async move {
            let mut cache_info = cloned_manager.inner.cache_info.lock().await;
            cache_info.add(
                name,
                CacheEntry::new(key, test_file_output.as_str(), &Duration::hours(6)),
            );
            drop(cache_info);

            let result: Result<Vec<u8>, CacheExecutionError<MediaError>> = cloned_manager
                .operation()
                .name(name)
                .key(key)
                .options(CacheOptions {
                    cache_type: CacheType::CacheLast,
                    expires_after: Duration::hours(6),
                })
                .execute(async {
                    tx.send(true).unwrap();
                    Err(MediaError::ProviderRequestFailed(
                        "this should not have been executed".to_string(),
                        500,
                    ))
                })
                .await;
            result
        }
        .await
        .unwrap();

        let _ = recv_timeout!(
            &mut rx,
            time::Duration::from_millis(200),
            "expected the mapper to have been executed"
        );
        assert_eq!(expected_result, data);
    }

    #[tokio::test]
    async fn test_execute_serializer() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = Arc::new(
            CacheManagerBuilder::default()
                .storage_path(temp_path)
                .build(),
        );
        let media = MovieOverview {
            imdb_id: "tt1112233".to_string(),
            title: "Lorem ipsum".to_string(),
            year: "".to_string(),
            rating: None,
            images: Default::default(),
        };

        let cloned_manager = cache_manager.clone();
        let cloned_media = media.clone();
        let result: Result<MovieOverview, CacheExecutionError<MediaError>> = cloned_manager
            .operation()
            .name("test")
            .key("lorem")
            .options(CacheOptions {
                cache_type: CacheType::CacheFirst,
                expires_after: Duration::hours(5),
            })
            .serializer()
            .execute(async { Ok(cloned_media) })
            .await;

        if let Ok(e) = result {
            assert_eq!(media, e)
        } else {
            assert!(
                false,
                "expected the cache operation to succeed, {:?}",
                result
            )
        }
    }

    #[tokio::test]
    async fn test_execute_serializer_error() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = Arc::new(
            CacheManagerBuilder::default()
                .storage_path(temp_path)
                .build(),
        );

        let cloned_manager = cache_manager.clone();
        let result: Result<MovieOverview, CacheExecutionError<MediaError>> = cloned_manager
            .operation()
            .name("test")
            .key("lorem")
            .options(CacheOptions {
                cache_type: CacheType::CacheFirst,
                expires_after: Duration::hours(5),
            })
            .serializer()
            .execute(async { Err(MediaError::NoAvailableProviders) })
            .await;

        if let Err(execution_error) = result {
            match execution_error {
                CacheExecutionError::Operation(e) => {
                    assert_eq!(MediaError::NoAvailableProviders, e)
                }
                _ => assert!(
                    false,
                    "expected CacheExecutionError::Operation, but got {:?} instead",
                    execution_error
                ),
            }
        } else {
            assert!(
                false,
                "expected the cache operation to succeed, {:?}",
                result
            )
        }
    }

    #[test]
    fn test_map_parser_error() {
        if let CacheExecutionError::Operation(e) = InnerCacheManager::map_cache_parser_error(
            CacheParserError::Operation(MediaError::NoAvailableProviders),
        ) {
            assert_eq!(MediaError::NoAvailableProviders, e);
        } else {
            assert!(false, "CacheExecutionError::Operation");
        }

        if let CacheExecutionError::Cache(e) = InnerCacheManager::map_cache_parser_error(
            CacheParserError::Parsing::<MediaError>("lorem".to_string()),
        ) {
            assert_eq!(CacheError::Parsing("lorem".to_string()), e);
        } else {
            assert!(false, "CacheExecutionError::Mapping");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_run_cleanup() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_filepath = copy_test_file(temp_path, "simple.jpg", Some("cache/simple.jpg"));
        let storage = Storage::from(&PathBuf::from(temp_path).join(DIRECTORY));
        let path = PathBuf::from(test_filepath.as_str());
        storage
            .options()
            .make_dirs(true)
            .serializer(FILENAME)
            .write(&CacheInfo {
                entries: vec![(
                    "lorem".to_string(),
                    vec![CacheEntry {
                        key: "ipsum".to_string(),
                        path: test_filepath,
                        expires_after: 60,
                        created_on: "2023-01-01T12:00".to_string(),
                    }],
                )]
                .into_iter()
                .collect(),
            })
            .unwrap();
        let _cache_manager = Arc::new(
            CacheManagerBuilder::default()
                .storage_path(temp_path)
                .build(),
        );

        assert_timeout!(time::Duration::from_millis(100), !path.exists());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_run_cleanup_non_existing_entry() {
        init_logger!();
        let cache_entry = "foo";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_filepath = PathBuf::from(temp_path).join("example.png");
        let storage = Storage::from(&PathBuf::from(temp_path).join(DIRECTORY));
        storage
            .options()
            .make_dirs(true)
            .serializer(FILENAME)
            .write(&CacheInfo {
                entries: vec![(
                    cache_entry.to_string(),
                    vec![CacheEntry {
                        key: "bar".to_string(),
                        path: test_filepath.to_str().unwrap().to_string(),
                        expires_after: 60,
                        created_on: "2023-01-01T12:00".to_string(),
                    }],
                )]
                .into_iter()
                .collect(),
            })
            .unwrap();

        let cache_manager = Arc::new(
            CacheManagerBuilder::default()
                .storage_path(temp_path)
                .build(),
        );

        let inner = cache_manager.inner.clone();
        inner.execute_cleanup().await;

        let cache = cache_manager.inner.cache_info.lock().await;

        if let Some(entry) = cache.entries(cache_entry) {
            assert_eq!(
                0,
                entry.len(),
                "expected the non existing entry to be removed"
            );
        } else {
            assert!(false, "expected the cache to contain the entry info")
        }
    }
}
