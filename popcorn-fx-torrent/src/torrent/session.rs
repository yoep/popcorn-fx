use crate::torrent::errors::Result;
use crate::torrent::fs::TorrentFileSystemStorage;
use crate::torrent::operation::TorrentTrackersOperation;
use crate::torrent::peer::{ProtocolExtensionFlags, TcpPeerDiscovery, UtpPeerDiscovery};
use crate::torrent::torrent::Torrent;
use crate::torrent::{
    ExtensionFactories, ExtensionFactory, InfoHash, Magnet, TorrentConfig, TorrentError,
    TorrentEvent, TorrentFlags, TorrentHandle, TorrentHealth, TorrentMetadata, TorrentOperation,
    TorrentOperationFactory, DEFAULT_TORRENT_EXTENSIONS, DEFAULT_TORRENT_OPERATIONS,
    DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
};

use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, trace, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::{select, time};

use crate::torrent::dht::{DhtTracker, DEFAULT_BOOTSTRAP_SERVERS};
use crate::torrent::dns::DnsResolver;
use crate::torrent::tracker::TrackerManager;
#[cfg(test)]
pub use mock::*;

const DEFAULT_TRACKER_TIMEOUT_SECONDS: u64 = 3;

/// A unique handle identifier of a [Session].
pub type SessionHandle = Handle;

/// The state of a torrent session.
#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum SessionState {
    /// This is the initial state of the session while it's creating the underlying listeners.
    #[display(fmt = "initializing")]
    Initializing,
    /// The running state of the session in which it's able to process torrents.
    #[display(fmt = "running")]
    Running,
    /// The error state which indicates that the session failed to initialize.
    #[display(fmt = "error")]
    Error,
}

/// The events of a torrent session.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SessionEvent {
    /// Indicates that a new torrent was added to the session.
    #[display(fmt = "torrent {} has been added", _0)]
    TorrentAdded(TorrentHandle),
    /// Indicates that a torrent has been removed from the session.
    #[display(fmt = "torrent {} has been removed", _0)]
    TorrentRemoved(TorrentHandle),
    /// Indicates that the session state has changed.
    #[display(fmt = "session state changed to {:?}", _0)]
    StateChanged(SessionState),
}

/// A torrent session which isolates torrents from each-other.
/// [Session] is able to process and managed torrents from multiple sources.
///
/// The session is always the owner of a [Torrent], meaning that it's able to drop a torrent at any time.
#[async_trait]
pub trait Session: Debug + Callback<SessionEvent> + Send + Sync {
    /// Retrieve the unique session identifier for this session.
    /// This handle can be used to identify a session.
    ///
    /// # Returns
    ///
    /// Returns the unique session handle for this session.
    fn handle(&self) -> SessionHandle;

    /// Get the current state of the session.
    ///
    /// # Returns
    ///
    /// It returns the state of the session.
    async fn state(&self) -> SessionState;

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
    async fn find_torrent_by_handle(&self, handle: &TorrentHandle) -> Option<Torrent>;

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
    async fn torrent_health_from_info(
        &self,
        torrent_info: &TorrentMetadata,
    ) -> Result<TorrentHealth>;

    /// Get the torrent health information for the given uri.
    /// The uri can either be a magnet uri or a filepath to a torrent file.
    ///
    /// If the uri points to a valid resolvable torrent information, than the seeders and leechers will be requested from the trackers.
    ///
    /// # Arguments
    ///
    /// * `uri` - The uri of the torrent to check.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health_from_uri(&self, uri: &str) -> Result<TorrentHealth>;

