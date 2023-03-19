use async_trait::async_trait;
use log::{debug, error, trace, warn};
use regex::Regex;
use reqwest::Client;
use url::Url;

use crate::core::media::{Category, Episode, MediaDetails, ShowDetails};
use crate::core::media::providers::enhancers::Enhancer;

/// The [Episode] enhancer which allows the retrieval of additional thumbs.
#[derive(Debug)]
pub struct EpisodeEnhancer {
    /// the regex used to retrieve the thumb
    regex: Regex,
    client: Client,
}

impl EpisodeEnhancer {
    async fn enhance(&self, mut episode: Episode) -> Episode {
        if episode.tvdb_id <= 0 {
            warn!("Unable to enhance episode, tvdb_id is unknown for {}", episode);
            return episode;
        }

        trace!("Enhancing episode {}", episode);
        let url = Self::build_url(&episode.tvdb_id);

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

    fn build_url(episode_id: &i32) -> Url {
        let mut url = Url::parse("https://thetvdb.com/series/lorem/episodes").unwrap();

        url.path_segments_mut()
            .unwrap()
            .push(episode_id.to_string().as_str());

        url
    }
}

#[async_trait]
impl Enhancer for EpisodeEnhancer {
    fn category(&self) -> Category {
        Category::Series
    }

    async fn enhance_details(&self, media: Box<dyn MediaDetails>) -> Box<dyn MediaDetails> {
        let mut show = media.into_any().downcast::<ShowDetails>()
            .expect("expected ShowDetails to be passed to the enhancer");

        show.episodes = futures::future::join_all(show.episodes.into_iter()
            .map(|e| self.enhance(e)))
            .await;

        show
    }
}

impl Default for EpisodeEnhancer {
    fn default() -> Self {
        Self {
            regex: Regex::new("https://artworks.thetvdb.com/banners/([a-zA-Z0-9/\\.]+)").unwrap(),
            client: Client::builder()
                .build()
                .expect("Client should have been created"),
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use crate::core::media::{Episode, Images, ShowDetails};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_category() {
        let enhancer = EpisodeEnhancer::default();

        assert_eq!(Category::Series, enhancer.category())
    }

    #[test]
    fn test_enhance_details() {
        init_logger();
        let tvdb_id = "9435216";
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
        let enhancer = EpisodeEnhancer::default();
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(enhancer.enhance_details(show))
            .into_any()
            .downcast::<ShowDetails>()
            .unwrap();

        assert_eq!(Some("https://artworks.thetvdb.com/banners/v4/episode/9435216/screencap/63fd00ab6f23b.jpg".to_string()), result.episodes.get(0).unwrap().thumb)
    }
}