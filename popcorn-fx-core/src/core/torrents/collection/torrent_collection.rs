use crate::core::storage::{Storage, StorageError};
use crate::core::torrents;
use crate::core::torrents::collection::{Collection, MagnetInfo};
use crate::core::torrents::Error;
use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

const FILENAME: &str = "torrent-collection.json";

/// The torrent collections stores magnet uri information.
/// This information can be queried later on for more information about the torrent itself.
#[derive(Debug, Clone)]
pub struct TorrentCollection {
    inner: Arc<InnerTorrentCollection>,
}

impl TorrentCollection {
    /// Create a new torrent collection.
    /// The collection will be stored within the storage directory location.
    ///
    /// # Arguments
    ///
    /// * `storage_directory` - The absolute path to store the torrent collection in.
    ///
    /// # Returns
    ///
    /// It returns a new torrent collection.
    pub fn new(storage_directory: &str) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerTorrentCollection {
            storage: Storage::from(storage_directory),
            cache: RwLock::new(None),
            command_sender,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver).await;
        });

        Self { inner }
    }

    /// Verify if the given uri is already stored.
    pub async fn is_stored(&self, uri: &str) -> bool {
        self.inner.is_stored(uri).await
    }

    /// Retrieve all stored magnets as owned instances.
    /// It returns the array of available [MagnetInfo] items, else the [Error].
    pub async fn all(&self) -> torrents::Result<Vec<MagnetInfo>> {
        match self.inner.load_collection_cache().await {
            Ok(_) => {
                let mutex = self.inner.cache.read().await;
                let cache = mutex.as_ref().expect("expected the cache to be present");

                Ok(cache.torrents.clone())
            }
            Err(e) => Err(e),
        }
    }

    /// Insert the given magnet info into the collection.
    pub async fn insert(&self, name: &str, magnet_uri: &str) {
        match self.inner.load_collection_cache().await {
            Ok(_) => {
                let mut mutex = self.inner.cache.write().await;
                let cache = mutex.as_mut().expect("expected the cache to be present");

                cache.insert(name, magnet_uri);
                self.inner.send_command(TorrentCollectionCommand::Save);
            }
            Err(e) => {
                error!("Failed to load torrent collection, {}", e);
            }
        }
    }

    /// Remove the given magnet uri from the collection.
    pub async fn remove(&self, magnet_uri: &str) {
        match self.inner.load_collection_cache().await {
            Ok(_) => {
                let mut mutex = self.inner.cache.write().await;
                let cache = mutex.as_mut().expect("expected the cache to be present");

                cache.remove(magnet_uri);
                self.inner.send_command(TorrentCollectionCommand::Save);
            }
            Err(e) => error!("Failed to remove the magnet from the collection, {}", e),
        }
    }
}

#[derive(Debug, PartialEq)]
enum TorrentCollectionCommand {
    /// Save the current torrent collection to the storage device.
    Save,
}

#[derive(Debug)]
struct InnerTorrentCollection {
    storage: Storage,
    cache: RwLock<Option<Collection>>,
    command_sender: UnboundedSender<TorrentCollectionCommand>,
    cancellation_token: CancellationToken,
}

impl InnerTorrentCollection {
    async fn start(&self, mut receiver: UnboundedReceiver<TorrentCollectionCommand>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = receiver.recv() => self.handle_command(command).await,
            }
        }
        self.save().await;
        debug!("Torrent collection main loop ended");
    }

    async fn handle_command(&self, command: TorrentCollectionCommand) {
        match command {
            TorrentCollectionCommand::Save => self.save().await,
        }
    }

    async fn is_stored(&self, uri: &str) -> bool {
        match self.load_collection_cache().await {
            Ok(_) => {
                let mutex = self.cache.read().await;
                let cache = mutex.as_ref().expect("expected the cache to be loaded");

                cache.contains(uri)
            }
            Err(e) => {
                error!("Failed to load torrent collection, {}", e);
                false
            }
        }
    }

    async fn save(&self) {
        trace!("Torrent collection is saving collection");
        let collection = self.cache.read().await;
        match self
            .storage
            .options()
            .serializer(FILENAME)
            .write_async(&*collection)
            .await
        {
            Ok(_) => info!("Torrent collection data has been saved"),
            Err(e) => error!("Failed to save torrent collection, {}", e),
        }
    }

    async fn load_collection_cache(&self) -> torrents::Result<()> {
        let mut cache = self.cache.write().await;

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
                    Err(Error::TorrentCollectionLoadingFailed(error))
                }
                _ => {
                    warn!("Unexpected error returned from storage, {}", e);
                    Ok(Collection::default())
                }
            },
        }
    }

    fn send_command(&self, command: TorrentCollectionCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Torrent collection failed to send command, {}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::init_logger;
    use crate::testing::copy_test_file;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_is_stored() {
        init_logger!();
        let magnet_uri = "magnet:?MyMagnetUri1";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let collection = TorrentCollection::new(temp_path);
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result = collection.is_stored(magnet_uri).await;

        assert_eq!(true, result)
    }

    #[tokio::test]
    async fn test_insert_new_item() {
        init_logger!();
        let name = "MyMagnet";
        let uri = "magnet:?LoremIpsumConn";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let collection = TorrentCollection::new(temp_path);
        let expected_result = vec![MagnetInfo {
            name: name.to_string(),
            magnet_uri: uri.to_string(),
        }];

        collection.insert(name, uri).await;

        let result = collection.is_stored(uri).await;
        assert_eq!(true, result);

        let magnets = collection
            .all()
            .await
            .expect("expected magnet to be returned");
        assert_eq!(expected_result, magnets)
    }

    #[tokio::test]
    async fn test_remove_magnet_uri() {
        init_logger!();
        let uri = "magnet:?MyMagnetUri1";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let collection = TorrentCollection::new(temp_path);
        copy_test_file(temp_path, "torrent-collection.json", None);
        let expected_result = vec![MagnetInfo {
            name: "MyMagnet2".to_string(),
            magnet_uri: "magnet:?MyMagnet2MagnetUrl".to_string(),
        }];

        collection.remove(uri).await;
        let result = collection
            .all()
            .await
            .expect("expected the magnets to be returned");

        assert_eq!(expected_result, result)
    }
}
