use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::core::media;
use crate::core::media::{MediaError, MediaIdentifier, MediaOverview, MediaType, MovieOverview, ShowOverview};
use crate::core::media::favorites::model::Favorites;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "favorites.json";

/// The callback to listen on events of the favorite service.
pub type FavoriteCallback = Box<dyn Fn(FavoriteEvent) + Send>;

#[derive(Debug, Clone)]
pub enum FavoriteEvent {
    /// Invoked when a media item's liked state has changed.
    ///
    /// - The IMDB ID of the media item for which the state changed.
    /// - The new state.
    LikedStateChanged(String, bool)
}

impl Display for FavoriteEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FavoriteEvent::LikedStateChanged(id, new_state) => write!(f, "Like state changed of {} to {}", id, new_state),
        }
    }
}

/// The favorite service is stores & retrieves liked media items based on the ID.
#[derive(Debug)]
pub struct FavoriteService {
    storage: Arc<Storage>,
    cache: Arc<Mutex<Option<Favorites>>>,
    callbacks: FavoriteCallbacks,
}

impl FavoriteService {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            cache: Arc::new(Mutex::new(None)),
            callbacks: FavoriteCallbacks::new(),
        }
    }

    /// Verify if the given [Favorable] media items is liked by the user.
    pub fn is_liked(&self, favorable: &impl MediaIdentifier) -> bool {
        let imdb_id = favorable.imdb_id();

        self.internal_is_liked(&imdb_id, &favorable.media_type())
    }

    /// Verify if the given [Favorable] media items is liked by the user.
    pub fn is_liked_boxed(&self, favorable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = favorable.imdb_id();
        let media_type = favorable.media_type();

        self.internal_is_liked(&imdb_id, &media_type)
    }

    /// Retrieve an array of owned liked [MediaOverview] items.
    ///
    /// It returns the liked media items when loaded, else the [MediaError].
    pub fn all(&self) -> media::Result<Vec<Box<dyn MediaOverview>>> {
        trace!("Retrieving all favorite media items");
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let favorites = cache.as_ref().expect("cache should have been present");
                let mut all: Vec<Box<dyn MediaOverview>> = vec![];
                let mut movies: Vec<Box<dyn MediaOverview>> = favorites.movies().iter()
                    .map(|e| e.clone())
                    .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                    .collect();
                let mut shows: Vec<Box<dyn MediaOverview>> = favorites.shows().iter()
                    .map(|e| e.clone())
                    .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                    .collect();

                all.append(&mut movies);
                all.append(&mut shows);

                Ok(all)
            }
            Err(e) => Err(MediaError::FavoritesLoadingFailed(e.to_string()))
        }
    }

    /// Retrieve the liked [MediaOverview] item by ID.
    ///
    /// It returns the media item when found, else [None].
    pub fn find_id(&self, imdb_id: &String) -> Option<Box<dyn MediaOverview>> {
        match self.all() {
            Ok(favorites) => {
                favorites.into_iter()
                    .find(|e| e.imdb_id().eq(imdb_id))
            }
            Err(_) => None
        }
    }

    /// Add the given media item to the favorites.
    /// Only overview items of type [MovieOverview] or [ShowOverview] are supported.
    pub fn add(&self, favorite: Box<dyn MediaIdentifier>) -> media::Result<()> {
        let _ = futures::executor::block_on(self.load_favorites_cache())?;
        let mutex = self.cache.clone();
        let mut cache = futures::executor::block_on(mutex.lock());
        let mut e = cache.as_mut().expect("cache should have been present");
        let imdb_id = favorite.imdb_id();
        let media_type = favorite.media_type();

        match media_type {
            MediaType::Movie => {
                match favorite.into_any().downcast::<MovieOverview>() {
                    Ok(media) => e.add_movie(&media),
                    Err(_) => {
                        return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
                    }
                }
            }
            MediaType::Show => {
                match favorite.into_any().downcast::<ShowOverview>() {
                    Ok(media) => e.add_show(&media),
                    Err(_) => {
                        return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
                    }
                }
            }
            _ => {
                return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
            }
        }

        // invoke callbacks
        self.callbacks.invoke(FavoriteEvent::LikedStateChanged(imdb_id, true));

        self.save(&mut e);
        Ok(())
    }

    /// Remove the media item from the favorites.
    pub fn remove(&self, favorite: Box<dyn MediaIdentifier>) {
        trace!("Removing media item {} from favorites", &favorite);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let imdb_id = favorite.imdb_id();
                let mutex = self.cache.clone();
                let mut cache = futures::executor::block_on(mutex.lock());
                let mut e = cache.as_mut().expect("cache should have been present");

                e.remove_id(&imdb_id);

                // invoke callbacks
                self.callbacks.invoke(FavoriteEvent::LikedStateChanged(imdb_id, false));

                self.save(&mut e);
            }
            Err(e) => error!("Failed to add {} as favorite, {}", favorite, e)
        }
    }

    /// Register the given callback to the favorite events.
    /// The callback will be invoked when an event happens within this service.
    pub fn register(&self, callback: FavoriteCallback) {
        self.callbacks.add(callback)
    }

    fn internal_is_liked(&self, imdb_id: &String, media_type: &MediaType) -> bool {
        trace!("Internally verifying if {} {} is liked", media_type, imdb_id);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let favorites = cache.as_ref().expect("cache should have been present");
                trace!("Checking is liked for media type {}", media_type);
                match media_type {
                    MediaType::Movie => {
                        favorites.movies()
                            .iter()
                            .any(|e| e.imdb_id().eq(imdb_id))
                    }
                    MediaType::Show => {
                        favorites.shows()
                            .iter()
                            .any(|e| e.imdb_id().eq(imdb_id))
                    }
                    _ => {
                        warn!("Media type {} is not supported as favorite", media_type);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("Unable to load {}, {}", FILENAME, e);
                false
            }
        }
    }

    fn save(&self, favorites: &Favorites) {
        match Handle::try_current() {
            Ok(e) => e.block_on(self.save_async(favorites)),
            Err(_) => {
                let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
                runtime.block_on(self.save_async(favorites));
            }
        }
    }

    async fn save_async(&self, favorites: &Favorites) {
        match self.storage.write(FILENAME, &favorites).await {
            Ok(_) => info!("Favorites have been saved"),
            Err(e) => error!("Failed to save favorites, {}", e)
        }
    }

    async fn load_favorites_cache(&self) -> media::Result<()> {
        let mutex = self.cache.clone();
        let mut cache = mutex.lock().await;

        if cache.is_none() {
            trace!("Loading favorites cache");
            return match self.load_favorites_from_storage() {
                Ok(e) => {
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e)
            };
        }

        trace!("Favorites cache already loaded, nothing to do");
        Ok(())
    }

    fn load_favorites_from_storage(&self) -> media::Result<Favorites> {
        match self.storage.read::<Favorites>(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::FileNotFound(file) => {
                        debug!("Creating new favorites file {}", file);
                        Ok(Favorites::empty())
                    }
                    StorageError::CorruptRead(_, error) => {
                        error!("Failed to load favorites, {}", error);
                        Err(MediaError::FavoritesLoadingFailed(error))
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", e);
                        Ok(Favorites::empty())
                    }
                }
            }
        }
    }
}

