use std::sync::Arc;

use log::{debug, error, trace, warn};
use tokio::sync::Mutex;

use crate::core::storage::{Storage, StorageError};
use crate::core::torrent;
use crate::core::torrent::collection::Collection;
use crate::core::torrent::TorrentError;

const FILENAME: &str = "torrent-collection.json";

/// The torrent collections stores magnet uri information.
/// This information can be queried later on for more information about the torrent itself.
#[derive(Debug)]
pub struct TorrentCollection {
    storage: Arc<Storage>,
    cache: Mutex<Option<Collection>>,
}

impl TorrentCollection {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            cache: Mutex::new(None),
        }
    }

    /// Verify if the given uri is already stored.
    pub fn is_stored(&self, uri: &str) -> bool {
        match futures::executor::block_on(self.load_collection_cache()) {
            Ok(_) => {
                let mutex = self.cache.blocking_lock();
                let cache = mutex.as_ref().expect("expected the cache to be loaded");

                cache.contains(uri)
            },
            Err(e) => {
                error!("Failed to load torrent collection, {}", e);
                false
            }
        }
    }

    async fn load_collection_cache(&self) -> torrent::Result<()> {
        let mut cache = self.cache.lock().await;

        if cache.is_none() {
            trace!("Loading torrent collection cache");
            return match self.load_collection_from_storage() {
                Ok(e) => {
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e)
            };
        }

        trace!("Torrent collection cache already loaded, nothing to do");
        Ok(())
    }

    fn load_collection_from_storage(&self) -> torrent::Result<Collection> {
        match self.storage.read::<Collection>(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::FileNotFound(file) => {
                        debug!("Creating new torrent collection file {}", file);
                        Ok(Collection::default())
                    }
                    StorageError::CorruptRead(_, error) => {
                        error!("Failed to load torrent collection, {}", error);
                        Err(TorrentError::TorrentCollectionLoadingFailed(error))
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", e);
                        Ok(Collection::default())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_is_stored() {
        init_logger();
        let magnet_uri = "magnet:?MyMagnetUri1";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from(temp_path));
        let collection = TorrentCollection::new(&storage);
        copy_test_file(temp_path, "torrent-collection.json");

        let result = collection.is_stored(magnet_uri);

        assert_eq!(true, result)
    }
}