    /// Resolve the given uri into torrent information.
    /// The uri can either be a magnet uri or a filepath to a torrent file.
    ///
    /// This doesn't create any underlying [Torrent] neither does it retrieve the metadata if it's incomplete.
    /// It's just a simple conversion of a `.torrent` file or magnet uri into [TorrentMetadata].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use popcorn_fx_torrent::torrent::Session;
    ///
    /// fn example(session: impl Session) {
    ///     let magnet_uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
    ///     let info = session.resolve(magnet_uri);
    ///     
    ///     let filepath = "/my/path/example.torrent";
    ///     let info = session.resolve(magnet_uri);
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `uri` - The uri to resolve.
    ///
    /// # Returns
    ///
    /// Returns the resolved torrent information on success.
    fn resolve(&self, uri: &str) -> Result<TorrentMetadata>;

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
    async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentMetadata>;

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
    async fn add_torrent_from_uri(&self, uri: &str, options: TorrentFlags) -> Result<Torrent>;

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
        torrent_info: TorrentMetadata,
        options: TorrentFlags,
    ) -> Result<Torrent>;

    /// Remove a torrent from this session.
    /// The handle will be ignored if it does not exist in this session.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the torrent to remove.
    async fn remove_torrent(&self, handle: &TorrentHandle);

    /// Get the total number of active connections within this session.
    ///
    /// # Returns
    ///
    /// It returns the total connections in-use.
    async fn total_connections(&self) -> usize;
}

/// The default Fx torrent session.
/// This is the standard [Session] implementation with default functionality for working with torrents.
///
/// See [FxTorrentSession::builder] for more information.
///
/// # Example
///
/// ```rust,no_run
/// use popcorn_fx_torrent::torrent::{FxTorrentSession, Result};
/// use popcorn_fx_torrent::torrent::peer::extension::metadata::MetadataExtension;
/// use popcorn_fx_torrent::torrent::peer::ProtocolExtensionFlags;
///
/// fn getting_started() -> Result<FxTorrentSession> {
///     FxTorrentSession::builder()
///         .base_path("/torrent/location/directory")
///         .client_name("MyClient")
///         .build()
/// }
/// ```
#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", inner)]
pub struct FxTorrentSession {
    inner: Arc<InnerSession>,
}

impl FxTorrentSession {
    /// Create a new torrent session builder.
    /// The builder always requires a `base_path` to be set, all other fields are optional and will use defaults if not set.
    ///
    /// This allows for easy setup of a torrent session, while still allow some flexibility in customization at runtime.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use popcorn_fx_torrent::torrent::{FxTorrentSession, Result};
    /// use popcorn_fx_torrent::torrent::peer::extension::metadata::MetadataExtension;
    /// use popcorn_fx_torrent::torrent::peer::ProtocolExtensionFlags;
    ///
    /// fn new_torrent_session() -> Result<FxTorrentSession> {
    ///     FxTorrentSession::builder()
    ///         .base_path("/torrent/location/directory")
    ///         .client_name("MyClient")
    ///         .protocol_extensions(ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast)
    ///         .extensions(vec![|| Box::new(MetadataExtension::new())])
    ///         .build()
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The `build` function of the builder will panic if the `base path` or `client name` is not set.
    /// Everything else is optional and uses default settings if not set.
    pub fn builder() -> FxTorrentSessionBuilder {
        FxTorrentSessionBuilder::new()
    }

    /// Create a new torrent sessions.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path to use for storing torrent data.
    /// * `client_name` - The client name of the session.
    /// * `protocol_extensions` - The protocol extensions to use for this session.
    /// * `extensions` - The peer extensions to use for this session.
    /// * `operations` - The torrent operations to use for this session.
    ///
    /// # Returns
    ///
    /// Returns the session when initialized successfully or an error on failure.
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(
        base_path: P,
        client_name: S,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: ExtensionFactories,
        operations: Vec<TorrentOperationFactory>,
    ) -> Self {
        let handle = SessionHandle::new();

        trace!("Creating new torrent session {}", handle);
        let inner = Arc::new(InnerSession {
            handle,
            state: Mutex::new(SessionState::Initializing),
            base_path: RwLock::new(base_path.as_ref().to_path_buf()),
            client_name: client_name.as_ref().to_string(),
            dht: Default::default(),
            tracker: TrackerManager::new(Duration::from_secs(DEFAULT_TRACKER_TIMEOUT_SECONDS)),
            torrents: Default::default(),
            protocol_extensions,
            extension_factories: extensions,
            torrent_operations: operations,
            callbacks: MultiThreadedCallback::new(),
        });

        let main_inner = inner.clone();
        tokio::spawn(async move {
            main_inner.start().await;
        });

        debug!("Created new torrent session {}", inner.handle);
        Self { inner }
    }

