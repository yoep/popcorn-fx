use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace, warn};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

use popcorn_fx_core::available_port;
use popcorn_fx_core::core::torrents::magnet::Magnet;
use popcorn_fx_core::core::{CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle};

use crate::torrents::errors::Result;
use crate::torrents::peers::PeerListener;
use crate::torrents::torrent::Torrent;
use crate::torrents::{
    InfoHash, TorrentError, TorrentFlags, TorrentHandle, TorrentHealth, TorrentInfo, TorrentRequest,
};

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

    /// Returns the torrent health of the given torrent metadata.
    ///
    /// # Arguments
    ///
    /// * `torrent_info` - The metadata information of the torrent to check.
    ///
    /// # Returns
    ///
    /// Returns a result containing the torrent health on success or an error on failure.
    async fn torrent_health(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;

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
    pub async fn new(runtime: Arc<Runtime>) -> Result<Self> {
        trace!("Trying to create a new torrent session");
        let port = available_port!(6881, 6889).ok_or(TorrentError::Io(
            "no available port found to start new peer listener".to_string(),
        ))?;
        let peer_listener = PeerListener::new(port, runtime.clone()).await?;
        let inner_session = InnerSession::new(peer_listener).await?;

        debug!("Created new torrent session {}", inner_session.handle);
        Ok(Self {
            inner: Arc::new(inner_session),
            runtime,
        })
    }

    pub async fn torrent(&self, handle: TorrentHandle) -> Option<Torrent> {
        self.inner.retrieve_torrent(handle).await
    }

    pub async fn find_torrent(&self, info_hash: InfoHash) -> Option<Torrent> {
        todo!()
    }

    pub async fn fetch_magnet(&self, magnet_uri: &str, timeout: Duration) -> Result<TorrentInfo> {
        trace!("Trying to fetch magnet {}", magnet_uri);
        let magnet =
            Magnet::from_str(magnet_uri).map_err(|e| TorrentError::TorrentParse(e.to_string()))?;
        let torrent_info = TorrentInfo::try_from(magnet)?;
        let handle = self
            .internal_add_torrent(torrent_info, TorrentFlags::default(), false)
            .await?;

        if let Some(torrent) = self.inner.retrieve_torrent(handle).await {
            torrent.start_announcing().await?;
        }

        warn!("Handle {} has already been dropped", handle);
        return Err(TorrentError::InvalidHandle(handle));
    }

    async fn internal_add_torrent(
        &self,
        torrent_info: TorrentInfo,
        options: TorrentFlags,
        send_callback_event: bool,
    ) -> Result<TorrentHandle> {
        trace!(
            "Trying to add {:?} to session {}",
            torrent_info,
            self.inner.handle
        );
        let request = TorrentRequest {
            metadata: torrent_info,
            options,
            peer_listener_port: self.inner.peer_listener.port(),
            timeout: None,
            runtime: Some(self.runtime.clone()),
        };
        let torrent = Torrent::try_from(request)?;
        let handle = torrent.handle();
        let inner = self.inner.clone();

        inner.add_torrent(torrent, send_callback_event).await;
        self.runtime.spawn(async move {
            if let Some(torrent) = inner.retrieve_torrent(handle).await {
                match torrent.start_announcing().await {
                    Ok(_) => {}
                    Err(e) => error!("Failed to start announcing torrent {}, {}", handle, e),
                }
            } else {
                warn!(
                    "Unable to start torrent, {} has already been dropped",
                    handle
                );
            }
        });

        Ok(handle)
    }
}

#[async_trait]
impl Session for DefaultSession {
    fn handle(&self) -> SessionHandle {
        self.inner.handle
    }

