use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, trace, warn};
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::media::{MediaResponse, Status, StatusEntry};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use rust_cast::{channels, ChannelMessage};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Interval};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::players::{
    MetadataValue, PlayRequest, PlayRequestBuilder, Player, PlayerEvent, PlayerState,
};
use popcorn_fx_core::core::subtitles::model::SubtitleType;
use popcorn_fx_core::core::subtitles::SubtitleServer;

use crate::chromecast;
use crate::chromecast::device::{FxCastDevice, DEFAULT_RECEIVER};
use crate::chromecast::transcode::{NoOpTranscoder, Transcoder};
use crate::chromecast::{
    ChromecastError, Image, LoadCommand, Media, MediaDetailedErrorCode, MediaError, Metadata,
    MovieMetadata, StreamType, TextTrackEdgeType, TextTrackStyle, TextTrackType, Track, TrackType,
};

const GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-chromecast-icon.png");
const DESCRIPTION: &str =
    "Chromecast streaming media device which allows the playback of videos on your TV.";
const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
const MEDIA_CHANNEL_NAMESPACE: &str = "urn:x-cast:com.google.cast.media";
const SUBTITLE_CONTENT_TYPE: &str = "text/vtt";
const MESSAGE_TYPE_ERROR: &str = "ERROR";
const METADATA_TRANSCODING: &str = "transcoding";
const METADATA_TRANSCODING_ORIGINAL_URL: &str = "";

/// The type of the factory function used to create the Chromecast client device.
pub type DeviceFactory<D> = Box<dyn Fn(String, u16) -> chromecast::Result<D> + Send + Sync>;

/// The Chromecast player allows the playback of media items on a specific Chromecast device.
#[derive(Debug, Display)]
#[display(fmt = "Chromecast player {}", "self.name()")]
pub struct ChromecastPlayer<D: FxCastDevice + 'static> {
    inner: Arc<InnerChromecastPlayer<D>>,
}

impl<D: FxCastDevice> ChromecastPlayer<D> {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        cast_model: impl Into<String>,
        cast_address: impl Into<String>,
        cast_port: u16,
        cast_device_factory: DeviceFactory<D>,
        subtitle_server: Arc<SubtitleServer>,
        transcoder: Arc<Box<dyn Transcoder>>,
        heartbeat_seconds: u64,
    ) -> chromecast::Result<Self> {
        let name = name.into();
        let cast_address = cast_address.into();
        let (command_sender, command_receiver) = unbounded_channel();

        trace!(
            "Trying to establish connection with Chromecast device {} on {}:{}...",
            name,
            cast_address,
            cast_port
        );
        let cast_device = cast_device_factory(cast_address.clone(), cast_port)?;
        debug!(
            "Connected to Chromecast device {} on {}:{}",
            name, cast_address, cast_port
        );
        if let Err(e) = cast_device.connect(DEFAULT_RECEIVER) {
            return Err(ChromecastError::Connection(e.to_string()));
        }
        if let Err(e) = cast_device.ping() {
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
            cast_device: RwLock::new(cast_device),
            cast_device_factory,
            cast_app: Default::default(),
            cast_media_session_id: Default::default(),
            subtitle_server,
            transcoder,
            command_sender,
            callbacks: MultiThreadedCallback::new(),
            status_check_token: Default::default(),
            cancellation_token: Default::default(),
        });

        let inner = instance.clone();
        tokio::spawn(async move {
            inner
                .start(
                    command_receiver,
                    interval(Duration::from_secs(heartbeat_seconds)),
                )
                .await;
        });

        Ok(Self { inner: instance })
    }

    pub fn builder() -> ChromecastPlayerBuilder<D> {
        ChromecastPlayerBuilder::builder()
    }

    async fn start_message_handler(
        inner: Arc<InnerChromecastPlayer<D>>,
        cancellation_token: CancellationToken,
    ) {
        loop {
            if cancellation_token.is_cancelled() {
                break;
            }

            let message = inner.cast_device.read().await.receive();
            inner.handle_event(message).await;
        }

        debug!("Chromecast {} message handler has been stopped", inner.name);
    }

    async fn start_status_updates(
        inner: Arc<InnerChromecastPlayer<D>>,
        cancellation_token: CancellationToken,
    ) {
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

#[async_trait]
impl<D: FxCastDevice + 'static> Player for ChromecastPlayer<D> {
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

    async fn state(&self) -> PlayerState {
        self.inner.state().await
    }

    async fn request(&self) -> Option<PlayRequest> {
        self.inner.request.lock().await.clone()
    }

    async fn current_volume(&self) -> Option<u32> {
        // TODO
        None
    }

    async fn play(&self, request: PlayRequest) {
        trace!(
            "Starting Chromecast {} playback for {:?}",
            self.name(),
            request
        );
        self.inner.update_state_async(PlayerState::Loading).await;

        match self.inner.start_app().await {
            Ok(app) => {
                if let Err(e) = self.inner.connect().await {
                    error!("Failed to connect to Chromecast device, {}", e);
                    self.inner.update_state_async(PlayerState::Error).await;
                    return;
                }

                // let inner = self.inner.clone();
                // let cancellation_token = self.inner.shutdown_token.clone();
                // self.inner.runtime.spawn(Self::start_message_handler(inner, cancellation_token));

                // serve the chromecast subtitle if one is present
                let subtitle_url = if let Some(subtitle) = request.subtitle().subtitle.as_ref() {
                    match self
                        .inner
                        .subtitle_server
                        .serve(subtitle.clone(), SubtitleType::Vtt)
                        .await
                    {
                        Ok(url) => Some(url),
                        Err(e) => {
                            error!("Failed to serve subtitle: {}", e);
                            None
                        }
                    }
                } else {
                    None
                };

                if let Err(e) = self.inner.load(&app, &request, subtitle_url).await {
                    error!("Failed to load Chromecast media, {}", e);
                    self.inner.update_state_async(PlayerState::Error).await;
                    return;
                }

                debug!("Starting Chromecast {} playback", self.name());
                let token = self.inner.generate_status_token().await;
                tokio::spawn(Self::start_status_updates(self.inner.clone(), token));
                self.inner.resume().await;

                {
                    trace!("Updating Chromecast player request to {:?}", request);
                    let mut mutex = self.inner.request.lock().await;
                    *mutex = Some(request)
                }
            }
            Err(e) => {
                error!("Failed to start Chromecast playback, {}", e);
                self.inner.update_state_async(PlayerState::Error).await;
            }
        }
    }

    async fn pause(&self) {
        self.inner.send_command(ChromecastPlayerCommand::Pause);
    }

    async fn resume(&self) {
        self.inner.send_command(ChromecastPlayerCommand::Resume);
    }

    async fn seek(&self, time: u64) {
        self.inner.send_command(ChromecastPlayerCommand::Seek(time));
    }

    async fn stop(&self) {
        self.inner.send_command(ChromecastPlayerCommand::Stop);
    }
}

