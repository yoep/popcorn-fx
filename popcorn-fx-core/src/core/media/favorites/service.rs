use std::sync::Arc;

use crate::core::config::Application;

const FILENAME: &str = "favorites.json";

/// The favorite service is stores & retrieves liked media items based on the ID.
#[derive(Debug, Clone)]
pub struct FavoriteService {
    settings: Arc<Application>
}

impl FavoriteService {
    pub fn new(settings: &Arc<Application>) -> Self {
        Self {
            settings: settings.clone(),
        }
    }
}

#[cfg(test)]
mod test {

}