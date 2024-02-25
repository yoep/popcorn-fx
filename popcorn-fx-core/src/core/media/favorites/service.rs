use std::fmt::Debug;

use derive_more::Display;
use log::{debug, error, info, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::{block_in_place, Callbacks, CoreCallback, CoreCallbacks, media};
use crate::core::media::{MediaError, MediaIdentifier, MediaOverview, MediaType, MovieOverview, ShowOverview};
use crate::core::media::favorites::model::Favorites;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "favorites.json";

/// The callback to listen on events of the favorite service.
pub type FavoriteCallback = CoreCallback<FavoriteEvent>;

/// The events that can be produced by the [FavoriteService].
#[derive(Debug, Clone, Display)]
pub enum FavoriteEvent {
    /// Invoked when a media item's liked state has changed.
    ///
    /// * The IMDB ID of the media item for which the state changed.
    /// * The new state.
    #[display(fmt = "Like state changed of {} to {}", _0, _1)]
    LikedStateChanged(String, bool)
}

#[cfg_attr(any(test, feature = "testing"), automock)]
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

    /// Update the existing liked items with the new given information.
    /// This will update only existing items (non-existing items won't be added).
    fn update(&self, favorites: Vec<Box<dyn MediaIdentifier>>);

    /// Retrieve a copy of the current [Favorites]/liked items.
    ///
    /// It returns the a copy when available, else [None].
    fn favorites(&self) -> Option<Favorites>;

    /// Register the given callback to the favorite events.
    /// The callback will be invoked when an event happens within this service.
    fn register(&self, callback: FavoriteCallback);
}

/// The standard favorite service which stores & retrieves liked media items based on the ID.
#[derive(Debug)]
pub struct DefaultFavoriteService {
    storage: Storage,
    favorites: Mutex<Favorites>,
    callbacks: CoreCallbacks<FavoriteEvent>,
}

impl DefaultFavoriteService {
    /// Create a new favorite service with default behavior.
    ///
    /// * `storage_directory` - The directory to use to read & store the favorites.
    pub fn new(storage_path: &str) -> Self {
        let storage = Storage::from(storage_path);
        let favorites = match storage.options()
            .serializer(FILENAME)
            .read::<Favorites>() {
            Ok(e) => {
                info!("Favorites have been loaded from {} ({} total items)", FILENAME, e.movies().len() + e.shows().len());
                e
            },
            Err(error) => {
                match error {
                    StorageError::NotFound(file) => {
                        debug!("Creating new favorites file {}", file);
                    }
                    StorageError::ReadingFailed(_, error) => {
                        error!("Failed to load favorites, {}", error);
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", error);
                    }
                }

                Favorites::default()
            }
        };

        Self {
            storage,
            favorites: Mutex::new(favorites),
            callbacks: CoreCallbacks::default(),
        }
    }

    fn save(&self, favorites: &Favorites) {
        block_in_place(self.save_async(favorites))
    }

    async fn save_async(&self, favorites: &Favorites) {
        match self.storage.options()
            .serializer(FILENAME)
            .write_async(favorites).await {
            Ok(_) => info!("Favorites have been saved"),
            Err(e) => error!("Failed to save favorites, {}", e)
        }
    }
}

impl FavoriteService for DefaultFavoriteService {
    fn is_liked(&self, id: &str) -> bool {
        trace!("Verifying if media item {} is liked", id);
        let favorites = futures::executor::block_on(self.favorites.lock());
        favorites.contains(id)
    }

    fn is_liked_dyn(&self, favorable: &Box<dyn MediaIdentifier>) -> bool {
        let imdb_id = favorable.imdb_id();

        self.is_liked(imdb_id)
    }

    fn all(&self) -> media::Result<Vec<Box<dyn MediaOverview>>> {
        trace!("Retrieving all favorite media items");
        let favorites = futures::executor::block_on(self.favorites.lock());
        let mut all: Vec<Box<dyn MediaOverview>> = vec![];
        trace!("Cloning a total of {} movie items", favorites.movies().len());
        let mut movies: Vec<Box<dyn MediaOverview>> = favorites.movies().iter()
            .map(|e| e.clone())
            .map(|e| Box::new(e) as Box<dyn MediaOverview>)
            .collect();
        trace!("Cloning a total of {} show items", favorites.shows().len());
        let mut shows: Vec<Box<dyn MediaOverview>> = favorites.shows().iter()
            .map(|e| e.clone())
            .map(|e| Box::new(e) as Box<dyn MediaOverview>)
            .collect();

        all.append(&mut movies);
        all.append(&mut shows);

        debug!("Retrieved a total of {} favorite items", all.len());
        Ok(all)
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
        trace!("Adding favorite media item {:?}", favorite);
        let mut favorites = futures::executor::block_on(self.favorites.lock());
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
        let imdb_id = favorite.imdb_id();
        let mut favorites = futures::executor::block_on(self.favorites.lock());

        favorites.remove_id(imdb_id);

        // invoke callbacks
        self.save(&mut favorites);
        self.callbacks.invoke(FavoriteEvent::LikedStateChanged(imdb_id.to_string(), false));
    }

