use std::fmt::Debug;
use std::sync::Arc;

use log::{debug, error, trace, warn};
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::media;
use crate::core::media::{MediaError, MediaIdentifier, MediaType};
use crate::core::media::watched::Watched;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "watched.json";

/// The watched service is responsible for tracking seen/unseen media items.
#[automock]
pub trait WatchedService: Debug + Send + Sync + 'static {
    /// Verify if the given ID has been seen.
    ///
    /// * `id`  - The ID of the watchable to verify.
    ///
    /// It returns `true` when the ID has been seen, else `false`.
    fn is_watched(&self, id: &str) -> bool;

    /// Verify if the given identifier item has been seen.
    ///
    /// It returns `true` when the media item has been seen, else `false`.
    fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool;

    /// Retrieve an array of owned watched media item ids.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn all(&self) -> media::Result<Vec<String>>;

    /// Retrieve an array of watched movie id's.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn watched_movies(&self) -> media::Result<Vec<String>>;

    /// Retrieve an array of watched show id's.
    ///
    /// It returns the watched id's when loaded, else the [MediaError].
    fn watched_shows(&self) -> media::Result<Vec<String>>;
}

/// The standard Popcorn FX watched service.
#[derive(Debug)]
pub struct WatchedServiceFx {
    storage: Arc<Storage>,
    cache: Arc<Mutex<Option<Watched>>>,
}

impl WatchedServiceFx {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Add the given media item to the watched list.
    pub fn add(&self, watchable: Box<dyn MediaIdentifier>) -> media::Result<()> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let mut cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_mut().expect("expected the cache to have been loaded");
                let id = watchable.imdb_id();

                match watchable.media_type() {
                    MediaType::Movie => watched.add_movie(id),
                    MediaType::Show => watched.add_show(id),
                    MediaType::Episode => watched.add_show(id),
                    _ => {
                        error!("Media type {} is not supported", watchable.media_type());
                    }
                }

                Ok(())
            }
            Err(e) => Err(e)
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

impl WatchedService for WatchedServiceFx {
    fn is_watched(&self, imdb_id: &str) -> bool {
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

    fn is_watched_dyn(&self, watchable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = watchable.imdb_id();

        self.is_watched(imdb_id.as_str())
    }

    fn all(&self) -> media::Result<Vec<String>> {
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

    fn watched_movies(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");

                Ok(watched.movies().clone())
            }
            Err(e) => Err(e)
        }
    }

    fn watched_shows(&self) -> media::Result<Vec<String>> {
        match futures::executor::block_on(self.load_watched_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let watched = cache.as_ref().expect("cache should have been present");

                Ok(watched.shows().clone())
            }
            Err(e) => Err(e)
        }
    }
}

impl Drop for WatchedServiceFx {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod test {
    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::testing::{init_logger, test_resource_directory};

    use super::*;

    #[test]
    fn test_is_watched_when_item_is_watched_should_return_true() {
        init_logger();
        let imdb_id = "tt548723".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>));

        assert!(result, "expected the media to have been watched")
    }

    #[test]
    fn test_is_watched_when_item_is_not_watched_should_return_false() {
        init_logger();
        let imdb_id = "tt548766".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>));

        assert!(!result, "expected the media to not have been watched")
    }

    #[test]
    fn test_is_watched_boxed() {
        init_logger();
        let imdb_id = "tt541345".to_string();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id,
            String::new(),
        );

        let result = service.is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>));

        assert!(result, "expected the media to have been watched")
    }

    #[test]
    fn test_all() {
        init_logger();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let expected_result = vec![
            "tt548723",
            "tt541345",
            "tt3915174",
        ];

        let result = service.all()
            .expect("expected the watched ids to have been returned");

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_add_movie() {
        init_logger();
        let imdb_id = "tt548795".to_string();
        let resource_directory = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::from_directory(resource_directory.path().to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let movie = MovieOverview::new(
            String::new(),
            imdb_id.clone(),
            String::new(),
        );

        service.add(Box::new(movie.clone()) as Box<dyn MediaIdentifier>)
            .expect("add should have succeeded");
        let result = service.is_watched_dyn(&(Box::new(movie) as Box<dyn MediaIdentifier>));

        assert!(result, "expected the media item to have been watched")
    }

    #[test]
    fn test_add_show() {
        init_logger();
        let imdb_id = "tt88877554".to_string();
        let resource_directory = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::from_directory(resource_directory.path().to_str().expect("expected resource path to be valid")));
        let service = WatchedServiceFx::new(&storage);
        let show = ShowOverview::new(
            imdb_id.clone(),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        );

        service.add(Box::new(show.clone()) as Box<dyn MediaIdentifier>)
            .expect("add should have succeeded");
        let result = service.is_watched_dyn(&(Box::new(show) as Box<dyn MediaIdentifier>));

        assert!(result, "expected the media item to have been watched")
    }
}