use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use log::{debug, info, trace, warn};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use popcorn_fx_core::core::players::PlayerManager;
use popcorn_fx_core::core::subtitles::SubtitleServer;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::chromecast::device::DefaultCastDevice;
use crate::chromecast::player::ChromecastPlayer;
#[cfg(feature = "transcoder")]
use crate::chromecast::transcode::VlcTranscoderDiscovery;
use crate::chromecast::transcode::{NoOpTranscoder, Transcoder};
use crate::{chromecast, Discovery, DiscoveryError, DiscoveryState};

pub(crate) const SERVICE_TYPE: &str = "_googlecast._tcp.local.";
const INFO_UNKNOWN: &str = "Unknown";

#[derive(Debug, Display)]
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
    ) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let transcoder = Arc::new(Self::resolve_transcoder());
        let inner = Arc::new(InnerChromecastDiscovery {
            player_manager,
            service_daemon,
            transcoder,
            subtitle_server,
            discovered_devices: Default::default(),
            state: Mutex::new(DiscoveryState::Stopped),
            command_sender,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver).await;
        });

        Self { inner }
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
    async fn state(&self) -> DiscoveryState {
        *self.inner.state.lock().await
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

            self.inner.update_state(DiscoveryState::Running).await;
            let inner = self.inner.clone();
            tokio::spawn(async move {
                trace!("Starting the Chromecast MDNS discovery service receiver");
                while let Ok(event) = receiver.recv_async().await {
                    inner.handle_event(event).await;
                }
                debug!("Chromecast device discovery receiver has been stopped");
            });
        } else {
            return Err(DiscoveryError::InvalidState(state));
        }

        Ok(())
    }

    fn stop_discovery(&self) -> crate::Result<()> {
        let _ = self
            .inner
            .command_sender
            .send(ChromecastDiscoveryCommand::Stop);
        Ok(())
    }
}

impl Drop for ChromecastDiscovery {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

/// A builder struct for creating a `ChromecastDiscovery` instance.
/// This struct provides fluent methods for setting optional components and a method for building the final `ChromecastDiscovery`.
#[derive(Debug, Default)]
pub struct ChromecastDiscoveryBuilder {
    player_manager: Option<Arc<Box<dyn PlayerManager>>>,
    subtitle_server: Option<Arc<SubtitleServer>>,
}

impl ChromecastDiscoveryBuilder {
    /// Creates a new instance of the builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the player manager to register new discovered instances.
    pub fn player_manager(mut self, player_manager: Arc<Box<dyn PlayerManager>>) -> Self {
        self.player_manager = Some(player_manager);
        self
    }

    /// Set the subtitle server to use for providing subtitles to the player.
    pub fn subtitle_server(mut self, subtitle_server: Arc<SubtitleServer>) -> Self {
        self.subtitle_server = Some(subtitle_server);
        self
    }

