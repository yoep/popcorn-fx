use derive_more::Display;
use serde::Deserialize;

use crate::core::media::{Favorable, Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating, Watchable};

#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "id: {}, tvdb_id: {}, imdb_id: {}, title: {}", id, tvdb_id, imdb_id, title)]
pub struct ShowOverview {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    imdb_id: String,
    tvdb_id: String,
    title: String,
    year: String,
    slug: String,
    original_language: String,
    num_seasons: i32,
    images: Images,
    rating: Option<Rating>,
}

impl ShowOverview {}

impl MediaIdentifier for ShowOverview {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl Watchable for ShowOverview {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for ShowOverview {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl MediaOverview for ShowOverview {}

#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "id: {}, tvdb_id: {}, imdb_id: {}, title: {}", id, tvdb_id, imdb_id, title)]
pub struct ShowDetails {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    imdb_id: String,
    tvdb_id: String,
    title: String,
    year: String,
    slug: String,
    original_language: String,
    num_seasons: i32,
    images: Images,
    rating: Option<Rating>,
    #[serde(rename(deserialize = "contextLocale"))]
    context_locale: String,
    synopsis: String,
    runtime: String,
}

impl ShowDetails {
    pub fn new(id: String, tvdb_id: String, title: String, imdb_id: String) -> Self {
        Self {
            id,
            tvdb_id,
            title,
            imdb_id,
            year: String::new(),
            slug: "".to_string(),
            original_language: "".to_string(),
            rating: None,
            context_locale: "".to_string(),
            synopsis: "".to_string(),
            images: Images::none(),
            num_seasons: 0,
            runtime: "".to_string(),
        }
    }

    pub fn imdb_id(&self) -> &String {
        &self.imdb_id
    }

    pub fn tvdb_id(&self) -> &String {
        &self.tvdb_id
    }

    pub fn year(&self) -> &String {
        &self.year
    }

    /// The currently known number of seasons for the show.
    pub fn number_of_seasons(&self) -> &i32 {
        &self.num_seasons
    }

    pub fn images(&self) -> &Images {
        &self.images
    }

    /// The rating of the show if available.
    pub fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }
}

impl MediaIdentifier for ShowDetails {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl Watchable for ShowDetails {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for ShowDetails {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl MediaOverview for ShowDetails {}

impl MediaDetails for ShowDetails {
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }
}