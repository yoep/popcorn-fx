use crate::core::media::{Category, Media};
use crate::core::media::providers::MediaProvider;

/// Manages available [MediaProvider]'s that can be used to retrieve [Media] items.
/// Multiple providers for the same [Category] can be registered to overrule an existing one.
#[derive(Debug)]
pub struct ProviderManager {
    providers: Vec<Box<dyn MediaProvider>>,
}

impl ProviderManager {
    /// Create a new manager for [MediaProvider]'s which is empty.
    /// This manager won't support anything out-of-the-box.
    ///
    /// If you want to create an instance with providers, use [ProviderManager::with_providers] instead.
    pub fn new() -> Self {
        Self {
            providers: vec![]
        }
    }

    /// Create a new manager which the given [MediaProvider]'s.
    pub fn with_providers(providers: Vec<Box<dyn MediaProvider>>) -> Self {
        Self {
            providers
        }
    }

    /// Retrieve the [MediaProvider] for the given [Category].
    ///
    /// It returns the [MediaProvider] if one is registered, else [None].
    pub fn get(&self, category: Category) -> Option<&Box<dyn MediaProvider>> {
        for provider in &self.providers {
            if provider.supports(&category) {
                return Some(provider);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::core::config::Application;
    use crate::core::media::providers::MovieProvider;

    use super::*;

    #[test]
    fn test_get_supported_category() {
        let settings = Arc::new(Application::default());
        let provider: Box<dyn MediaProvider> = Box::new(MovieProvider::new(&settings));
        let manager = ProviderManager::with_providers(vec![provider]);

        let result = manager.get(Category::MOVIES);

        assert!(result.is_some(), "Expected a supported provider to have been found")
    }

    #[test]
    fn test_get_not_supported_category() {
        let manager = ProviderManager::new();

        let result = manager.get(Category::MOVIES);

        assert!(result.is_none(), "Expected no supported provider to have been found")
    }
}