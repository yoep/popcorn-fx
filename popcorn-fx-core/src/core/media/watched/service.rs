use std::sync::Arc;

use log::{debug, error, trace, warn};
use tokio::sync::Mutex;

use crate::core::media;
use crate::core::media::{MediaError, MediaIdentifier};
use crate::core::media::watched::Watched;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "watched.json";

/// The watch service is responsible for tracking watched media items.
#[derive(Debug)]
pub struct WatchedService {
    storage: Arc<Storage>,
    cache: Arc<Mutex<Option<Watched>>>,
}

impl WatchedService {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Verify if the watchable has been seen or not.
    pub fn is_watched(&self, watchable: &impl MediaIdentifier) -> bool {
        let imdb_id = watchable.imdb_id();

        self.internal_is_watched(imdb_id.as_str())
    }

    /// Verify if the watchable has been seen or not.
    pub fn is_watched_boxed(&self, watchable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = watchable.imdb_id();

        self.internal_is_watched(imdb_id.as_str())
    }

    /// Retrieve an array of owned watched media item ids.
    ///
    /// It returns the watched ids when loaded, else the [MediaError].
    pub fn all(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");
                let mut movies = watched.movies().clone();
                let mut shows = watched.shows().clone();
                let mut all: Vec<String> = vec![];

                all.append(&mut movies);
                all.append(&mut shows);

                Ok(all)
            }
            Err(e) => Err(e)
        }
    }

    fn internal_is_watched(&self, imdb_id: &str) -> bool {
        trace!("Verifying if ID {} is watched", imdb_id);
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");

                watched.contains(imdb_id)
            }
            Err(e) => {
                warn!("Unable to load {}, {}", FILENAME, e);
                false
            }
        }
    }

    async fn load_watched_cache(&self) -> media::Result<()> {
        let mutex = self.cache.clone();
        let mut cache = mutex.lock().await;

        if cache.is_none() {
            trace!("Loading watched cache");
            return match self.load_watched_from_storage() {
                Ok(e) => {
                    debug!("Loaded watched items {:?}", &e);
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e)
            };
        }

        trace!("Watched cache has already been loaded, nothing to do");
        Ok(())
    }

    fn load_watched_from_storage(&self) -> media::Result<Watched> {
        match self.storage.read::<Watched>(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::FileNotFound(file) => {
                        debug!("Creating new watched file {}", file);
                        Ok(Watched::empty())
                    }
                    StorageError::CorruptRead(_, error) => {
                        error!("Failed to load watched items, {}", error);
                        Err(MediaError::WatchedLoadingFailed(error))
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", e);
                        Ok(Watched::empty())
                    }
                }
            }
        }
    }
}

impl Drop for WatchedService {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod test {
    use crate::core::media::MovieOverview;
    use crate::testing::{init_logger, test_resource_directory};

    use super::*;

    #[test]
    fn test_is_watched_when_item_is_watched_should_return_true() {
        init_logger();
        let imdb_id = "tt548723".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedService::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched(&movie);

        assert!(result, "expected the media to have been watched")
    }

    #[test]
    fn test_is_watched_when_item_is_not_watched_should_return_false() {
        init_logger();
        let imdb_id = "tt548766".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedService::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched(&movie);

        assert!(!result, "expected the media to not have been watched")
    }

    #[test]
    fn test_is_watched_boxed() {
        init_logger();
        let imdb_id = "tt541345".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedService::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched_boxed(&(Box::new(movie) as Box<dyn MediaIdentifier>));

        assert!(result, "expected the media to have been watched")
    }

    #[test]
    fn test_all() {
        init_logger();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedService::new(&storage);
        let expected_result = vec![
            "tt548723",
            "tt541345",
            "tt3915174",
        ];

        let result = service.all()
            .expect("expected the watched ids to have been returned");

        assert_eq!(expected_result, result)
    }
}