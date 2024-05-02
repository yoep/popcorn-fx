use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Weak};
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace, warn};
use rust_cast::{ChannelMessage, channels};
use rust_cast::channels::media;
use rust_cast::channels::media::{MediaResponse, StatusEntry};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use tokio::{runtime, time};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleType};
use popcorn_fx_core::core::subtitles::SubtitleServer;

use crate::chromecast;
use crate::chromecast::{ChromecastError, Image, LoadCommand, Media, MediaDetailedErrorCode, MediaError, Metadata, MovieMetadata, StreamType, TextTrackEdgeType, TextTrackStyle, TextTrackType, Track, TrackType};
use crate::chromecast::device::{CastDeviceEvent, DefaultCastDevice, FxCastDevice};
use crate::chromecast::transcode::{NoOpTranscoder, Transcoder};

const GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-chromecast-icon.png");
const DESCRIPTION: &str =
    "Chromecast streaming media device which allows the playback of videos on your TV.";
const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
const MEDIA_CHANNEL_NAMESPACE: &str = "urn:x-cast:com.google.cast.media";
const SUBTITLE_CONTENT_TYPE: &str = "text/vtt";
const MESSAGE_TYPE_ERROR: &str = "ERROR";

/// The Chromecast player allows the playback of media items on a specific Chromecast device.
#[derive(Debug, Display)]
#[display(fmt = "Chromecast player {}", "self.name()")]
pub struct ChromecastPlayer<D: FxCastDevice + 'static> {
    inner: Arc<InnerChromecastPlayer<D>>,
}

impl ChromecastPlayer<DefaultCastDevice> {
    pub async fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        cast_model: impl Into<String>,
        cast_address: impl Into<String>,
        cast_port: u16,
        subtitle_server: Arc<SubtitleServer>,
        transcoder: Arc<Box<dyn Transcoder>>,
        runtime: Runtime,
    ) -> chromecast::Result<Self> {
        let cast_address = cast_address.into();
        let device = DefaultCastDevice::new(
            cast_address.clone(),
            cast_port.clone(),
            DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
            &runtime).await?;

        Self::new_with_device(
            id,
            name,
            cast_model,
            cast_address,
            cast_port,
            device,
            subtitle_server,
            transcoder,
            runtime,
        )
    }
}

impl<D: FxCastDevice> ChromecastPlayer<D> {
    pub fn new_with_device(
        id: impl Into<String>,
        name: impl Into<String>,
        cast_model: impl Into<String>,
        cast_address: impl Into<String>,
        cast_port: u16,
        cast_device: D,
        subtitle_server: Arc<SubtitleServer>,
        transcoder: Arc<Box<dyn Transcoder>>,
        runtime: Runtime,
    ) -> chromecast::Result<Self> {
        let name = name.into();
        let cost_model = cast_model.into();
        let cast_address = cast_address.into();
        let (tx_events, rx_events) = channel(10);

        debug!("Connected to Chromecast device {} on {}:{}", name, cast_address, cast_port);
        if let Err(e) = block_in_place(cast_device.ping()) {
            return Err(ChromecastError::Connection(e.to_string()));
        }

        trace!("Subscribing to Chromecast {} device events", name);
        cast_device.subscribe(Box::new(move |e| block_in_place(tx_events.send(e)).unwrap()));
        cast_device.start_heartbeat_loop(&runtime);

        trace!("Creating new Chromecast player for {}", name);
        let inner = Arc::new(InnerChromecastPlayer {
            id: id.into(),
            name,
            cast_model: cost_model,
            request: Default::default(),
            state: Mutex::new(PlayerState::Ready),
            cast_device,
            cast_app: Default::default(),
            cast_media_session_id: Default::default(),
            subtitle_server,
            transcoder,
            callbacks: Default::default(),
            status_check_token: Default::default(),
            shutdown_token: Default::default(),
            runtime,
        });

        let event_instance = inner.clone();
        let cancellation_token = event_instance.shutdown_token.clone();
        inner.runtime.spawn(Self::start_device_events(
            event_instance,
            rx_events,
            cancellation_token,
        ));

        Ok(Self {
            inner,
        })
    }

    pub fn builder() -> ChromecastPlayerBuilder {
        ChromecastPlayerBuilder::builder()
    }

