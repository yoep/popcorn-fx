use std::fmt::Debug;
use std::sync::Arc;

use derive_more::Display;
use log::{debug, error, info, trace, warn};
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::{block_in_place, CoreCallback, CoreCallbacks, media};
use crate::core::media::{MediaError, MediaIdentifier, MediaOverview, MediaType, MovieOverview, ShowOverview};
use crate::core::media::favorites::model::Favorites;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "favorites.json";

/// The callback to listen on events of the favorite service.
pub type FavoriteCallback = CoreCallback<FavoriteEvent>;

#[derive(Debug, Clone, Display)]
pub enum FavoriteEvent {
    /// Invoked when a media item's liked state has changed.
    ///
    /// - The IMDB ID of the media item for which the state changed.
    /// - The new state.
    #[display(fmt = "Like state changed of {} to {}", _0, _1)]
    LikedStateChanged(String, bool)
}

#[automock]
pub trait FavoriteService: Debug + Send + Sync {
    /// Verify if the given media item id is liked.
    fn is_liked(&self, id: &str) -> bool;

    /// Verify if the given [Favorable] media items is liked by the user.
    fn is_liked_dyn(&self, favorable: &Box<dyn MediaIdentifier>) -> bool;

    /// Retrieve an array of owned liked [MediaOverview] items.
    ///
    /// It returns the liked media items when loaded, else the [MediaError].
    fn all(&self) -> media::Result<Vec<Box<dyn MediaOverview>>>;

    /// Retrieve the liked [MediaOverview] item by ID.
    ///
    /// It returns the media item when found, else [None].
    fn find_id(&self, imdb_id: &str) -> Option<Box<dyn MediaOverview>>;

    /// Add the given media item to the favorites.
    /// Only overview items of type [MovieOverview] or [ShowOverview] are supported.
    fn add(&self, favorite: Box<dyn MediaIdentifier>) -> media::Result<()>;

    /// Remove the media item from the favorites.
    /// Not liked favorite item will just be ignored and not result in an error.
    fn remove(&self, favorite: Box<dyn MediaIdentifier>);

    /// Register the given callback to the favorite events.
    /// The callback will be invoked when an event happens within this service.
    fn register(&self, callback: FavoriteCallback);
}

/// The standard favorite service which stores & retrieves liked media items based on the ID.
#[derive(Debug)]
pub struct DefaultFavoriteService {
    storage: Arc<Storage>,
    cache: Arc<Mutex<Option<Favorites>>>,
    callbacks: CoreCallbacks<FavoriteEvent>,
}

impl DefaultFavoriteService {
    /// Create a new favorite service with default behavior.
    ///
    /// * `storage_directory` - The directory to use to read & store the favorites.
    pub fn new(storage_path: &str) -> Self {
        Self {
            storage: Arc::new(Storage::from(storage_path)),
            cache: Arc::new(Mutex::new(None)),
            callbacks: CoreCallbacks::default(),
        }
    }

    fn save(&self, favorites: &Favorites) {
        block_in_place(self.save_async(favorites))
    }

    async fn save_async(&self, favorites: &Favorites) {
        match self.storage.write_async(FILENAME, &favorites).await {
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
        match self.storage.read(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::NotFound(file) => {
                        debug!("Creating new favorites file {}", file);
                        Ok(Favorites::default())
                    }
                    StorageError::ReadingFailed(_, error) => {
                        error!("Failed to load favorites, {}", error);
                        Err(MediaError::FavoritesLoadingFailed(error))
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", e);
                        Ok(Favorites::default())
                    }
                }
            }
        }
    }
}

