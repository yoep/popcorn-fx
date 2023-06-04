use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use mockall::automock;

use crate::core::media::{Category, MediaDetails};

/// The enhancer allows [Media] items to be enhanced before they're returned by the [ProviderManager].
///
/// ## async
///
/// The Enhancer should be able to be send across threads in a safe manner.
/// This means that each implementation must guarantee [Send] & [Sync] compatibility.
#[automock]
#[async_trait]
pub trait Enhancer : Debug + Display + Send + Sync {
    /// Verify if this enhancer supports the given [Category].
    /// 
    /// Returns true when this enhance supports the given category.
    fn supports(&self, category: &Category) -> bool;

    /// Enhance the given [MediaDetails].
    ///
    /// The enhancement process should <b>never panic nor error</b>.
    /// When the enhancement fails, it should return the original media item.
    async fn enhance_details(&self, media: Box<dyn MediaDetails>) -> Box<dyn MediaDetails>;
}

impl Display for MockEnhancer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockEnhancer")
    }
}