use std::collections::HashMap;

use derive_more::Display;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::core::media::{
    Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating, TorrentInfo,
};

pub const DEFAULT_AUDIO_LANGUAGE: &str = "en";

/// The simple version of a media item representing a movie.
/// It contains only the basic information needed for search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(
    "MovieOverview: {{imdb_id: {}, title: {}, year: {}}}",
    imdb_id,
    title,
    year
)]
pub struct MovieOverview {
    /// The title of the movie
    pub title: String,
    /// The IMDB identifier of the movie
    pub imdb_id: String,
    /// The year the movie has been released
    pub year: String,
    pub rating: Option<Rating>,
    pub images: Images,
}

impl MovieOverview {
    pub fn new(title: String, imdb_id: String, year: String) -> Self {
        Self {
            title,
            imdb_id,
            year,
            rating: None,
            images: Images::none(),
        }
    }

    pub fn new_detailed(
        title: String,
        imdb_id: String,
        year: String,
        rating: Option<Rating>,
        images: Images,
    ) -> Self {
        Self {
            title,
            imdb_id,
            year,
            rating,
            images,
        }
    }
}

impl MediaIdentifier for MovieOverview {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for MovieOverview {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e),
        }
    }

    fn year(&self) -> &str {
        self.year.as_str()
    }

    fn images(&self) -> &Images {
        &self.images
    }
}

/// The detailed version of a media item representing a movie.
/// It contains all information need for a movie description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(
    "MovieDetails: {{imdb_id: {}, title: {}, year: {}, runtime: {}, torrents: {}}}",
    imdb_id,
    title,
    year,
    runtime,
    torrents.len()
)]
pub struct MovieDetails {
    pub title: String,
    pub imdb_id: String,
    pub year: String,
    pub runtime: String,
    pub genres: Vec<String>,
    pub synopsis: String,
    pub rating: Option<Rating>,
    pub images: Images,
    pub trailer: String,
    #[serde(default, deserialize_with = "serde_torrents::deserialize")]
    pub torrents: HashMap<String, HashMap<String, TorrentInfo>>,
}

impl MovieDetails {
    pub fn new(title: String, imdb_id: String, year: String) -> Self {
        Self {
            title,
            imdb_id,
            year,
            runtime: String::new(),
            genres: vec![],
            synopsis: String::new(),
            rating: None,
            images: Images::none(),
            trailer: String::new(),
            torrents: HashMap::new(),
        }
    }

    pub fn new_detailed(
        title: String,
        imdb_id: String,
        year: String,
        runtime: String,
        genres: Vec<String>,
        synopsis: String,
        rating: Option<Rating>,
        images: Images,
        trailer: String,
    ) -> Self {
        Self {
            title,
            imdb_id,
            year,
            runtime,
            genres,
            synopsis,
            rating,
            images,
            trailer,
            torrents: HashMap::new(),
        }
    }

    pub fn trailer(&self) -> &String {
        &self.trailer
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn torrents(&self) -> &HashMap<String, HashMap<String, TorrentInfo>> {
        &self.torrents
    }
}

impl MediaIdentifier for MovieDetails {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for MovieDetails {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e),
        }
    }

    fn year(&self) -> &str {
        self.year.as_str()
    }

    fn images(&self) -> &Images {
        &self.images
    }
}

impl MediaDetails for MovieDetails {
    /// Retrieve the description of the [Media] item.
    /// The description is html decoded before it's returned.
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }

    fn runtime(&self) -> u32 {
        match self.runtime.parse::<u32>() {
            Ok(e) => e,
            Err(e) => {
                warn!("Runtime value {} is invalid, {}", &self.runtime, e);
                0
            }
        }
    }
}

impl From<&MovieDetails> for MovieOverview {
    fn from(value: &MovieDetails) -> Self {
        MovieOverview::new_detailed(
            value.title.clone(),
            value.imdb_id.clone(),
            value.year.clone(),
            value.rating.clone(),
            value.images.clone(),
        )
    }
}

mod serde_torrents {
    use super::*;
    use serde::de::{MapAccess, SeqAccess, Visitor};
    use serde::Deserializer;
    use std::fmt::Formatter;

    struct TorrentsVisitor;
    impl<'de> Visitor<'de> for TorrentsVisitor {
        type Value = HashMap<String, HashMap<String, TorrentInfo>>;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "expected a map of torrents")
        }

        fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Ok(Self::Value::default())
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut torrents = HashMap::new();
            while let Some((key, value)) =
                map.next_entry::<String, HashMap<String, TorrentInfo>>()?
            {
                torrents.insert(key, value);
            }
            Ok(torrents)
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<String, HashMap<String, TorrentInfo>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        D::deserialize_any(deserializer, TorrentsVisitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod movie_details {
        use super::*;

        #[test]
        fn test_deserialize_empty_torrents() {
            let imdb_id = "tt1560220";
            let value = r#"{"_id":"tt1560220","imdb_id":"tt1560220","tmdb_id":338967,"title":"","year":"2019","original_language":"en","exist_translations":["it","ru","ua"],"contextLocale":false,"synopsis":"","runtime":"99","released":1570579200,"certification":"R","torrents":[],"trailer":"http://www.youtube.com/watch?v=ZlW9yhUKlkQ","genres":["comedy","horror"],"images":{"poster":"http://image.tmdb.org/t/p/w500/dtRbVsUb5O12WWO54SRpiMtHKC0.jpg","fanart":"http://image.tmdb.org/t/p/w500/e7tMI0zVKJB2TS74TaBifIZIkCp.jpg","banner":"http://image.tmdb.org/t/p/w500/dtRbVsUb5O12WWO54SRpiMtHKC0.jpg"},"rating":{"percentage":69,"watching":0,"votes":16609,"loved":0,"hated":0}}"#;

            let result: MovieDetails =
                serde_json::from_str(value).expect("expected the movie details to be deserialized");

            assert_eq!(imdb_id, result.imdb_id, "expected the imdb id to match");
            assert!(
                result.torrents.is_empty(),
                "expected the torrent to be empty, but got {:?}",
                result.torrents
            );
        }
    }
}