impl FavoriteService for DefaultFavoriteService {
    fn is_liked(&self, id: &str) -> bool {
        trace!("Verifying if media item {} is liked", id);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.cache.clone();
                let cache = futures::executor::block_on(mutex.lock());
                let favorites = cache.as_ref().expect("cache should have been present");

                favorites.contains(id)
            }
            Err(e) => {
                warn!("Unable to load {}, {}", FILENAME, e);
                false
            }
        }
    }

    fn is_liked_dyn(&self, favorable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = favorable.imdb_id();

        self.is_liked(imdb_id)
    }

    fn all(&self) -> media::Result<Vec<Box<dyn MediaOverview>>> {
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

    fn find_id(&self, imdb_id: &str) -> Option<Box<dyn MediaOverview>> {
        match self.all() {
            Ok(favorites) => {
                favorites.into_iter()
                    .find(|e| e.imdb_id() == imdb_id)
            }
            Err(_) => None
        }
    }

    fn add(&self, favorite: Box<dyn MediaIdentifier>) -> media::Result<()> {
        futures::executor::block_on(self.load_favorites_cache())?;
        let mutex = self.cache.clone();
        let mut cache = futures::executor::block_on(mutex.lock());
        let favorites = cache.as_mut().expect("cache should have been present");
        let imdb_id = favorite.imdb_id().to_string();
        let media_type = favorite.media_type();

        match media_type {
            MediaType::Movie => {
                match favorite.into_any().downcast::<MovieOverview>() {
                    Ok(media) => favorites.add_movie(&media),
                    Err(_) => {
                        return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
                    }
                }
            }
            MediaType::Show => {
                match favorite.into_any().downcast::<ShowOverview>() {
                    Ok(media) => favorites.add_show(&media),
                    Err(_) => {
                        return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
                    }
                }
            }
            _ => {
                return Err(MediaError::FavoriteAddFailed(imdb_id, format!("media type {} is not supported", media_type)));
            }
        }

        self.save(&favorites);
        self.callbacks.invoke(FavoriteEvent::LikedStateChanged(imdb_id, true));
        Ok(())
    }

    fn remove(&self, favorite: Box<dyn MediaIdentifier>) {
        trace!("Removing media item {} from favorites", &favorite);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let imdb_id = favorite.imdb_id();
                let mutex = self.cache.clone();
                let mut cache = futures::executor::block_on(mutex.lock());
                let mut e = cache.as_mut().expect("cache should have been present");

                e.remove_id(imdb_id);

                // invoke callbacks
                self.save(&mut e);
                self.callbacks.invoke(FavoriteEvent::LikedStateChanged(imdb_id.to_string(), false));
            }
            Err(e) => error!("Failed to add {} as favorite, {}", favorite, e)
        }
    }

    fn register(&self, callback: FavoriteCallback) {
        self.callbacks.add(callback)
    }
}

impl Drop for DefaultFavoriteService {
    fn drop(&mut self) {
        let mutex = self.cache.clone();

        block_in_place(async move {
            let favorites = mutex.lock().await;

            if favorites.is_some() {
                debug!("Saving favorites on exit");
                let e = favorites.as_ref().expect("Expected the favorites to be present");
                self.save_async(e).await
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::core::media::MovieOverview;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_is_liked_when_favorable_is_not_liked_should_return_false() {
        init_logger();
        let imdb_id = String::from("tt9387250");
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "settings.json", None);
        let service = DefaultFavoriteService::new(temp_path);

        let result = service.is_liked(imdb_id.as_str());

        assert_eq!(false, result)
    }

    #[test]
    fn test_is_liked_when_favorable_is_liked_should_return_true() {
        init_logger();
        let imdb_id = String::from("tt1156398");
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let service = DefaultFavoriteService::new(temp_path);

        let result = service.is_liked(imdb_id.as_str());

        assert_eq!(true, result)
    }

    #[test]
    fn test_all() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let service = DefaultFavoriteService::new(temp_path);
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
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultFavoriteService::new(temp_path);
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
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultFavoriteService::new(temp_path);
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

    #[test]
    fn test_register_when_add_is_called_should_invoke_callback() {
        init_logger();
        let id = "tt1122333";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultFavoriteService::new(temp_path);
        let (tx, rx) = channel();
        let movie: Box<dyn MediaIdentifier> = Box::new(MovieOverview::new(
            String::new(),
            id.to_string(),
            String::new(),
        ));

        service.register(Box::new(move |e| {
            tx.send(e).unwrap();
        }));
        service.add(movie).unwrap();

        let result = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        match result {
            FavoriteEvent::LikedStateChanged(imdb_id, state) => {
                assert_eq!(id.to_string(), imdb_id);
                assert_eq!(true, state)
            }
        }
    }
}