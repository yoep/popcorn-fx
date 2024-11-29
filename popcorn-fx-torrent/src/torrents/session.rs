use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

use crate::torrents::errors::Result;
use crate::torrents::fs::DefaultTorrentFileStorage;
use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::PeerListener;
use crate::torrents::torrent::Torrent;
use crate::torrents::{
    InfoHash, RequestStrategy, TorrentError, TorrentEvent, TorrentFlags, TorrentHandle,
    TorrentInfo, TorrentOperation, TorrentOperations, DEFAULT_TORRENT_EXTENSIONS,
    DEFAULT_TORRENT_OPERATIONS, DEFAULT_TORRENT_REQUEST_STRATEGIES,
};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
#[cfg(test)]
pub use mock::*;
use popcorn_fx_core::available_port;
use popcorn_fx_core::core::torrents::magnet::Magnet;
use popcorn_fx_core::core::torrents::TorrentHealth;
use popcorn_fx_core::core::{CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::{select, time};

/// A unique handle identifier of a [Session].
pub type SessionHandle = Handle;

/// A callback handler for the [Session] events.
pub type SessionCallback = CoreCallback<SessionEvent>;

/// The torrent session events.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SessionEvent {
    /// Indicates that a new torrent was added to the session.
    #[display(fmt = "Torrent added: {}", _0)]
    TorrentAdded(TorrentHandle),
    /// Indicates that a torrent has been removed from the session.
    #[display(fmt = "Torrent removed: {}", _0)]
    TorrentRemoved(TorrentHandle),
}

/// A torrent session which isolates torrents from each-other.
/// [Session] is able to process and managed torrents from multiple sources.
///
/// The session is always the owner of a [Torrent], meaning that it's able to drop a torrent at any time.
#[async_trait]
pub trait Session: Debug + Callbacks<SessionEvent> + Send + Sync {
    /// Retrieve the unique session identifier for this session.
    /// This handle can be used to identify a session.
    ///
    /// # Returns
    ///
    /// Returns the unique session handle for this session.
    fn handle(&self) -> SessionHandle;

    /// Get the torrent based on the given handle.
    /// It returns a weak reference to the torrent, which can be invalidated at any moment.
    /// To check if a torrent is still valid, use the [Torrent::is_valid] method.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the torrent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the torrent if found, else `None`.
    async fn find_torrent_by_handle(&self, handle: TorrentHandle) -> Option<Torrent>;

    /// Get the torrent based on the given info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash of the torrent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a weak reference to the torrent if found, else `None`.
    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent>;

    /// Get the calculated torrent health based on the given torrent metadata.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to check.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health_from_info(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;

    /// Get the torrent health of the given torrent magnet link.
    /// This will request the seeders and leechers from the magnet's tracker.
    ///
    /// # Arguments
    ///
    /// * `magnet_uri` - The magnet URI of the torrent to fetch the health of.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health_from_uri(&self, magnet_uri: &str) -> Result<TorrentHealth>;

    /// Get the torrent information for the given magnet URI.
    ///
    /// # Arguments
    ///
    /// * `magnet_uri` - The magnet URI of the torrent to fetch.
    /// * `timeout` - The timeout to use when fetching the torrent information.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent information on success or an error on failure.
    async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentInfo>;

    /// Add a new torrent to this session for the given uri.
    /// The uri can either be a path to a torrent file or a magnet link.
    ///
    /// # Arguments
    ///
    /// * `uri` - The uri of the torrent to add.
    /// * `options` - The torrent options to use when adding the torrent.
    ///
    /// # Returns
    ///
    /// Returns the created torrent handle if successful.
    async fn add_torrent_from_uri(&self, uri: &str, options: TorrentFlags)
        -> Result<TorrentHandle>;

