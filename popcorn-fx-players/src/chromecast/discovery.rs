use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use log::{debug, info, trace, warn};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use popcorn_fx_core::core::block_in_place;
use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::SubtitleServer;

use crate::{chromecast, Discovery, DiscoveryError, DiscoveryState};
use crate::chromecast::device::DefaultCastDevice;
use crate::chromecast::player::ChromecastPlayer;
use crate::chromecast::transcode::{NoOpTranscoder, Transcoder};
#[cfg(feature = "transcoder")]
use crate::chromecast::transcode::VlcTranscoderDiscovery;

pub(crate) const SERVICE_TYPE: &str = "_googlecast._tcp.local.";
const INFO_UNKNOWN: &str = "Unknown";

#[derive(Display)]
#[display(fmt = "Chromecast device discovery")]
pub struct ChromecastDiscovery {
    inner: Arc<InnerChromecastDiscovery>,
}

impl ChromecastDiscovery {
    pub fn builder() -> ChromecastDiscoveryBuilder {
        ChromecastDiscoveryBuilder::builder()
    }

    pub fn new(
        service_daemon: ServiceDaemon,
        player_manager: Arc<Box<dyn PlayerManager>>,
        subtitle_server: Arc<SubtitleServer>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let transcoder = Arc::new(Self::resolve_transcoder());

        Self {
            inner: Arc::new(InnerChromecastDiscovery {
                player_manager,
                service_daemon,
                transcoder,
                subtitle_server,
                discovered_devices: Default::default(),
                state: Mutex::new(DiscoveryState::Stopped),
                runtime,
            }),
        }
    }

    #[cfg(feature = "transcoder")]
    fn resolve_transcoder() -> Box<dyn Transcoder> {
        VlcTranscoderDiscovery::discover()
            .map(|e| {
                info!("Using VLC transcoder for Chromecast devices");
                Box::new(e) as Box<dyn Transcoder>
            })
            .unwrap_or_else(|| {
                info!("VLC transcoder not found. Using no-op transcoder for Chromecast devices");
                Box::new(NoOpTranscoder {})
            })
    }

    #[cfg(not(feature = "transcoder"))]
    fn resolve_transcoder() -> Box<dyn Transcoder> {
        Box::new(NoOpTranscoder {})
    }
}

#[async_trait]
impl Discovery for ChromecastDiscovery {
    fn state(&self) -> DiscoveryState {
        let mutex = block_in_place(self.inner.state.lock());
        mutex.clone()
    }

    async fn start_discovery(&self) -> crate::Result<()> {
        let state: DiscoveryState;

        {
            let mutex = self.inner.state.lock().await;
            state = mutex.clone();
        }

        if state != DiscoveryState::Running {
            trace!("Starting Chromecast device discovery");
            let receiver = self
                .inner
                .service_daemon
                .browse(SERVICE_TYPE)
                .map_err(|e| DiscoveryError::Initialization(e.to_string()))?;

            self.inner.update_state_async(DiscoveryState::Running).await;
            let inner = self.inner.clone();
            self.inner.runtime.spawn(async move {
                while let Ok(event) = receiver.recv() {
                    inner.handle_event(event).await;
                }
            });
        } else {
            return Err(DiscoveryError::InvalidState(state));
        }

        Ok(())
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        let state: DiscoveryState;

        {
            let mutex = block_in_place(self.inner.state.lock());
            state = mutex.clone();
        }

        if state == DiscoveryState::Running {
            self.inner
                .service_daemon
                .stop_browse(SERVICE_TYPE)
                .map_err(|e| DiscoveryError::Terminate(e.to_string()))?;
            block_in_place(self.inner.update_state_async(DiscoveryState::Stopped));
        } else {
            trace!("Unable to stop Chromecast discovery because it is not running");
        }

        Ok(())
    }
}

impl Drop for ChromecastDiscovery {
    fn drop(&mut self) {
        let _ = self.stop_discovery();
    }
}

/// A builder struct for creating a `ChromecastDiscovery` instance.
/// This struct provides fluent methods for setting optional components and a method for building the final `ChromecastDiscovery`.
#[derive(Debug, Default)]
pub struct ChromecastDiscoveryBuilder {
    player_manager: Option<Arc<Box<dyn PlayerManager>>>,
    subtitle_server: Option<Arc<SubtitleServer>>,
    runtime: Option<Arc<Runtime>>,
}

impl ChromecastDiscoveryBuilder {
    /// Creates a new instance of the builder.
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn player_manager(mut self, player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        self.player_manager = Some(player_manager);
        self
    }

    pub fn subtitle_server(mut self, subtitle_server: Arc<SubtitleServer>) -> Self {
        self.subtitle_server = Some(subtitle_server);
        self
    }