impl<D: FxCastDevice + 'static> Callback<PlayerEvent> for ChromecastPlayer<D> {
    fn subscribe(&self) -> Subscription<PlayerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl<D: FxCastDevice + 'static> Drop for ChromecastPlayer<D> {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, PartialEq)]
enum ChromecastPlayerCommand {
    Pause,
    Resume,
    Seek(u64),
    Stop,
}

pub struct ChromecastPlayerBuilder<D: FxCastDevice> {
    id: Option<String>,
    name: Option<String>,
    cast_model: Option<String>,
    cast_address: Option<String>,
    cast_port: Option<u16>,
    cast_device_factory: Option<DeviceFactory<D>>,
    subtitle_server: Option<Arc<SubtitleServer>>,
    transcoder: Option<Arc<Box<dyn Transcoder>>>,
    heartbeat_seconds: Option<u64>,
}

impl<D: FxCastDevice> ChromecastPlayerBuilder<D> {
    pub fn builder() -> Self {
        Self {
            id: None,
            name: None,
            cast_model: None,
            cast_address: None,
            cast_port: None,
            cast_device_factory: None,
            subtitle_server: None,
            transcoder: None,
            heartbeat_seconds: None,
        }
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

    pub fn cast_device_factory(mut self, cast_device_factory: DeviceFactory<D>) -> Self {
        self.cast_device_factory = Some(cast_device_factory);
        self
    }

    pub fn subtitle_server(mut self, subtitle_server: Arc<SubtitleServer>) -> Self {
        self.subtitle_server = Some(subtitle_server);
        self
    }

    pub fn transcoder(mut self, transcoder: Arc<Box<dyn Transcoder>>) -> Self {
        self.transcoder = Some(transcoder);
        self
    }

    pub fn heartbeat_seconds(mut self, heartbeat_seconds: u64) -> Self {
        self.heartbeat_seconds = Some(heartbeat_seconds);
        self
    }

    pub fn build(self) -> chromecast::Result<ChromecastPlayer<D>> {
        let id = self.id.expect("expected an id to be set");
        let name = self.name.expect("expected a name to be set");
        let cast_model = self.cast_model.expect("expected a cast model to be set");
        let cast_address = self
            .cast_address
            .expect("expected a cast address to be set");
        let cast_port = self.cast_port.expect("expected a cast port to be set");
        let cast_device_factory = self
            .cast_device_factory
            .expect("expected a cast device factory to be set");
        let subtitle_server = self
            .subtitle_server
            .expect("expected a subtitle server to have been set");
        let heartbeat_seconds = self
            .heartbeat_seconds
            .unwrap_or(DEFAULT_HEARTBEAT_INTERVAL_SECONDS);
        let transcoder = self.transcoder.unwrap_or_else(|| {
            warn!("No transcoder set, using no-op transcoder");
            Arc::new(Box::new(NoOpTranscoder {}))
        });

        ChromecastPlayer::new(
            id,
            name,
            cast_model,
            cast_address,
            cast_port,
            cast_device_factory,
            subtitle_server,
            transcoder,
            heartbeat_seconds,
        )
    }
}

struct InnerChromecastPlayer<D: FxCastDevice> {
    id: String,
    name: String,
    request: Mutex<Option<PlayRequest>>,
    state: Mutex<PlayerState>,
    cast_model: String,
    cast_address: String,
    cast_port: u16,
    cast_device: RwLock<D>,
    cast_device_factory: DeviceFactory<D>,
    cast_app: Mutex<Option<Application>>,
    cast_media_session_id: Mutex<Option<i32>>,
    subtitle_server: Arc<SubtitleServer>,
    transcoder: Arc<Box<dyn Transcoder>>,
    status_check_token: Mutex<CancellationToken>,
    command_sender: UnboundedSender<ChromecastPlayerCommand>,
    callbacks: MultiThreadedCallback<PlayerEvent>,
    cancellation_token: CancellationToken,
}

impl<D: FxCastDevice> InnerChromecastPlayer<D> {
    async fn start(
        &self,
        mut command_receiver: UnboundedReceiver<ChromecastPlayerCommand>,
        mut heartbeat_interval: Interval,
    ) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                _ = heartbeat_interval.tick() => self.send_heartbeat().await,
            }
        }
        self.stop().await;
        debug!("Chromecast player main loop ended");
    }

    async fn handle_command(&self, command: ChromecastPlayerCommand) {
        match command {
            ChromecastPlayerCommand::Pause => self.pause().await,
            ChromecastPlayerCommand::Resume => self.resume().await,
            ChromecastPlayerCommand::Seek(time) => self.seek(time).await,
            ChromecastPlayerCommand::Stop => self.stop().await,
        }
    }

    async fn state(&self) -> PlayerState {
        *self.state.lock().await
    }

    async fn update_state_async(&self, state: PlayerState) {
        let mut event_state: Option<PlayerState> = None;

        {
            let mut mutex = self.state.lock().await;
            if *mutex != state {
                trace!("Updating Chromecast {} state to {:?}", self.name, state);
                *mutex = state.clone();
                debug!(
                    "Chromecast {} state has been updated to {:?}",
                    self.name, state
                );
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
                    let cast_device = self.cast_device.read().await;

                    trace!("Establishing connection to the device receiver");
                    if let Err(e) = cast_device.connect(DEFAULT_RECEIVER) {
                        return Err(ChromecastError::AppInitializationFailed(e.to_string()));
                    }

                    // verify if the default app is already running
                    // if so, we use the existing app information instead of launching a new app
                    trace!("Retrieving Chromecast {} device status", self.name);
                    match cast_device.device_status() {
                        Ok(status) => {
                            if let Some(app) = status.applications.into_iter().find(|e| {
                                e.app_id == CastDeviceApp::DefaultMediaReceiver.to_string()
                            }) {
                                debug!("Chromecast default media receiver app is already running");
                                *mutex = Some(app.clone());
                                return Ok(app);
                            } else {
                                trace!("Chromecast default media receiver app is not yet running");
                            }
                        }
                        Err(e) => error!(
                            "Failed to retrieve Chromecast {} device status, {}",
                            self.name, e
                        ),
                    }

                    trace!("Launching chromecast app {:?}", app);
                    return match cast_device.launch_app(&app) {
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
        })
        .await
    }

    async fn send_heartbeat(&self) {
        let ping_result: chromecast::Result<()>;

        {
            let mutex = self.cast_device.read().await;
            trace!("Sending Chromecast {} heartbeat", self.name);
            ping_result = mutex.ping();
        }

        if let Err(e) = ping_result {
            warn!("Failed to ping Chromecast {}, {}", self.name, e);
        }
    }

    async fn start_transcoding(&self) {
        let mut mutex = self.request.lock().await;
        // don't keep the cast_app lock as it will cause issues when trying to resume the media playback
        let app = self.cast_app.lock().await.clone();

        if let Some(app) = app.as_ref() {
            if let Some(request) = mutex.take() {
                trace!("Starting transcoding process for {:?}", request);
                let request_url = request.url();
                match self.transcoder.transcode(request_url).await {
                    Ok(output) => {
                        debug!("Received transcoding output {:?}", output);
                        let request = PlayRequestBuilder::from(&request)
                            .url(output.url)
                            .metadata_bool(METADATA_TRANSCODING, true)
                            .metadata_str(METADATA_TRANSCODING_ORIGINAL_URL, request_url)
                            .build();

                        // serve the chromecast subtitle if one is present
                        let subtitle_url: Option<String>;
                        if request.subtitle().enabled {
                            subtitle_url = self.subtitle_url(&request).await;
                        } else {
                            subtitle_url = None;
                        }

                        match self.load(app, &request, subtitle_url).await {
                            Ok(_) => {
                                *mutex = Some(request);
                                drop(mutex);

                                let _ = self.resume().await;
                            }
                            Err(e) => {
                                error!("Failed to start media transcoding, {}", e);
                                self.update_state_async(PlayerState::Error).await
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start media transcoding, {}", e);
                        self.update_state_async(PlayerState::Error).await
                    }
                }
            } else {
                error!("Failed to start media transcoding, no request available");
                self.update_state_async(PlayerState::Error).await
            }
        } else {
            error!("Failed to start media transcoding, no app is running");
            self.update_state_async(PlayerState::Error).await
        }
    }

    async fn connect(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            let mutex = self.cast_app.lock().await;
            let cast_device = self.cast_device.read().await;

            if let Some(app) = mutex.as_ref() {
                trace!("Connecting to chromecast app {:?}", app);
                cast_device.connect(app.transport_id.to_string())?;

                debug!("Connected to chromecast app {:?}", app);
                return Ok(());
            }

            Err(ChromecastError::AppNotInitialized)
        })
        .await
    }

    async fn load(
        &self,
        app: &Application,
        request: &PlayRequest,
        subtitle_url: Option<String>,
    ) -> chromecast::Result<()> {
        self.try_command(|| async {
            let cast_device = self.cast_device.read().await;
            let active_track_ids = if subtitle_url.is_some() {
                Some(vec![0])
            } else {
                None
            };
            let media = Self::request_to_media_payload(request, subtitle_url.clone());
            let load = LoadCommand {
                request_id: 0,
                session_id: app.session_id.to_string(),
                payload_type: (),
                media,
                autoplay: true,
                current_time: request
                    .auto_resume_timestamp()
                    .map(|e| Self::parse_to_chromecast_time(e))
                    .unwrap_or(0f32),
                active_track_ids,
            };

            trace!("Sending load command {:?}", load);
            if let Err(e) = cast_device.broadcast_message(MEDIA_CHANNEL_NAMESPACE, &load) {
                return Err(ChromecastError::AppInitializationFailed(e.to_string()));
            }

            Ok(())
        })
        .await
    }

    async fn stop_app(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            let mut mutex = self.cast_app.lock().await;
            let cast_device = self.cast_device.read().await;

            if let Some(app) = mutex.take() {
                debug!("Stopping chromecast app {:?}", app);
                cast_device.stop_app(app.session_id)?;
            }

            Ok(())
        })
        .await
    }

    async fn pause(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                match self
                    .try_command(|| async {
                        let cast_device = self.cast_device.read().await;
                        cast_device.pause(app.transport_id.to_string(), media_session_id.clone())
                    })
                    .await
                {
                    Ok(status) => self.on_player_state_changed(&status).await,
                    Err(e) => error!("Failed to pause Chromecast {}, {}", self.name, e),
                }
            } else {
                warn!(
                    "Unable to pause Chromecast {}, media session id is unknown",
                    self.name
                );
            }
        }
    }

    async fn resume(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                match self
                    .try_command(|| async {
                        let cast_device = self.cast_device.read().await;
                        cast_device.play(app.transport_id.to_string(), media_session_id.clone())
                    })
                    .await
                {
                    Ok(status) => {
                        trace!("Received resume status {:?}", status);
                        self.on_player_state_changed(&status).await;
                    }
                    Err(e) => error!("Failed to resume Chromecast {} playback, {}", self.name, e),
                }
            } else {
                warn!(
                    "Unable to resume Chromecast {}, media session id is unknown",
                    self.name
                );
            }
        }
    }

    async fn seek(&self, time: u64) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                if let Err(e) = self
                    .try_command(|| async {
                        let chromecast_time = Self::parse_to_chromecast_time(time);
                        let cast_device = self.cast_device.read().await;

                        cast_device.seek(
                            app.transport_id.to_string(),
                            media_session_id.clone(),
                            Some(chromecast_time),
                            None,
                        )
                    })
                    .await
                {
                    error!("Failed to seek Chromecast {} playback, {}", self.name, e);
                }
            } else {
                warn!(
                    "Unable to seek Chromecast {}, media session id is unknown",
                    self.name
                );
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
        tokio::select! {
            _ = time::sleep(Duration::from_secs(5)) => Err(ChromecastError::CommandTimeout("status".to_string())),
            status = self.try_command(|| async {
                if let Some(app) = self.cast_app.lock().await.as_ref() {
                    let mutex = self.cast_device.read().await;

                    trace!("Requesting Chromecast {} status info", self.name);
                    return mutex.media_status(app.transport_id.to_string(), None);
                }

                Err(ChromecastError::AppNotInitialized)
            }) => status,
        }
    }

    async fn handle_status_update(&self, status: Status) {
        trace!(
            "Received Chromecast {} status update {:?}",
            self.name,
            status
        );
        if let Some(e) = status.entries.get(0) {
            {
                let mut mutex = self.cast_media_session_id.lock().await;
                if mutex.is_none() {
                    *mutex = Some(e.media_session_id.clone());
                    debug!(
                        "Received Chromecast media session id {}",
                        e.media_session_id
                    );
                }
            }

            // update the playback state of the player
            self.on_player_state_changed(e).await;

            if let Some(time) = e.current_time {
                self.callbacks
                    .invoke(PlayerEvent::TimeChanged(Self::parse_to_popcorn_fx_time(
                        time,
                    )));
            }
            if let Some(media) = &e.media {
                if let Some(duration) = media.duration {
                    self.callbacks.invoke(PlayerEvent::DurationChanged(
                        Self::parse_to_popcorn_fx_time(duration),
                    ));
                }
            }
        } else {
            warn!("Received empty status update for Chromecast {}", self.name);
        }
    }

    async fn on_player_state_changed(&self, e: &StatusEntry) {
        match e.player_state {
            channels::media::PlayerState::Idle => self.update_state_async(PlayerState::Ready).await,
            channels::media::PlayerState::Playing => {
                self.update_state_async(PlayerState::Playing).await
            }
            channels::media::PlayerState::Buffering => {
                self.update_state_async(PlayerState::Buffering).await
            }
            channels::media::PlayerState::Paused => {
                self.update_state_async(PlayerState::Paused).await
            }
        }
    }

    /// Tries to serve the subtitle URL for the given request.
    ///
    /// This function converts the given subtitle to the expected Chromecast subtitle format.
    ///
    /// # Arguments
    ///
    /// * `request` - The request which might hold the subtitle information to serve.
    ///
    /// # Returns
    ///
    /// The subtitle URL if available, or `None` if the subtitle is not present or could not be served.
    async fn subtitle_url(&self, request: &PlayRequest) -> Option<String> {
        if let Some(url) = request.subtitle().subtitle.as_ref().cloned() {
            match self.subtitle_server.serve(url, SubtitleType::Vtt).await {
                Ok(e) => Some(e),
                Err(e) => {
                    error!("Failed to serve subtitle, {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    async fn handle_event(&self, event: chromecast::Result<ChannelMessage>) {
        trace!("Handling Chromecast {} event {:?}", self.name, event);
        match event {
            Ok(e) => match e {
                ChannelMessage::Media(response) => self.handle_media_event(response).await,
                ChannelMessage::Heartbeat(response) => self.handle_heartbeat_event(response).await,
                _ => {}
            },
            Err(e) => {
                error!("Chromecast message error: {}", e);
                self.update_state_async(PlayerState::Error).await
            }
        }
    }

    async fn handle_media_event(&self, event: MediaResponse) {
        trace!("Handling media response event {:?}", event);
        match event {
            MediaResponse::Status(status) => {
                self.handle_status_update(status).await;
            }
            MediaResponse::NotImplemented(code, value) => {
                if code == MESSAGE_TYPE_ERROR {
                    match serde_json::from_value::<MediaError>(value) {
                        Ok(e) => self.handle_media_error(e).await,
                        Err(e) => error!("Failed to deserialize MediaError: {}", e),
                    }
                } else {
                    debug!("Received unknown media message type {}", code);
                }
            }
            _ => {}
        }
    }

    async fn handle_heartbeat_event(&self, event: HeartbeatResponse) {
        trace!("Handling heartbeat response event {:?}", event);
        if let HeartbeatResponse::Ping = event {
            // if we receive a ping message from the chromecast device, we need to answer with
            // a pong message in a timely manner to prevent the device from closing the connection
            if let Err(e) = self.cast_device.read().await.pong() {
                error!("Failed to send pong heartbeat, {}", e);
            }
        }
    }

    async fn handle_media_error(&self, error: MediaError) {
        debug!("Handling media error {:?}", error);
        if error.detailed_error_code == MediaDetailedErrorCode::MediaSrcNotSupported {
            let is_transcoding_request: bool;

            {
                let mutex = self.request.lock().await;
                is_transcoding_request = mutex
                    .as_ref()
                    .and_then(|req| req.metadata().get("transcoding").cloned())
                    .and_then(|value| match value {
                        MetadataValue::Bool(b) => Some(b),
                        _ => None,
                    })
                    .unwrap_or(false);
            }

            // verify if we have some information known about the play request
            // if not, the error that occurred was no longer related to this Chromecast playback
            if !is_transcoding_request {
                warn!("Media source is not supported by the Chromecast device, starting transcoding of the media");
                self.start_transcoding().await;
            } else {
                warn!("Chromecast device failed to play transcoding media");
            }
        } else {
            error!("Received media error {:?}", error);
            self.update_state_async(PlayerState::Error).await
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
    where
        F: Future<Output = chromecast::Result<O>> + Send,
        O: Send,
    {
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
        let mut mutex = self.cast_device.write().await;
        let address = self.cast_address.clone();
        let port = self.cast_port.clone();

        trace!(
            "Reestablishing connection to the Chromecast device {}",
            self.name
        );
        let cast_device = (self.cast_device_factory)(address, port)?;
        *mutex = cast_device;
        debug!("Reconnected to the Chromecast device {}", self.name);

        Ok(())
    }

    fn send_command(&self, command: ChromecastPlayerCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Chromecast player failed to send command, {}", e);
        }
    }

    fn request_to_media_payload(request: &PlayRequest, subtitle_url: Option<String>) -> Media {
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
                images,
                release_date: None,
                thumb: request.thumbnail(),
                thumbnail_url: request.thumbnail(),
                poster_url: request.background(),
            })),
            custom_data: None,
            duration: None,
            text_track_style: Some(TextTrackStyle {
                background_color: Some("#00000000".to_string()),
                custom_data: None,
                edge_color: Some("#000000FF".to_string()),
                edge_type: Some(TextTrackEdgeType::Outline),
                font_family: None,
                font_scale: None,
                foreground_color: Some("#FFFFFFFF".to_string()),
                window_color: None,
            }),
            tracks: subtitle_url.map(|e| {
                vec![Track {
                    track_id: 0,
                    track_type: TrackType::Text,
                    track_content_id: e.to_string(),
                    track_content_type: SUBTITLE_CONTENT_TYPE.to_string(),
                    subtype: TextTrackType::Subtitles,
                    language: "en".to_string(),
                    name: "English".to_string(),
                }]
            }),
        }
    }

    fn create_media_subtitle(request: &PlayRequest) -> String {
        let separator = if request.caption().is_some() {
            " - "
        } else {
            ""
        };

        format!(
            "{}{}{}",
            request.caption().unwrap_or(String::new()),
            separator,
            request.quality().unwrap_or(String::new())
        )
    }

    fn parse_to_popcorn_fx_time(time: f32) -> u64 {
        (time * 1000f32) as u64
    }

    fn parse_to_chromecast_time(time: u64) -> f32 {
        time as f32 / 1000f32
    }
}