    /// Try to find an existing torrent within the session based on the info hash,
    /// or create a new torrent from the given torrent information.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to add.
    /// * `options` - The torrent options to use when adding the torrent.
    /// * `peer_timeout` - The peer timeout to use when adding the torrent.
    /// * `send_callback_event` - Whether to send a callback event when the torrent is added.
    ///
    /// # Returns
    ///
    /// Returns a torrent (weak reference) on success.
    async fn find_or_add_torrent(
        &self,
        torrent_info: TorrentMetadata,
        options: TorrentFlags,
        peer_timeout: Option<Duration>,
        send_callback_event: bool,
    ) -> Result<Torrent> {
        self.inner.assert_state().await?;

        trace!(
            "Trying to add {:?} to session {}",
            torrent_info,
            self.inner.handle
        );
        // check if the info hash is already known
        if let Some(torrent) = self
            .inner
            .find_torrent_by_info_hash(&torrent_info.info_hash)
            .await
        {
            debug!(
                "Torrent info hash {} already exists in session {}",
                torrent_info.info_hash, self.inner.handle
            );
            return Ok(torrent);
        }

        let info_hash = torrent_info.info_hash.clone();
        let mut config = TorrentConfig::builder().client_name(self.inner.client_name.as_str());

        if let Some(peer_timeout) = peer_timeout {
            config = config.peer_connection_timeout(peer_timeout);
        }

        let storage_path = self.inner.base_path.read().await.clone();
        trace!(
            "Session {} is creating new torrent for info hash {}",
            self,
            info_hash
        );
        let tcp_peer_discovery = self.inner.create_tcp_peer_discovery().await?;
        let utp_peer_discovery = self.inner.create_utp_peer_discovery().await?;
        let dht_tracker = self.inner.dht.lock().await.clone();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .options(options)
            .config(config.build())
            .peer_discoveries(vec![
                Box::new(tcp_peer_discovery),
                Box::new(utp_peer_discovery),
            ])
            .protocol_extensions(self.inner.protocol_extensions)
            .extensions(self.inner.extensions())
            .operations(self.inner.torrent_operations())
            .storage(Box::new(TorrentFileSystemStorage::new(storage_path)))
            .tracker_manager(self.inner.tracker.clone())
            .dht_option(dht_tracker)
            .build()?;
        let result_torrent = torrent.clone();

        self.inner
            .add_torrent(info_hash, torrent, send_callback_event)
            .await;

        Ok(result_torrent)
    }

    async fn wait_for_metadata(torrent: &Torrent) -> TorrentMetadata {
        let mut receiver = torrent.subscribe();

        loop {
            if let Some(event) = receiver.recv().await {
                if let TorrentEvent::MetadataChanged(metadata) = &*event {
                    return metadata.clone();
                }
            }
        }
    }
}

#[async_trait]
impl Session for FxTorrentSession {
    fn handle(&self) -> SessionHandle {
        self.inner.handle
    }

    async fn state(&self) -> SessionState {
        *self.inner.state.lock().await
    }

    async fn find_torrent_by_handle(&self, handle: &TorrentHandle) -> Option<Torrent> {
        self.inner.find_torrent_by_handle(handle).await
    }

    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent> {
        self.inner.find_torrent_by_info_hash(info_hash).await
    }

