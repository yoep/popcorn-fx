use std::sync::Arc;

use chrono::{Duration, Local};
use itertools::Itertools;
use log::{debug, info, trace, warn};
use tokio::runtime::Runtime;

use crate::core::media::{Category, MediaIdentifier, MediaType, MovieDetails, ShowDetails};
use crate::core::media::favorites::FavoriteService;
use crate::core::media::favorites::model::Favorites;
use crate::core::media::providers::ProviderManager;

const UPDATE_CACHE_INTERVAL: fn() -> Duration = || Duration::hours(72);

/// The favorite cache updater is responsible for updating the favorites cache if needed.
/// It makes use of the [Provider] for retrieving the latest information.
#[derive(Debug)]
pub struct FavoriteCacheUpdater {
    inner: Arc<InnerCacheUpdater>,
    runtime: Arc<Runtime>,
}

impl FavoriteCacheUpdater {
    pub fn builder() -> FavoriteCacheUpdaterBuilder {
        FavoriteCacheUpdaterBuilder::default()
    }

    fn start_cache_update_check(&self) {
        let inner = self.inner.clone();
        self.runtime.spawn(async move {
            match inner.cache() {
                Some(cache) => {
                    let last_update_diff = Local::now() - cache.last_update();

                    trace!("Favorite cache last updated {} hours ago", last_update_diff.num_hours());
                    if last_update_diff >= UPDATE_CACHE_INTERVAL() {
                        debug!("Starting favorite cache update");
                        let updated_items = inner.update_media_items(cache).await;
                        let total_items = updated_items.len();
                        inner.service.update(updated_items);
                        info!("Updated a total of {} favorite media items", total_items)
                    } else {
                        debug!("Favorites are already up-to-date, not updating cache");
                    }
                }
                None => debug!("No favorites cache available to update"),
            }
        });
    }
}

#[derive(Debug)]
pub struct FavoriteCacheUpdaterBuilder {
    runtime: Option<Arc<Runtime>>,
    favorite_service: Option<Arc<Box<dyn FavoriteService>>>,
    provider_manager: Option<Arc<ProviderManager>>,
}

impl FavoriteCacheUpdaterBuilder {
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn favorite_service(mut self, favorite_service: Arc<Box<dyn FavoriteService>>) -> Self {
        self.favorite_service = Some(favorite_service);
        self
    }

    pub fn provider_manager(mut self, provider_manager: Arc<ProviderManager>) -> Self {
        self.provider_manager = Some(provider_manager);
        self
    }

    pub fn build(self) -> FavoriteCacheUpdater {
        let instance = FavoriteCacheUpdater {
            runtime: self.runtime
                .or_else(|| Some(Arc::new(Runtime::new().unwrap())))
                .unwrap(),
            inner: Arc::new(InnerCacheUpdater {
                service: self.favorite_service.unwrap().clone(),
                providers: self.provider_manager.unwrap().clone(),
            }),
        };

        instance.start_cache_update_check();
        instance
    }
}

impl Default for FavoriteCacheUpdaterBuilder {
    fn default() -> Self {
        FavoriteCacheUpdaterBuilder {
            runtime: None,
            favorite_service: None,
            provider_manager: None,
        }
    }
}

#[derive(Debug)]
struct InnerCacheUpdater {
    /// The favorite service containing the cache
    service: Arc<Box<dyn FavoriteService>>,
    /// The provider manager which can provide the new media details
    providers: Arc<ProviderManager>,
}

impl InnerCacheUpdater {
    /// Retrieve a cached [Favorites] instance
    fn cache(&self) -> Option<Favorites> {
        self.service.favorites()
    }

    async fn update_media_items(&self, cache: Favorites) -> Vec<Box<dyn MediaIdentifier>> {
        trace!("Merging all favorites into one MediaIdentifier array");
        let mut media_items: Vec<Box<dyn MediaIdentifier>> = vec![];
        media_items.append(&mut cache.movies.into_iter()
            .map(|e| Box::new(e) as Box<dyn MediaIdentifier>)
            .collect_vec());
        media_items.append(&mut cache.shows.into_iter()
            .map(|e| Box::new(e) as Box<dyn MediaIdentifier>)
            .collect_vec());

        debug!("Updating a total of {} favorite items", media_items.len());
        futures::future::join_all(media_items.into_iter()
            .map(|media| async {
                let category = Category::from(media.media_type());

                match self.providers.retrieve_details(&category, media.imdb_id()).await {
                    Ok(e) => {
                        trace!("Retrieved updated media item {}", e);
                        match e.media_type() {
                            MediaType::Movie => Box::new(e.into_any()
                                .downcast::<MovieDetails>()
                                .expect("expected a MovieDetails item")
                                .to_overview()) as Box<dyn MediaIdentifier>,
                            MediaType::Show => Box::new(e.into_any()
                                .downcast::<ShowDetails>()
                                .expect("expected a ShowDetails item")
                                .to_overview()) as Box<dyn MediaIdentifier>,
                            _ => {
                                warn!("Received unknown media type {}, ignoring update for {}", e.media_type(), media.imdb_id());
                                media
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to update media item {}, {}", media.imdb_id(), e);
                        media
                    }
                }
            }))
            .await
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;

    use crate::core::media::{MediaOverview, MovieOverview};
    use crate::core::media::favorites::MockFavoriteService;
    use crate::core::media::providers::MockMediaProvider;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_update_cache() {
        init_logger();
        let movie_id = "tt12121222";
        let title = "Lorem ipsum";
        let year = "2010";
        let mut movie_provider = MockMediaProvider::new();
        movie_provider.expect_supports()
            .returning(|e: &Category| e == &Category::Movies);
        movie_provider.expect_retrieve_details()
            .returning(|_: &str| Ok(Box::new(MovieDetails {
                imdb_id: movie_id.to_string(),
                title: title.to_string(),
                year: year.to_string(),
                runtime: "".to_string(),
                genres: vec![],
                synopsis: "".to_string(),
                rating: None,
                images: Default::default(),
                trailer: "".to_string(),
                torrents: Default::default(),
            })));
        let (tx, rx) = channel();
        let mut favorites = MockFavoriteService::new();
        favorites.expect_favorites()
            .returning(|| Some(Favorites {
                movies: vec![
                    MovieOverview {
                        title: "".to_string(),
                        imdb_id: movie_id.to_string(),
                        year: "".to_string(),
                        rating: None,
                        images: Default::default(),
                    }
                ],
                shows: vec![],
                last_cache_update: "2020-01-01T10:15:00.000000".to_string(),
            }));
        favorites.expect_update()
            .returning(move |items: Vec<Box<dyn MediaIdentifier>>| {
                tx.send(items).unwrap()
            });
        let _updater = FavoriteCacheUpdater::builder()
            .favorite_service(Arc::new(Box::new(favorites)))
            .provider_manager(Arc::new(ProviderManager::builder()
                .with_provider(Arc::new(Box::new(movie_provider)))
                .build()))
            .build();

        let updated_items = rx.recv_timeout(core::time::Duration::from_millis(200))
            .expect("expected to receive updated media items");
        let movies = updated_items.into_iter()
            .map(|e| {
                e.into_any()
                    .downcast::<MovieOverview>()
                    .unwrap()
            })
            .collect_vec();
        let movie = movies.get(0).unwrap();

        assert_eq!(movie_id, movie.imdb_id());
        assert_eq!(title.to_string(), movie.title());
        assert_eq!(&year.to_string(), movie.year());
    }
}