    /// Build a new chromecast device discovery instance.
    ///
    /// # Panics
    ///
    /// This function panics when the underlying service daemon couldn't be created.
    pub fn build(self) -> ChromecastDiscovery {
        let service_daemon = ServiceDaemon::new().expect("Failed to create daemon");

        ChromecastDiscovery::new(
            service_daemon,
            self.player_manager
                .expect("expected a player manager to have been set"),
            self.subtitle_server
                .expect("expected a subtitle server to have been set"),
        )
    }
}

#[derive(Debug, PartialEq)]
enum ChromecastDiscoveryCommand {
    Stop,
}

struct InnerChromecastDiscovery {
    player_manager: Arc<Box<dyn PlayerManager>>,
    service_daemon: ServiceDaemon,
    transcoder: Arc<Box<dyn Transcoder>>,
    subtitle_server: Arc<SubtitleServer>,
    discovered_devices: Mutex<Vec<String>>,
    state: Mutex<DiscoveryState>,
    command_sender: UnboundedSender<ChromecastDiscoveryCommand>,
    cancellation_token: CancellationToken,
}

impl InnerChromecastDiscovery {
    async fn start(&self, mut command_receiver: UnboundedReceiver<ChromecastDiscoveryCommand>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
            }
        }
        self.stop_discovery().await;
        debug!("Chromecast discovery main loop ended");
    }

    async fn handle_command(&self, command: ChromecastDiscoveryCommand) {
        match command {
            ChromecastDiscoveryCommand::Stop => self.stop_discovery().await,
        }
    }

    async fn stop_discovery(&self) {
        trace!("Stopping Chromecast device discovery");
        let state: DiscoveryState;

        {
            let mutex = self.state.lock().await;
            state = mutex.clone();
        }

        if state == DiscoveryState::Running {
            trace!("Stopping the Chromecast MDNS discovery service");
            let _ = self
                .service_daemon
                .stop_browse(SERVICE_TYPE)
                .map_err(|e| DiscoveryError::Terminate(e.to_string()));
            self.update_state(DiscoveryState::Stopped).await;
            debug!("Chromecast device discovery has been stopped");
        } else {
            trace!("Unable to stop Chromecast discovery because it is not running");
        }
    }

    async fn update_state(&self, state: DiscoveryState) {
        let mut mutex = self.state.lock().await;
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
            .cast_device_factory(Box::new(|addr, port| DefaultCastDevice::new(addr, port)))
            .build()
        {
            Ok(player) => {
                if let Err(e) = self.player_manager.add_player(Box::new(player)) {
                    warn!("Failed to add Chromecast player {:?}, {}", info, e);
                }

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl Debug for InnerChromecastDiscovery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromecastDiscovery")
            .field("player_manager", &self.player_manager)
            .field("transcoder", &self.transcoder)
            .field("subtitle_server", &self.subtitle_server)
            .field("discovered_devices", &self.discovered_devices)
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::core::players::MockPlayerManager;
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::{assert_timeout, init_logger, recv_timeout};
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::chromecast::tests::TestInstance;

    use super::*;

    #[tokio::test]
    async fn test_state() {
        init_logger!();
        let player_manager = MockPlayerManager::new();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        let result = discovery.state().await;

        assert_eq!(DiscoveryState::Stopped, result);
    }

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_start_discovery() {
        init_logger!();
        let mut player_buf = vec![];
        let (tx, mut rx) = unbounded_channel();
        let mut player_manager = MockPlayerManager::new();
        player_manager.expect_add_player().returning(move |e| {
            debug!("--- Received Chromecast player: {:?}", e);
            if e.name() == "Chromecast test device" {
                tx.send(e).unwrap();
            } else {
                player_buf.push(e);
            }
            Ok(())
        });
        let mut test_instance = TestInstance::new_mdns().await;
        let mdns = test_instance.mdns.take().unwrap();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        discovery.start_discovery().await.unwrap();

        let result = recv_timeout!(&mut rx, Duration::from_secs(3));
        info!("--- Chromecast player received: {:?}", result);
        discovery.stop_discovery().unwrap();

        assert_eq!("Chromecast test device", result.name());
        mdns.daemon.shutdown().unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_stop_discovery() {
        init_logger!();
        let player_manager = MockPlayerManager::new();
        let mut test_instance = TestInstance::new_mdns().await;
        let mdns = test_instance.mdns.take().unwrap();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let discovery = ChromecastDiscovery::builder()
            .player_manager(Arc::new(Box::new(player_manager)))
            .subtitle_server(subtitle_server)
            .build();

        discovery.start_discovery().await.unwrap();
        let result = discovery.stop_discovery();

        assert_eq!(Ok(()), result);
        assert_timeout!(
            Duration::from_millis(200),
            discovery.state().await == DiscoveryState::Stopped,
            "expected the discovery to have been stopped"
        );
        mdns.daemon.shutdown().unwrap();
    }
}