    async fn start_device_events(
        inner: Arc<InnerChromecastPlayer<D>>,
        mut receiver: Receiver<CastDeviceEvent>,
        cancellation_token: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => break,
                event = receiver.recv() => {
                    match event {
                        Some(e) => inner.handle_event(e).await,
                        None => break,
                    }
                }
            }
        }

        debug!("Chromecast {} device events receiver has been stopped", inner.name);
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

impl<D: FxCastDevice> Callbacks<PlayerEvent> for ChromecastPlayer<D> {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
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

    fn state(&self) -> PlayerState {
        self.inner.state()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = block_in_place(self.inner.request.lock());
        mutex.as_ref().map(|e| Arc::downgrade(e))
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
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

                // serve the chromecast subtitle if one is present
                let subtitle_url: Option<String>;
                if request.subtitles_enabled() {
                    subtitle_url = self.inner.subtitle_url(&request);
                } else {
                    subtitle_url = None;
                }

                if let Err(e) = self.inner.load(&app, &request, subtitle_url).await {
                    error!("Failed to load Chromecast media, {}", e);
                    self.inner.update_state_async(PlayerState::Error).await;
                    return;
                }

                debug!("Starting Chromecast {} playback", self.name());
                let token = self.inner.generate_status_token().await;
                self.inner
                    .runtime
                    .spawn(Self::start_status_updates(self.inner.clone(), token));
                self.inner.resume().await;

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
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            inner.pause().await;
        });
    }

    fn resume(&self) {
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            inner.resume().await;
        });
    }

    fn seek(&self, time: u64) {
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            inner.seek(time).await;
        });
    }

    fn stop(&self) {
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            inner.stop().await;
        });
    }
}

impl<D: FxCastDevice> Drop for ChromecastPlayer<D> {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        self.inner.shutdown_token.cancel();
        self.stop();
    }
}

/// A builder for creating a new `ChromecastPlayer` instance.
pub struct ChromecastPlayerBuilder {
    id: Option<String>,
    name: Option<String>,
    cast_model: Option<String>,
    cast_address: Option<String>,
    cast_port: Option<u16>,
    subtitle_server: Option<Arc<SubtitleServer>>,
    transcoder: Option<Arc<Box<dyn Transcoder>>>,
    heartbeat_seconds: Option<u64>,
    runtime: Option<Runtime>,
}

impl ChromecastPlayerBuilder {
    /// Creates a new empty `ChromecastPlayerBuilder`.
    pub fn builder() -> Self {
        Self {
            id: None,
            name: None,
            cast_model: None,
            cast_address: None,
            cast_port: None,
            subtitle_server: None,
            transcoder: None,
            heartbeat_seconds: None,
            runtime: None,
        }
    }

    /// Sets the ID of the Chromecast device.
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the name of the Chromecast device.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the model of the Chromecast device.
    pub fn cast_model<S: Into<String>>(mut self, cast_model: S) -> Self {
        self.cast_model = Some(cast_model.into());
        self
    }

    /// Sets the address of the Chromecast device.
    pub fn cast_address<S: Into<String>>(mut self, cast_address: S) -> Self {
        self.cast_address = Some(cast_address.into());
        self
    }

    /// Sets the port of the Chromecast device.
    pub fn cast_port(mut self, cast_port: u16) -> Self {
        self.cast_port = Some(cast_port);
        self
    }

    /// Sets the subtitle server.
    pub fn subtitle_server(mut self, subtitle_server: Arc<SubtitleServer>) -> Self {
        self.subtitle_server = Some(subtitle_server);
        self
    }

    /// Sets the transcoder for media playback.
    pub fn transcoder(mut self, transcoder: Arc<Box<dyn Transcoder>>) -> Self {
        self.transcoder = Some(transcoder);
        self
    }

    /// Sets the heartbeat interval in seconds.
    pub fn heartbeat_seconds(mut self, heartbeat_seconds: u64) -> Self {
        self.heartbeat_seconds = Some(heartbeat_seconds);
        self
    }