    fn update(&self, favorites: Vec<Box<dyn MediaIdentifier>>) {
        let mut cache = futures::executor::block_on(self.favorites.lock());

        for media in favorites.into_iter() {
            if !cache.contains(media.imdb_id()) {
                warn!("Unable to update favorite {}, media is not stored as a favorite item", media.imdb_id());
                continue;
            }

            match media.media_type() {
                MediaType::Movie => {
                    let movie = media.into_any()
                        .downcast::<MovieOverview>()
                        .expect("expected MovieOverview");
                    cache.remove_id(movie.imdb_id());
                    cache.add_movie(&*movie);
                }
                MediaType::Show => {
                    let show = media.into_any()
                        .downcast::<ShowOverview>()
                        .expect("expected ShowOverview");
                    cache.remove_id(show.imdb_id());
                    cache.add_show(&*show);
                }
                _ => warn!("Unable to update media item {} type {}", media.imdb_id(), media.media_type())
            }
        }

        cache.last_cache_update = Favorites::current_datetime();
        debug!("Favorites have been updated at {}", &cache.last_cache_update);
    }

    fn favorites(&self) -> Option<Favorites> {
        Some(futures::executor::block_on(self.favorites.lock()).clone())
    }

    fn register(&self, callback: FavoriteCallback) {
        self.callbacks.add(callback);
    }
}

impl Drop for DefaultFavoriteService {
    fn drop(&mut self) {
        block_in_place(async move {
            let favorites = self.favorites.lock().await;
            debug!("Saving favorites on exit");
            self.save_async(&favorites).await
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::core::media::{Images, MovieOverview, Rating};
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
    fn test_find_id() {
        init_logger();
        let imdb_id = String::from("tt8111666");
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let service = DefaultFavoriteService::new(temp_path);

        let result = service.find_id(imdb_id.as_str());

        match result {
            Some(e) => {
                assert_eq!(imdb_id, e.imdb_id());
                assert_eq!("Ipsum".to_string(), e.title())
            }
            None => assert!(false, "expected the ID to have been found")
        }
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
        assert_eq!("Lorem".to_string(), result.title());
        assert_eq!(MediaType::Movie, result.media_type());
    }

    #[test]
    fn test_add_new_movie_item() {
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
    fn test_add_new_show_item() {
        init_logger();
        let imdb_id = "tt12345678";
        let title = "lorem ipsum";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultFavoriteService::new(temp_path);
        let show = Box::new(ShowOverview::new(
            String::from(imdb_id),
            String::from(imdb_id),
            String::from(title),
            String::new(),
            2,
            Default::default(),
            None,
        )) as Box<dyn MediaIdentifier>;

        service.add(show)
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
    fn test_favorites() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let service = DefaultFavoriteService::new(temp_path);
        let movies = vec![MovieOverview {
            title: "Lorem".to_string(),
            imdb_id: "tt1156398".to_string(),
            year: "2009".to_string(),
            rating: Some(Rating {
                percentage: 72,
                watching: 1,
                votes: 22330,
                loved: 0,
                hated: 0,
            }),
            images: Images {
                poster: "http://localhost/img.jpg".to_string(),
                fanart: "http://localhost/img.jpg".to_string(),
                banner: "http://localhost/img.jpg".to_string(),
            },
        }];

        let favorites = service.favorites()
            .expect("expected favorites to be present");

        assert_eq!(movies, favorites.movies)
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

    #[test]
    fn test_update() {
        init_logger();
        let movie_id = "tt111122244";
        let show_id = "tt111125555";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let movie = MovieOverview {
            imdb_id: movie_id.to_string(),
            title: "lorem".to_string(),
            year: "".to_string(),
            rating: None,
            images: Default::default(),
        };
        let updated_movie = MovieOverview {
            imdb_id: movie_id.to_string(),
            title: "ipsum".to_string(),
            year: "2019".to_string(),
            rating: None,
            images: Default::default(),
        };
        let show = ShowOverview {
            imdb_id: show_id.to_string(),
            tvdb_id: "".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Default::default(),
            rating: None,
        };
        let updated_show = ShowOverview {
            imdb_id: show_id.to_string(),
            tvdb_id: show_id.to_string(),
            title: "lipsum dolor".to_string(),
            year: "2011".to_string(),
            num_seasons: 3,
            images: Default::default(),
            rating: None,
        };
        let service = DefaultFavoriteService::new(temp_path);

        service.add(Box::new(movie)).expect("expected the movie to have been added");
        service.add(Box::new(show)).expect("expected the show to have been added");
        service.update(vec![Box::new(updated_movie.clone()), Box::new(updated_show.clone())]);

        let movie_result = service.find_id(movie_id)
            .expect("expected movie to be found")
            .into_any()
            .downcast::<MovieOverview>()
            .unwrap();
        let show_result = service.find_id(show_id)
            .expect("expected show to be found")
            .into_any()
            .downcast::<ShowOverview>()
            .unwrap();

        assert_eq!(updated_movie, *movie_result);
        assert_eq!(updated_show, *show_result);
    }
}