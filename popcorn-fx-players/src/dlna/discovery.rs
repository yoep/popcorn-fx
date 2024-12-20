use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use rupnp::Device;
use rupnp::http::uri::InvalidUri;
use ssdp_client::{Error, SearchResponse, SearchTarget, URN};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time;
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::block_in_place;
use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::SubtitleServer;

use crate::{Discovery, DiscoveryState};
use crate::dlna::{DlnaError, DlnaPlayer, errors};

pub(crate) const SSDP_QUERY_URN: URN = URN::device("schemas-upnp-org", "MediaRenderer", 1);
pub(crate) const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);
const DEFAULT_INTERVAL_SECONDS: u64 = 120;

/// Represents a DLNA discovery service responsible for discovering DLNA devices within the local network.
#[derive(Display)]
#[display(fmt = "DLNA device discovery")]
pub struct DlnaDiscovery {
    inner: Arc<InnerDlnaDiscovery>,
    runtime: Arc<Runtime>,
}

impl DlnaDiscovery {
    /// Creates a new `DlnaDiscoveryBuilder` to build a `DlnaDiscovery` instance.
    pub fn builder() -> DlnaDiscoveryBuilder {
        DlnaDiscoveryBuilder::builder()
    }
}

#[async_trait]
impl Discovery for DlnaDiscovery {
    fn state(&self) -> DiscoveryState {
        self.inner.state()
    }

    async fn start_discovery(&self) -> crate::Result<()> {
        let state = self.inner.state();

        if state != DiscoveryState::Running {
            debug!("Starting DLNA devices discovery");
            let inner = self.inner.clone();
            self.runtime.spawn(async move {
                inner.update_state(DiscoveryState::Running);
                loop {
                    if inner.cancel_token.is_cancelled() {
                        break;
                    }

                    if let Err(e) = inner.execute_search().await {
                        error!("Failed to discover DLNA devices, {}", e);
                    }

                    if inner.cancel_token.is_cancelled() {
                        break;
                    }
                    time::sleep(Duration::from_secs(inner.interval_seconds)).await;
                }
                inner.update_state(DiscoveryState::Stopped);
            });

            Ok(())
        } else {
            Err(crate::DiscoveryError::InvalidState(state))
        }
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        let state = self.inner.state();

        if state == DiscoveryState::Running && !self.inner.cancel_token.is_cancelled() {
            trace!("Stopping DLNA devices discovery");
            self.inner.cancel_token.cancel();
        }

        Ok(())
    }
}

impl Drop for DlnaDiscovery {
    fn drop(&mut self) {
        let _ = self.stop_discovery();
    }
}

/// Builder for configuring DLNA discovery.
#[derive(Debug, Default)]
pub struct DlnaDiscoveryBuilder {
    player_manager: Option<Arc<Box<dyn PlayerManager>>>,
    subtitle_server: Option<Arc<SubtitleServer>>,
    runtime: Option<Arc<Runtime>>,
    interval_seconds: Option<u64>,
}

impl DlnaDiscoveryBuilder {
    /// Creates a new instance of the builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the runtime for the DLNA discovery.
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Sets the interval between DLNA discovery checks, in seconds.
    pub fn interval_seconds(mut self, interval_seconds: u64) -> Self {
        self.interval_seconds = Some(interval_seconds);
        self
    }

    /// Sets the player manager for the DLNA discovery.
    pub fn player_manager(mut self, player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        self.player_manager = Some(player_manager);
        self
    }

    /// Sets the subtitle server for the DLNA discovery.
    pub fn subtitle_server(mut self, subtitle_server: Arc<SubtitleServer>) -> Self {
        self.subtitle_server = Some(subtitle_server);
        self
    }

    /// Builds the DLNA discovery instance.
    ///
    /// # Panics
    ///
    /// Panics if the player manager is not set.
    pub fn build(self) -> DlnaDiscovery {
        let runtime = self
            .runtime
            .unwrap_or_else(|| Arc::new(Runtime::new().expect("expected a valid runtime")));
        let interval_seconds = self.interval_seconds.unwrap_or(DEFAULT_INTERVAL_SECONDS);

        DlnaDiscovery {
            inner: Arc::new(InnerDlnaDiscovery {
                player_manager: self
                    .player_manager
                    .expect("expected a player manager to have been set"),
                interval_seconds,
                discovered_devices: Default::default(),
                subtitle_server: self
                    .subtitle_server
                    .expect("expected a subtitle server to have been set"),
                state: Mutex::new(DiscoveryState::Stopped),
                cancel_token: Default::default(),
            }),
            runtime,
        }
    }
}

struct InnerDlnaDiscovery {
    interval_seconds: u64,
    player_manager: Arc<Box<dyn PlayerManager>>,
    discovered_devices: Mutex<Vec<String>>,
    subtitle_server: Arc<SubtitleServer>,
    state: Mutex<DiscoveryState>,
    cancel_token: CancellationToken,
}

