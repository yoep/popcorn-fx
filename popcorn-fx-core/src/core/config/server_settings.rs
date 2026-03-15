use serde::Deserialize;
use serde::Serialize;

/// The api server preferences of the user for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerSettings {
    #[serde(default)]
    pub movie_api_servers: Vec<String>,
    #[serde(default)]
    pub serie_api_servers: Vec<String>,
    #[serde(default)]
    pub update_api_servers_automatically: bool,
}

impl ServerSettings {
    /// Returns the slice of the api servers for movies.
    pub fn movie_api_servers_as_slice(&self) -> &[String] {
        &self.movie_api_servers
    }

    /// Returns the slice of the api servers for series.
    pub fn serie_api_servers_as_slice(&self) -> &[String] {
        &self.serie_api_servers
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            movie_api_servers: Default::default(),
            serie_api_servers: Default::default(),
            update_api_servers_automatically: false,
        }
    }
}