    pub fn build(self) -> ChromecastDiscovery {
        let runtime = self
            .runtime
            .unwrap_or_else(|| Arc::new(tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("chromecast-discovery")
                .build()
                .expect("expected a new runtime")));
        let service_daemon = ServiceDaemon::new().expect("Failed to create daemon");

        ChromecastDiscovery::new(
            service_daemon,
            self.player_manager.expect("expected a player manager to have been set"),
            self.subtitle_server.expect("expected a subtitle server to have been set"),
            runtime,
        )
    }
}

struct InnerChromecastDiscovery {
    player_manager: Arc<Box<dyn PlayerManager>>,
    service_daemon: ServiceDaemon,
    transcoder: Arc<Box<dyn Transcoder>>,
    subtitle_server: Arc<SubtitleServer>,
    discovered_devices: Mutex<Vec<String>>,
    state: Mutex<DiscoveryState>,
    runtime: Arc<Runtime>,
}

impl InnerChromecastDiscovery {
    async fn update_state_async(&self, state: DiscoveryState) {
        let mut mutex = block_in_place(self.state.lock());
        debug!("Updating Chromecast discovery state to {:?}", state);
        *mutex = state.clone();
        info!("Chromecast discovery state changed to {:?}", state);
    }

    async fn handle_event(&self, event: ServiceEvent) {
        if let ServiceEvent::ServiceResolved(info) = event {
            trace!("Discovered Chromecast device: {:?}", info);
            if let Some(addr) = info
                .get_addresses()
                .into_iter()
                .find_or_first(|e| e.is_ipv4())
                .map(|e| e.to_string())
            {
                let mut mutex = self.discovered_devices.lock().await;
                let id = info.get_fullname().to_string();
                let port = info.get_port();

                if !mutex.contains(&id) {
                    match self.register_device(info, addr, port).await {
                        Ok(_) => mutex.push(id),
                        Err(e) => warn!("Failed to connect to Chromecast device: {}", e),
                    }
                } else {
                    trace!("Chromecast device {} is already known", id);
                }
            } else {
                warn!("Chromecast device {:?} has no available IPv4 address", info);
            }
        }
    }

    async fn register_device<S: Into<String>>(
        &self,
        info: ServiceInfo,
        addr: S,
        port: u16,
    ) -> chromecast::Result<()> {
        let device_id = info.get_fullname();
        let device_name = info.get_property_val_str("fn").unwrap_or(INFO_UNKNOWN);
        let device_model = info.get_property_val_str("md").unwrap_or(INFO_UNKNOWN);

        match ChromecastPlayer::<DefaultCastDevice>::builder()
            .id(device_id)
            .name(device_name)
            .cast_model(device_model)
            .cast_address(addr.into())
            .cast_port(port)
            .subtitle_server(self.subtitle_server.clone())
            .transcoder(self.transcoder.clone())
            .build().await
        {
            Ok(player) => {
                if !self.player_manager.add_player(Box::new(player)) {
                    warn!("Failed to add Chromecast player {:?}", info);
                }

                Ok(())
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use popcorn_fx_core::core::players::MockPlayerManager;
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::testing::init_logger;

    use crate::chromecast::tests::TestInstance;

    use super::*;

    #[test]
    fn test_state() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let player_manager = MockPlayerManager::new();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .runtime(runtime.clone())
            .subtitle_server(subtitle_server)
            .build();

        let result = discovery.state();

        assert_eq!(DiscoveryState::Stopped, result);
    }

    #[test]
    fn test_start_discovery() {
        init_logger();
        let (tx, rx) = channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player().returning(move |e| {
            debug!("Received player: {:?}", e);
            if e.name() == "Chromecast test device" {
                tx.send(e).unwrap();
            }
            true
        });
        let mut test_instance = TestInstance::new_mdns();
        let mdns = test_instance.mdns.take().unwrap();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .runtime(test_instance.runtime.clone())
            .subtitle_server(subtitle_server)
            .build();

        test_instance.runtime.block_on(discovery.start_discovery()).unwrap();
        let result = rx.recv_timeout(Duration::from_secs(2)).unwrap();

        assert_eq!("Chromecast test device", result.name());
        mdns.shutdown().unwrap();
    }

    #[test]
    fn test_stop_discovery() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let player_manager = MockPlayerManager::new();
        let mut test_instance = TestInstance::new_mdns();
        let mdns = test_instance.mdns.take().unwrap();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .runtime(runtime.clone())
            .subtitle_server(subtitle_server)
            .build();

        runtime.block_on(discovery.start_discovery()).unwrap();
        let result = discovery.stop_discovery();

        assert_eq!(Ok(()), result);
        assert_eq!(DiscoveryState::Stopped, discovery.state());
        mdns.shutdown().unwrap();
    }
}
