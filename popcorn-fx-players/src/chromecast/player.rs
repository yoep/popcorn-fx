use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace, warn};
use rust_cast::{CastDevice, ChannelMessage, channels};
use rust_cast::channels::media::{MediaResponse, Status};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use tokio::{runtime, time};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};

use crate::chromecast;
use crate::chromecast::{ChromecastError, Image, LoadCommand, Media, Metadata, MovieMetadata, StreamType};

const GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-chromecast-icon.png");
const DESCRIPTION: &str = "Chromecast streaming media device which allows the playback of videos on your TV.";
const DEFAULT_HEARTBEAT_INTERVAL_MILLIS: u64 = 10 * 1000;
const MEDIA_CHANNEL_NAMESPACE: &str = "urn:x-cast:com.google.cast.media";

#[derive(Debug, Display)]
#[display(fmt = "Chromecast player {}", "self.name()")]
pub struct ChromecastPlayer {
    inner: Arc<InnerChromecastPlayer>,
}

impl ChromecastPlayer {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        cast_model: impl Into<String>,
        cast_address: impl Into<String>,
        cast_port: u16,
        heartbeat_millis: u64,
        runtime: Arc<Runtime>) -> chromecast::Result<Self> {
        let name = name.into();
        let cast_address = cast_address.into();

        trace!("Trying to establish connection with Chromecast device {} on {}:{}...", name, cast_address, cast_port);
        match CastDevice::connect_without_host_verification(cast_address.clone(), cast_port) {
            Ok(cast_device) => {
                debug!("Connected to Chromecast device {} on {}:{}", name, cast_address, cast_port);
                if let Err(e) = cast_device.connection.connect("receiver-0") {
                    return Err(ChromecastError::Connection(e.to_string()));
                }
                if let Err(e) = cast_device.heartbeat.ping() {
                    return Err(ChromecastError::Connection(e.to_string()));
                }

                let instance = Arc::new(InnerChromecastPlayer {
                    id: id.into(),
                    name,
                    cast_model: cast_model.into(),
                    cast_address,
                    cast_port,
                    request: Default::default(),
                    state: Mutex::new(PlayerState::Ready),
                    cast_device: Mutex::new(cast_device),
                    cast_app: Default::default(),
                    cast_media_session_id: Default::default(),
                    callbacks: Default::default(),
                    runtime,
                    status_check_token: Default::default(),
                    shutdown_token: Default::default(),
                });

                let inner = instance.clone();
                let cancellation_token = instance.shutdown_token.clone();
                instance.runtime.spawn(Self::start_heartbeat(inner, cancellation_token, heartbeat_millis));

                Ok(Self {
                    inner: instance
                })
            }
            Err(e) => Err(ChromecastError::Connection(e.to_string())),
        }
    }

    pub fn builder() -> ChromecastPlayerBuilder {
        ChromecastPlayerBuilder::builder()
    }

    async fn start_heartbeat(inner: Arc<InnerChromecastPlayer>, cancellation_token: CancellationToken, heartbeat_millis: u64) {
        loop {
            if cancellation_token.is_cancelled() {
                break;
            }

            let ping_result: Result<(), rust_cast::errors::Error>;

            {
                let mutex = inner.cast_device.lock().await;
                trace!("Sending Chromecast {} heartbeat", inner.name);
                ping_result = mutex.heartbeat.ping();
            }

            if let Err(e) = ping_result {
                warn!("Failed to ping Chromecast {}, {}", inner.name, e);
            }
            time::sleep(Duration::from_millis(heartbeat_millis)).await;
        }

        debug!("Chromecast {} heartbeat has been stopped", inner.name);
    }

    async fn start_status_updates(inner: Arc<InnerChromecastPlayer>, cancellation_token: CancellationToken) {
        loop {
            if cancellation_token.is_cancelled() {
                break;
            }

            match inner.status().await {
                Ok(e) => inner.handle_status_update(e).await,
                Err(e) => {
                    error!("Failed to retrieve chromecast status, {}", e);
                    break;
                }
            }
            time::sleep(Duration::from_secs(1)).await;
        }

        debug!("Chromecast {} status check has been stopped", inner.name);
    }
}

impl Callbacks<PlayerEvent> for ChromecastPlayer {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
    }
}

