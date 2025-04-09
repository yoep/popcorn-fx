use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use rupnp::http::uri::InvalidUri;
use rupnp::Device;
use ssdp_client::{Error, SearchResponse, SearchTarget, URN};
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::SubtitleServer;

use crate::dlna::{errors, DlnaError, DlnaPlayer};
use crate::{Discovery, DiscoveryState};

pub(crate) const SSDP_QUERY_URN: URN = URN::device("schemas-upnp-org", "MediaRenderer", 1);
pub(crate) const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);
const DEFAULT_INTERVAL_SECONDS: u64 = 120;

/// Represents a DLNA discovery service responsible for discovering DLNA devices within the local network.
#[derive(Display, Clone)]
#[display(fmt = "DLNA device discovery")]
pub struct DlnaDiscovery {
    inner: Arc<InnerDlnaDiscovery>,
}

impl DlnaDiscovery {
    /// Creates a new `DlnaDiscoveryBuilder` to build a `DlnaDiscovery` instance.
    pub fn builder() -> DlnaDiscoveryBuilder {
        DlnaDiscoveryBuilder::builder()
    }
}

#[async_trait]
impl Discovery for DlnaDiscovery {
    async fn state(&self) -> DiscoveryState {
        self.inner.state().await
    }

    async fn start_discovery(&self) -> crate::Result<()> {
        let state = self.inner.state().await;

        if state != DiscoveryState::Running {
            debug!("Starting DLNA devices discovery");
            let inner = self.inner.clone();
            tokio::spawn(async move {
                inner.update_state(DiscoveryState::Running).await;
                let mut interval = interval(Duration::from_secs(inner.interval_seconds));
                loop {
                    select! {
                        _ = inner.cancellation_token.cancelled() => break,
                        _ = interval.tick() => {
                            if let Err(e) = inner.execute_search().await {
                                error!("Failed to discover DLNA devices, {}", e);
                            }
                        }
                    }
                }
                inner.update_state(DiscoveryState::Stopped).await;
                debug!("DLNA device discovery stopped");
            });

            Ok(())
        } else {
            Err(crate::DiscoveryError::InvalidState(state))
        }
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        if !self.inner.cancellation_token.is_cancelled() {
            trace!("Stopping DLNA devices discovery");
            self.inner.cancellation_token.cancel();
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
    interval_seconds: Option<u64>,
}

impl DlnaDiscoveryBuilder {
    /// Creates a new instance of the builder.
    pub fn builder() -> Self {
        Self::default()
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
                cancellation_token: Default::default(),
            }),
        }
    }
}

struct InnerDlnaDiscovery {
    interval_seconds: u64,
    player_manager: Arc<Box<dyn PlayerManager>>,
    discovered_devices: Mutex<Vec<String>>,
    subtitle_server: Arc<SubtitleServer>,
    state: Mutex<DiscoveryState>,
    cancellation_token: CancellationToken,
}

impl InnerDlnaDiscovery {
    async fn state(&self) -> DiscoveryState {
        *self.state.lock().await
    }

    async fn update_state(&self, state: DiscoveryState) {
        let mut mutex = self.state.lock().await;
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
            if let Err(e) = self.player_manager.add_player(Box::new(player)) {
                warn!("Failed to add player to DLNA player, {}", e);
            } else {
                info!("Registered new DLNA player {}", name);
            }
        } else {
            info!("DLNA device {} doesn't support AV transport service", name)
        }

        let mut mutex = self.discovered_devices.lock().await;
        mutex.push(device_url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlna::tests::{MockUdpServer, DEFAULT_SSDP_DESCRIPTION_RESPONSE};
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use popcorn_fx_core::core::players::{MockPlayerManager, Player};
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::{assert_timeout, init_logger};
    use std::sync::mpsc::channel;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_state() {
        init_logger!();
        let player_manager = MockPlayerManager::new();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let server = DlnaDiscovery::builder()
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = server.state().await;

        assert_eq!(DiscoveryState::Stopped, result);
    }

    // FIXME: timeout in Github Actions
    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_execute_search() {
        init_logger!();
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

            Ok(())
        });
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let _dlna_server = MockUdpServer::new()
            .device_name("test")
            .upnp_server_addr(server.address().clone())
            .build();
        let server = DlnaDiscovery::builder()
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = server.inner.execute_search().await;
        assert_eq!(false, result.is_err(), "expected no error");

        let player = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!("test", player.name());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_stop_discovery() {
        init_logger!();
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player().returning(|_| Ok(()));
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let server = DlnaDiscovery::builder()
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = server.start_discovery().await;
        assert_eq!(
            true,
            result.is_ok(),
            "expected the server to have been started"
        );
        assert_timeout!(
            Duration::from_millis(200),
            DiscoveryState::Running == server.inner.state().await
        );

        server.stop_discovery().unwrap();
        assert_eq!(
            true,
            server.inner.cancellation_token.is_cancelled(),
            "server should be stopped"
        );
        assert_timeout!(
            Duration::from_secs(10),
            DiscoveryState::Stopped == server.inner.state().await,
            "expected the discovery server to have been stopped"
        );
    }
}