    /// Add a new torrent to this session for the given metadata information.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to add.
    /// * `options` - The torrent options to use when adding the torrent.
    ///
    /// # Returns
    ///
    /// Returns the created torrent handle if successful.
    async fn add_torrent_from_info(
        &self,
        torrent_info: TorrentInfo,
        options: TorrentFlags,
    ) -> Result<TorrentHandle>;

    /// Remove a torrent from this session.
    /// The handle will be ignored if it does not exist in this session.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the torrent to remove.
    fn remove_torrent(&self, handle: TorrentHandle);
}

#[derive(Debug, Display)]
#[display(fmt = "Session {}", "inner.handle")]
pub struct DefaultSession {
    inner: Arc<InnerSession>,
    runtime: Arc<Runtime>,
}

impl DefaultSession {
    /// Create a new torrent session builder.
    /// The builder always requires a `base_path` to be set, all other fields are optional and will use defaults if not set.
    ///
    /// This allows for easy setup of a torrent session, while still allow some flexibility in customization at runtime.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    ///
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_torrent::torrents::{DefaultSession, Result};
    /// use popcorn_fx_torrent::torrents::peers::extensions::metadata::MetadataExtension;
    ///
    /// fn new_torrent_session(shared_runtime: Arc<Runtime>) -> Result<DefaultSession> {
    ///     DefaultSession::builder()
    ///         .base_path("/torrent/location/directory")
    ///         .extensions(vec![Box::new(MetadataExtension::new())])
    ///         .runtime(shared_runtime)
    ///         .build()
    /// }
    /// ```
    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
    }

    /// Create a new torrent sessions.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path to use for storing torrent data.
    /// * `extensions` - The peer extensions to use for this session.
    /// * `torrent_operations` - The torrent operations to use for this session.
    /// * `request_strategies` - The request strategies to use for this session.
    /// * `runtime` - The tokio runtime to use for this session.
    ///
    /// # Returns
    ///
    /// Returns the session when initialized successfully or an error on failure.
    pub async fn new<P: AsRef<Path>>(
        base_path: P,
        extensions: Extensions,
        torrent_operations: Vec<Box<dyn TorrentOperation>>,
        request_strategies: Vec<Box<dyn RequestStrategy>>,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!("Trying to create a new torrent session");
        let port = available_port!(6881, 6889).ok_or(TorrentError::Io(
            "no available port found to start new peer listener".to_string(),
        ))?;
        let peer_listener = PeerListener::new(port, runtime.clone()).await?;
        let torrent_storage_location = base_path.as_ref().to_path_buf();
        let inner_session = InnerSession::new(
            torrent_storage_location,
            peer_listener,
            extensions,
            torrent_operations,
            request_strategies,
        )
        .await?;

        debug!("Created new torrent session {}", inner_session.handle);
        Ok(Self {
            inner: Arc::new(inner_session),
            runtime,
        })
    }

    pub async fn torrent(&self, handle: TorrentHandle) -> Option<Torrent> {
        self.inner.find_torrent_by_handle(handle).await
    }

    /// Try to find an existing torrent within the session for the torrent info, or create a new one if not found.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to add.
    /// * `options` - The torrent options to use when adding the torrent.
    /// * `peer_timeout` - The peer timeout to use when adding the torrent.
    /// * `tracker_timeout` - The tracker timeout to use when adding the torrent.
    /// * `send_callback_event` - Whether to send a callback event when the torrent is added.
    ///
    /// # Returns
    ///
    /// Returns a result containing the handle of the newly added or existing torrent on success.
    async fn find_or_add_torrent(
        &self,
        torrent_info: TorrentInfo,
        options: TorrentFlags,
        peer_timeout: Option<Duration>,
        tracker_timeout: Option<Duration>,
        send_callback_event: bool,
    ) -> Result<TorrentHandle> {
        trace!(
            "Trying to add {:?} to session {}",
            torrent_info,
            self.inner.handle
        );
        // check if the info hash is already known
        if let Some(handle) = self
            .inner
            .find_handle_by_info_hash(&torrent_info.info_hash)
            .await
        {
            debug!(
                "Torrent info hash {} already exists in session {}",
                torrent_info.info_hash, self.inner.handle
            );
            return Ok(handle);
        }

        let info_hash = torrent_info.info_hash.clone();
        let mut request = Torrent::request()
            .metadata(torrent_info)
            .options(options)
            .peer_listener_port(self.inner.peer_listener.port())
            .extensions(self.inner.extensions())
            .operations(self.inner.torrent_operations())
            .storage(Box::new(DefaultTorrentFileStorage::new(
                &self.inner.base_path,
            )))
            .runtime(self.runtime.clone());

        if let Some(peer_timeout) = peer_timeout {
            request = request.peer_timeout(peer_timeout);
        }
        if let Some(tracker_timeout) = tracker_timeout {
            request = request.tracker_timeout(tracker_timeout);
        }

        let torrent = Torrent::try_from(request)?;
        let handle = torrent.handle();

        self.inner
            .add_torrent(info_hash, torrent, send_callback_event)
            .await;

        Ok(handle)
    }

    async fn wait_for_metadata(
        torrent: &Torrent,
        rx: Receiver<TorrentEvent>,
        timeout: Duration,
    ) -> Result<TorrentInfo> {
        loop {
            let event = rx
                .recv_timeout(timeout)
                .map_err(|_| TorrentError::Timeout)?;

            if let TorrentEvent::MetadataChanged = event {
                let torrent_info = torrent.metadata().await?;
                if torrent_info.info.is_some() {
                    return Ok(torrent_info);
                }
            }
        }
    }

    fn magnet_to_torrent_info(magnet_uri: &str) -> Result<TorrentInfo> {
        let magnet =
            Magnet::from_str(magnet_uri).map_err(|e| TorrentError::TorrentParse(e.to_string()))?;
        let torrent_info = TorrentInfo::try_from(magnet)?;

        Ok(torrent_info)
    }
}