#[async_trait]
impl Player for ChromecastPlayer {
    fn id(&self) -> &str {
        self.inner.id.as_str()
    }

    fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn graphic_resource(&self) -> Vec<u8> {
        GRAPHIC_RESOURCE.to_vec()
    }

    fn state(&self) -> PlayerState {
        self.inner.state()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = block_in_place(self.inner.request.lock());
        mutex.as_ref()
            .map(|e| Arc::downgrade(e))
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        trace!("Starting Chromecast {} playback for {:?}", self.name(), self.request());
        self.inner.update_state_async(PlayerState::Loading).await;

        match self.inner.start_app().await {
            Ok(app) => {
                if let Err(e) = self.inner.connect().await {
                    error!("Failed to connect to Chromecast device, {}", e);
                    self.inner.update_state_async(PlayerState::Error).await;
                    return;
                }

                if let Err(e) = self.inner.load(&app, &request).await {
                    error!("Failed to load Chromecast media, {}", e);
                    self.inner.update_state_async(PlayerState::Error).await;
                    return;
                }

                debug!("Starting Chromecast {} playback", self.name());
                self.inner.resume().await;
                let token = self.inner.generate_status_token().await;
                self.inner.runtime.spawn(Self::start_status_updates(self.inner.clone(), token));

                {
                    trace!("Updating Chromecast player request to {:?}", request);
                    let mut mutex = self.inner.request.lock().await;
                    *mutex = Some(Arc::new(request))
                }
            }
            Err(e) => {
                error!("Failed to start Chromecast playback, {}", e);
                self.inner.update_state_async(PlayerState::Error).await;
            }
        }
    }

    fn pause(&self) {
        block_in_place(self.inner.pause())
    }

    fn resume(&self) {
        block_in_place(self.inner.resume())
    }

    fn seek(&self, time: u64) {
        block_in_place(self.inner.seek(time))
    }

    fn stop(&self) {
        block_in_place(self.inner.stop())
    }
}

#[derive(Default)]
pub struct ChromecastPlayerBuilder {
    id: Option<String>,
    name: Option<String>,
    cast_model: Option<String>,
    cast_address: Option<String>,
    cast_port: Option<u16>,
    heartbeat_millis: Option<u64>,
    runtime: Option<Arc<Runtime>>,
}

impl ChromecastPlayerBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn cast_model<S: Into<String>>(mut self, cast_model: S) -> Self {
        self.cast_model = Some(cast_model.into());
        self
    }

    pub fn cast_address<S: Into<String>>(mut self, cast_address: S) -> Self {
        self.cast_address = Some(cast_address.into());
        self
    }

    pub fn cast_port(mut self, cast_port: u16) -> Self {
        self.cast_port = Some(cast_port);
        self
    }

    pub fn heartbeat_millis(mut self, heartbeat_millis: u64) -> Self {
        self.heartbeat_millis = Some(heartbeat_millis);
        self
    }

    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn build(self) -> chromecast::Result<ChromecastPlayer> {
        let id = self.id.expect("expected an id to be set");
        let name = self.name.expect("expected a name to be set");
        let cast_model = self.cast_model.expect("expected a cast model to be set");
        let cast_address = self.cast_address.expect("expected a cast address to be set");
        let cast_port = self.cast_port.expect("expected a cast port to be set");
        let heartbeat_millis = self.heartbeat_millis.unwrap_or(DEFAULT_HEARTBEAT_INTERVAL_MILLIS);
        let runtime = self.runtime.unwrap_or_else(|| Arc::new(runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name(format!("chromecast-{}", name))
            .build()
            .expect("failed to create runtime")));

        ChromecastPlayer::new(
            id,
            name,
            cast_model,
            cast_address,
            cast_port,
            heartbeat_millis,
            runtime,
        )
    }
}

struct InnerChromecastPlayer {
    id: String,
    name: String,
    request: Mutex<Option<Arc<Box<dyn PlayRequest>>>>,
    state: Mutex<PlayerState>,
    cast_model: String,
    cast_address: String,
    cast_port: u16,
    cast_device: Mutex<CastDevice<'static>>,
    cast_app: Mutex<Option<Application>>,
    cast_media_session_id: Mutex<Option<i32>>,
    callbacks: CoreCallbacks<PlayerEvent>,
    runtime: Arc<Runtime>,
    status_check_token: Mutex<CancellationToken>,
    shutdown_token: CancellationToken,
}