    async fn torrent_health(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth> {
        trace!("Retrieving torrent health for {:?}", torrent_info);
        let request = TorrentRequest {
            metadata: torrent_info,
            options: TorrentFlags::None,
            peer_listener_port: self.inner.peer_listener.port(),
            timeout: Some(Duration::from_secs(3)),
            runtime: Some(self.runtime.clone()),
        };
        let torrent = Torrent::try_from(request)?;

        let announcement = torrent.announce().await?;

        debug!(
            "Converting announcement to torrent health for {:?}",
            announcement
        );
        Ok(TorrentHealth::from(&announcement))
    }

    async fn add_torrent(
        &self,
        torrent_info: TorrentInfo,
        options: TorrentFlags,
    ) -> Result<TorrentHandle> {
        self.internal_add_torrent(torrent_info, options, true).await
    }

    fn remove_torrent(&self, handle: TorrentHandle) {
        let inner = self.inner.clone();
        self.runtime
            .spawn(async move { inner.remove_torrent(handle).await });
    }
}

impl Callbacks<SessionEvent> for DefaultSession {
    fn add(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
    }
}

#[derive(Debug)]
struct InnerSession {
    /// The unique session identifier
    handle: SessionHandle,
    torrents: RwLock<Vec<Torrent>>,
    peer_listener: PeerListener,
    callbacks: CoreCallbacks<SessionEvent>,
}

impl InnerSession {
    async fn new(peer_listener: PeerListener) -> Result<Self> {
        Ok(Self {
            handle: Default::default(),
            peer_listener,
            torrents: Default::default(),
            callbacks: Default::default(),
        })
    }

    async fn retrieve_torrent(&self, handle: TorrentHandle) -> Option<Torrent> {
        let mutex = self.torrents.read().await;
        mutex.iter().find(|e| e.handle() == handle).cloned()
    }

    async fn add_torrent(&self, torrent: Torrent, send_callback_event: bool) {
        let handle = torrent.handle();

        {
            let mut mutex = self.torrents.write().await;
            debug!("Adding torrent {}", handle);
            mutex.push(torrent);
        }

        if send_callback_event {
            self.callbacks.invoke(SessionEvent::TorrentAdded(handle));
        }
    }

    async fn remove_torrent(&self, handle: TorrentHandle) {
        let mut event_handle: Option<TorrentHandle> = None;

        {
            let mut mutex = self.torrents.write().await;
            let position = mutex.iter().position(|e| e.handle() == handle);
            if let Some(position) = position {
                debug!("Removing torrent {}", handle);
                event_handle = Some(mutex.remove(position).handle());
            }
        }

        if let Some(handle) = event_handle {
            self.callbacks.invoke(SessionEvent::TorrentRemoved(handle));
        }
    }
}

impl Callbacks<SessionEvent> for InnerSession {
    fn add(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use log::info;
    use mockall::mock;

    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use crate::torrents::TorrentHealthState;

    use super::*;

    mock! {
        #[derive(Debug)]
        pub Session {}

        #[async_trait]
        impl Session for Session {
            fn handle(&self) -> SessionHandle;
            async fn torrent_health(&self, torrent_info: TorrentInfo) -> Result<TorrentHealth>;
            async fn add_torrent(&self, torrent_info: TorrentInfo, options: TorrentFlags) -> Result<TorrentHandle>;
            fn remove_torrent(&self, handle: TorrentHandle);
        }

        impl Callbacks<SessionEvent> for Session {
            fn add(&self, callback: CoreCallback<SessionEvent>) -> CallbackHandle;
            fn remove(&self, handle: CallbackHandle);
        }
    }

    #[test]
    fn test_session_fetch_magnet() {
        init_logger();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let runtime = Arc::new(Runtime::new().unwrap());
        let session = runtime
            .block_on(DefaultSession::new(runtime.clone()))
            .unwrap();

        let result = runtime
            .block_on(session.fetch_magnet(uri, Duration::from_secs(10)))
            .unwrap();
    }

    #[test]
    fn test_session_torrent_health() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentInfo::try_from(data.as_slice()).unwrap();
        let session = runtime
            .block_on(DefaultSession::new(runtime.clone()))
            .unwrap();

        let result = runtime
            .block_on(session.torrent_health(info))
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
            .block_on(DefaultSession::new(runtime.clone()))
            .unwrap();

        session.add(Box::new(move |event| {
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
            .block_on(DefaultSession::new(runtime.clone()))
            .unwrap();

        session.add(Box::new(move |event| {
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