impl<D: FxCastDevice> Debug for InnerChromecastPlayer<D> {
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
            .field("cancellation_token", &self.cancellation_token)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromecast::device::MockFxCastDevice;
    use crate::chromecast::tests::TestInstance;
    use crate::chromecast::transcode::{MockTranscoder, TranscodeOutput, TranscodeType};
    use popcorn_fx_core::core::media::MovieOverview;
    use popcorn_fx_core::core::players::PlayRequest;
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo};
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::testing::MockTorrentStream;
    use popcorn_fx_core::{assert_timeout, init_logger, recv_timeout};
    use rust_cast::channels::media::StatusEntry;
    use rust_cast::channels::receiver::Volume;
    use rust_cast::channels::{media, receiver};
    use serde_json::Number;
    use std::sync::mpsc::channel;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_player_new() {
        init_logger!();
        let subtitle_provider = MockSubtitleProvider::new();
        let transcoder = MockTranscoder::new();

        let result = ChromecastPlayer::new(
            "MyChromecastId",
            "MyChromecastName",
            "MyChromecastModel",
            "127.0.0.1",
            9870,
            Box::new(|_, _| Ok(create_default_device())),
            Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider)))),
            Arc::new(Box::new(transcoder)),
            500,
        );

        if let Ok(_) = result {
        } else {
            assert!(false, "expected a new player, but got {:?} instead", result);
        }
    }

    #[tokio::test]
    async fn test_player_id() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        let result = player.id();

        assert_eq!("MyChromecastId", result);
    }

    #[tokio::test]
    async fn test_player_name() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        let result = player.name();

        assert_eq!("MyChromecastName", result);
    }

    #[tokio::test]
    async fn test_player_description() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        let result = player.description();

        assert_eq!(DESCRIPTION, result);
    }

    #[tokio::test]
    async fn test_player_graphic_resource() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        let result = player.graphic_resource();

        assert_eq!(GRAPHIC_RESOURCE.to_vec(), result);
    }

    #[tokio::test]
    async fn test_player_state() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        let result = player.state().await;

        assert_eq!(PlayerState::Ready, result);
    }

    #[tokio::test]
    async fn test_player_play() {
        init_logger!();
        let url = "http://localhost:8900/my-video.mkv";
        let (tx_command, rx_command) = channel::<LoadCommand>();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = MockFxCastDevice::new();
            default_device_responses(&mut device);
            device
                .expect_device_status()
                .return_const(Ok(receiver::Status {
                    request_id: 1,
                    applications: vec![],
                    is_active_input: false,
                    is_stand_by: true,
                    volume: Volume {
                        level: None,
                        muted: None,
                    },
                }));
            device.expect_launch_app().return_const(Ok(Application {
                app_id: "MyAppId".to_string(),
                session_id: "MySessionId".to_string(),
                transport_id: "MyTransportId".to_string(),
                namespaces: vec![],
                display_name: "".to_string(),
                status_text: "".to_string(),
            }));
            let sender = tx_command.clone();
            device.expect_broadcast_message::<LoadCommand>().returning(
                move |_namespace, command| {
                    sender.send(command.clone()).unwrap();
                    Ok(())
                },
            );
            device.expect_play::<String>().return_const(Ok(StatusEntry {
                media_session_id: 0,
                media: None,
                playback_rate: 0.0,
                player_state: media::PlayerState::Playing,
                current_item_id: None,
                loading_item_id: None,
                preloaded_item_id: None,
                idle_reason: None,
                extended_status: None,
                current_time: None,
                supported_media_commands: 0,
            }));
            default_device_status_response(&mut device);
            device
        }));
        let movie = MovieOverview {
            title: "MyMovie".to_string(),
            imdb_id: "tt011000".to_string(),
            year: "1028".to_string(),
            rating: None,
            images: Default::default(),
        };
        let request = PlayRequest::builder()
            .url(url.to_string())
            .title("FooBar")
            .caption("MyCaption")
            .thumb("http://localhost/my-thumb.png")
            .background("http://localhost/my-background.png")
            .auto_resume_timestamp(28000)
            .subtitles_enabled(true)
            .media(Box::new(movie))
            .quality("720p")
            .torrent_stream(Box::new(MockTorrentStream::new()))
            .build();
        let (tx, mut rx) = unbounded_channel();
        let player = test_instance.player.take().unwrap();

        let mut receiver = player.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let PlayerEvent::StateChanged(state) = &*event {
                        tx.send(*state).unwrap();
                    }
                } else {
                    break;
                }
            }
        });
        player.play(request).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(PlayerState::Loading, result);

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(PlayerState::Playing, result);

        let command = rx_command.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(url.to_string(), command.media.url);
    }

    #[tokio::test]
    async fn test_player_pause() {
        init_logger!();
        let transport_id = "FooBar";
        let (tx, mut rx) = unbounded_channel();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device
                .expect_pause::<String>()
                .times(1)
                .returning(move |destination, _| {
                    sender.send(destination).unwrap();
                    Ok(status_entry(media::PlayerState::Paused))
                });
            device
        }));
        let player = test_instance.player.take().unwrap();

        *player.inner.cast_app.lock().await = Some(Application {
            app_id: "MyAppId".to_string(),
            session_id: "MySessionId".to_string(),
            transport_id: transport_id.to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *player.inner.cast_media_session_id.lock().await = Some(1);

        player.pause().await;
        assert_timeout!(
            Duration::from_millis(200),
            player.state().await == PlayerState::Paused,
            "expected the player to be paused"
        );

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(transport_id.to_string(), result);
    }

    #[tokio::test]
    async fn test_player_resume() {
        init_logger!();
        let mut test_instance = TestInstance::new_player(Box::new(|| create_default_device()));
        let player = test_instance.player.take().unwrap();

        player.resume().await;
    }

    #[tokio::test]
    async fn test_player_seek() {
        init_logger!();
        let transport_id = "LoremIpsum";
        let (tx, mut rx) = unbounded_channel();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device
                .expect_seek::<String>()
                .times(1)
                .returning(move |_, _, time, _| {
                    sender.send(time).unwrap();
                    Ok(status_entry(media::PlayerState::Playing))
                });
            device
        }));
        let player = test_instance.player.take().unwrap();

        *player.inner.cast_app.lock().await = Some(Application {
            app_id: "Foo".to_string(),
            session_id: "Bar".to_string(),
            transport_id: transport_id.to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *player.inner.cast_media_session_id.lock().await = Some(1);

        player.seek(14000).await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(Some(14f32), result);
    }

    #[tokio::test]
    async fn test_player_stop() {
        init_logger!();
        let session_id = "Bar";
        let (tx, rx) = channel();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device
                .expect_stop_app::<String>()
                .times(1)
                .returning(move |session_id| {
                    sender.send(session_id).unwrap();
                    Ok(())
                });
            device
        }));
        let player = test_instance.player.take().unwrap();

        *player.inner.cast_app.lock().await = Some(Application {
            app_id: "Foo".to_string(),
            session_id: session_id.to_string(),
            transport_id: "Dolor".to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *player.inner.cast_media_session_id.lock().await = Some(1);

        player.stop().await;
        assert_timeout!(
            Duration::from_millis(250),
            player.state().await == PlayerState::Stopped,
            "expected the player to be stopped"
        );

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(session_id, result);
    }

    #[tokio::test]
    async fn test_player_handle_event_message() {
        init_logger!();
        let original_url = "http://localhost:9876/my-video.mp4";
        let transcoding_url = "http://localhost:9875/my-transcoded-video.mp4";
        let subtitle_url = "http://localhost:9876/my-subtitle.srt";
        let request = PlayRequest::builder()
            .url(original_url)
            .title("My Video")
            .subtitles_enabled(true)
            .subtitle(Subtitle::new(
                vec![],
                Some(
                    SubtitleInfo::builder()
                        .imdb_id("tt12345678")
                        .language(SubtitleLanguage::English)
                        .build(),
                ),
                "MySubtitleFile.srt".to_string(),
            ))
            .build();
        let response = MediaResponse::NotImplemented(
            MESSAGE_TYPE_ERROR.to_string(),
            serde_json::Value::Object(
                vec![
                    (
                        "detailedErrorCode".to_string(),
                        serde_json::Value::Number(Number::from(104)),
                    ),
                    (
                        "type".to_string(),
                        serde_json::Value::String(MESSAGE_TYPE_ERROR.to_string()),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        );
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_convert()
            .times(2)
            .return_const(Ok(subtitle_url.to_string()));
        let (tx_ready, mut rx_ready) = unbounded_channel();
        let (tx_transcode, mut rx_transcode) = unbounded_channel();
        let mut load_transcoding_url = Some(());
        let mut transcoder = MockTranscoder::new();
        transcoder.expect_transcode().times(1).returning(move |e| {
            tx_transcode.send(e.to_string()).unwrap();
            Ok(TranscodeOutput {
                url: transcoding_url.to_string(),
                output_type: TranscodeType::Live,
            })
        });
        transcoder.expect_stop().times(1).return_const(());
        let mut test_instance = TestInstance::new_player_with_additions(
            Box::new(move || {
                let mut device = create_default_device();
                device
                    .expect_device_status()
                    .return_const(Ok(receiver::Status {
                        request_id: 1,
                        applications: vec![],
                        is_active_input: true,
                        is_stand_by: true,
                        volume: Volume {
                            level: None,
                            muted: None,
                        },
                    }));
                device.expect_launch_app().return_const(Ok(Application {
                    app_id: "MyAppId".to_string(),
                    session_id: "MySessionId".to_string(),
                    transport_id: "MyTransportId".to_string(),
                    namespaces: vec![],
                    display_name: "".to_string(),
                    status_text: "".to_string(),
                }));
                let value = tx_ready.clone();
                device
                    .expect_broadcast_message::<LoadCommand>()
                    .times(2)
                    .returning(move |_, _| {
                        if let None = load_transcoding_url.take() {
                            value.send(()).unwrap();
                        }
                        Ok(())
                    });
                device
                    .expect_play::<String>()
                    .times(2)
                    .return_const(Ok(StatusEntry {
                        media_session_id: 10,
                        media: None,
                        playback_rate: 0.0,
                        player_state: media::PlayerState::Playing,
                        current_item_id: None,
                        loading_item_id: None,
                        preloaded_item_id: None,
                        idle_reason: None,
                        extended_status: None,
                        current_time: None,
                        supported_media_commands: 0,
                    }));
                device
                    .expect_media_status::<String>()
                    .return_const(Ok(media::Status {
                        request_id: 1,
                        entries: vec![],
                    }));
                device
            }),
            Box::new(provider),
            Box::new(transcoder),
        );
        let player = test_instance.player.take().unwrap();

        player.play(request).await;
        player
            .inner
            .handle_event(Ok(ChannelMessage::Media(response)))
            .await;

        let transcode_url = recv_timeout!(&mut rx_transcode, Duration::from_millis(250));
        assert_eq!(original_url, transcode_url);

        recv_timeout!(
            &mut rx_ready,
            Duration::from_millis(250),
            "expected the transcoding url to have been loaded"
        );

        if let Some(request) = player.request().await {
            let request_url = request.url();
            assert_eq!(
                transcoding_url, request_url,
                "expected the request url to be the transcoding live url"
            );
            assert_eq!(
                Some(&MetadataValue::Bool(true)),
                request.metadata().get(METADATA_TRANSCODING)
            );
        } else {
            assert!(false, "expected a PlayRequest to have been present")
        }
    }

    #[tokio::test]
    async fn test_player_handle_event_error() {
        init_logger!();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        player
            .inner
            .handle_event(Err(ChromecastError::Connection("FooBar".to_string())))
            .await;
        let result = player.state().await;

        assert_eq!(PlayerState::Error, result);
    }

    #[tokio::test]
    async fn test_player_start_app_already_running() {
        init_logger!();
        let session_id = "MySessionId123456";
        let transport_id = "MyTransportId";
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = MockFxCastDevice::new();
            default_device_responses(&mut device);
            default_device_status_response(&mut device);
            device
                .expect_device_status()
                .return_const(Ok(receiver::Status {
                    request_id: 1,
                    applications: vec![Application {
                        app_id: CastDeviceApp::DefaultMediaReceiver.to_string().to_string(),
                        session_id: session_id.to_string(),
                        transport_id: transport_id.to_string(),
                        namespaces: vec![],
                        display_name: "Existing default media receiver".to_string(),
                        status_text: "MyExistingDefaultReceiverInstance".to_string(),
                    }],
                    is_active_input: true,
                    is_stand_by: true,
                    volume: Volume {
                        level: None,
                        muted: None,
                    },
                }));
            device.expect_launch_app().times(0).return_const(Err(
                ChromecastError::AppInitializationFailed(
                    "Should not have been invoked".to_string(),
                ),
            ));
            device
        }));
        let player = test_instance.player.take().unwrap();

        let result = player.inner.start_app().await.unwrap();

        assert_eq!(
            CastDeviceApp::DefaultMediaReceiver.to_string(),
            result.app_id
        );
        assert_eq!(session_id.to_string(), result.session_id);
        assert_eq!(transport_id.to_string(), result.transport_id);
    }

    fn create_default_test_instance() -> TestInstance {
        TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            device.expect_ping().return_const(Ok(()));
            device
        }))
    }

    fn create_default_device() -> MockFxCastDevice {
        let mut device = MockFxCastDevice::new();
        default_device_responses(&mut device);
        device
    }

    fn default_device_responses(device: &mut MockFxCastDevice) {
        device.expect_connect::<&str>().return_const(Ok(()));
        device.expect_connect::<String>().return_const(Ok(()));
        device.expect_ping().return_const(Ok(()));
    }

    fn default_device_status_response(device: &mut MockFxCastDevice) {
        device
            .expect_media_status::<String>()
            .times(1..)
            .return_const(Ok(Status {
                request_id: 0,
                entries: vec![StatusEntry {
                    media_session_id: 0,
                    media: None,
                    playback_rate: 1.0,
                    player_state: media::PlayerState::Playing,
                    current_item_id: None,
                    loading_item_id: None,
                    preloaded_item_id: None,
                    idle_reason: None,
                    extended_status: None,
                    current_time: Some(1.0),
                    supported_media_commands: 0,
                }],
            }));
    }

    fn status_entry(state: media::PlayerState) -> StatusEntry {
        StatusEntry {
            media_session_id: 0,
            media: None,
            playback_rate: 0.0,
            player_state: state,
            current_item_id: None,
            loading_item_id: None,
            preloaded_item_id: None,
            idle_reason: None,
            extended_status: None,
            current_time: None,
            supported_media_commands: 0,
        }
    }
}
