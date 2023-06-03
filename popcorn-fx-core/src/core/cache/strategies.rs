use std::error::Error;
use std::fmt::Debug;
use std::future::Future;

use log::{debug, trace};

use crate::core::cache::{CacheError, CacheExecutionError};

#[derive(Debug)]
pub struct CacheFirstStrategy {}

impl CacheFirstStrategy {
    /// Executes the cache first strategy asynchronously.
    ///
    /// # Arguments
    ///
    /// * `cache_data` - The closure to retrieve the data from the cache.
    /// * `operation` - The closure representing the operation to execute if the cache data is not available.
    ///
    /// # Returns
    ///
    /// The result of the cache first strategy execution, which is a `Vec<u8>` representing the data obtained
    /// either from the cache or the executed operation.
    ///
    /// # Errors
    ///
    /// This method can return a `CacheExecutionError` if there is an error executing the cache first strategy.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use futures::Future;
    ///
    /// async fn cache_data() -> Result<Vec<u8>, crate::CacheError> {
    ///     // Implementation code here...
    ///     # Ok(vec![])
    /// }
    ///
    /// async fn operation() -> Result<Vec<u8>, Box<dyn Error>> {
    ///     // Implementation code here...
    ///     # Ok(vec![])
    /// }
    ///
    /// let result = CacheFirstStrategy::execute(cache_data, operation);
    /// match result {
    ///     Ok(data) => {
    ///         // Process the obtained data...
    ///     }
    ///     Err(err) => {
    ///         // Handle the cache execution error...
    ///     }
    /// }
    /// ```
    pub async fn execute<E, C, O>(cache_data: C, operation: O) -> Result<Vec<u8>, CacheExecutionError<E>>
        where E: Error,
              C: Future<Output=Result<Vec<u8>, CacheError>>,
              O: Future<Output=Result<Vec<u8>, CacheExecutionError<E>>> {
        trace!("Executing cache first strategy");
        let bytes = cache_data.await;

        if let Err(e) = bytes {
            debug!("Cache data couldn't be read, {}", e);
            return operation.await;
        }

        Ok(bytes.unwrap())
    }
}

#[derive(Debug)]
pub struct CacheLastStrategy {}

impl CacheLastStrategy {
    /// Executes the cache last strategy asynchronously.
    ///
    /// # Arguments
    ///
    /// * `cache_data` - The closure to retrieve the data from the cache.
    /// * `operation` - The closure representing the operation to execute if the cache data is not available.
    ///
    /// # Returns
    ///
    /// The result of the cache last strategy execution, which is a `Vec<u8>` representing the data obtained
    /// either from the cache or the executed operation.
    ///
    /// # Errors
    ///
    /// This method can return a `CacheExecutionError` if there is an error executing the cache last strategy.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use futures::Future;
    ///
    /// async fn cache_data() -> Result<Vec<u8>, crate::CacheError> {
    ///     // Implementation code here...
    ///     # Ok(vec![])
    /// }
    ///
    /// async fn operation() -> Result<Vec<u8>, Box<dyn Error>> {
    ///     // Implementation code here...
    ///     # Ok(vec![])
    /// }
    ///
    /// let result = CacheLastStrategy::execute(cache_data, operation);
    /// match result {
    ///     Ok(data) => {
    ///         // Process the obtained data...
    ///     }
    ///     Err(err) => {
    ///         // Handle the cache execution error...
    ///     }
    /// }
    /// ```
    pub async fn execute<E, C, O>(cache_data: C, operation: O) -> Result<Vec<u8>, CacheExecutionError<E>>
        where E: Error,
              C: Future<Output=Result<Vec<u8>, CacheError>>,
              O: Future<Output=Result<Vec<u8>, CacheExecutionError<E>>> {
        trace!("Executing cache last strategy");
        let result = operation.await;

        match result {
            Ok(e) => Ok(e),
            Err(e) => {
                debug!("Cache operation failed, using cache data instead, reason: {}", e);
                cache_data.await.map_err(|e| CacheExecutionError::Cache(e))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::media::MediaError;
    use crate::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_cache_first() {
        init_logger();

        match CacheFirstStrategy::execute(async { Ok(vec![0]) }, async { Err(CacheExecutionError::Operation(MediaError::ProviderNotFound("should not have been invoked".to_string()))) }).await {
            Ok(e) => assert_eq!(vec![0], e),
            Err(e) => assert!(false, "expected OK, but got {:?} instead", e)
        };
        match CacheFirstStrategy::execute(async { Err(CacheError::NotFound("".to_string())) }, async { Ok::<Vec<u8>, CacheExecutionError<MediaError>>(vec![1]) }).await {
            Ok(e) => assert_eq!(vec![1], e),
            Err(e) => assert!(false, "expected OK, but got {:?} instead", e)
        };
        match CacheFirstStrategy::execute(async { Err(CacheError::NotFound("".to_string())) }, async { Err(CacheExecutionError::Operation(MediaError::ProviderNotFound("lorem".to_string()))) }).await {
            Ok(e) => assert!(false, "expected an error to be returned"),
            Err(cache_error) => match cache_error {
                CacheExecutionError::Operation(e) => assert_eq!(MediaError::ProviderNotFound("lorem".to_string()), e),
                _ => assert!(false, "expected CacheExecutionError::Operation but got {:?} instead", cache_error)
            }
        };
    }

    #[tokio::test]
    async fn test_cache_last() {
        init_logger();
        let (tx, rx) = channel();

        let result = CacheLastStrategy::execute(async move {
            tx.send(true).unwrap();
            Ok(vec![0])
        }, async { Ok::<Vec<u8>, CacheExecutionError<MediaError>>(vec![1]) })
            .await
            .unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(50)).is_err(), "the cache data should not have been invoked");

        match CacheLastStrategy::execute(async { Ok(vec![0]) }, async { Err(CacheExecutionError::Operation(MediaError::ProviderNotFound("should not have been invoked".to_string()))) }).await {
            Ok(e) => assert_eq!(vec![0], e),
            Err(e) => assert!(false, "expected OK, but got {:?} instead", e)
        };
        match CacheLastStrategy::execute(async { Err(CacheError::NotFound("".to_string())) }, async { Err(CacheExecutionError::Operation(MediaError::ProviderNotFound("lorem".to_string()))) }).await {
            Ok(e) => assert!(false, "expected an error to be returned"),
            Err(cache_error) => match cache_error {
                CacheExecutionError::Cache(e) => assert_eq!(CacheError::NotFound("".to_string()), e),
                _ => assert!(false, "expected CacheExecutionError::Cache but got {:?} instead", cache_error)
            }
        };
    }
}