    async fn torrent_health_from_info(
        &self,
        torrent_info: &TorrentMetadata,
    ) -> Result<TorrentHealth> {
        trace!("Retrieving torrent health for {:?}", torrent_info);
        // try to retrieve the existing torrent based on its info hash
        // otherwise, we'll create a new torrent
        let torrent = match self
            .inner
            .find_torrent_by_info_hash(&torrent_info.info_hash)
            .await
        {
            Some(e) => e,
            None => Torrent::request()
                .metadata(torrent_info.clone())
                .options(TorrentFlags::none())
                .config(
                    TorrentConfig::builder()
                        .client_name(self.inner.client_name.as_str())
                        .peers_lower_limit(0)
                        .peers_upper_limit(0)
                        .peer_connection_timeout(Duration::from_secs(0))
                        .build(),
                )
                .protocol_extensions(self.inner.protocol_extensions)
                .extensions(self.inner.extensions())
                .operations(vec![Box::new(TorrentTrackersOperation::new())])
                .storage(Box::new(TorrentFileSystemStorage::new(
                    &self.inner.base_path.read().await.clone(),
                )))
                .tracker_manager(self.inner.tracker.clone())
                .dht_option(self.inner.dht.lock().await.clone())
                .build()?,
        };

        let metrics = torrent.scrape().await?;

        debug!(
            "Converting announcement to torrent health for {:?}",
            metrics
        );
        Ok(TorrentHealth::from(metrics.complete, metrics.incomplete))
    }

    async fn torrent_health_from_uri(&self, uri: &str) -> Result<TorrentHealth> {
        trace!("Retrieving torrent health for {:?}", uri);
        let torrent_info = self.resolve(uri)?;
        self.torrent_health_from_info(&torrent_info).await
    }

    fn resolve(&self, uri: &str) -> Result<TorrentMetadata> {
        trace!("Resolving torrent uri {}", uri);
        Magnet::from_str(uri)
            .map_err(Into::<TorrentError>::into)
            .and_then(|e| TorrentMetadata::try_from(e))
            .map(|e| Ok::<TorrentMetadata, TorrentError>(e))
            .unwrap_or_else(|_| {
                PathBuf::from_str(uri)
                    .map_err(|e| TorrentError::Io(io::Error::new(io::ErrorKind::InvalidInput, e)))
                    .and_then(|filepath| {
                        std::fs::OpenOptions::new()
                            .create(false)
                            .read(true)
                            .open(filepath)
                            .map_err(|e| TorrentError::Io(e))
                    })
                    .and_then(|mut file| {
                        let mut buffer = vec![];
                        if let Err(e) = file.read_to_end(&mut buffer) {
                            return Err(TorrentError::Io(e));
                        }

                        Ok(buffer)
                    })
                    .and_then(|bytes| TorrentMetadata::try_from(bytes.as_slice()))
            })
    }

    async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentMetadata> {
        self.inner.assert_state().await?;

        trace!("Trying to fetch magnet {}", magnet_uri);
        let torrent_info = self.resolve(magnet_uri)?;
        let torrent = self
            .find_or_add_torrent(
                torrent_info,
                TorrentFlags::Metadata,
                Some(Duration::from_secs(3)),
                false,
            )
            .await?;

        // check if the metadata is already fetched
        let torrent_info = torrent.metadata().await?;
        if torrent_info.info.is_some() {
            return Ok(torrent_info);
        }

        // make sure the torrent tries to download the metadata
        torrent.add_options(TorrentFlags::Metadata).await;

        // otherwise, wait for the MetadataChanged event
        trace!("Trying to fetch metadata for {}", magnet_uri);
        select! {
            _ = time::sleep(timeout) => Err(TorrentError::Timeout),
            result = Self::wait_for_metadata(&torrent) => Ok(result),
        }
    }

    async fn add_torrent_from_uri(&self, uri: &str, options: TorrentFlags) -> Result<Torrent> {
        self.inner.assert_state().await?;

        let torrent_info = self.resolve(uri)?;
        self.add_torrent_from_info(torrent_info, options).await
    }

    async fn add_torrent_from_info(
        &self,
        torrent_info: TorrentMetadata,
        options: TorrentFlags,
    ) -> Result<Torrent> {
        self.inner.assert_state().await?;

        self.find_or_add_torrent(torrent_info, options, None, true)
            .await
    }

