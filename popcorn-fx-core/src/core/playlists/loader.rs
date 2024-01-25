use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, RwLock, Weak};

use async_trait::async_trait;
use log::debug;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::playlists::PlaylistItem;

pub const HIGHEST_ORDER: Order = i32::MIN;
pub const DEFAULT_ORDER: Order = 0;
pub const LOWEST_ORDER: Order = i32::MAX;

/// A trait for defining loading strategies for media items in a playlist.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait LoadingStrategy: Debug + Display {
    /// Process the playlist item.
    ///
    /// This method takes a `PlaylistItem` as input and may enhance the item, replace it, or start the playback.
    /// If an updated `PlaylistItem` is returned, it indicates that the next strategy in the chain should
    /// continue processing this item. If `None` is returned, it indicates that this strategy has started
    /// the playback for the given item, and further processing in the chain is stopped.
    ///
    /// # Arguments
    ///
    /// * `item` - The `PlaylistItem` to be processed by the loading strategy.
    ///
    /// # Returns
    ///
    /// An optional `PlaylistItem`. If `Some(item)` is returned, it indicates that the strategy
    /// has processed the item, and the item may have been modified or replaced. If `None` is returned,
    /// it indicates that the strategy has initiated playback for the item, and processing in the chain stops.
    async fn process(&self, item: PlaylistItem) -> Option<PlaylistItem>;
}

#[cfg(any(test, feature = "testing"))]
impl Display for MockLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockLoadingStrategy")
    }
}

pub type Order = i32;

#[derive(Debug, Default)]
pub struct LoadingChain {
    chain: RwLock<Vec<ChainAction>>,
}

impl LoadingChain {
    pub fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        debug!("Adding loading strategy {} to the chain", strategy);
        let mut chain = self.chain.write().unwrap();
        chain.push(ChainAction {
            order,
            strategy: Arc::new(strategy),
        });
        chain.sort()
    }

    pub fn strategies(&self) -> Vec<Weak<Box<dyn LoadingStrategy>>> {
        let chain = self.chain.read().unwrap();
        chain.iter()
            .map(|e| Arc::downgrade(&e.strategy))
            .collect()
    }
}

unsafe impl Send for LoadingChain {}

unsafe impl Sync for LoadingChain {}

impl From<Vec<Box<dyn LoadingStrategy>>> for LoadingChain {
    fn from(value: Vec<Box<dyn LoadingStrategy>>) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<Box<dyn LoadingStrategy>> for LoadingChain {
    fn from_iter<T: IntoIterator<Item=Box<dyn LoadingStrategy>>>(iter: T) -> Self {
        let instance = Self::default();
        let mut order: Order = DEFAULT_ORDER;

        for strategy in iter {
            instance.add(strategy, order);
            order += 1;
        }

        instance
    }
}

#[derive(Debug)]
struct ChainAction {
    order: Order,
    strategy: Arc<Box<dyn LoadingStrategy>>,
}

impl Eq for ChainAction {}

impl PartialEq<Self> for ChainAction {
    fn eq(&self, other: &Self) -> bool {
        self.order.eq(&other.order)
    }
}

impl PartialOrd<Self> for ChainAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for ChainAction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_chain_from_vector() {
        let strat1 = Box::new(MockLoadingStrategy::new()) as Box<dyn LoadingStrategy>;
        let strat2 = Box::new(MockLoadingStrategy::new()) as Box<dyn LoadingStrategy>;
        let strategies = vec![strat1, strat2];

        let chain: LoadingChain = LoadingChain::from(strategies);

        assert_eq!(2, chain.strategies().len());
    }
}