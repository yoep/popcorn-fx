use async_trait::async_trait;
use log::{debug, error, trace};
use reqwest::Client;
use url::Url;

use crate::core::media::MediaOverview;

const POSTER_HOLDER: &[u8] = include_bytes!("../../../resources/posterholder.png");
const ART_HOLDER: &[u8] = include_bytes!("../../../resources/artholder.png");
const BACKGROUND_HOLDER: &[u8] = include_bytes!("../../../resources/background.jpg");

/// The image loader is responsible for loading image data from local or remote locations.
/// The loaded data can then be converted into a graphic that can be shown to the user.
#[async_trait]
pub trait ImageLoader {
    /// Load the fanart image for the given media item.
    ///
    /// It returns the fanart when available, else the default placeholder.
    async fn load_fanart(&self, media: &Box<dyn MediaOverview>) -> Vec<u8>;
}

/// The default image loader implementation used by Popcorn FX.
#[derive(Debug)]
pub struct DefaultImageLoader {
    client: Client,
}

impl DefaultImageLoader {
    async fn retrieve_image_data(&self, image_url: &str, default_value: &[u8]) -> Vec<u8> {
        trace!("Parsing image url {}", image_url);
        match Url::parse(image_url) {
            Ok(url) => {
                debug!("Retrieving image data from {:?}", url);
                match self.client.get(url)
                    .send()
                    .await {
                    Ok(response) => {
                        trace!("Retrieved image data with status {}", response.status());
                        if response.status().is_success() {
                            debug!("Retrieved image data from {}", image_url);
                            match response.bytes().await {
                                Ok(bytes) => {
                                    bytes.to_vec()
                                }
                                Err(e) => {
                                    error!("Failed to retrieve the image binary data, {}", e);
                                    default_value.to_vec()
                                }
                            }
                        } else {
                            error!("Received invalid response status {} for image url {}", response.status(), image_url);
                            default_value.to_vec()
                        }
                    }
                    Err(e) => {
                        error!("Failed to load image data, {}", e);
                        default_value.to_vec()
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse image url, {}", e);
                default_value.to_vec()
            }
        }
    }
}

#[async_trait]
impl ImageLoader for DefaultImageLoader {
    async fn load_fanart(&self, media: &Box<dyn MediaOverview>) -> Vec<u8> {
        trace!("Loading fanart image for {:?}", media);
        let fanart_url = media.images().fanart();

        self.retrieve_image_data(fanart_url, BACKGROUND_HOLDER).await
    }
}

impl Default for DefaultImageLoader {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .build()
                .expect("expected a new client")
        }
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tokio::runtime::Runtime;

    use crate::core::media::{Images, MovieOverview};
    use crate::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_load_fanart() {
        init_logger();
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
        let loader = DefaultImageLoader::default();
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(async move {
            loader.load_fanart(&media).await
        });

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_load_fanart_invalid_url() {
        init_logger();
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
        let loader = DefaultImageLoader::default();
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(async move {
            loader.load_fanart(&media).await
        });

        assert_eq!(BACKGROUND_HOLDER, result)
    }

    #[test]
    fn test_load_fanart_invalid_response() {
        init_logger();
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
        let loader = DefaultImageLoader::default();
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(async move {
            loader.load_fanart(&media).await
        });

        assert_eq!(BACKGROUND_HOLDER, result)
    }
}