    async fn remove_torrent(&self, handle: &TorrentHandle) {
        self.inner.remove_torrent(handle).await
    }

    async fn total_connections(&self) -> usize {
        let torrents = self.inner.torrents.read().await;
        let mut total_connections = 0;

        for torrent in torrents.values() {
            total_connections += torrent.active_peer_connections().await;
        }

        total_connections
    }
}

impl Callback<SessionEvent> for FxTorrentSession {
    fn subscribe(&self) -> Subscription<SessionEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<SessionEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for FxTorrentSession {
    fn drop(&mut self) {
        // check if we're the last 2 references to the session
        // if so, terminate the main loop of the session
        if Arc::strong_count(&self.inner) == 2 {
            self.inner.tracker.close();
        }
    }
}

/// The torrent session builder for configuring an [FxTorrentSession].
///
/// # Required fields
///
/// The following fields are required to be configured.
///
/// - `base_path` - The path location of where torrent file data will be stored.
/// - `client_name` - The client name which is communicated between torrent peers.
///
/// All other fields make use of defaults when not set.
#[derive(Debug, Default)]
pub struct FxTorrentSessionBuilder {
    base_path: Option<PathBuf>,
    client_name: Option<String>,
    protocol_extensions: Option<ProtocolExtensionFlags>,
    extension_factories: Option<ExtensionFactories>,
    operation_factories: Option<Vec<TorrentOperationFactory>>,
}

impl FxTorrentSessionBuilder {
    /// Create a new builder instance to construct a [FxTorrentSession].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the path location for the torrent file data storage of the session.
    pub fn base_path<P: AsRef<Path>>(&mut self, base_path: P) -> &mut Self {
        self.base_path = Some(base_path.as_ref().to_path_buf());
        self
    }

    /// Set the client name for the session.
    /// This is the name of the client that is exchanged with peers that support `LTEP`.
    pub fn client_name<S: AsRef<str>>(&mut self, client_name: S) -> &mut Self {
        self.client_name = Some(client_name.as_ref().to_string());
        self
    }

    /// Set the protocol extensions for the session.
    pub fn protocol_extensions(
        &mut self,
        protocol_extensions: ProtocolExtensionFlags,
    ) -> &mut Self {
        self.protocol_extensions = Some(protocol_extensions);
        self
    }

    /// Add an extension to the session.
    pub fn extension(&mut self, extension: ExtensionFactory) -> &mut Self {
        self.extension_factories
            .get_or_insert(Vec::new())
            .push(extension);
        self
    }

    /// Set the extensions for the session.
    pub fn extensions(&mut self, extensions: ExtensionFactories) -> &mut Self {
        self.extension_factories
            .get_or_insert(Vec::new())
            .extend(extensions);
        self
    }

    /// Add an operation to the session.
    pub fn operation(&mut self, operation: TorrentOperationFactory) -> &mut Self {
        self.operation_factories
            .get_or_insert(Vec::new())
            .push(operation);
        self
    }

    /// Set the torrent operation factories for the session.
    /// These are the operations which are executed on the main loop of the torrent.
    pub fn operations(&mut self, torrent_operations: Vec<TorrentOperationFactory>) -> &mut Self {
        self.operation_factories = Some(torrent_operations);
        self
    }