impl Drop for FavoriteService {
    fn drop(&mut self) {
        let mutex = self.cache.clone();
        let execute = async move {
            let favorites = mutex.lock().await;

            if favorites.is_some() {
                debug!("Saving favorites on exit");
                let e = favorites.as_ref().expect("Expected the favorites to be present");
                self.save_async(e).await
            }
        };

        match Handle::try_current() {
            Ok(e) => {
                trace!("Using handle on exit");
                e.block_on(execute)
            },
            Err(_) => {
                let runtime = tokio::runtime::Runtime::new().expect("expected a new runtime");
                runtime.block_on(execute)
            }
        }
    }
}

struct FavoriteCallbacks {
    callbacks: Arc<Mutex<Vec<FavoriteCallback>>>,
}

impl FavoriteCallbacks {
    fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(vec![])),
        }
    }

    fn add(&self, callback: FavoriteCallback) {
        trace!("Registering new callback for favorite events");
        match Handle::try_current() {
            Ok(e) => {
                e.block_on(self.add_async(callback));
            }
            Err(_) => {
                // let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to be created");
                futures::executor::block_on(self.add_async(callback));
            }
        }
    }

    async fn add_async(&self, callback: FavoriteCallback) {
        let callbacks = self.callbacks.clone();
        let mut mutex = callbacks.lock().await;

        mutex.push(callback);
        debug!("Added new callback for FavoriteEvent events, new total callbacks {}", mutex.len());
    }

    fn invoke(&self, event: FavoriteEvent) {
        let callbacks = self.callbacks.clone();
        let execute = async move {
            let mutex = callbacks.lock().await;

            debug!("Calling a total of {} callbacks for: {}", mutex.len(), &event);
            for callback in mutex.iter() {
                callback(event.clone());
            }
        };

        match Handle::try_current() {
            Ok(e) => e.block_on(execute),
            Err(_) => {
                let runtime = tokio::runtime::Runtime::new().expect("expected a new runtime");
                runtime.block_on(execute)
            }
        }
    }
}

