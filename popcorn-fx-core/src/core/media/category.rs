#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
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