    pub fn runtime(mut self, runtime: Runtime) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Builds the `ChromecastPlayer` instance.
    pub async fn build(self) -> chromecast::Result<ChromecastPlayer<DefaultCastDevice>> {
        let id = self.id.expect("expected an id to be set");
        let name = self.name.expect("expected a name to be set");
        let cast_model = self.cast_model.expect("expected a cast model to be set");
        let cast_address = self
            .cast_address
            .expect("expected a cast address to be set");
        let cast_port = self.cast_port.expect("expected a cast port to be set");
        let runtime = self.runtime.unwrap_or_else(|| {
            runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(10)
                .thread_name(format!("chromecast-{}", name))
                .build()
                .expect("expected a new runtime")
        });
        let subtitle_server = self.subtitle_server.expect("expected a subtitle server to have been set");
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
            subtitle_server,
            transcoder,
            runtime,
        ).await
    }
}

struct InnerChromecastPlayer<D: FxCastDevice> {
    id: String,
    name: String,
    request: Mutex<Option<Arc<Box<dyn PlayRequest>>>>,
    state: Mutex<PlayerState>,
    cast_model: String,
    cast_device: D,
    cast_app: Mutex<Option<Application>>,
    cast_media_session_id: Mutex<Option<i32>>,
    subtitle_server: Arc<SubtitleServer>,
    transcoder: Arc<Box<dyn Transcoder>>,
    callbacks: CoreCallbacks<PlayerEvent>,
    status_check_token: Mutex<CancellationToken>,
    shutdown_token: CancellationToken,
    runtime: Runtime,
}

impl<D: FxCastDevice> InnerChromecastPlayer<D> {
    fn state(&self) -> PlayerState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
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
        let app = CastDeviceApp::DefaultMediaReceiver;
        let mut mutex = self.cast_app.lock().await;

        match mutex.clone() {
            None => {
                let cast_device = &self.cast_device;

                // verify if the default app is already running
                // if so, we use the existing app information instead of launching a new app
                trace!("Retrieving Chromecast {} device status", self.name);
                match cast_device.device_status().await {
                    Ok(status) => {
                        if let Some(app) = status.applications.into_iter()
                            .find(|e| e.app_id == CastDeviceApp::DefaultMediaReceiver.to_string())
                        {
                            debug!("Chromecast default media receiver app is already running");
                            *mutex = Some(app.clone());
                            return Ok(app);
                        } else {
                            trace!("Chromecast default media receiver app is not yet running");
                        }
                    }
                    Err(e) => error!("Failed to retrieve Chromecast {} device status, {}", self.name, e),
                }

                trace!("Launching Chromecast {} application {:?}", self.name, app);
                return match cast_device.launch_app(&app).await {
                    Ok(app) => {
                        trace!("Connecting to the application transport id {:?}", app.transport_id);
                        cast_device.connect(app.transport_id.clone()).await
                            .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))?;

                        debug!("Chromecast {} application {:?} has been launched", self.name, app);
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
                        let request = Arc::new(Box::new(TranscodingPlayRequest {
                            url: output.url,
                            request,
                        }) as Box<dyn PlayRequest>);

                        // serve the chromecast subtitle if one is present
                        let subtitle_url: Option<String>;
                        if request.subtitles_enabled() {
                            subtitle_url = self.subtitle_url(&request);
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
        let mutex = self.cast_app.lock().await;
        let cast_device = &self.cast_device;

        if let Some(app) = mutex.as_ref() {
            trace!("Connecting to chromecast app {:?}", app);
            cast_device.connect(app.transport_id.to_string()).await?;

            debug!("Connected to chromecast app {:?}", app);
            return Ok(());
        }

        Err(ChromecastError::AppNotInitialized)
    }

    async fn load(&self,
                  app: &Application,
                  request: &Box<dyn PlayRequest>,
                  subtitle_url: Option<String>) -> chromecast::Result<()> {
        let cast_device = &self.cast_device;
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
        if let Err(e) = cast_device.broadcast_message(MEDIA_CHANNEL_NAMESPACE, &load).await
        {
            return Err(ChromecastError::AppInitializationFailed(e.to_string()));
        }

        Ok(())
    }

    async fn stop_app(&self) -> chromecast::Result<()> {
        trace!("Trying to stop the Chromecast running application");
        let mut mutex = self.cast_app.lock().await;
        let cast_device = &self.cast_device;

        if let Some(app) = mutex.take() {
            let app_id = app.app_id.clone();
            trace!("Stopping chromecast {} app {:?}", self.name, app);
            cast_device.stop_app(app.session_id).await?;
            debug!("Stopped chromecast {} app {}", self.name, app_id);
        }

        Ok(())
    }

    async fn pause(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                match self.cast_device.pause(app.transport_id.to_string(), media_session_id.clone()).await {
                    Ok(status) => self.on_player_state_changed(&status).await,
                    Err(e) => error!("Failed to pause Chromecast {}, {}", self.name, e),
                }
            } else {
                warn!("Unable to pause Chromecast {}, media session id is unknown", self.name);
            }
        } else {
            warn!("Unable to pause Chromecast {}, app id is unknown", self.name);
        }
    }

