use std::sync::Arc;
use std::time::Duration;

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

use crate::dlna::{DlnaError, DlnaPlayer, errors};

const SSDP_QUERY_URN: URN = URN::device("schemas-upnp-org", "MediaRenderer", 1);
const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);
const DEFAULT_INTERVAL_SECONDS: u64 = 30;

/// Represents the state of a DLNA server.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum DlnaServerState {
    /// The DLNA server is running.
    Running,
    /// The DLNA server is stopped.
    Stopped,
    /// An error occurred with the DLNA server.
    Error,
}

/// Represents a DLNA server responsible for discovering DLNA devices.
pub struct DlnaServer {
    inner: Arc<InnerDlnaServer>,
    runtime: Arc<Runtime>,
}

impl DlnaServer {
    /// Creates a new `DlnaServerBuilder` to build a `DlnaServer` instance.
    pub fn builder() -> DlnaServerBuilder {
        DlnaServerBuilder::builder()
    }

    /// Starts the DLNA devices discovery process.
    pub fn start_discovery(&self) -> errors::Result<()> {
        let state = self.inner.state();

        if state != DlnaServerState::Running {
            debug!("Starting DLNA devices discovery");
            let inner = self.inner.clone();
            self.runtime.spawn(async move {
                inner.update_state(DlnaServerState::Running);
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
                inner.update_state(DlnaServerState::Stopped);
            });

            Ok(())
        } else {
            Err(DlnaError::InvalidState(state))
        }
    }

    /// Stops the DLNA devices discovery process.
    pub fn stop_discovery(&self) {
        let state = self.inner.state();

        if state == DlnaServerState::Running && !self.inner.cancel_token.is_cancelled() {
            trace!("Stopping DLNA devices discovery");
            self.inner.cancel_token.cancel()
        }
    }
}

impl Drop for DlnaServer {
    fn drop(&mut self) {
        self.stop_discovery()
    }
}

#[derive(Debug, Default)]
pub struct DlnaServerBuilder {
    player_manager: Option<Arc<Box<dyn PlayerManager>>>,
    runtime: Option<Arc<Runtime>>,
    interval_seconds: Option<u64>,
}

impl DlnaServerBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn interval_seconds(mut self, interval_seconds: u64) -> Self {
        self.interval_seconds = Some(interval_seconds);
        self
    }

    pub fn player_manager(mut self, player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        self.player_manager = Some(player_manager);
        self
    }

    pub fn build(self) -> DlnaServer {
        let runtime = self.runtime.unwrap_or_else(|| Arc::new(Runtime::new().expect("expected a valid runtime")));
        let interval_seconds = self.interval_seconds.unwrap_or(DEFAULT_INTERVAL_SECONDS);

        DlnaServer {
            inner: Arc::new(InnerDlnaServer {
                player_manager: self.player_manager.expect("expected a player manager to have been set"),
                interval_seconds,
                state: Mutex::new(DlnaServerState::Stopped),
                cancel_token: Default::default(),
            }),
            runtime,
        }
    }
}

struct InnerDlnaServer {
    interval_seconds: u64,
    player_manager: Arc<Box<dyn PlayerManager>>,
    state: Mutex<DlnaServerState>,
    cancel_token: CancellationToken,
}

impl InnerDlnaServer {
    fn state(&self) -> DlnaServerState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    fn update_state(&self, state: DlnaServerState) {
        trace!("Updating DLNA server state to {:?}", state);
        let mut mutex = block_in_place(self.state.lock());
        *mutex = state.clone();
        info!("DLNA server state changed to {}", state);
    }

    async fn execute_search(&self) -> Result<(), DlnaError> {
        let search_target = SearchTarget::URN(SSDP_QUERY_URN);

        debug!("Executing DLNA devices search with target {}", search_target.to_string());
        let mut responses = ssdp_client::search(&search_target, Duration::from_secs(self.interval_seconds), 3, Some(4))
            .await
            .map_err(|e| {
                DlnaError::Discovery(e.to_string())
            })?;

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
            .and_then(|e| e.location().parse().map_err(|err: InvalidUri| {
                DlnaError::Uri(err.to_string())
            }))?;
        debug!("Requesting DLNA device info from {}", uri);
        let device = Device::from_url(uri).await
            .map_err(|e| DlnaError::Device(e.to_string()))?;
        self.add_player(device);

        Ok(())
    }

    fn add_player(&self, device: Device) {
        let name = device.friendly_name().to_string();
        if let Some(service) = device.find_service(&AV_TRANSPORT).cloned() {
            trace!("Creating new player from {:?}", device);
            let player = DlnaPlayer::new(device, service);

            trace!("Adding new DLNA player {:?}", player);
            self.player_manager.add_player(Box::new(player));
            info!("Registered new DLNA player {}", name);
        } else {
            debug!("Device {} doesn't support AV data", name)
        }
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::assert_timeout;
    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::core::players::MockPlayerManager;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_execute_search() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let player_manager = Arc::new(Box::new(MockPlayerManager::new()) as Box<dyn PlayerManager>);
        let server = DlnaServer::builder()
            .runtime(runtime.clone())
            .interval_seconds(1)
            .player_manager(player_manager)
            .build();

        let result = block_in_place(server.inner.execute_search());

        assert_eq!(false, result.is_err(), "expected no error");
    }

    #[test]
    fn test_stop_discovery() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player()
            .return_const(true);
        let server = DlnaServer::builder()
            .runtime(runtime.clone())
            .interval_seconds(1)
            .player_manager(Arc::new(Box::new(player_manager)))
            .build();

        let result = server.start_discovery();
        assert_eq!(true, result.is_ok(), "expected the server to have been started");
        assert_timeout!(Duration::from_millis(200), DlnaServerState::Running == server.inner.state());

        server.stop_discovery();
        assert_eq!(true, server.inner.cancel_token.is_cancelled(), "server should be stopped");
        assert_timeout!(Duration::from_millis(1500), DlnaServerState::Stopped == server.inner.state());
    }
}