    /// Create a new torrent session from this builder.
    /// The only required field within this builder is the base path for the torrent storage.
    ///
    /// # Returns
    ///
    /// It returns an error when one of the required is not set.
    pub fn build(&mut self) -> Result<FxTorrentSession> {
        let base_path = self.base_path.take().ok_or(TorrentError::InvalidSession(
            "base path is required".to_string(),
        ))?;
        let client_name = self.client_name.take().filter(|e| !e.is_empty()).ok_or(
            TorrentError::InvalidSession("client name is required".to_string()),
        )?;
        let protocol_extensions = self
            .protocol_extensions
            .unwrap_or_else(DEFAULT_TORRENT_PROTOCOL_EXTENSIONS);
        let extensions = self
            .extension_factories
            .take()
            .unwrap_or_else(DEFAULT_TORRENT_EXTENSIONS);
        let torrent_operations = self
            .operation_factories
            .take()
            .unwrap_or_else(DEFAULT_TORRENT_OPERATIONS);

        Ok(FxTorrentSession::new(
            base_path,
            client_name,
            protocol_extensions,
            extensions,
            torrent_operations,
        ))
    }
}

// TODO: add options which support configuring timeouts etc
#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerSession {
    /// The unique session identifier
    handle: SessionHandle,
    /// The state of the session
    state: Mutex<SessionState>,
    /// The base path for the torrent storage of the session
    base_path: RwLock<PathBuf>,
    /// The client name of the session, exchanged with peers that support `LTEP`
    client_name: String,
    /// The DHT node server of the session
    dht: Mutex<Option<DhtTracker>>,
    /// The tracker of the session
    tracker: TrackerManager,
    /// The currently active torrents within the session
    torrents: RwLock<HashMap<InfoHash, Torrent>>,
    /// The enabled protocol extensions of the session
    protocol_extensions: ProtocolExtensionFlags,
    /// The factories to create new extensions for a torrent
    extension_factories: ExtensionFactories,
    /// The torrent operations for the session
    torrent_operations: Vec<TorrentOperationFactory>,
    /// The event callbacks of the session
    callbacks: MultiThreadedCallback<SessionEvent>,
}

impl InnerSession {
    /// Start the main loop of the session.
    async fn start(&self) {
        if let Err(e) = self.initialize_dht_tracker().await {
            warn!("Session {} failed to initialize, {}", self, e);
            self.update_state(SessionState::Error).await;
            return;
        }

        self.update_state(SessionState::Running).await;
    }

    /// Initialize the DHT tracker of the session.
    #[cfg(feature = "dht")]
    async fn initialize_dht_tracker(&self) -> Result<()> {
        let mut dht_tracker_builder = DhtTracker::builder();

        for addr in DEFAULT_BOOTSTRAP_SERVERS() {
            match DnsResolver::from_str(addr) {
                Ok(resolver) => resolver
                    .resolve()
                    .await
                    .into_iter()
                    .flatten()
                    .for_each(|addr| {
                        dht_tracker_builder.routing_node(addr);
                    }),
                Err(e) => debug!("Failed to resolve IP of node bootstrap \"{}\", {}", addr, e),
            }
        }

        let dht_tracker = dht_tracker_builder.build().await?;

        *self.dht.lock().await = Some(dht_tracker);
        Ok(())
    }

    #[cfg(not(feature = "dht"))]
    async fn initialize_dht_tracker(&self) -> Result<()> {
        Ok(())
    }

    /// Verify that the session is in the expected state to execute actions.
    async fn assert_state(&self) -> Result<()> {
        let state = self.state.lock().await;
        if *state != SessionState::Running {
            return Err(TorrentError::InvalidSessionState(*state));
        }

        Ok(())
    }

    /// Update the state of the session.
    /// This will trigger an event callback for the session if the state is different from the current state.
    async fn update_state(&self, state: SessionState) {
        let mut mutex = self.state.lock().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        self.callbacks.invoke(SessionEvent::StateChanged(state));
    }

    /// Get the enabled peer extensions of the session.
    fn extensions(&self) -> ExtensionFactories {
        self.extension_factories.clone()
    }

    /// Get the torrent processing operation.
    fn torrent_operations(&self) -> Vec<Box<dyn TorrentOperation>> {
        self.torrent_operations.iter().map(|e| e()).collect()
    }

    async fn find_torrent_by_handle(&self, handle: &TorrentHandle) -> Option<Torrent> {
        self.torrents
            .read()
            .await
            .iter()
            .find(|(_, e)| e.handle() == *handle)
            .map(|(_, e)| e.clone())
    }

    /// Try to find the torrent by the given info hash.
    /// It returns a weak reference to the torrent if it is found, otherwise None.
    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent> {
        (*self.torrents.read().await)
            .get(info_hash)
            .map(|e| e.clone())
    }