#[async_trait]
impl Session for DefaultSession {
    fn handle(&self) -> SessionHandle {
        self.inner.handle
    }

    async fn find_torrent_by_handle(&self, handle: TorrentHandle) -> Option<Torrent> {
        self.inner.find_torrent_by_handle(handle).await
    }

    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent> {
        self.inner.find_torrent_by_info_hash(info_hash).await
    }

    async fn torrent_health_from_info(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth> {
        trace!("Retrieving torrent health for {:?}", torrent_info);
        // try to retrieve the existing torrent based on its info hash
        // otherwise, we'll create a new torrent
        let torrent = self
            .inner
            .find_torrent_by_info_hash(&torrent_info.info_hash)
            .await
            .map_or_else(
                || {
                    let request = Torrent::request()
                        .metadata(torrent_info)
                        .options(TorrentFlags::None)
                        .peer_listener_port(self.inner.peer_listener.port())
                        .extensions(self.inner.extensions())
                        .operations(self.inner.torrent_operations())
                        .storage(Box::new(DefaultTorrentFileStorage::new(
                            &self.inner.base_path,
                        )))
                        .peer_timeout(Duration::from_secs(3))
                        .tracker_timeout(Duration::from_secs(2))
                        .runtime(self.runtime.clone());

                    Torrent::try_from(request)
                },
                |e| Ok(e),
            )?;

        let announcement = torrent.announce().await?;

        debug!(
            "Converting announcement to torrent health for {:?}",
            announcement
        );
        Ok(TorrentHealth::from(
            announcement.total_seeders as u32,
            announcement.total_leechers as u32,
        ))
    }

    async fn torrent_health_from_uri(&self, magnet_uri: &str) -> Result<TorrentHealth> {
        trace!("Retrieving torrent health for {:?}", magnet_uri);
        let torrent_info = Self::magnet_to_torrent_info(magnet_uri)?;

        self.torrent_health_from_info(torrent_info).await
    }

    async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentInfo> {
        trace!("Trying to fetch magnet {}", magnet_uri);
        let torrent_info = Self::magnet_to_torrent_info(magnet_uri)?;
        let handle = self
            .find_or_add_torrent(
                torrent_info,
                TorrentFlags::Metadata,
                Some(Duration::from_secs(3)),
                Some(Duration::from_secs(2)),
                false,
            )
            .await?;
        let torrent = self
            .inner
            .find_torrent_by_handle(handle)
            .await
            .ok_or(TorrentError::InvalidHandle(handle))?;
        let (tx, rx) = channel();

        let callback_handle = torrent.add_callback(Box::new(move |event| {
            if let Err(e) = tx.send(event) {
                debug!("Failed to send torrent event, {}", e);
            }
        }));

        // check if the metadata is already fetched
        let torrent_info = torrent.metadata().await?;
        if torrent_info.info.is_some() {
            return Ok(torrent_info);
        }

        // otherwise, wait for the MetadataChanged event
        select! {
            _ = time::sleep(timeout) => {
                <Torrent as Callbacks<TorrentEvent>>::remove_callback(&torrent, callback_handle);
                Err(TorrentError::Timeout)
            },
            result = Self::wait_for_metadata(&torrent, rx, timeout) => {
                <Torrent as Callbacks<TorrentEvent>>::remove_callback(&torrent, callback_handle);
                result
            }
        }
    }

    async fn add_torrent_from_uri(
        &self,
        uri: &str,
        options: TorrentFlags,
    ) -> Result<TorrentHandle> {
        todo!()
    }

    async fn add_torrent_from_info(
        &self,
        torrent_info: TorrentInfo,
        options: TorrentFlags,
    ) -> Result<TorrentHandle> {
        self.find_or_add_torrent(torrent_info, options, None, None, true)
            .await
    }

    fn remove_torrent(&self, handle: TorrentHandle) {
        let inner = self.inner.clone();
        self.runtime
            .spawn(async move { inner.remove_torrent(handle).await });
    }
}

impl Callbacks<SessionEvent> for DefaultSession {
    fn add_callback(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle {
        self.inner.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.inner.remove_callback(handle)
    }
}

#[derive(Debug, Default)]
pub struct SessionBuilder {
    base_path: Option<PathBuf>,
    extensions: Option<Extensions>,
    torrent_operation: Option<Vec<Box<dyn TorrentOperation>>>,
    request_strategy: Option<Vec<Box<dyn RequestStrategy>>>,
    runtime: Option<Arc<Runtime>>,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base path for the torrent storage of the session.
    pub fn base_path<P: AsRef<Path>>(mut self, base_path: P) -> Self {
        self.base_path = Some(base_path.as_ref().to_path_buf());
        self
    }