impl InnerChromecastPlayer {
    fn state(&self) -> PlayerState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    fn update_state(&self, state: PlayerState) {
        block_in_place(self.update_state_async(state))
    }

    async fn update_state_async(&self, state: PlayerState) {
        let mut event_state: Option<PlayerState> = None;

        {
            let mut mutex = self.state.lock().await;
            if *mutex != state {
                trace!("Updating Chromecast {} state to {:?}", self.name, state);
                *mutex = state.clone();
                debug!("Chromecast {} state has been updated to {:?}", self.name, state);
                event_state = Some(state);
            }
        }

        if let Some(state) = event_state {
            self.callbacks.invoke(PlayerEvent::StateChanged(state));
        }
    }

    async fn start_app(&self) -> chromecast::Result<Application> {
        self.try_command(|| async {
            let app = CastDeviceApp::DefaultMediaReceiver;
            let mut mutex = self.cast_app.lock().await;

            match mutex.clone() {
                None => {
                    let cast_device = self.cast_device.lock().await;

                    trace!("Establishing connection to the device receiver");
                    if let Err(e) = cast_device.connection.connect("receiver-0") {
                        return Err(ChromecastError::AppInitializationFailed(e.to_string()));
                    }

                    trace!("Launching chromecast app {:?}", app);
                    return match cast_device.receiver.launch_app(&app) {
                        Ok(app) => {
                            debug!("Chromecast app {:?} has been launched", app);
                            *mutex = Some(app.clone());
                            Ok(app)
                        }
                        Err(e) => Err(ChromecastError::AppInitializationFailed(e.to_string())),
                    };
                }
                Some(app) => {
                    debug!("Chromecast default media receiver app is already running");
                    Ok(app)
                }
            }
        }).await
    }

    async fn connect(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            let mutex = self.cast_app.lock().await;
            let cast_device = self.cast_device.lock().await;

            if let Some(app) = mutex.as_ref() {
                trace!("Connecting to chromecast app {:?}", app);
                cast_device
                    .connection
                    .connect(app.transport_id.as_str())
                    .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))?;

