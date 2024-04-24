use std::cmp::Ordering;
use std::sync::{Arc, RwLock, Weak};

use log::debug;

use crate::core::loader::LoadingStrategy;

pub const HIGHEST_ORDER: Order = i32::MIN;
pub const DEFAULT_ORDER: Order = 0;
pub const LOWEST_ORDER: Order = i32::MAX;

/// Represents the order in which loading strategies are applied within the loading chain.
pub type Order = i32;

/// A struct that manages a chain of loading strategies.
#[derive(Debug, Default)]
pub struct LoadingChain {
    chain: RwLock<Vec<ChainAction>>,
}

impl LoadingChain {
    /// Add a loading strategy to the chain with the specified `order`.
    pub fn add(&self, strategy: Box<dyn LoadingStrategy>, order: Order) {
        debug!("Adding loading strategy {} to the chain", strategy);
        let mut chain = self.chain.write().unwrap();
        chain.push(ChainAction {
            order,
            strategy: Arc::new(strategy),
        });
        chain.sort()
    }

    /// Get a vector of weak references to the loading strategies in the chain.
    pub fn strategies(&self) -> Vec<Weak<Box<dyn LoadingStrategy>>> {
        let chain = self.chain.read().unwrap();
        chain.iter().map(|e| Arc::downgrade(&e.strategy)).collect()
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
    fn from_iter<T: IntoIterator<Item = Box<dyn LoadingStrategy>>>(iter: T) -> Self {
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
    use crate::core::loader::{LoadingStrategy, MockLoadingStrategy};

    use super::*;

    #[test]
    fn test_loading_chain_from_vector() {
        let strat1 = Box::new(MockLoadingStrategy::new()) as Box<dyn LoadingStrategy>;
        let strat2 = Box::new(MockLoadingStrategy::new()) as Box<dyn LoadingStrategy>;
        let strategies = vec![strat1, strat2];

        let chain: LoadingChain = LoadingChain::from(strategies);

        assert_eq!(2, chain.strategies().len());
    }

    #[test]
    fn test_loading_chain_add() {
        let strategy = Box::new(MockLoadingStrategy::new()) as Box<dyn LoadingStrategy>;
        let chain = LoadingChain::default();

        assert_eq!(0, chain.strategies().len());
        chain.add(strategy, DEFAULT_ORDER);
        assert_eq!(1, chain.strategies().len());
    }
}
