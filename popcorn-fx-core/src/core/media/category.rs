use derive_more::Display;

/// The available categories of [crate::core::media::Media] items.
/// These can be used as filter to retrieve data from the API.
#[repr(i32)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{}", (self.name()))]
pub enum Category {
    MOVIES = 0,
    SERIES = 1,
    ANIME = 2,
    FAVORITES = 3,
}

impl Category {
    /// Retrieve the name of the category.
    pub fn name(&self) -> String {
        match self {
            Category::MOVIES => "movies".to_string(),
            Category::SERIES => "series".to_string(),
            Category::ANIME => "animes".to_string(),
            Category::FAVORITES => "favorites".to_string(),
        }
    }
}