impl Debug for FavoriteCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mutex = futures::executor::block_on(self.callbacks.lock());
        write!(f, "FavoriteCallbacks {{callbacks: {}}}", mutex.len())
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::media::MovieOverview;
    use crate::testing::{init_logger, test_resource_directory};

    use super::*;

    #[test]
    fn test_is_liked_when_favorable_is_not_liked_should_return_false() {
        init_logger();
        let imdb_id = String::from("tt9387250");
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&storage);
        let media = MovieOverview::new(
            String::new(),
            imdb_id.clone(),
            String::new(),
        );

        let result = service.is_liked(&media);

        assert_eq!(false, result)
    }

    #[test]
    fn test_is_liked_when_favorable_is_liked_should_return_true() {
        init_logger();
        let imdb_id = String::from("tt1156398");
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&storage);
        let media = MovieOverview::new(
            String::new(),
            imdb_id.clone(),
            String::new(),
        );

        let result = service.is_liked(&media);

        assert_eq!(true, result)
    }

    #[test]
    fn test_all() {
        init_logger();
        let resource_directory = test_resource_directory();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&storage);
        let result = service.all()
            .expect("Expected the favorites to have been retrieved");

        let result = result.get(0).expect("expected at least one result");

        assert_eq!("tt1156398".to_string(), result.imdb_id());
        assert_eq!("Zombieland".to_string(), result.title());
        assert_eq!(MediaType::Movie, result.media_type());
    }

    #[test]
    fn test_add_not_favorite_media() {
        init_logger();
        let imdb_id = "tt12345678";
        let title = "lorem ipsum";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let storage = Arc::new(Storage::from_directory(temp_dir.path().to_str().expect("expected temp dir path to be valid")));
        let service = FavoriteService::new(&storage);
        let movie = Box::new(MovieOverview::new(
            String::from(title),
            String::from(imdb_id),
            String::new(),
        )) as Box<dyn MediaIdentifier>;

        service.add(movie)
            .expect("expected the favorite media item add to have succeeded");
        let result = service.all()
            .expect("expected the favorites to have been loaded");

        assert_eq!(1, result.len());
        let media = result.get(0).unwrap();
        assert_eq!(imdb_id.to_string(), media.imdb_id());
        assert_eq!(title.to_string(), media.title());
    }

    #[test]
    fn test_remove_favorite_media() {
        init_logger();
        let imdb_id = "tt12345666";
        let title = "lorem ipsum";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let storage = Arc::new(Storage::from_directory(temp_dir.path().to_str().expect("expected temp dir path to be valid")));
        let service = FavoriteService::new(&storage);
        let movie = MovieOverview::new(
            String::from(title),
            String::from(imdb_id),
            String::new(),
        );

        service.add(Box::new(movie.clone()))
            .expect("expected the media to have been added to liked items");
        service.remove(Box::new(movie));
        let result = service.all()
            .expect("expected the favorites to have been loaded");

        assert_eq!(0, result.len());
    }
}