    /// Add or replace the torrent in the session based on the info hash.
    ///
    /// ## Caution
    ///
    /// This might replace an existing torrent with the same info hash.
    /// The original strong reference torrent will be dropped in this scenario, invalidating the original torrent.
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

    async fn remove_torrent(&self, handle: &TorrentHandle) {
        trace!("Session {} is trying to remove torrent {}", self, handle);
        let torrent_info_hash: Option<InfoHash>;

        {
            let mut mutex = self.torrents.write().await;
            torrent_info_hash = mutex
                .iter()
                .find(|(_, torrent)| torrent.handle() == *handle)
                .map(|(info_hash, _)| info_hash)
                .cloned();

            if let Some(info_hash) = &torrent_info_hash {
                mutex.remove(&info_hash);
                debug!("Session {} removed torrent {}", self, handle);
            }
        }

        if let Some(_) = torrent_info_hash {
            self.callbacks.invoke(SessionEvent::TorrentRemoved(*handle));
        } else {
            trace!("Session {} has no torrent {}", self, handle);
        }
    }

    /// Try to create a TCP peer discovery which listens for incoming connections within the configured port range of the session.
    /// This function might try multiple times to find a free port and return an error if none is available.
    ///
    /// # Returns
    ///
    /// It returns a [TcpPeerDiscovery] on success, else the underlying `bind` failure.
    async fn create_tcp_peer_discovery(&self) -> Result<TcpPeerDiscovery> {
        TcpPeerDiscovery::new()
            .await
            .map_err(|e| TorrentError::Peer(e))
    }

    /// Try to create a uTP peer discovery which listens for incoming connections within the configured port range of the session.
    /// This function might try multiple times to find a free port and return an error if none is available.
    ///
    /// # Returns
    ///
    /// It returns a [UtpPeerDiscovery] on success, else the underlying `bind` failure.
    async fn create_utp_peer_discovery(&self) -> Result<UtpPeerDiscovery> {
        UtpPeerDiscovery::new()
            .await
            .map_err(|e| TorrentError::Peer(e))
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
            async fn state(&self) -> SessionState;
            async fn find_torrent_by_handle(&self, handle: &TorrentHandle) -> Option<Torrent>;
            async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent>;
            async fn torrent_health_from_info(&self, torrent_info: &TorrentMetadata) -> Result<TorrentHealth>;
            async fn torrent_health_from_uri(&self, uri: &str) -> Result<TorrentHealth>;
            fn resolve(&self, uri: &str) -> Result<TorrentMetadata>;
            async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentMetadata>;
            async fn add_torrent_from_uri(&self, uri: &str, options: TorrentFlags) -> Result<Torrent>;
            async fn add_torrent_from_info(&self, torrent_info: TorrentMetadata, options: TorrentFlags) -> Result<Torrent>;
            async fn remove_torrent(&self, handle: &TorrentHandle);
            async fn total_connections(&self) -> usize;
        }

        impl Callback<SessionEvent> for Session {
            fn subscribe(&self) -> Subscription<SessionEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<SessionEvent>);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::torrent::TorrentHealthState;
    use crate::{create_torrent, timeout};

    use crate::init_logger;
    use log::info;
    use popcorn_fx_core::testing::{read_test_file_to_bytes, test_resource_filepath};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_session_find_torrent() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();
        let info_hash = info.info_hash.clone();
        let session = create_session(temp_path).await;

        let _ = session
            .add_torrent_from_info(info, TorrentFlags::default())
            .await
            .expect("expected the torrent to have been added");
        let result = session.find_torrent_by_info_hash(&info_hash).await;