    async fn resume(&self) {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            if let Some(media_session_id) = self.cast_media_session_id.lock().await.as_ref() {
                match self.cast_device.play(app.transport_id.to_string(), media_session_id.clone()).await
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
                let chromecast_time = Self::parse_to_chromecast_time(time);
                if let Err(e) = self.cast_device.seek(
                    app.transport_id.to_string(),
                    media_session_id.clone(),
                    Some(chromecast_time),
                    None,
                ).await {
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

        // stop the transcoder
        let state = self.state();
        if state != PlayerState::Stopped && state != PlayerState::Ready {
            self.transcoder.stop().await;
        }

        // stop the chromecast app
        if let Err(e) = self.stop_app().await {
            error!("Failed to stop Chromecast playback, {}", e);
            self.update_state_async(PlayerState::Error).await;
        } else {
            debug!("Chromecast device {} has been stopped", self.name);
            self.update_state_async(PlayerState::Stopped).await;
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

    async fn status(&self) -> chromecast::Result<media::Status> {
        if let Some(app) = self.cast_app.lock().await.as_ref() {
            trace!("Requesting Chromecast {} status info", self.name);
            return self.cast_device.media_status(app.transport_id.to_string(), None).await;
        }

        Err(ChromecastError::AppNotInitialized)
    }

    async fn handle_status_update(&self, status: media::Status) {
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
            channels::media::PlayerState::Idle => {
                self.update_state_async(PlayerState::Ready).await
            }
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
    fn subtitle_url(&self, request: &Box<dyn PlayRequest>) -> Option<String> {
        request.subtitle()
            .map(|e| e.clone())
            .and_then(|e| {
                match self.subtitle_server.serve(e, SubtitleType::Vtt) {
                    Ok(e) => Some(e),
                    Err(e) => {
                        error!("Failed to serve subtitle, {}", e);
                        None
                    }
                }
            })
    }

    async fn handle_event(&self, event: CastDeviceEvent) {
        trace!("Handling Chromecast {} event {:?}", self.name, event);
        match event {
            CastDeviceEvent::Message(e) => {
                match e {
                    ChannelMessage::Media(response) => self.handle_media_event(response).await,
                    _ => {}
                }
            }
            CastDeviceEvent::Error(e) => {
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

    async fn handle_media_error(&self, error: MediaError) {
        debug!("Handling media error {:?}", error);
        if error.detailed_error_code == MediaDetailedErrorCode::MediaSrcNotSupported {
            let mut is_transcoding_request: Option<bool> = None;

            {
                let mutex = self.request.lock().await;
                if let Some(request) = mutex.as_ref() {
                    is_transcoding_request = Some(request.downcast_ref::<TranscodingPlayRequest>().is_some());
                }
            }

            // verify if we have some information known about the play request
            // if not, the error that occurred was no longer related to this Chromecast playback
            if let Some(is_transcoding) = is_transcoding_request {
                // prevent that we transcode the already transcoding media
                if !is_transcoding {
                    warn!("Media source is not supported by the Chromecast device, starting transcoding of the media");
                    self.start_transcoding().await;
                } else {
                    warn!("Chromecast device failed to play transcoding media");
                }
            }
        } else {
            error!("Received media error {:?}", error);
            self.update_state_async(PlayerState::Error).await
        }
    }

    fn request_to_media_payload(request: &Box<dyn PlayRequest>, subtitle_url: Option<String>) -> Media {
        let mut images: Vec<Image> = Vec::new();
        let stream_type = Self::parse_stream_type_from_request(request);
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
            stream_type,
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
            tracks: subtitle_url.map(|e| vec![Track {
                track_id: 0,
                track_type: TrackType::Text,
                track_content_id: e.to_string(),
                track_content_type: SUBTITLE_CONTENT_TYPE.to_string(),
                subtype: TextTrackType::Subtitles,
                language: "en".to_string(),
                name: "English".to_string(),
            }]),
        }
    }

    fn create_media_subtitle(request: &Box<dyn PlayRequest>) -> String {
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

    fn parse_stream_type_from_request(request: &Box<dyn PlayRequest>) -> StreamType {
        if request.downcast_ref::<TranscodingPlayRequest>().is_some() {
            StreamType::Live
        } else {
            StreamType::Buffered
        }
    }
}

impl<D: FxCastDevice> Callbacks<PlayerEvent> for InnerChromecastPlayer<D> {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
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
            .field("cast_app", &self.cast_app)
            .field("callbacks", &self.callbacks)
            .field("cancellation_token", &self.shutdown_token)
            .finish()
    }
}

#[derive(Debug, Display)]
#[display(fmt = "Transcoding play request for {}", url)]
struct TranscodingPlayRequest {
    pub url: String,
    pub request: Arc<Box<dyn PlayRequest>>,
}

impl PlayRequest for TranscodingPlayRequest {
    fn url(&self) -> &str {
        self.url.as_str()
    }

    fn title(&self) -> &str {
        self.request.title()
    }

    fn caption(&self) -> Option<String> {
        self.request.caption()
    }

    fn thumbnail(&self) -> Option<String> {
        self.request.thumbnail()
    }

    fn background(&self) -> Option<String> {
        self.request.background()
    }

    fn quality(&self) -> Option<String> {
        self.request.quality()
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        if let Some(_) = self.request.auto_resume_timestamp() {
            warn!("Auto resume timestamps are not supported for live transcoding media playbacks");
        }

        // the auto resume timestamp is not supported for media when it's being transcoded
        // this will otherwise require us to support seeking within live transcodings
        None
    }

    fn subtitles_enabled(&self) -> bool {
        self.request.subtitles_enabled()
    }

    fn subtitle(&self) -> Option<&Subtitle> {
        self.request.subtitle()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use rust_cast::channels::{media, receiver};
    use rust_cast::channels::media::StatusEntry;
    use rust_cast::channels::receiver::Volume;
    use serde_json::Number;

    use popcorn_fx_core::assert_timeout_eq;
    use popcorn_fx_core::core::Handle;
    use popcorn_fx_core::core::media::MovieOverview;
    use popcorn_fx_core::core::players::{PlayMediaRequest, PlayUrlRequest};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
    use popcorn_fx_core::testing::init_logger;

    use crate::chromecast::device::MockFxCastDevice;
    use crate::chromecast::tests::TestInstance;
    use crate::chromecast::transcode::{MockTranscoder, TranscodeOutput, TranscodeType};

    use super::*;

    #[test]
    fn test_player_new() {
        init_logger();
        let subtitle_provider = MockSubtitleProvider::new();
        let transcoder = MockTranscoder::new();
        let mut device = MockFxCastDevice::new();
        device.expect_connect::<&str>()
            .return_const(Ok(()));
        device.expect_ping()
            .return_const(Ok(()));
        device.expect_subscribe()
            .times(1)
            .return_const(Handle::new());
        device.expect_start_heartbeat_loop()
            .return_const(());
        let runtime = Runtime::new().unwrap();

        let result = ChromecastPlayer::new_with_device(
            "MyChromecastId",
            "MyChromecastName",
            "MyChromecastModel",
            "127.0.0.1",
            9870,
            device,
            Arc::new(SubtitleServer::new(&Arc::new(Box::new(subtitle_provider)))),
            Arc::new(Box::new(transcoder)),
            runtime,
        );

        if let Ok(_) = result {} else {
            assert!(false, "expected a new player, but got {:?} instead", result);
        }
    }

    #[test]
    fn test_player_id() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        let result = player.id();

        assert_eq!("MyChromecastId", result);
    }

    #[test]
    fn test_player_name() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        let result = player.name();

        assert_eq!("MyChromecastName", result);
    }

    #[test]
    fn test_player_description() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        let result = player.description();

        assert_eq!(DESCRIPTION, result);
    }

    #[test]
    fn test_player_graphic_resource() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        let result = player.graphic_resource();

        assert_eq!(GRAPHIC_RESOURCE.to_vec(), result);
    }

    #[test]
    fn test_player_state() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        let result = player.state();

        assert_eq!(PlayerState::Ready, result);
    }

    #[test]
    fn test_player_play() {
        init_logger();
        let url = "http://localhost:8900/my-video.mkv";
        let (tx_command, rx_command) = channel::<LoadCommand>();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = MockFxCastDevice::new();
            default_device_responses(&mut device);
            device.expect_launch_app()
                .return_const(Ok(Application {
                    app_id: "MyAppId".to_string(),
                    session_id: "MySessionId".to_string(),
                    transport_id: "MyTransportId".to_string(),
                    namespaces: vec![],
                    display_name: "".to_string(),
                    status_text: "".to_string(),
                }));
            let sender = tx_command.clone();
            device.expect_broadcast_message::<LoadCommand>()
                .returning(move |_namespace, command| {
                    sender.send(command.clone()).unwrap();
                    Ok(())
                });
            device.expect_play::<String>()
                .return_const(Ok(StatusEntry {
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
            device.expect_device_status()
                .return_const(Ok(receiver::Status {
                    request_id: 1,
                    applications: vec![],
                    is_active_input: true,
                    is_stand_by: true,
                    volume: Volume { level: None, muted: None },
                }));
            device
        }));
        let movie = MovieOverview {
            title: "MyMovie".to_string(),
            imdb_id: "tt011000".to_string(),
            year: "1028".to_string(),
            rating: None,
            images: Default::default(),
        };
        let request = Box::new(PlayMediaRequest {
            base: PlayUrlRequest {
                url: url.to_string(),
                title: "FooBar".to_string(),
                caption: Some("MyCaption".to_string()),
                thumb: Some("http://localhost/my-thumb.png".to_string()),
                background: Some("http://localhost/my-background.png".to_string()),
                auto_resume_timestamp: Some(28000),
                subtitles_enabled: true,
                subtitle: None,
            },
            parent_media: None,
            media: Box::new(movie),
            quality: "720p".to_string(),
            torrent_stream: Default::default(),
        });
        let (tx, rx) = channel();
        let player = test_instance.player.take().unwrap();

        player.add(Box::new(move |event| {
            if let PlayerEvent::StateChanged(state) = event {
                tx.send(state).unwrap();
            }
        }));
        test_instance.runtime.block_on(player.play(request));

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlayerState::Loading, result);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlayerState::Playing, result);

        let command = rx_command.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(url.to_string(), command.media.url);
    }

    #[test]
    fn test_player_pause() {
        init_logger();
        let transport_id = "FooBar";
        let (tx, rx) = channel();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device.expect_subscribe()
                .return_const(CallbackHandle::new());
            device.expect_pause::<String>()
                .times(1)
                .returning(move |destination, _| {
                    sender.send(destination).unwrap();
                    Ok(status_entry(media::PlayerState::Paused))
                });
            device.expect_stop_app::<String>()
                .return_const(Ok(()));
            device
        }));
        let player = test_instance.player.take().unwrap();

        *block_in_place(player.inner.cast_app.lock()) = Some(Application {
            app_id: "MyAppId".to_string(),
            session_id: "MySessionId".to_string(),
            transport_id: transport_id.to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *block_in_place(player.inner.cast_media_session_id.lock()) = Some(1);

        player.pause();
        assert_timeout_eq!(Duration::from_millis(200), PlayerState::Paused, player.state());

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(transport_id.to_string(), result);
    }

    #[test]
    fn test_player_resume() {
        init_logger();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            device.expect_subscribe()
                .return_const(CallbackHandle::new());
            device.expect_play::<String>()
                .times(1)
                .returning(move |_, _| {
                    Ok(status_entry(media::PlayerState::Playing))
                });
            device.expect_stop_app::<String>()
                .return_const(Ok(()));
            device
        }));
        let player = test_instance.player.take().unwrap();

        *block_in_place(player.inner.cast_app.lock()) = Some(Application {
            app_id: "Foo".to_string(),
            session_id: "Bar".to_string(),
            transport_id: "Transport21".to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *block_in_place(player.inner.cast_media_session_id.lock()) = Some(1);
        
        player.resume();
        
        assert_timeout_eq!(Duration::from_millis(200), PlayerState::Playing, player.state());
    }

    #[test]
    fn test_player_seek() {
        init_logger();
        let transport_id = "LoremIpsum";
        let (tx, rx) = channel();
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device.expect_subscribe()
                .return_const(CallbackHandle::new());
            device.expect_seek::<String>()
                .times(1)
                .returning(move |_, _, time, _| {
                    sender.send(time).unwrap();
                    Ok(status_entry(media::PlayerState::Playing))
                });
            device.expect_stop_app::<String>()
                .return_const(Ok(()));
            device
        }));
        let player = test_instance.player.take().unwrap();

        *block_in_place(player.inner.cast_app.lock()) = Some(Application {
            app_id: "Foo".to_string(),
            session_id: "Bar".to_string(),
            transport_id: transport_id.to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        *block_in_place(player.inner.cast_media_session_id.lock()) = Some(1);

        player.seek(14000);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(Some(14f32), result);
    }

    #[test]
    fn test_player_stop() {
        init_logger();
        let session_id = "Bar";
        let (tx, rx) = channel();
        let subtitle_provider = MockSubtitleProvider::new();
        let mut transcoder = MockTranscoder::new();
        transcoder.expect_stop()
            .times(1)
            .return_const(());
        let mut test_instance = TestInstance::new_player_with_additions(Box::new(move || {
            let mut device = create_default_device();
            let sender = tx.clone();
            device.expect_stop_app::<String>()
                .times(1)
                .returning(move |session_id| {
                    sender.send(session_id).unwrap();
                    Ok(())
                });
            device.expect_subscribe()
                .return_const(CallbackHandle::new());
            device
        }), Box::new(subtitle_provider), Box::new(transcoder));
        let player = test_instance.player.take().unwrap();

        // make sure the player state is not stopped or ready for stopping the transcoder
        *block_in_place(player.inner.state.lock()) = PlayerState::Playing;
        // make sure we have an app ID present to invoke the stop command
        *block_in_place(player.inner.cast_app.lock()) = Some(Application {
            app_id: "Foo".to_string(),
            session_id: session_id.to_string(),
            transport_id: "Dolor".to_string(),
            namespaces: vec![],
            display_name: "".to_string(),
            status_text: "".to_string(),
        });
        // make sure we have a session ID present to invoke the stop command
        *block_in_place(player.inner.cast_media_session_id.lock()) = Some(1);

        player.stop();
        assert_timeout_eq!(Duration::from_millis(200), PlayerState::Stopped, player.state());

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(session_id, result);
    }

    #[test]
    fn test_player_handle_event_message() {
        init_logger();
        let original_url = "http://localhost:9876/my-video.mp4";
        let transcoding_url = "http://localhost:9875/my-transcoded-video.mp4";
        let subtitle_url = "http://localhost:9876/my-subtitle.srt";
        let request = Box::new(PlayUrlRequest::builder()
            .url(original_url)
            .title("My Video")
            .subtitles_enabled(true)
            .subtitle(Subtitle::new(
                vec![],
                Some(SubtitleInfo::builder()
                    .imdb_id("tt12345678")
                    .language(SubtitleLanguage::English)
                    .build()),
                "MySubtitleFile.srt".to_string(),
            ))
            .build());
        let response = MediaResponse::NotImplemented(MESSAGE_TYPE_ERROR.to_string(), serde_json::Value::Object(vec![
            ("detailedErrorCode".to_string(), serde_json::Value::Number(Number::from(104))),
            ("type".to_string(), serde_json::Value::String(MESSAGE_TYPE_ERROR.to_string())),
        ].into_iter().collect()));
        let mut provider = MockSubtitleProvider::new();
        provider.expect_convert()
            .times(2)
            .return_const(Ok(subtitle_url.to_string()));
        let (tx, rx) = channel();
        let mut transcoder = MockTranscoder::new();
        transcoder.expect_transcode()
            .times(1)
            .returning(move |e| {
                tx.send(e.to_string()).unwrap();
                Ok(TranscodeOutput {
                    url: transcoding_url.to_string(),
                    output_type: TranscodeType::Live,
                })
            });
        transcoder.expect_stop()
            .times(1)
            .return_const(());
        let mut test_instance = TestInstance::new_player_with_additions(Box::new(move || {
            let mut device = create_default_device();
            device.expect_device_status()
                .return_const(Ok(receiver::Status {
                    request_id: 1,
                    applications: vec![],
                    is_active_input: true,
                    is_stand_by: true,
                    volume: Volume { level: None, muted: None },
                }));
            device.expect_launch_app()
                .return_const(Ok(Application {
                    app_id: "MyAppId".to_string(),
                    session_id: "MySessionId".to_string(),
                    transport_id: "MyTransportId".to_string(),
                    namespaces: vec![],
                    display_name: "".to_string(),
                    status_text: "".to_string(),
                }));
            device.expect_broadcast_message::<LoadCommand>()
                .times(2)
                .return_const(Ok(()));
            device.expect_play::<String>()
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
            device.expect_media_status::<String>()
                .return_const(Ok(media::Status {
                    request_id: 1,
                    entries: vec![],
                }));
            device
        }), Box::new(provider), Box::new(transcoder));
        let player = test_instance.player.take().unwrap();
        let runtime = test_instance.runtime.clone();

        runtime.block_on(player.play(request));
        runtime.block_on(player.inner.handle_event(CastDeviceEvent::Message(ChannelMessage::Media(response))));

        let transcode_url = rx.recv_timeout(Duration::from_millis(250)).unwrap();
        assert_eq!(original_url, transcode_url);

        let request_url = player.request()
            .and_then(|e| e.upgrade())
            .map(|e| e.url().to_string())
            .unwrap();
        assert_eq!(transcoding_url.to_string(), request_url, "expected the request url to be the transcoding live url");
    }

    #[test]
    fn test_player_handle_event_error() {
        init_logger();
        let mut test_instance = create_default_test_instance();
        let player = test_instance.player.take().unwrap();

        test_instance.runtime.block_on(player.inner.handle_event(CastDeviceEvent::Error("FooBar".to_string())));
        let result = player.state();

        assert_eq!(PlayerState::Error, result);
    }

    #[test]
    fn test_player_start_app_already_running() {
        init_logger();
        let session_id = "MySessionId123456";
        let transport_id = "MyTransportId";
        let mut test_instance = TestInstance::new_player(Box::new(move || {
            let mut device = MockFxCastDevice::new();
            default_device_responses(&mut device);
            default_device_status_response(&mut device);
            device.expect_device_status()
                .return_const(Ok(receiver::Status {
                    request_id: 1,
                    applications: vec![
                        Application {
                            app_id: CastDeviceApp::DefaultMediaReceiver.to_string().to_string(),
                            session_id: session_id.to_string(),
                            transport_id: transport_id.to_string(),
                            namespaces: vec![],
                            display_name: "Existing default media receiver".to_string(),
                            status_text: "MyExistingDefaultReceiverInstance".to_string(),
                        }
                    ],
                    is_active_input: true,
                    is_stand_by: true,
                    volume: Volume { level: None, muted: None },
                }));
            device.expect_launch_app()
                .times(0)
                .return_const(Err(ChromecastError::AppInitializationFailed("Should not have been invoked".to_string())));
            device
        }));
        let player = test_instance.player.take().unwrap();

        let result = test_instance.runtime.block_on(player.inner.start_app()).unwrap();

        assert_eq!(CastDeviceApp::DefaultMediaReceiver.to_string(), result.app_id);
        assert_eq!(session_id.to_string(), result.session_id);
        assert_eq!(transport_id.to_string(), result.transport_id);
    }

    fn create_default_test_instance() -> TestInstance {
        TestInstance::new_player(Box::new(move || {
            let mut device = create_default_device();
            device.expect_subscribe()
                .return_const(CallbackHandle::new());
            device.expect_start_heartbeat_loop()
                .return_const(());
            device
        }))
    }

    fn create_default_device() -> MockFxCastDevice {
        let mut device = MockFxCastDevice::new();
        default_device_responses(&mut device);
        device
    }

    fn default_device_responses(device: &mut MockFxCastDevice) {
        device.expect_connect::<&str>()
            .return_const(Ok(()));
        device.expect_connect::<String>()
            .return_const(Ok(()));
        device.expect_ping()
            .return_const(Ok(()));
        device.expect_start_heartbeat_loop()
            .return_const(());
        device.expect_subscribe()
            .return_const(CallbackHandle::new());
    }

    fn default_device_status_response(device: &mut MockFxCastDevice) {
        device.expect_media_status::<String>()
            .times(1..)
            .return_const(Ok(media::Status {
                request_id: 0,
                entries: vec![
                    StatusEntry {
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
                    }
                ],
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