    /// Set the extensions for the session.
    pub fn extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set the torrent operations for the session.
    pub fn torrent_operations(
        mut self,
        torrent_operations: Vec<Box<dyn TorrentOperation>>,
    ) -> Self {
        self.torrent_operation = Some(torrent_operations);
        self
    }

    /// Set the request strategies for the session.
    pub fn request_strategies(mut self, request_strategies: Vec<Box<dyn RequestStrategy>>) -> Self {
        self.request_strategy = Some(request_strategies);
        self
    }

    /// Set the runtime for the session.
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Create a new torrent session from this builder.
    /// The only required field within this builder is the base path for the torrent storage.
    pub async fn build(self) -> Result<DefaultSession> {
        let base_path = self.base_path.expect("expected the base path to be set");
        let extensions = self.extensions.unwrap_or_else(DEFAULT_TORRENT_EXTENSIONS);
        let torrent_operations = self
            .torrent_operation
            .unwrap_or_else(DEFAULT_TORRENT_OPERATIONS);
        let request_strategies = self
            .request_strategy
            .unwrap_or_else(DEFAULT_TORRENT_REQUEST_STRATEGIES);
        let runtime = self.runtime.unwrap_or_else(|| {
            Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(25)
                    .thread_name("session")
                    .build()
                    .expect("expected a new runtime"),
            )
        });

