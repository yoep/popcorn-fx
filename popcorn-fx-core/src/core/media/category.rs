use derive_more::Display;
use serde::{Deserialize, Serialize};

/// The available categories of [crate::core::media::Media] items.
/// These can be used as filter to retrieve data from the API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
#[display("{}", (self.name()))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    Movies = 0,
    Series = 1,
    Favorites = 2,
}

impl Category {
    /// Retrieve the name of the category.
    pub fn name(&self) -> String {
        match self {
            Category::Movies => "movies".to_string(),
            Category::Series => "series".to_string(),
            Category::Favorites => "favorites".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name_movies() {
        let category = Category::Movies;
        let expected_result = "movies".to_string();

        let result = category.name();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_name_series() {
        let category = Category::Series;
        let expected_result = "series".to_string();

        let result = category.name();

        assert_eq!(expected_result, result)
    }
}
