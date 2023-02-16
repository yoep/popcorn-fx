use derive_more::Display;
use serde::Deserialize;

#[derive(Debug, Display, Clone, PartialEq, Deserialize)]
#[display(fmt = "uris: {:?}, genres: {:?}, sort_by: {:?}", uris, genres, sort_by)]
pub struct ProviderProperties {
    /// The provider uri's to use
    uris: Vec<String>,
    genres: Vec<String>,
    sort_by: Vec<String>,
}

impl ProviderProperties {
    pub fn new(uris: Vec<String>, genres: Vec<String>, sort_by: Vec<String>) -> Self {
        Self {
            uris,
            genres,
            sort_by,
        }
    }

    pub fn uris(&self) -> &Vec<String> {
        &self.uris
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }
}