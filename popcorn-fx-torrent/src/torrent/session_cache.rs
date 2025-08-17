use crate::torrent::{InfoHash, TorrentMetadata};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Instant;

/// A torrent session cache used for storing data about torrents within the session.
pub trait SessionCache: Debug + Send {
    /// Try to find the metadata of the given info hash within the torrent session cache.
    fn find_metadata(&self, info_hash: &InfoHash) -> Option<&TorrentMetadata>;

    /// Store the metadata within the session cache.
    fn store_metadata(&mut self, metadata: &TorrentMetadata);
}

/// The default torrent FX session cache storing data about torrents.
#[derive(Debug)]
pub struct FxSessionCache {
    cache: HashMap<InfoHash, MetadataHolder>,
    cache_limit: usize,
}

impl FxSessionCache {
    /// Create a new FX session cache with the given limit.
    /// Once the limit is reached, the oldest item within the cache will be removed.
    pub fn new(cache_limit: usize) -> Self {
        Self {
            cache: Default::default(),
            cache_limit,
        }
    }
}

impl SessionCache for FxSessionCache {
    fn find_metadata(&self, info_hash: &InfoHash) -> Option<&TorrentMetadata> {
        self.cache.get(info_hash).map(|e| &e.metadata)
    }

    fn store_metadata(&mut self, metadata: &TorrentMetadata) {
        // if the cache limit is going to be exceeded
        // remove the oldest item from the cache
        if self.cache.len() == self.cache_limit {
            if let Some(info_hash) = self
                .cache
                .iter()
                .sorted_by(|(_, a), (_, b)| (&a.last_changed).cmp(&b.last_changed))
                .next()
                .map(|(info_hash, _)| info_hash.clone())
            {
                self.cache.remove(&info_hash);
            }
        }

        let info_hash = metadata.info_hash.clone();
        self.cache.insert(
            info_hash,
            MetadataHolder {
                metadata: metadata.clone(),
                last_changed: Instant::now(),
            },
        );
    }
}

/// A torrent session cache which doesn't store any data.
#[derive(Debug)]
pub struct NoSessionCache;

impl NoSessionCache {
    pub fn new() -> Self {
        Self {}
    }
}

impl SessionCache for NoSessionCache {
    fn find_metadata(&self, _: &InfoHash) -> Option<&TorrentMetadata> {
        // no-op
        None
    }

    fn store_metadata(&mut self, _: &TorrentMetadata) {
        // no-op
    }
}

#[derive(Debug)]
struct MetadataHolder {
    metadata: TorrentMetadata,
    last_changed: Instant,
}

impl PartialEq for MetadataHolder {
    fn eq(&self, other: &MetadataHolder) -> bool {
        self.metadata == other.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod find_metadata {
        use super::*;
        use crate::torrent::tests::read_test_file_to_bytes;

        #[test]
        fn test_not_cached() {
            let metadata =
                TorrentMetadata::try_from(read_test_file_to_bytes("ubuntu-udp.torrent").as_slice())
                    .unwrap();
            let session_cache = FxSessionCache::new(10);

            let result = session_cache.find_metadata(&metadata.info_hash);

            assert_eq!(
                None, result,
                "expected the info hash to not have been found"
            );
        }
    }

    mod store_metadata {
        use super::*;
        use crate::torrent::tests::read_test_file_to_bytes;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_store() {
            let metadata =
                TorrentMetadata::try_from(read_test_file_to_bytes("debian.torrent").as_slice())
                    .unwrap();
            let mut session_cache = FxSessionCache::new(10);

            session_cache.store_metadata(&metadata);

            assert_ne!(
                None,
                session_cache.find_metadata(&metadata.info_hash),
                "expected the metadata to have been stored"
            );
        }

        #[test]
        fn test_cache_limit_reached() {
            let metadata1 =
                TorrentMetadata::try_from(read_test_file_to_bytes("debian.torrent").as_slice())
                    .unwrap();
            let metadata2 =
                TorrentMetadata::try_from(read_test_file_to_bytes("debian-udp.torrent").as_slice())
                    .unwrap();
            let metadata3 = TorrentMetadata::try_from(
                read_test_file_to_bytes("ubuntu-https.torrent").as_slice(),
            )
            .unwrap();
            let mut session_cache = FxSessionCache::new(2);

            session_cache.store_metadata(&metadata1);
            thread::sleep(Duration::from_millis(10));
            session_cache.store_metadata(&metadata2);
            assert_eq!(
                2,
                session_cache.cache.len(),
                "expected 2 items to have been stored"
            );

            session_cache.store_metadata(&metadata3);
            assert_eq!(
                None,
                session_cache.cache.get(&metadata1.info_hash),
                "expected the oldest metadata to have been removed"
            );
            assert_ne!(None, session_cache.cache.get(&metadata2.info_hash));
            assert_ne!(None, session_cache.cache.get(&metadata3.info_hash));
        }
    }
}
