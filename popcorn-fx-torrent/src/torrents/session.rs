use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

use crate::torrents::errors::Result;
use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::PeerListener;
use crate::torrents::torrent::Torrent;
use crate::torrents::{
    InfoHash, TorrentError, TorrentEvent, TorrentFlags, TorrentHandle, TorrentInfo, TorrentRequest,
};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace, warn};
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

#[derive(Debug, Display, Clone, PartialEq)]
pub enum SessionEvent {
    #[display(fmt = "Torrent added: {}", _0)]
    TorrentAdded(TorrentHandle),
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

    /// Retrieve the torrent based on the given info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash of the torrent to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a weak reference to the torrent if found, else `None`.
    async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent>;

    /// Returns the torrent health of the given torrent metadata.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to check.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health_from_info(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;

    /// Returns the torrent health of the given torrent magnet link.
    ///
    /// # Arguments
    ///
    /// * `magnet_uri` - The magnet URI of the torrent to fetch the health of.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health_from_uri(&self, magnet_uri: &str) -> Result<TorrentHealth>;

    /// Retrieve the torrent information for the given magnet URI.
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

    /// Add a new torrent to this session for the given metadata information.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to add.
    /// * `options` - The torrent options to use when adding the torrent.
    ///
    /// # Returns
    ///
    /// Returns a result containing the handle of the newly added torrent on success or an error on failure.
    async fn add_torrent(
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
    pub async fn new(extensions: Extensions, runtime: Arc<Runtime>) -> Result<Self> {
        trace!("Trying to create a new torrent session");
        let port = available_port!(6881, 6889).ok_or(TorrentError::Io(
            "no available port found to start new peer listener".to_string(),
        ))?;
        let peer_listener = PeerListener::new(port, runtime.clone()).await?;
        let inner_session = InnerSession::new(peer_listener, extensions).await?;

        debug!("Created new torrent session {}", inner_session.handle);
        Ok(Self {
            inner: Arc::new(inner_session),
            runtime,
        })
    }

    pub async fn torrent(&self, handle: TorrentHandle) -> Option<Torrent> {
        self.inner.find_torrent_by_handle(handle).await
    }

    pub async fn find_torrent_by_handle(&self, handle: TorrentHandle) -> Option<Torrent> {
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
        let request = TorrentRequest {
            metadata: torrent_info,
            options,
            peer_listener_port: self.inner.peer_listener.port(),
            extensions: self.inner.extensions(),
            peer_timeout,
            tracker_timeout,
            runtime: Some(self.runtime.clone()),
        };
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
                    let request = TorrentRequest {
                        metadata: torrent_info,
                        options: TorrentFlags::None,
                        peer_listener_port: self.inner.peer_listener.port(),
                        extensions: self.inner.extensions(),
                        peer_timeout: Some(Duration::from_secs(3)),
                        tracker_timeout: Some(Duration::from_secs(2)),
                        runtime: Some(self.runtime.clone()),
                    };

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

        torrent.add_callback(Box::new(move |event| {
            tx.send(event).unwrap();
        }));

        // check if the metadata is already fetched
        let torrent_info = torrent.metadata().await?;
        if torrent_info.info.is_some() {
            return Ok(torrent_info);
        }

        // otherwise, wait for the MetadataChanged event
        select! {
            _ = time::sleep(timeout) => Err(TorrentError::Timeout),
            result = Self::wait_for_metadata(&torrent, rx, timeout) => result
        }
    }

    async fn add_torrent(
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

// TODO: add options which support configuring timeouts etc
#[derive(Debug)]
struct InnerSession {
    /// The unique session identifier
    handle: SessionHandle,
    torrents: RwLock<HashMap<InfoHash, Torrent>>,
    peer_listener: PeerListener,
    extensions: Extensions,
    callbacks: CoreCallbacks<SessionEvent>,
}

impl InnerSession {
    async fn new(peer_listener: PeerListener, extensions: Extensions) -> Result<Self> {
        Ok(Self {
            handle: Default::default(),
            peer_listener,
            extensions,
            torrents: Default::default(),
            callbacks: Default::default(),
        })
    }

    fn extensions(&self) -> Extensions {
        self.extensions.iter().map(|e| e.clone_box()).collect()
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
            debug!("Adding torrent {}", handle);
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
pub mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::torrents::peers::extensions::metadata::MetadataExtension;
    use log::info;
    use mockall::mock;
    use popcorn_fx_core::core::torrents::TorrentHealthState;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    mock! {
        #[derive(Debug)]
        pub Session {}

        #[async_trait]
        impl Session for Session {
            fn handle(&self) -> SessionHandle;
            async fn find_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Option<Torrent>;
            async fn torrent_health_from_info(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;
            async fn torrent_health_from_uri(&self, magnet_uri: &str) -> Result<TorrentHealth>;
            async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentInfo>;
            async fn add_torrent(&self, torrent_info: TorrentInfo, options: TorrentFlags) -> Result<TorrentHandle>;
            fn remove_torrent(&self, handle: TorrentHandle);
        }

        impl Callbacks<SessionEvent> for Session {
            fn add_callback(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle;
            fn remove_callback(&self, handle: CallbackHandle);
        }
    }

    #[test]
    fn test_find_torrent() {
        init_logger();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let info_hash = info.info_hash.clone();
        let runtime = Arc::new(Runtime::new().unwrap());
        let session = runtime
            .block_on(DefaultSession::new(
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            ))
            .unwrap();

        let _ = runtime
            .block_on(session.add_torrent(info, TorrentFlags::default()))
            .expect("expected the torrent to have been added");
        let result = runtime.block_on(session.find_torrent_by_info_hash(&info_hash));

        assert_ne!(None, result);
    }

    #[test]
    fn test_session_fetch_magnet() {
        init_logger();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let runtime = Arc::new(Runtime::new().unwrap());
        let session = runtime
            .block_on(DefaultSession::new(
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            ))
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
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let session = runtime
            .block_on(DefaultSession::new(vec![], runtime.clone()))
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
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let (tx, rx) = channel();
        let session = runtime
            .block_on(DefaultSession::new(vec![], runtime.clone()))
            .unwrap();

        session.add_callback(Box::new(move |event| {
            tx.send(event).unwrap();
        }));

        let handle = runtime
            .block_on(session.add_torrent(info, TorrentFlags::default()))
            .expect("expected a torrent handle");

        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentAdded(handle));
    }

    #[test]
    fn test_session_remove_torrent() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let (tx, rx) = channel();
        let session = runtime
            .block_on(DefaultSession::new(vec![], runtime.clone()))
            .unwrap();

        session.add_callback(Box::new(move |event| {
            tx.send(event).unwrap();
        }));
        let handle = runtime
            .block_on(session.add_torrent(info, TorrentFlags::default()))
            .expect("expected a torrent handle");

        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentAdded(handle));

        session.remove_torrent(handle);
        let event = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(event, SessionEvent::TorrentRemoved(handle));
    }
}
