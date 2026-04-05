use crate::core::media::{
    Episode, Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating,
};
use derive_more::Display;
use log::warn;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

/// The show media information of a specific serie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(
    "{{ShowOverview: imdb_id: {}, tvdb_id: {}, title: {}}}",
    imdb_id,
    tvdb_id,
    title
)]
pub struct ShowOverview {
    pub imdb_id: String,
    pub tvdb_id: String,
    pub title: String,
    pub year: String,
    pub num_seasons: u32,
    pub images: Images,
    pub rating: Option<Rating>,
}

impl ShowOverview {
    pub fn new(
        imdb_id: String,
        tvdb_id: String,
        title: String,
        year: String,
        num_seasons: u32,
        images: Images,
        rating: Option<Rating>,
    ) -> Self {
        if imdb_id.is_empty() {
            panic!("Show IMDB ID cannot be empty")
        }

        Self {
            imdb_id,
            tvdb_id,
            title,
            year,
            num_seasons,
            images,
            rating,
        }
    }

    pub fn tvdb_id(&self) -> &String {
        &self.tvdb_id
    }

    /// The currently known number of seasons for the show.
    pub fn number_of_seasons(&self) -> &u32 {
        &self.num_seasons
    }
}

impl MediaIdentifier for ShowOverview {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for ShowOverview {
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

/// The details of a show/serie which contains one or more [Episode] items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(
    "{{ShowDetails: imdb_id: {}, tvdb_id: {}, title: {}}}",
    imdb_id,
    tvdb_id,
    title
)]
pub struct ShowDetails {
    pub imdb_id: String,
    pub tvdb_id: String,
    pub title: String,
    pub year: String,
    pub num_seasons: u32,
    pub images: Images,
    pub rating: Option<Rating>,
    #[serde(rename = "contextLocale")]
    pub context_locale: ContextLocale,
    pub synopsis: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "serde_empty_string::deserialize"
    )]
    pub runtime: Option<String>,
    pub status: String,
    pub genres: Vec<String>,
    pub episodes: Vec<Episode>,
}

impl ShowDetails {
    pub fn tvdb_id(&self) -> &String {
        &self.tvdb_id
    }

    /// The currently known number of seasons for the show.
    pub fn number_of_seasons(&self) -> &u32 {
        &self.num_seasons
    }

    pub fn status(&self) -> &String {
        &self.status
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn episodes(&self) -> &Vec<Episode> {
        &self.episodes
    }

    // TODO: replace with [From] trait
    pub fn to_overview(&self) -> ShowOverview {
        ShowOverview::new(
            self.imdb_id.clone(),
            self.tvdb_id.clone(),
            self.title.clone(),
            self.year.clone(),
            self.num_seasons.clone(),
            self.images.clone(),
            self.rating.clone(),
        )
    }
}

impl MediaIdentifier for ShowDetails {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for ShowDetails {
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

impl MediaDetails for ShowDetails {
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }

    fn runtime(&self) -> u32 {
        match self.runtime.as_ref().map(|e| e.parse::<u32>()).transpose() {
            Ok(runtime) => runtime.unwrap_or(0),
            Err(e) => {
                warn!("Runtime value {:?} is invalid, {}", &self.runtime, e);
                0
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextLocale {
    Disabled,
    Locale(String),
}

impl Serialize for ContextLocale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ContextLocale::Disabled => serializer.serialize_bool(false),
            ContextLocale::Locale(locale) => serializer.serialize_str(locale),
        }
    }
}

impl<'de> Deserialize<'de> for ContextLocale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ContextLocaleVisitor;
        impl<'de> Visitor<'de> for ContextLocaleVisitor {
            type Value = ContextLocale;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a boolean or string")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if value {
                    return Err(E::custom("expected bool false"));
                }

                Ok(ContextLocale::Disabled)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ContextLocale::Locale(v.to_string()))
            }
        }

        deserializer.deserialize_any(ContextLocaleVisitor)
    }
}

mod serde_empty_string {
    use serde::de::{Error, Visitor};
    use serde::Deserializer;
    use std::fmt::Formatter;

    struct EmptyStringVisitor;
    impl<'de> Visitor<'de> for EmptyStringVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "expected a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(v.to_string()).filter(|e| !e.is_empty()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(v).filter(|e| !e.is_empty()))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_string(EmptyStringVisitor {})?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod show_details {
        use super::*;

        #[test]
        fn test_deserialize_context_locale_disabled() {
            let imdb_id = "tt15428778";
            let value = r#"{"_id":"tt15428778","imdb_id":"tt15428778","tmdb_id":207863,"tvdb_id":"413074","title":"","year":"2023","slug":"","original_language":"en","exist_translations":["ru","ua"],"num_seasons":2,"images":{"poster":"http://image.tmdb.org/t/p/w500/eDl1veju2Hf3tyFmGAedtGXb9Yv.jpg","fanart":"http://image.tmdb.org/t/p/w500/9DHo5qXkG0titQmr2PF92N3aYYk.jpg","banner":"http://image.tmdb.org/t/p/w500/eDl1veju2Hf3tyFmGAedtGXb9Yv.jpg"},"rating":{"percentage":73,"watching":0,"votes":1427,"loved":0,"hated":0},"contextLocale":false,"__v":0,"synopsis":"","runtime":"","country":"US","network":"AMC","last_updated":1768147497,"air_day":"","air_time":"","status":"Returning Series","genres":["drama","sci-fi & fantasy"],"episodes":[]}"#;

            let result: ShowDetails =
                serde_json::from_str(value).expect("expected the show details to be deserialized");

            assert_eq!(imdb_id, result.imdb_id, "expected the imdb id to match");
            assert_eq!(
                ContextLocale::Disabled,
                result.context_locale,
                "expected the context locale to be disabled"
            );
        }
    }
}