        DefaultSession::new(
            base_path,
            extensions,
            torrent_operations,
            request_strategies,
            runtime,
        )
        .await
    }
}

// TODO: add options which support configuring timeouts etc
#[derive(Debug)]
struct InnerSession {
    /// The unique session identifier
    handle: SessionHandle,
    /// The base path for the torrent storage of the session
    base_path: PathBuf,
    /// The currently active torrents within the session
    torrents: RwLock<HashMap<InfoHash, Torrent>>,
    /// The peer listener for the session
    peer_listener: PeerListener,
    /// The currently active extensions for the session
    extensions: Extensions,
    /// The torrent operations for the session
    torrent_operations: Vec<Box<dyn TorrentOperation>>,
    /// The request strategies for the session
    request_strategies: Vec<Box<dyn RequestStrategy>>,
    /// The event callbacks of the session
    callbacks: CoreCallbacks<SessionEvent>,
}

impl InnerSession {
    async fn new(
        base_path: PathBuf,
        peer_listener: PeerListener,
        extensions: Extensions,
        torrent_operations: Vec<Box<dyn TorrentOperation>>,
        request_strategies: Vec<Box<dyn RequestStrategy>>,
    ) -> Result<Self> {
        Ok(Self {
            handle: Default::default(),
            base_path,
            peer_listener,
            extensions,
            torrent_operations,
            request_strategies,
            torrents: Default::default(),
            callbacks: Default::default(),
        })
    }

    fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_boxed()).collect()
    }

    fn torrent_operations(&self) -> TorrentOperations {
        self.torrent_operations
            .iter()
            .map(|e| e.clone_boxed())
            .collect()
    }

    async fn find_torrent_by_handle(&self, handle: TorrentHandle) -> Option<Torrent> {
        self.torrents
            .read()
            .await
            .iter()
            .find(|(_, e)| e.handle() == handle)
            .map(|(_, e)| e.clone())
    }

    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent> {
        (*self.torrents.read().await)
            .get(info_hash)
            .map(|e| e.clone())
    }

    async fn find_handle_by_info_hash(&self, info_hash: &InfoHash) -> Option<TorrentHandle> {
        (*self.torrents.read().await)
            .get(info_hash)
            .map(|e| e.handle())
    }

    /// Add the given torrent to the session.
    /// This might replace an existing torrent with the same info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash of the torrent to add.
    /// * `torrent` - The torrent to add.
    /// * `send_callback_event` - Whether to send a callback event when the torrent is added.
    async fn add_torrent(&self, info_hash: InfoHash, torrent: Torrent, send_callback_event: bool) {
        let handle = torrent.handle();

        {
            let mut mutex = self.torrents.write().await;
            debug!(
                "Adding torrent {} with options {:?}",
                handle,
                torrent.options().await
            );
            mutex.insert(info_hash, torrent);
        }

        if send_callback_event {
            self.callbacks.invoke(SessionEvent::TorrentAdded(handle));
        }
    }

    async fn remove_torrent(&self, handle: TorrentHandle) {
        let mut torrent_info_hash: Option<InfoHash> = None;

        {
            let mut mutex = self.torrents.write().await;
            for (info_hash, torrent) in mutex.iter() {
                if torrent.handle() == handle {
                    torrent_info_hash = Some(info_hash.clone());
                    break;
                }
            }

            if let Some(info_hash) = &torrent_info_hash {
                debug!("Removing torrent {}", handle);
                mutex.remove(&info_hash);
            }
        }

        if let Some(_) = torrent_info_hash {
            self.callbacks.invoke(SessionEvent::TorrentRemoved(handle));
        }
    }
}

impl Callbacks<SessionEvent> for InnerSession {
    fn add_callback(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.callbacks.remove_callback(handle)
    }
}

#[cfg(test)]
mod mock {
    use super::*;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub Session {}

