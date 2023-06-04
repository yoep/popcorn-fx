use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use log::{debug, trace, warn};
use reqwest::Client;
use url::Url;

use crate::core::cache::{CacheManager, CacheOptions, CacheType};
use crate::core::images::ImageError;
use crate::core::media::MediaOverview;

const POSTER_PLACEHOLDER: &[u8] = include_bytes!("../../../resources/posterholder.png");
const ART_PLACEHOLDER: &[u8] = include_bytes!("../../../resources/artholder.png");
const BACKGROUND_HOLDER: &[u8] = include_bytes!("../../../resources/background.jpg");
const CACHE_NAME: &str = "images";

/// The `ImageLoader` trait is responsible for loading image data from local or remote locations.
///
/// Implementations of this trait provide methods for loading the fanart and poster images for a given media item.
/// The loaded data can then be converted into a graphic that can be shown to the user.
///
/// # Asynchronous
///
/// All methods in this trait are asynchronous and return a `Future` that will resolve to the image data when it's available.
#[async_trait]
pub trait ImageLoader {
    /// Retrieve the default poster (placeholder) image data.
    ///
    /// This method returns a `Vec<u8>` containing the data for the default poster holder image.
    /// The default poster holder image is typically used as a fallback when a poster image is not available for a media item or is still being loaded.
    fn default_poster(&self) -> Vec<u8>;

    /// Retrieve the default artwork (placeholder) image data.
    ///
    /// This method returns a `Vec<u8>` containing the data for the default artwork placeholder image.
    /// The default artwork placeholder image is typically used as a fallback when artwork image is not available for a media item or is still being loaded.
    fn default_artwork(&self) -> Vec<u8>;

    /// Load the fanart image for the given media item.
    ///
    /// If fanart image data is available for the media item, it is returned as a `Vec<u8>`.
    /// Otherwise, a placeholder byte array is returned.
    ///
    /// # Arguments
    ///
    /// * `media` - a reference to a boxed `dyn MediaOverview` object that represents the media item to load.
    async fn load_fanart(&self, media: &Box<dyn MediaOverview>) -> Vec<u8>;

    /// Load the poster image for the given media item.
    ///
    /// If poster image data is available for the media item, it is returned as a `Vec<u8>`.
    /// Otherwise, a placeholder byte array is returned.
    ///
    /// # Arguments
    ///
    /// * `media` - a reference to a boxed `dyn MediaOverview` object that represents the media item to load.
    async fn load_poster(&self, media: &Box<dyn MediaOverview>) -> Vec<u8>;

    /// Load the image data from the given URL.
    ///
    /// This method fetches the image data from the provided URL location and converts it to binary data.
    /// If the operation succeeds, it returns the image binary data wrapped in a Some variant.
    /// If the operation fails, it returns None.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL from where to fetch the image data.
    ///
    /// # Returns
    ///
    /// * `Some(Vec<u8>)` - The binary data of the image on success.
    /// * `None` - If the operation fails.
    async fn load(&self, url: &str) -> Option<Vec<u8>>;
}

/// The DefaultImageLoader struct is an implementation of the ImageLoader trait and is responsible for loading image data from local or remote locations.
/// This implementation is the default image loader used by the Popcorn FX library.
///
/// Most methods implemented from the [ImageLoader] trait are asynchronous and return a Future that will resolve to the image data when it's available.
#[derive(Debug)]
pub struct DefaultImageLoader {
    client: Client,
    cache_manager: Arc<CacheManager>,
}

impl DefaultImageLoader {
    /// Creates a new `DefaultImageLoader` instance.
    ///
    /// # Arguments
    ///
    /// * `cache_manager` - The cache manager for storing and retrieving image data.
    ///
    /// # Returns
    ///
    /// A new `DefaultImageLoader` instance.
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self {
            client: Client::builder()
                .build()
                .expect("expected a new client"),
            cache_manager,
        }
    }

    /// Retrieves the image data from the cache or fetches it remotely if not available in the cache.
    ///
    /// # Arguments
    ///
    /// * `image_url` - The URL of the image to retrieve.
    ///
    /// # Returns
    ///
    /// The image data as a `Vec<u8>`, or `None` if the data could not be retrieved.
    async fn retrieve_image_data(&self, image_url: &str) -> Option<Vec<u8>> {
        match self.cache_manager.operation()
            .name(CACHE_NAME)
            .key(image_url)
            .options(CacheOptions {
                cache_type: CacheType::CacheFirst,
                expires_after: Duration::days(1),
            })
            .execute(self.fetch_remote_image_data(image_url))
            .await {
            Ok(e) => Some(e),
            Err(e) => {
                warn!("Failed to retrieve image data, {}", e);
                None
            }
        }
    }

    async fn fetch_remote_image_data(&self, image_url: &str) -> Result<Vec<u8>, ImageError> {
        trace!("Parsing image url {}", image_url);
        let url = Url::parse(image_url)
            .map_err(|e| ImageError::ParseUrl(e.to_string()))?;

        debug!("Retrieving image data from {:?}", url);
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| ImageError::Load(e.to_string()))?;

        trace!("Retrieved image data with status {}", response.status());
        if response.status().is_success() {
            debug!("Retrieved image data from {}", image_url);
            match response.bytes().await {
                Ok(bytes) => Ok(bytes.to_vec()),
                Err(e) => Err(ImageError::Load(format!("failed to retrieve the image binary data, {}", e))),
            }
        } else {
            warn!("Received invalid response status {} for image url {}", response.status(), image_url);
            Err(ImageError::Load(format!("received response status {}", response.status())))
        }
    }
}