        assert_ne!(None, result);
    }

    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_session_fetch_magnet() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=EADAF0EFEA39406914414D359E0EA16416409BD7&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let _torrent_with_metadata = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::Metadata,
            TorrentConfig::default()
        );

        let session = create_session(temp_path).await;

        let result = session
            .fetch_magnet(uri, Duration::from_secs(40))
            .await
            .unwrap();

        assert_ne!(
            None, result.info,
            "expected the metadata to have been present"
        );
    }

    #[tokio::test]
    async fn test_session_torrent_health_from_file() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();
        let session = create_session(temp_path).await;

        let result = session
            .torrent_health_from_info(&info)
            .await
            .expect("expected a torrent health");

        info!("Got torrent health result {:?}", result);
        assert_ne!(TorrentHealthState::Unknown, result.state);
        assert_ne!(0, result.seeds, "expected seeders to have been found");
    }

    #[tokio::test]
    async fn test_session_torrent_health_from_magnet() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();
        let info = TorrentMetadata::try_from(magnet).unwrap();
        let session = create_session(temp_path).await;

        let result = session
            .torrent_health_from_info(&info)
            .await
            .expect("expected a torrent health");

        info!("Got torrent health result {:?}", result);
        assert_ne!(TorrentHealthState::Unknown, result.state);
        assert_ne!(0, result.seeds, "expected seeders to have been found");
    }

    #[tokio::test]
    async fn test_session_resolve() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let session = create_session(temp_path).await;

        let filepath = test_resource_filepath("debian.torrent");
        let result = session
            .resolve(filepath.to_str().unwrap())
            .expect("expected the torrent info to have been resolved");
        let expected_info_hash =
            InfoHash::from_str("6D4795DEE70AEB88E03E5336CA7C9FCF0A1E206D").unwrap();
        assert_eq!(expected_info_hash, result.info_hash);

        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let result = session
            .resolve(uri)
            .expect("expected the torrent info to have been resolved");
        let expected_info_hash =
            InfoHash::from_str("EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        assert_eq!(expected_info_hash, result.info_hash);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_session_add_torrent() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();
        let (tx, mut rx) = unbounded_channel();
        let session = create_session(temp_path).await;

        let mut receiver = session.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let _ = tx.send((*event).clone());
            }
        });

        let torrent = session
            .add_torrent_from_info(info, TorrentFlags::none())
            .await
            .expect("expected a torrent handle");

        let event = select! {
            _ = tokio::time::sleep(Duration::from_millis(500)) => panic!("receive event timed out"),
            event = rx.recv() => event.unwrap(),
        };
        assert_eq!(event, SessionEvent::TorrentAdded(torrent.handle()));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_session_remove_torrent() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();
        let (tx, mut rx) = unbounded_channel();
        let session = create_session(temp_path).await;

        let mut receiver = session.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let _ = tx.send((*event).clone());
            }
        });
        let torrent = session
            .add_torrent_from_info(info, TorrentFlags::none())
            .await
            .expect("expected a torrent handle");
        let handle = torrent.handle();

        let event = timeout!(
            rx.recv(),
            Duration::from_millis(250),
            "expected to receive a session event"
        )
        .unwrap();
        assert_eq!(event, SessionEvent::TorrentAdded(handle));

        session.remove_torrent(&handle).await;

        let event = timeout!(
            rx.recv(),
            Duration::from_millis(250),
            "expected to receive a session event"
        )
        .unwrap();
        assert_eq!(event, SessionEvent::TorrentRemoved(handle));
    }

    async fn create_session(temp_path: &str) -> FxTorrentSession {
        let session = FxTorrentSession::builder()
            .base_path(temp_path)
            .client_name("test")
            .extensions(DEFAULT_TORRENT_EXTENSIONS())
            .build()
            .expect("expected a session to have been created");
        let (tx, rx) = oneshot::channel();

        let mut receiver = session.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let SessionEvent::StateChanged(state) = &*event {
                    tx.send(*state).unwrap();
                    return;
                }
            }
        });

        let state =
            timeout!(rx, Duration::from_millis(500)).expect("expected to receive a session state");
        if state != SessionState::Running {
            assert!(
                false,
                "expected the session to have been running, but got {:?} instead",
                state
            );
        }

        session
    }
}
