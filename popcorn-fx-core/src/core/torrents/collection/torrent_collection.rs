use log::{debug, error, info, trace, warn};
use tokio::sync::Mutex;

use crate::core::{block_in_place, torrents};
use crate::core::storage::{Storage, StorageError};
use crate::core::torrents::collection::{Collection, MagnetInfo};
use crate::core::torrents::TorrentError;

const FILENAME: &str = "torrent-collection.json";

/// The torrent collections stores magnet uri information.
/// This information can be queried later on for more information about the torrent itself.
#[derive(Debug)]
pub struct TorrentCollection {
    storage: Storage,
    cache: Mutex<Option<Collection>>,
}

impl TorrentCollection {
    pub fn new(storage_directory: &str) -> Self {
        Self {
            storage: Storage::from(storage_directory),
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
            }
            Err(e) => {
                error!("Failed to load torrent collection, {}", e);
                false
            }
        }
    }

    /// Retrieve all stored magnets as owned instances.
    /// It returns the array of available [MagnetInfo] items, else the [TorrentError].
    pub fn all(&self) -> torrents::Result<Vec<MagnetInfo>> {
        match futures::executor::block_on(self.load_collection_cache()) {
            Ok(_) => {
                let mutex = self.cache.blocking_lock();
                let cache = mutex.as_ref().expect("expected the cache to be present");

                Ok(cache.torrents.clone())
            }
            Err(e) => Err(e),
        }
    }

    /// Insert the given magnet info into the collection.
    pub fn insert(&self, name: &str, magnet_uri: &str) {
        match futures::executor::block_on(self.load_collection_cache()) {
            Ok(_) => {
                let mut mutex = self.cache.blocking_lock();
                let cache = mutex.as_mut().expect("expected the cache to be present");

                cache.insert(name, magnet_uri);
                self.save(cache);
            }
            Err(e) => {
                error!("Failed to load torrent collection, {}", e);
            }
        }
    }

    /// Remove the given magnet uri from the collection.
    pub fn remove(&self, magnet_uri: &str) {
        match futures::executor::block_on(self.load_collection_cache()) {
            Ok(_) => {
                let mut mutex = self.cache.blocking_lock();
                let cache = mutex.as_mut().expect("expected the cache to be present");

                cache.remove(magnet_uri);
                self.save(cache);
            }
            Err(e) => error!("Failed to remove the magnet from the collection, {}", e),
        }
    }

    async fn load_collection_cache(&self) -> torrents::Result<()> {
        let mut cache = self.cache.lock().await;

        if cache.is_none() {
            trace!("Loading torrent collection cache");
            return match self.load_collection_from_storage() {
                Ok(e) => {
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e),
            };
        }

        trace!("Torrent collection cache already loaded, nothing to do");
        Ok(())
    }

    fn load_collection_from_storage(&self) -> torrents::Result<Collection> {
        match self
            .storage
            .options()
            .serializer(FILENAME)
            .read::<Collection>()
        {
            Ok(e) => Ok(e),
            Err(e) => match e {
                StorageError::NotFound(file) => {
                    debug!("Creating new torrent collection file {}", file);
                    Ok(Collection::default())
                }
                StorageError::ReadingFailed(_, error) => {
                    error!("Failed to load torrent collection, {}", error);
                    Err(TorrentError::TorrentCollectionLoadingFailed(error))
                }
                _ => {
                    warn!("Unexpected error returned from storage, {}", e);
                    Ok(Collection::default())
                }
            },
        }
    }

    fn save(&self, collection: &Collection) {
        block_in_place(self.save_async(collection))
    }

    async fn save_async(&self, collection: &Collection) {
        match self
            .storage
            .options()
            .serializer(FILENAME)
            .write_async(collection)
            .await
        {
            Ok(_) => info!("Torrent collection data has been saved"),
            Err(e) => error!("Failed to save torrent collection, {}", e),
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
        let collection = TorrentCollection::new(temp_path);
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result = collection.is_stored(magnet_uri);

        assert_eq!(true, result)
    }

    #[test]
    fn test_insert_new_item() {
        init_logger();
        let name = "MyMagnet";
        let uri = "magnet:?LoremIpsumConn";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let collection = TorrentCollection::new(temp_path);
        let expected_result = vec![MagnetInfo {
            name: name.to_string(),
            magnet_uri: uri.to_string(),
        }];

        collection.insert(name, uri);

        let result = collection.is_stored(uri);
        assert_eq!(true, result);

        let magnets = collection.all().expect("expected magnet to be returned");
        assert_eq!(expected_result, magnets)
    }

    #[test]
    fn test_remove_magnet_uri() {
        init_logger();
        let uri = "magnet:?MyMagnetUri1";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let collection = TorrentCollection::new(temp_path);
        copy_test_file(temp_path, "torrent-collection.json", None);
        let expected_result = vec![MagnetInfo {
            name: "MyMagnet2".to_string(),
            magnet_uri: "magnet:?MyMagnet2MagnetUrl".to_string(),
        }];

        collection.remove(uri);
        let result = collection
            .all()
            .expect("expected the magnets to be returned");

        assert_eq!(expected_result, result)
    }
}