                debug!("Connected to chromecast app {:?}", app);
                return Ok(());
            }

            Err(ChromecastError::AppNotInitialized)
        }).await
    }

    async fn load(&self, app: &Application, request: &Box<dyn PlayRequest>) -> chromecast::Result<()> {
        return self.try_command(|| async {
            {
                let cast_device = self.cast_device.lock().await;
                let media = Self::request_to_media_payload(request);
                let load = LoadCommand {
                    request_id: 0,
                    session_id: app.session_id.to_string(),
                    payload_type: (),
                    media,
                    autoplay: true,
                    current_time: request.auto_resume_timestamp()
                        .map(|e| Self::parse_to_chromecast_time(e))
                        .unwrap_or(0f32),
                    active_track_ids: None,
                };
                trace!("Parsing load command payload {:?}", load);
                let load_payload = serde_json::to_string(&load)
                    .map_err(|e| ChromecastError::Parsing(e.to_string()))?;

                trace!("Sending load command {}", load_payload);
                if let Err(e) = cast_device.receiver.broadcast_message(MEDIA_CHANNEL_NAMESPACE, &load) {
                    return Err(ChromecastError::AppInitializationFailed(e.to_string()));
                }
            }

            Ok(())
        }).await;
    }

    async fn stop_app(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            let mut mutex = block_in_place(self.cast_app.lock());
            let cast_device = block_in_place(self.cast_device.lock());

            if let Some(app) = mutex.take() {
                debug!("Stopping chromecast app {:?}", app);
                cast_device.receiver.stop_app(app.session_id)
                    .map_err(|e| ChromecastError::AppTerminationFailed(e.to_string()))?;
            }

            Ok(())
        }).await
    }

    async fn pause(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                if let Err(e) = self.try_command(|| async {
                    let cast_device = self.cast_device.lock().await;
                    cast_device.media.pause(app.transport_id.as_str(), media_session_id.clone())
                        .map_err(|e| ChromecastError::Connection(e.to_string()))
                }).await {
                    error!("Failed to pause Chromecast {} playback, {}", self.name, e);
                }
            } else {
                warn!("Unable to pause Chromecast {}, media session id is unknown", self.name);
            }
        }
    }

    async fn resume(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                if let Err(e) = self.try_command(|| async {
                    let cast_device = self.cast_device.lock().await;
                    cast_device.media.play(app.transport_id.as_str(), media_session_id.clone())
                        .map_err(|e| ChromecastError::Connection(e.to_string()))
                }).await {
                    error!("Failed to resume Chromecast {} playback, {}", self.name, e);
                }
            } else {
                warn!("Unable to resume Chromecast {}, media session id is unknown", self.name);
            }
        }
    }

    async fn seek(&self, time: u64) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                if let Err(e) = self.try_command(|| async {
                    let chromecast_time = Self::parse_to_chromecast_time(time);
                    let cast_device = self.cast_device.lock().await;

                    cast_device.media.seek(app.transport_id.as_str(), media_session_id.clone(), Some(chromecast_time), None)
                        .map_err(|e| ChromecastError::Connection(e.to_string()))
                }).await {
                    error!("Failed to seek Chromecast {} playback, {}", self.name, e);
                }
            } else {
                warn!("Unable to seek Chromecast {}, media session id is unknown", self.name);
            }
        }
    }

    async fn stop(&self) {
        {
            let mutex = self.status_check_token.lock().await;
            trace!("Cancelling status check token for {}", self.name);
            mutex.cancel();
        }
        {
            let mut mutex = self.cast_media_session_id.lock().await;
            trace!("Removing media session id for {}", self.name);
            let _ = mutex.take();
        }

        if let Err(e) = self.stop_app().await {
            error!("Failed to stop Chromecast playback, {}", e);
            self.update_state_async(PlayerState::Error).await
        } else {
            self.update_state_async(PlayerState::Stopped).await
        }
    }

    async fn generate_status_token(&self) -> CancellationToken {
        let token = CancellationToken::new();
        {
            let mut mutex = self.status_check_token.lock().await;
            trace!("Cancelling original Chromecast status token");
            mutex.cancel();
            *mutex = token.clone();
        }

        debug!("Generated new Chromecast status token");
        token
    }

    async fn status(&self) -> chromecast::Result<Status> {
        let mutex = self.cast_device.lock().await;

        self.try_command(|| async {
            if let Some(app) = self.cast_app.lock().await.as_ref() {
                trace!("Requesting Chromecast {} status info", self.name);
                return mutex.media.get_status(app.transport_id.as_str(), None)
                    .map_err(|e| ChromecastError::Connection(e.to_string()));
            }

            Err(ChromecastError::AppNotInitialized)
        }).await
    }

    async fn handle_status_update(&self, status: Status) {
        trace!("Received Chromecast {} status update {:?}", self.name, status);
        if let Some(e) = status.entries.get(0) {
            {
                let mut mutex = self.cast_media_session_id.lock().await;
                if mutex.is_none() {
                    *mutex = Some(e.media_session_id.clone());
                    debug!("Received Chromecast media session id {}", e.media_session_id);
                }
            }

            match e.player_state {
                channels::media::PlayerState::Idle => self.update_state_async(PlayerState::Ready).await,
                channels::media::PlayerState::Playing => self.update_state_async(PlayerState::Playing).await,
                channels::media::PlayerState::Buffering => self.update_state_async(PlayerState::Buffering).await,
                channels::media::PlayerState::Paused => self.update_state_async(PlayerState::Paused).await,
            }

            if let Some(time) = e.current_time {
                self.callbacks.invoke(PlayerEvent::TimeChanged(Self::parse_to_popcorn_fx_time(time)));
            }
            if let Some(media) = &e.media {
                if let Some(duration) = media.duration {
                    self.callbacks.invoke(PlayerEvent::DurationChanged(Self::parse_to_popcorn_fx_time(duration)));
                }
            }
        } else {
            warn!("Received empty status update for Chromecast {}", self.name);
        }
    }

    /// Try to execute the given Chromecast command.
    ///
    /// If the command fails, it will try to reestablish the connection and try the command
    /// again one last time.
    ///
    /// # Arguments
    ///
    /// * `command_fn` - A function that returns a future representing the command to execute.
    ///
    /// # Returns
    ///
    /// - `Ok(O)` if the command was executed successfully.
    /// - `Err(chromecast::Error)` if an error occurred during the command execution.
    async fn try_command<F, O>(&self, command_fn: impl Fn() -> F) -> chromecast::Result<O>
        where F: Future<Output=chromecast::Result<O>> + Send,
              O: Send, {
        match command_fn().await {
            Ok(e) => Ok(e),
            Err(e) => {
                debug!("Failed to execute command: {}", e);
                self.reestablish_connection().await?;
                command_fn().await
            }
        }
    }

    /// Reestablishes the connection to the Chromecast device.
    /// It will reconnect to the device using the known `cast_address` and `cast_port`.
    /// If the connection is established, the old [CastDevice] will be replaced with the new one.
    ///
    /// **It's recommended to use [Self::try_command] instead of calling this method directly.**
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the connection is successfully reestablished.
    /// - `Err(chromecast::Error)` if an error occurs during the connection process.
    async fn reestablish_connection(&self) -> chromecast::Result<()> {
        let mut mutex = self.cast_device.lock().await;
        let address = self.cast_address.clone();
        let port = self.cast_port.clone();

        trace!("Reestablishing connection to the Chromecast device {}", self.name);
        let cast_device = CastDevice::connect_without_host_verification(address, port)
            .map_err(|e| ChromecastError::Connection(e.to_string()))?;
        *mutex = cast_device;
        debug!("Reconnected to the Chromecast device {}", self.name);

        Ok(())
    }

    fn request_to_media_payload(request: &Box<dyn PlayRequest>) -> Media {
        let mut images: Vec<Image> = Vec::new();
        let subtitle = Self::create_media_subtitle(request);

        if let Some(e) = request.thumbnail() {
            images.push(Image {
                url: e,
                height: None,
                width: None,
            });
        }

        Media {
            url: request.url().to_string(),
            stream_type: StreamType::Buffered,
            content_type: "application/octet-stream".to_string(),
            metadata: Some(Metadata::Movie(MovieMetadata {
                metadata_type: (),
                title: Some(request.title().to_string()),
                subtitle: Some(subtitle),
                studio: None,
                images: Some(images).filter(|e| !e.is_empty()),
                release_date: None,
                thumb: request.thumbnail(),
                thumbnail_url: request.thumbnail(),
                poster_url: request.background(),
            })),
            custom_data: None,
            duration: None,
            text_track_style: None,
        }
    }

    fn create_media_subtitle(request: &Box<dyn PlayRequest>) -> String {
        let separator = if request.caption().is_some() {
            " - "
        } else {
            ""
        };

        format!("{}{}{}",
                request.caption().unwrap_or(String::new()),
                separator,
                request.quality().unwrap_or(String::new()))
    }

    fn parse_to_popcorn_fx_time(time: f32) -> u64 {
        (time * 1000f32) as u64
    }

    fn parse_to_chromecast_time(time: u64) -> f32 {
        time as f32 / 1000f32
    }
}

