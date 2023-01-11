use std::sync::Arc;

use log::{error, trace, warn};

use crate::core::config::Application;
use crate::core::media;
use crate::core::media::{Favorable, MediaError, MediaIdentifier, MediaOverview, MediaType};
use crate::core::media::favorites::model::Favorites;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "favorites.json";

/// The favorite service is stores & retrieves liked media items based on the ID.
#[derive(Debug)]
pub struct FavoriteService {
    settings: Arc<Application>,
    storage: Arc<Storage>,
}

impl FavoriteService {
    pub fn new(settings: &Arc<Application>, storage: &Arc<Storage>) -> Self {
        Self {
            settings: settings.clone(),
            storage: storage.clone(),
        }
    }

    /// Verify if the given [Favorable] media items is liked by the user.
    pub fn is_liked(&self, favorable: &impl Favorable) -> bool {
        let imdb_id = favorable.imdb_id();

        match self.load_favorites() {
            Ok(favorites) => {
                match favorable.media_type() {
                    MediaType::Movie => {
                        favorites.movies()
                            .iter()
                            .any(|e| e.imdb_id().eq(&imdb_id))
                    }
                    MediaType::Show => {
                        favorites.shows()
                            .iter()
                            .any(|e| e.imdb_id().eq(&imdb_id))
                    }
                    _ => {
                        warn!("Media type {} is not supported as favorite", favorable.media_type());
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

    /// Retrieve an array of owned liked [MediaOverview] items.
    ///
    /// It returns the liked media items when loaded, else the [MediaError].
    pub fn all(&self) -> media::Result<Vec<Box<dyn MediaOverview>>> {
        match self.load_favorites() {
            Ok(favorites) => {
                let mut all: Vec<Box<dyn MediaOverview>> = vec![];
                let mut movies: Vec<Box<dyn MediaOverview>> = favorites.movies().iter()
                    .map(|e| Box::new(e.clone()) as Box<dyn MediaOverview>)
                    .collect();
                let mut shows: Vec<Box<dyn MediaOverview>> = favorites.shows().iter()
                    .map(|e| Box::new(e.clone()) as Box<dyn MediaOverview>)
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

    fn load_favorites(&self) -> media::Result<Favorites> {
        match self.storage.read::<Favorites>(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::FileNotFound(file) => {
                        trace!("Favorites file {} not found, using new favorites instead", file);
                        Ok(Favorites::empty())
                    }
                    StorageError::CorruptData(_, error) => {
                        error!("Failed to load favorites, {}", error);
                        Err(MediaError::FavoritesLoadingFailed(error))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::MovieOverview;
    use crate::test::{init_logger, test_resource_directory};

    use super::*;

    #[test]
    fn test_is_liked_when_favorable_is_not_liked_should_return_false() {
        init_logger();
        let imdb_id = String::from("tt9387250");
        let resource_directory = test_resource_directory();
        let settings = Arc::new(Application::default());
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&settings, &storage);
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
        let settings = Arc::new(Application::default());
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&settings, &storage);
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
        let settings = Arc::new(Application::default());
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let service = FavoriteService::new(&settings, &storage);
        let result = service.all()
            .expect("Expected the favorites to have been retrieved");

        let result = result.get(0).expect("expected at least one result");

        assert_eq!("tt1156398".to_string(), result.imdb_id());
        assert_eq!("Zombieland".to_string(), result.title());
        assert_eq!(MediaType::Movie, result.media_type());
    }
}