impl InnerDlnaDiscovery {
    fn state(&self) -> DiscoveryState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    fn update_state(&self, state: DiscoveryState) {
        let mut mutex = block_in_place(self.state.lock());
        trace!("Updating DLNA server state to {:?}", state);
        *mutex = state.clone();
        info!("DLNA discovery state changed to {}", state);
    }

    async fn execute_search(&self) -> Result<(), DlnaError> {
        let search_target = SearchTarget::URN(SSDP_QUERY_URN);

        debug!(
            "Executing DLNA devices search with target {}",
            search_target.to_string()
        );
        let mut responses = ssdp_client::search(
            &search_target,
            Duration::from_secs(self.interval_seconds),
            3,
            Some(4),
        )
        .await
        .map_err(|e| DlnaError::Discovery(e.to_string()))?;

        trace!("Received DLNA device responses");
        while let Some(response) = responses.next().await {
            if let Err(e) = self.handle_response(response).await {
                warn!("Failed to handle DLNA device response, {}", e);
            }
        }

        Ok(())
    }

    async fn handle_response(&self, response: Result<SearchResponse, Error>) -> errors::Result<()> {
        trace!("Received DLNA response {:?}", response);
        let uri = response
            .map_err(|e| DlnaError::Device(e.to_string()))
            .and_then(|e| {
                e.location()
                    .parse()
                    .map_err(|err: InvalidUri| DlnaError::Uri(err.to_string()))
            })?;
        debug!("Requesting DLNA device info from {}", uri);
        let device = Device::from_url(uri)
            .await
            .map_err(|e| DlnaError::Device(e.to_string()))?;

        if !self.is_already_discovered(&device).await {
            self.add_player(device).await
        } else {
            trace!(
                "DLNA device {} has already been discovered",
                device.friendly_name()
            );
        }

        Ok(())
    }

    async fn is_already_discovered(&self, device: &Device) -> bool {
        let mutex = self.discovered_devices.lock().await;
        mutex.contains(&device.url().to_string())
    }

    async fn add_player(&self, device: Device) {
        let name = device.friendly_name().to_string();
        let device_url = device.url().to_string();

        if let Some(service) = device.find_service(&AV_TRANSPORT).cloned() {
            trace!("Creating new player from {:?}", device);
            let player = DlnaPlayer::new(device, service, self.subtitle_server.clone());

            trace!("Adding new DLNA player {:?}", player);
            self.player_manager.add_player(Box::new(player));
            info!("Registered new DLNA player {}", name);
        } else {
            info!("DLNA device {} doesn't support AV transport service", name)
        }

        let mut mutex = self.discovered_devices.lock().await;
        mutex.push(device_url);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use httpmock::Method::GET;
    use httpmock::MockServer;

    use popcorn_fx_core::assert_timeout;
    use popcorn_fx_core::core::players::{MockPlayerManager, Player};
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::testing::init_logger;

    use crate::dlna::tests::{DEFAULT_SSDP_DESCRIPTION_RESPONSE, MockUdpServer};

    use super::*;

    #[test]
    fn test_state() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let player_manager = MockPlayerManager::new();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let server = DlnaDiscovery::builder()
            .runtime(runtime.clone())
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = server.state();

        assert_eq!(DiscoveryState::Stopped, result);
    }

    #[test]
    fn test_execute_search() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/description.xml");
            then.status(200)
                .header("Content-Type", "text/xml")
                .body(DEFAULT_SSDP_DESCRIPTION_RESPONSE);
        });
        let (tx, rx) = channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player().returning(move |e| {
            if let Ok(player) = e.downcast::<DlnaPlayer>() {
                if player.name() == "test" {
                    tx.send(player).unwrap();
                }
            }

            true
        });
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let _dlna_server = MockUdpServer::new()
            .runtime(runtime.clone())
            .device_name("test")
            .upnp_server_addr(server.address().clone())
            .build();
        let server = DlnaDiscovery::builder()
            .runtime(runtime.clone())
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = runtime.block_on(server.inner.execute_search());
        assert_eq!(false, result.is_err(), "expected no error");

        let player = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!("test", player.name());
    }

    #[test]
    fn test_stop_discovery() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player().return_const(true);
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let server = DlnaDiscovery::builder()
            .runtime(runtime.clone())
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = runtime.block_on(server.start_discovery());
        assert_eq!(
            true,
            result.is_ok(),
            "expected the server to have been started"
        );
        assert_timeout!(
            Duration::from_millis(200),
            DiscoveryState::Running == server.inner.state()
        );

        server.stop_discovery().unwrap();
        assert_eq!(
            true,
            server.inner.cancel_token.is_cancelled(),
            "server should be stopped"
        );
        assert_timeout!(
            Duration::from_millis(1500),
            DiscoveryState::Stopped == server.inner.state()
        );
    }
}