impl Callbacks<PlayerEvent> for InnerChromecastPlayer {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

impl Debug for InnerChromecastPlayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerChromecastPlayer")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("request", &self.request)
            .field("state", &self.state)
            .field("cast_model", &self.cast_model)
            .field("cast_address", &self.cast_address)
            .field("cast_port", &self.cast_port)
            .field("cast_app", &self.cast_app)
            .field("callbacks", &self.callbacks)
            .field("runtime", &self.runtime)
            .field("cancellation_token", &self.shutdown_token)
            .finish()
    }
}

impl Drop for InnerChromecastPlayer {
    fn drop(&mut self) {
        block_in_place(self.stop());
        self.shutdown_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use httpmock::MockServer;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_player_id() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        let player = create_player(&server, runtime.clone());

        let result = player.id();

        assert_eq!("MyChromecastId", result);
    }

    #[test]
    fn test_create_subtitle() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        let player = create_player(&server, runtime.clone());
    }

    fn create_player(server: &MockServer, runtime: Arc<Runtime>) -> ChromecastPlayer {
        ChromecastPlayer::builder()
            .id("MyChromecastId")
            .name("MyChromecastName")
            .cast_model("Chromecast")
            .cast_address(server.host())
            .cast_port(server.port())
            .runtime(runtime)
            .heartbeat_millis(1000)
            .build()
            .unwrap()
    }
}