#[async_trait]
impl ImageLoader for DefaultImageLoader {
    fn default_poster(&self) -> Vec<u8> {
        POSTER_PLACEHOLDER.to_vec()
    }

    fn default_artwork(&self) -> Vec<u8> {
        ART_PLACEHOLDER.to_vec()
    }

    async fn load_fanart(&self, media: &Box<dyn MediaOverview>) -> Vec<u8> {
        trace!("Loading fanart image for {:?}", media);
        let fanart_url = media.images().fanart();

        self.retrieve_image_data(fanart_url).await
            .or_else(|| Some(BACKGROUND_HOLDER.to_vec()))
            .unwrap()
    }

    async fn load_poster(&self, media: &Box<dyn MediaOverview>) -> Vec<u8> {
        trace!("Loading poster image for {:?}", media);
        let poster_url = media.images().poster();

        self.retrieve_image_data(poster_url).await
            .or_else(|| Some(POSTER_PLACEHOLDER.to_vec()))
            .unwrap()
    }

    async fn load(&self, url: &str) -> Option<Vec<u8>> {
        trace!("Loading image data from url for {}", url);
        self.retrieve_image_data(url).await
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    use crate::core::cache::CacheManagerBuilder;
    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_default_poster() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);

        assert_eq!(POSTER_PLACEHOLDER.to_vec(), loader.default_poster())
    }

    #[test]
    fn test_load_fanart() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        let expected_result = read_test_file_to_bytes("image.png");
        server.mock(|when, then| {
            when.method(GET)
                .path("/fanart.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let media = Box::new(MovieOverview {
            title: "lorem ipsum".to_string(),
            imdb_id: "tt121212".to_string(),
            year: "2010".to_string(),
            rating: None,
            images: Images {
                poster: "".to_string(),
                fanart: server.url("/fanart.png"),
                banner: "".to_string(),
            },
        }) as Box<dyn MediaOverview>;
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);
        let runtime = Runtime::new().unwrap();

        let (result, _) = runtime.block_on(async move {
            (loader.load_fanart(&media).await, loader)
        });

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_load_fanart_invalid_url() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let media = Box::new(MovieOverview {
            title: "lorem ipsum".to_string(),
            imdb_id: "tt121212".to_string(),
            year: "2010".to_string(),
            rating: None,
            images: Images {
                poster: "".to_string(),
                fanart: ":invalid#url".to_string(),
                banner: "".to_string(),
            },
        }) as Box<dyn MediaOverview>;
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);
        let runtime = Runtime::new().unwrap();

        let (result, _) = runtime.block_on(async move {
            (loader.load_fanart(&media).await, loader)
        });

        assert_eq!(BACKGROUND_HOLDER, result)
    }

    #[test]
    fn test_load_fanart_invalid_response() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/fanart.png");
            then.status(500)
                .body("");
        });
        let media = Box::new(MovieOverview {
            title: "lorem ipsum".to_string(),
            imdb_id: "tt121212".to_string(),
            year: "2010".to_string(),
            rating: None,
            images: Images {
                poster: "".to_string(),
                fanart: server.url("/fanart.png"),
                banner: "".to_string(),
            },
        }) as Box<dyn MediaOverview>;
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);
        let runtime = Runtime::new().unwrap();

        let (result, _) = runtime.block_on(async move {
            (loader.load_fanart(&media).await, loader)
        });

        assert_eq!(BACKGROUND_HOLDER, result)
    }

    #[test]
    fn test_load_poster() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        let expected_result = read_test_file_to_bytes("image.png");
        server.mock(|when, then| {
            when.method(GET)
                .path("/poster.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let media = Box::new(ShowOverview {
            imdb_id: "".to_string(),
            tvdb_id: "".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Images {
                poster: server.url("/poster.png"),
                fanart: "".to_string(),
                banner: "".to_string(),
            },
            rating: None,
        }) as Box<dyn MediaOverview>;
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);
        let runtime = Runtime::new().unwrap();

        let (result, _) = runtime.block_on(async move {
            (loader.load_poster(&media).await, loader)
        });

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_load_url() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        let expected_result = read_test_file_to_bytes("image.png");
        server.mock(|when, then| {
            when.method(GET)
                .path("/my-image.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let url = server.url("/my-image.png");
        let cache_manager = Arc::new(CacheManager::builder()
            .storage_path(temp_path)
            .build());
        let loader = DefaultImageLoader::new(cache_manager);
        let runtime = Runtime::new().unwrap();

        let (result, _) = runtime.block_on(async move {
            (loader.load(url.as_str()).await, loader)
        });

        assert_eq!(Some(expected_result), result)
    }
}