        #[async_trait]
        impl Session for Session {
            fn handle(&self) -> SessionHandle;
            async fn find_torrent_by_handle(&self, handle: TorrentHandle) -> Option<Torrent>;
            async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent>;
            async fn torrent_health_from_info(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;
            async fn torrent_health_from_uri(&self, magnet_uri: &str) -> Result<TorrentHealth>;
            async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentInfo>;
            async fn add_torrent_from_uri(&self, uri: &str, options: TorrentFlags) -> Result<TorrentHandle>;
            async fn add_torrent_from_info(&self, torrent_info: TorrentInfo, options: TorrentFlags) -> Result<TorrentHandle>;
            fn remove_torrent(&self, handle: TorrentHandle);
        }

        impl Callbacks<SessionEvent> for Session {
            fn add_callback(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle;
            fn remove_callback(&self, handle: CallbackHandle);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::torrents::peers::extensions::metadata::MetadataExtension;
    use log::info;
    use popcorn_fx_core::core::torrents::TorrentHealthState;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_find_torrent() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let info_hash = info.info_hash.clone();
        let runtime = Arc::new(Runtime::new().unwrap());
        let session = runtime
            .block_on(
                DefaultSession::builder()
                    .base_path(temp_path)
                    .runtime(runtime.clone())
                    .build(),
            )
            .unwrap();

        let _ = runtime
            .block_on(session.add_torrent_from_info(info, TorrentFlags::default()))
            .expect("expected the torrent to have been added");
        let result = runtime.block_on(session.find_torrent_by_info_hash(&info_hash));

        assert_ne!(None, result);
    }

    #[test]
    fn test_session_fetch_magnet() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let runtime = Arc::new(Runtime::new().unwrap());
        let session = runtime
            .block_on(
                DefaultSession::builder()
                    .base_path(temp_path)
                    .runtime(runtime.clone())
                    .build(),
            )
            .unwrap();

        let result = runtime
            .block_on(session.fetch_magnet(uri, Duration::from_secs(30)))
            .unwrap();

        assert_ne!(
            None, result.info,
            "expected the metadata to have been present"
        );
    }

    #[test]
    fn test_session_torrent_health() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let session = runtime
            .block_on(
                DefaultSession::builder()
                    .base_path(temp_path)
                    .extensions(vec![])
                    .runtime(runtime.clone())
                    .build(),
            )
            .unwrap();

        let result = runtime
            .block_on(session.torrent_health_from_info(info))
            .expect("expected a torrent health");

        info!("torrent health: {:?}", result);
        assert_ne!(TorrentHealthState::Unknown, result.state);
        assert_ne!(0, result.seeds, "expected seeders to have been found");
    }

    #[test]
    fn test_session_add_torrent() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let (tx, rx) = channel();
        let session = runtime
            .block_on(
                DefaultSession::builder()
                    .base_path(temp_path)
                    .extensions(vec![])
                    .runtime(runtime.clone())
                    .build(),
            )
            .unwrap();

        session.add_callback(Box::new(move |event| {
            tx.send(event).unwrap();
        }));

        let handle = runtime
            .block_on(session.add_torrent_from_info(info, TorrentFlags::default()))
            .expect("expected a torrent handle");

        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentAdded(handle));
    }

    #[test]
    fn test_session_remove_torrent() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let (tx, rx) = channel();
        let session = runtime
            .block_on(
                DefaultSession::builder()
                    .base_path(temp_path)
                    .extensions(vec![])
                    .runtime(runtime.clone())
                    .build(),
            )
            .unwrap();

        session.add_callback(Box::new(move |event| {
            tx.send(event).unwrap();
        }));
        let handle = runtime
            .block_on(session.add_torrent_from_info(info, TorrentFlags::default()))
            .expect("expected a torrent handle");

        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentAdded(handle));

        session.remove_torrent(handle);
        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentRemoved(handle));
    }
}
