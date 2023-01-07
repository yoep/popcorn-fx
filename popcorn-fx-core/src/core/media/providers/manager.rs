use std::collections::HashMap;

use crate::core::media::{Category, Movie};
use crate::core::media::providers::Provider;

/// Manages the available [Provider<T>]'s.
#[derive(Debug)]
pub struct ProviderManager {
    providers: HashMap<Category, Box<dyn Provider<Movie>>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new()
        }
    }

    pub fn with_providers(providers: HashMap<Category, Box<dyn Provider<Movie>>>) -> Self {
        Self {
            providers
        }
    }

    /// Retrieve the [Provider] for the given [Category].
    ///
    /// It returns the [Provider] if one is registered, else [None]. 
    pub fn get(&self, category: Category) -> Option<&Box<dyn Provider<Movie>>> {
        self.providers.get(&category)
    }
}