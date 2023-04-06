use std::any::TypeId;

use async_trait::async_trait;
use log::{debug, error, trace, warn};
use regex::Regex;
use reqwest::Client;
use url::Url;

use crate::core::config::EnhancerProperties;
use crate::core::media::{Category, Episode, MediaDetails, ShowDetails};
use crate::core::media::providers::enhancers::Enhancer;

/// The [Episode] thumb enhancer which allows the retrieval of thumbs for episode media items.
#[derive(Debug)]
pub struct ThumbEnhancer {
    /// The properties for this enhancer
    properties: EnhancerProperties,
    /// the regex used to retrieve the thumb
    regex: Regex,
    client: Client,
}

impl ThumbEnhancer {
    /// Create a new episode enhancer which will use TVDB information based on the given enhancer properties.
    pub fn new(properties: EnhancerProperties) -> Self {
        Self {
            properties,
            regex: Regex::new("https://artworks.thetvdb.com/banners/([a-zA-Z0-9/\\.]+)").unwrap(),
            client: Client::builder()
                .build()
                .expect("Client should have been created"),
        }
    }

    async fn enhance(&self, mut episode: Episode) -> Episode {
        if episode.tvdb_id <= 0 {
            warn!("Unable to enhance episode, tvdb_id is unknown for {}", episode);
            return episode;
        }

        trace!("Enhancing episode {}", episode);
        let url = self.build_url(&episode.tvdb_id);

        trace!("Retrieving additional TVDB info from {}", url);
        match self.client.get(url)
            .send()
            .await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(body) => self.handle_body(&mut episode, body),
                        Err(e) => error!("Failed to retrieve body, {}", e)
                    }
                } else {
                    error!("Received invalid response for enhancement, status {}", response.status());
                }
            }
            Err(e) => error!("Failed to retrieve the episode details, {}", e)
        }

        episode
    }

    fn handle_body(&self, episode: &mut Episode, body: String) {
        match self.regex.find(body.as_str()) {
            None => warn!("Thumb url not found for {}", episode.tvdb_id),
            Some(url) => {
                let url = url.as_str();
                debug!("Enhancing episode {} with thumb {}", episode.tvdb_id, url);
                episode.thumb = Some(url.to_string())
            }
        }
    }

    fn build_url(&self, episode_id: &i32) -> Url {
        let mut url = Url::parse(self.properties.uri.as_str()).unwrap();

        url.path_segments_mut()
            .unwrap()
            .push(episode_id.to_string().as_str());

        url
    }
}

#[async_trait]
impl Enhancer for ThumbEnhancer {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::Series || category == &Category::Favorites
    }

    async fn enhance_details(&self, media: Box<dyn MediaDetails>) -> Box<dyn MediaDetails> {
        if (*media).type_id() == TypeId::of::<ShowDetails>() {
            let mut show = media.into_any().downcast::<ShowDetails>()
                .expect("expected the media item to be ShowDetails");

            show.episodes = futures::future::join_all(show.episodes.into_iter()
                .map(|e| self.enhance(e)))
                .await;

            return show;
        }

        media
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tokio::runtime::Runtime;

    use crate::core::media::{Episode, Images, MovieDetails, ShowDetails};
    use crate::testing::{init_logger, read_test_file_to_string};

    use super::*;

    #[test]
    fn test_supports() {
        let enhancer = ThumbEnhancer::new(EnhancerProperties {
            uri: "".to_string(),
        });

        assert!(enhancer.supports(&Category::Series), "expected the series to have been supported");
        assert!(enhancer.supports(&Category::Favorites), "expected the favorites to have been supported");
    }

    #[test]
    fn test_enhance_details_show_details() {
        init_logger();
        let tvdb_id = "9435216";
        let server = MockServer::start();
        let show = Box::new(ShowDetails {
            imdb_id: "tt12124578".to_string(),
            tvdb_id: "392256".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Images::none(),
            rating: None,
            context_locale: "".to_string(),
            synopsis: "".to_string(),
            runtime: "".to_string(),
            status: "".to_string(),
            genres: vec![],
            episodes: vec![
                Episode {
                    season: 1,
                    episode: 1,
                    first_aired: 16875554,
                    title: "".to_string(),
                    overview: "".to_string(),
                    tvdb_id: 9435216,
                    tvdb_id_value: tvdb_id.to_string(),
                    thumb: None,
                    torrents: Default::default(),
                }
            ],
            liked: None,
        });
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/{}", tvdb_id));
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(read_test_file_to_string("tvdb_response.html"));
        });
        let enhancer = ThumbEnhancer::new(EnhancerProperties {
            uri: server.url(""),
        });
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(enhancer.enhance_details(show))
            .into_any()
            .downcast::<ShowDetails>()
            .unwrap();

        assert_eq!(Some("https://artworks.thetvdb.com/banners/v4/episode/9435216/screencap/63fd00ab6f23b.jpg".to_string()), result.episodes.get(0).unwrap().thumb)
    }

    #[test]
    fn test_enhance_details_movie_details() {
        init_logger();
        let movie = Box::new(MovieDetails {
            title: "".to_string(),
            imdb_id: "".to_string(),
            year: "".to_string(),
            runtime: "".to_string(),
            genres: vec![],
            synopsis: "".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "".to_string(),
            torrents: Default::default(),
        });
        let enhancer = ThumbEnhancer::new(EnhancerProperties {
            uri: "".to_string(),
        });
        let runtime = Runtime::new().unwrap();

        let _ = runtime.block_on(enhancer.enhance_details(movie))
            .into_any()
            .downcast::<MovieDetails>()
            .expect("should have returned to correct same given movie media");
    }
}