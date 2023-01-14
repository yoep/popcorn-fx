use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::core::media;
use crate::core::media::{MediaError, MediaIdentifier, MediaOverview, MediaType, MovieOverview, ShowOverview};
use crate::core::media::favorites::model::Favorites;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "favorites.json";

/// The favorite service is stores & retrieves liked media items based on the ID.
#[derive(Debug)]
pub struct FavoriteService {
    storage: Arc<Storage>,
    mutex: Arc<Mutex<Option<Favorites>>>,
}

impl FavoriteService {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            mutex: Arc::new(Mutex::new(None)),
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
                let mutex = self.mutex.clone();
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

    /// Add the given [Favorable] media item to the favorites.
    pub fn add(&self, favorite: Box<dyn MediaIdentifier>) {
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.mutex.clone();
                let mut cache = futures::executor::block_on(mutex.lock());
                let mut e = cache.as_mut().expect("cache should have been present");
                match favorite.media_type() {
                    MediaType::Movie => {
                        e.add_movie(&favorite.into_any()
                            .downcast::<MovieOverview>()
                            .expect("expected the favorite to be a movie overview"));
                    }
                    MediaType::Show => {
                        e.add_show(&favorite.into_any()
                            .downcast::<ShowOverview>()
                            .expect("expected the favorite to be a show overview"));
                    }
                    _ => error!("Unable to add media to favorites, media type {} is not supported", favorite.media_type())
                }

                self.save(&mut e);
            }
            Err(e) => {
                error!("Failed to add {} as favorite, {}", favorite, e);
            }
        }
    }

    /// Remove the media item from the favorites.
    pub fn remove(&self, favorite: Box<dyn MediaIdentifier>) {
        trace!("Removing media item {} from favorites", &favorite);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.mutex.clone();
                let mut cache = futures::executor::block_on(mutex.lock());
                let mut e = cache.as_mut().expect("cache should have been present");

                e.remove_id(&favorite.imdb_id());
                self.save(&mut e);
            }
            Err(e) => error!("Failed to add {} as favorite, {}", favorite, e)
        }
    }

    fn internal_is_liked(&self, imdb_id: &String, media_type: &MediaType) -> bool {
        trace!("Internally verifying if {} {} is liked", media_type, imdb_id);
        match futures::executor::block_on(self.load_favorites_cache()) {
            Ok(_) => {
                let mutex = self.mutex.clone();
                trace!("Acquiring favorites cache lock");
                let cache = futures::executor::block_on(mutex.lock());
                trace!("Acquired favorites cache lock");
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
        let mutex = self.mutex.clone();
        trace!("Acquiring favorites cache lock");
        let mut cache = mutex.lock().await;

        trace!("Acquired cache lock, checking cache state");
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

    fn load_favorites_from_storage(&self) -> Result<Favorites, MediaError> {
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

        service.add(movie);
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

        service.add(Box::new(movie.clone()));
        service.remove(Box::new(movie));
        let result = service.all()
            .expect("expected the favorites to have been loaded");

        assert_eq!(0, result.len());
    }
}