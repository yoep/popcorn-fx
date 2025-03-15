use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Weak};
use std::time::Duration;

use async_trait::async_trait;
use chbs::config::BasicConfigBuilder;
use chbs::prelude::ToScheme;
use chbs::probability::Probability;
use chbs::word::{WordList, WordSampler};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder, Error, Response};
use serde_xml_rs::from_str;
use thiserror::Error;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use url::Url;

use popcorn_fx_core::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider};

use crate::vlc::VlcStatus;

pub const VLC_ID: &str = "vlc";
const VLC_GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-vlc-icon.png");
const VLC_DESCRIPTION: &str = "VLC is a free and open source cross-platform multimedia player";
const VLC_HOST: &str = "localhost";
const STATUS_URI: &str = "/requests/status.xml";
const COMMAND_NAME_PARAM: &str = "command";
const COMMAND_VALUE_PARAM: &str = "val";
const COMMAND_PLAY_PAUSE: &str = "pl_pause";
const COMMAND_STOP: &str = "pl_stop";
const COMMAND_SEEK: &str = "seek";
const COMMAND_VOLUME: &str = "volume";

/// Represents an external VLC player instance.
#[derive(Debug, Display)]
#[display(fmt = "VLC player")]
pub struct VlcPlayer {
    inner: Arc<InnerVlcPlayer>,
}

impl VlcPlayer {
    pub fn builder() -> VlcPlayerBuilder {
        VlcPlayerBuilder::builder()
    }
}

#[async_trait]
impl Player for VlcPlayer {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn graphic_resource(&self) -> Vec<u8> {
        self.inner.graphic_resource()
    }

    async fn state(&self) -> PlayerState {
        self.inner.state().await
    }

    async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        self.inner.request().await
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        self.inner.play(request).await;
    }

    fn pause(&self) {
        self.inner.send_command(VlcPlayerCommand::Pause)
    }

    fn resume(&self) {
        self.inner.send_command(VlcPlayerCommand::Resume)
    }

    fn seek(&self, time: u64) {
        self.inner.send_command(VlcPlayerCommand::Seek(time))
    }

    fn stop(&self) {
        self.inner.send_command(VlcPlayerCommand::Stop)
    }
}

impl Callback<PlayerEvent> for VlcPlayer {
    fn subscribe(&self) -> Subscription<PlayerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for VlcPlayer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

/// Builder for creating new [VlcPlayer] instances.
///
/// # Example
///
/// ```rust
/// use popcorn_fx_players::vlc::VlcPlayer;
///
/// VlcPlayer::builder()
///     .password("MyPassword")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct VlcPlayerBuilder {
    subtitle_manager: Option<Arc<Box<dyn SubtitleManager>>>,
    subtitle_provider: Option<Arc<Box<dyn SubtitleProvider>>>,
    password: Option<String>,
    address: Option<SocketAddr>,
}

impl VlcPlayerBuilder {
    /// Returns a new instance of `VlcPlayerBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the subtitle manager for the VLC player.
    pub fn subtitle_manager(mut self, subtitle_manager: Arc<Box<dyn SubtitleManager>>) -> Self {
        self.subtitle_manager = Some(subtitle_manager);
        self
    }

    /// Sets the subtitle provider for the VLC player.
    pub fn subtitle_provider(mut self, subtitle_provider: Arc<Box<dyn SubtitleProvider>>) -> Self {
        self.subtitle_provider = Some(subtitle_provider);
        self
    }

    /// Sets the password for the VLC player.
    pub fn password<S>(mut self, password: S) -> Self
    where
        S: Into<String>,
    {
        self.password = Some(password.into());
        self
    }

    /// Sets the address on which the VLC player API will be started.
    pub fn address(mut self, address: SocketAddr) -> Self {
        self.address = Some(address);
        self
    }

    /// Builds the `VlcPlayer` instance.
    pub fn build(self) -> VlcPlayer {
        let address = self.address.unwrap_or_else(|| {
            let listener =
                TcpListener::bind("localhost:0").expect("expected a TCP address to be bound");
            listener.local_addr().expect("expected a valid socket")
        });
        let password = self.password.unwrap_or_else(|| {
            BasicConfigBuilder::<WordSampler>::default()
                .words(5usize)
                .separator("-")
                .word_provider(WordList::default().sampler())
                .capitalize_first(Probability::half())
                .capitalize_words(Probability::Never)
                .build()
                .unwrap()
                .to_scheme()
                .generate()
        });
        let client = ClientBuilder::new()
            .default_headers(
                HeaderMap::try_from(
                    &vec![("User-Agent".to_string(), "popcorn-fx".to_string())]
                        .into_iter()
                        .collect::<HashMap<String, String>>(),
                )
                .expect("expected a valid header map"),
            )
            .build()
            .unwrap();
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerVlcPlayer {
            options: format!(
                "--http-host={} --http-port={} --extraintf=http --http-password={}",
                VLC_HOST,
                address.port(),
                password
            ),
            password,
            client,
            socket: address,
            request: Default::default(),
            process: Default::default(),
            state: Default::default(),
            callbacks: MultiThreadedCallback::new(),
            subtitle_manager: self
                .subtitle_manager
                .expect("exself.inner.send_command(VlcPlayerCommand::Pause)pected the subtitle_manager to have been set"),
            subtitle_provider: self
                .subtitle_provider
                .expect("expected the subtitle_provider to have been set"),
            command_sender,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver).await;
        });

        VlcPlayer { inner }
    }
}

#[derive(Debug, PartialEq)]
enum VlcPlayerCommand {
    /// Pause the vlc player playback
    Pause,
    /// Resume the vlc player playback
    Resume,
    /// Seek the given time millis within the playback
    Seek(u64),
    /// Stop the current vlc player playback
    Stop,
}

#[derive(Debug, Display)]
#[display(fmt = "inner VLC player")]
struct InnerVlcPlayer {
    password: String,
    client: Client,
    socket: SocketAddr,
    options: String,
    request: Mutex<Option<Arc<Box<dyn PlayRequest>>>>,
    process: Mutex<Option<Child>>,
    state: Mutex<PlayerState>,
    callbacks: MultiThreadedCallback<PlayerEvent>,
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
    command_sender: UnboundedSender<VlcPlayerCommand>,
    cancellation_token: CancellationToken,
}

impl InnerVlcPlayer {
    async fn start(&self, mut command_receiver: UnboundedReceiver<VlcPlayerCommand>) {
        let mut status_interval = interval(Duration::from_secs(1));
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                _ = status_interval.tick() => self.check_status().await,
            }
        }
        self.stop().await;
        debug!("Vpl player main loop ended");
    }

    async fn handle_command(&self, command: VlcPlayerCommand) {
        match command {
            VlcPlayerCommand::Pause => self.pause().await,
            VlcPlayerCommand::Resume => self.resume().await,
            VlcPlayerCommand::Seek(time) => self.seek(time).await,
            VlcPlayerCommand::Stop => self.stop().await,
        }
    }

    fn build_uri(&self, params: Vec<(&str, &str)>) -> Url {
        Url::parse_with_params(
            format!("http://{}:{}{}", VLC_HOST, self.socket.port(), STATUS_URI).as_str(),
            params,
        )
        .expect("expected a valid uri to have been created")
    }

    async fn check_status(&self) {
        let state = *self.state.lock().await;
        if matches!(
            state,
            PlayerState::Stopped | PlayerState::Error | PlayerState::Unknown
        ) {
            return;
        }

        trace!("Checking external VLC player status for {:?}", self);
        match self.retrieve_status().await {
            Ok(status) => {
                debug!("Received external VLC status {:?}", status);
                self.update_state_async(PlayerState::from(status.state))
                    .await;
                self.callbacks
                    .invoke(PlayerEvent::TimeChanged(status.time * 1000));
                self.callbacks
                    .invoke(PlayerEvent::DurationChanged(status.length * 1000));
                self.callbacks
                    .invoke(PlayerEvent::VolumeChanged(status.volume));
            }
            Err(e) => {
                warn!("Vlc player failed to retrieve VLC playback status, {}", e);
                self.stop().await;
            }
        }
    }

    async fn retrieve_status(&self) -> Result<VlcStatus, VlcError> {
        let uri = self.build_uri(vec![]);
        debug!("Retrieving status from {}", uri);
        self.execute_request(uri)
            .await
            .map_err(|e| VlcError::Request(e.to_string()))?
            .text()
            .await
            .map_err(|e| VlcError::Request(e.to_string()))
            .and_then(|body| {
                from_str::<VlcStatus>(body.as_str())
                    .map_err(|err| VlcError::Parsing(err.to_string()))
            })
    }

    async fn execute_request(&self, uri: Url) -> Result<Response, Error> {
        self.client
            .get(uri)
            .basic_auth("", Some(self.password.as_str()))
            .timeout(Duration::from_millis(750))
            .send()
            .await
    }

    async fn execute_command(&self, command: VlcCommand) {
        let uri = self.build_uri(command.as_query_params());
        debug!("Exeucting VLC command {}", uri);
        match self.execute_request(uri).await {
            Ok(_) => debug!("VLC command {} has been executed with success", command),
            Err(e) => warn!("Failed to executed VLC command {}, {}", command, e),
        }
    }

    async fn update_state_async(&self, state: PlayerState) {
        let mut mutex = self.state.lock().await;
        if *mutex != state {
            *mutex = state.clone();
        } else {
            return;
        }
        drop(mutex);

        self.callbacks.invoke(PlayerEvent::StateChanged(state));
    }

    fn id(&self) -> &str {
        VLC_ID
    }

    fn name(&self) -> &str {
        "VLC"
    }

    fn description(&self) -> &str {
        VLC_DESCRIPTION
    }

    fn graphic_resource(&self) -> Vec<u8> {
        VLC_GRAPHIC_RESOURCE.to_vec()
    }

    async fn state(&self) -> PlayerState {
        *self.state.lock().await
    }

    async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = self.request.lock().await;
        mutex.as_ref().map(|e| Arc::downgrade(e))
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        trace!("Trying to start VLC playback for {:?}", request);
        let filename = Path::new(request.url())
            .file_name()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string());
        let mut command = Command::new("vlc");

        command.arg(request.url());
        for arg in self.options.as_str().split(" ") {
            command.arg(arg);
        }

        if let Some(subtitle) = request.subtitle().info.as_ref() {
            let matcher = SubtitleMatcher::from_string(filename, request.quality());
            match self.subtitle_provider.download(subtitle, &matcher).await {
                Ok(uri) => {
                    debug!("Adding VLC player subtitle file {}", uri);
                    command.arg(format!("--sub-file={}", uri));
                }
                Err(e) => warn!("Failed to download VLC player subtitle file, {}", e),
            }
        }

        {
            debug!("Launching VLC command {:?}", command);
            let mut mutex = self.process.lock().await;
            *mutex = command
                .spawn()
                .map(|e| {
                    info!("VLC play process has been started");
                    Some(e)
                })
                .map_err(|e| {
                    error!("Failed to spawn VLC process, {}", e);
                    e
                })
                .unwrap_or(None);
        }

        {
            trace!("Updating VLC request to {:?}", request);
            let mut mutex = self.request.lock().await;
            *mutex = Some(Arc::new(request))
        }
    }

    async fn pause(&self) {
        self.execute_command(VlcCommand::builder().name(COMMAND_PLAY_PAUSE).build())
            .await
    }

    async fn resume(&self) {
        self.execute_command(VlcCommand::builder().name(COMMAND_PLAY_PAUSE).build())
            .await
    }

    async fn seek(&self, mut time: u64) {
        if time <= 999 {
            warn!("Invalid seek time {} given", time);
            time = 0;
        } else {
            time = time / 1000;
        }

        self.execute_command(VlcCommand::builder().name(COMMAND_SEEK).value(time).build())
            .await
    }

    async fn stop(&self) {
        debug!("Vlc player is stopping the current playback");
        self.execute_command(VlcCommand::builder().name(COMMAND_STOP).build())
            .await;

        {
            let mut mutex = self.process.lock().await;
            if let Some(mut process) = mutex.take() {
                if let Err(err) = process.kill() {
                    warn!("VLC player failed to stop the VLC process, {}", err);
                }
            }
        }

        self.callbacks
            .invoke(PlayerEvent::StateChanged(PlayerState::Stopped));
    }

    fn send_command(&self, command: VlcPlayerCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Vlv player failed to send command, {}", e);
        }
    }
}

#[derive(Debug, Clone, Error)]
enum VlcError {
    #[error("failed to send request, {0}")]
    Request(String),
    #[error("failed to parse response body, {0}")]
    Parsing(String),
}

#[derive(Debug, Display)]
#[display(fmt = "{}={:?}", name, value)]
struct VlcCommand {
    pub name: String,
    pub value: Option<String>,
}

impl VlcCommand {
    pub fn builder() -> VlcCommandBuilder {
        VlcCommandBuilder::builder()
    }

    pub fn as_query_params(&self) -> Vec<(&str, &str)> {
        let mut query_params = vec![(COMMAND_NAME_PARAM, self.name.as_str())];

        if let Some(value) = self.value.as_ref() {
            query_params.push((COMMAND_VALUE_PARAM, value.as_str()))
        }

        query_params
    }
}

#[derive(Debug, Default)]
struct VlcCommandBuilder {
    name: Option<String>,
    value: Option<String>,
}

impl VlcCommandBuilder {
    fn builder() -> Self {
        Self::default()
    }

    fn name<T: ToString>(mut self, name: T) -> Self {
        self.name = Some(name.to_string());
        self
    }

    fn value<T: ToString>(mut self, value: T) -> Self {
        self.value = Some(value.to_string());
        self
    }

    fn build(self) -> VlcCommand {
        VlcCommand {
            name: self.name.expect("name is not set"),
            value: self.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;

    use popcorn_fx_core::core::players::{MockPlayRequest, PlaySubtitleRequest};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
    use popcorn_fx_core::core::subtitles::{MockSubtitleProvider, SubtitlePreference};
    use popcorn_fx_core::testing::MockSubtitleManager;
    use popcorn_fx_core::{assert_timeout, init_logger, recv_timeout};

    use super::*;

    #[test]
    fn test_id() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        assert_eq!("vlc", player.id());
    }

    #[test]
    fn test_name() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        assert_eq!("VLC", player.name());
    }

    #[test]
    fn test_description() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        assert_eq!(VLC_DESCRIPTION, player.description());
    }

    #[test]
    fn test_graphic_resource() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        assert!(
            player.graphic_resource().len() > 0,
            "expected a graphic resource to have been returned"
        );
    }

    #[tokio::test]
    async fn test_state() {
        init_logger!();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        let result = player.state().await;

        assert_eq!(PlayerState::Unknown, result);
    }

    #[tokio::test]
    async fn test_play() {
        init_logger!();
        let title = "FooBarTitle";
        let language = SubtitleLanguage::Finnish;
        let mut request = MockPlayRequest::new();
        let subtitle_url = "http://localhost:8080/subtitle.srt";
        request
            .expect_url()
            .return_const("http://localhost:8080/myvideo.mp4".to_string());
        request.expect_title().return_const(title.to_string());
        request
            .expect_quality()
            .return_const(Some("720p".to_string()));
        request.expect_subtitle().return_const(PlaySubtitleRequest {
            enabled: true,
            info: Some(
                SubtitleInfo::builder()
                    .imdb_id("tt8976123")
                    .language(language)
                    .build(),
            ),
            subtitle: None,
        });
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference()
            .return_const(SubtitlePreference::Language(language));
        let mut provider = MockSubtitleProvider::new();
        provider
            .expect_download()
            .times(1)
            .return_const(Ok(subtitle_url.to_string()));
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        player.play(Box::new(request)).await;

        let result = player.inner.process.lock().await;
        assert!(
            result.is_some(),
            "expected the VLC process to have been spawned"
        );

        let result = player
            .request()
            .await
            .and_then(|e| e.upgrade())
            .expect("expected the request to have been stored");
        assert_eq!(title.to_string(), result.title());
    }

    #[tokio::test]
    async fn test_stop() {
        init_logger!();
        let server = MockServer::start();
        let mock = server.mock(move |when, then| {
            when.method(GET)
                .path(STATUS_URI)
                .query_param(COMMAND_NAME_PARAM, COMMAND_STOP);
            then.status(200);
        });
        let mut request = MockPlayRequest::new();
        request
            .expect_url()
            .return_const("http://localhost:8080/myvideo.mp4".to_string());
        request.expect_subtitle().return_const(PlaySubtitleRequest {
            enabled: false,
            info: None,
            subtitle: None,
        });
        let mut manager = MockSubtitleManager::new();
        manager
            .expect_preference()
            .return_const(SubtitlePreference::Language(SubtitleLanguage::None));
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.play(Box::new(request)).await;
        player.stop();

        assert_timeout!(
            Duration::from_millis(200),
            player.inner.process.lock().await.is_none(),
            "expected the VLC process to have been killed"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_check_status() {
        init_logger!();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(STATUS_URI);
            then.status(200)
                .header("Content-Type", "application/xml")
                .body(
                    r#"<?xml version="1.0" encoding="utf-8" standalone="yes" ?>
<root>
    <time>1</time>
    <length>6300</length>
    <state>playing</state>
    <volume>256</volume>
</root>"#,
                );
        });
        let (tx_status, mut rx_status) = unbounded_channel();
        let (tx_time, mut rx_time) = unbounded_channel();
        let (tx_duration, mut rx_duration) = unbounded_channel();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        let mut receiver = player.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    PlayerEvent::DurationChanged(e) => tx_duration.send(*e).unwrap(),
                    PlayerEvent::TimeChanged(e) => tx_time.send(*e).unwrap(),
                    PlayerEvent::StateChanged(e) => tx_status.send(*e).unwrap(),
                    _ => {}
                }
            }
        });

        player.inner.check_status().await;

        let result = recv_timeout!(
            &mut rx_status,
            Duration::from_millis(200),
            "expected to receive the player state"
        );
        assert_eq!(PlayerState::Playing, result);

        let result = recv_timeout!(
            &mut rx_time,
            Duration::from_millis(200),
            "expected to receive the player time"
        );
        assert_eq!(1000u64, result);

        let result = recv_timeout!(
            &mut rx_duration,
            Duration::from_millis(200),
            "expected to receive the player duration"
        );
        assert_eq!(6300000u64, result);
    }

    #[test]
    fn test_pause() {
        init_logger!();
        let server = MockServer::start();
        let mock = server.mock(move |when, then| {
            when.method(GET)
                .path(STATUS_URI)
                .query_param(COMMAND_NAME_PARAM, COMMAND_PLAY_PAUSE);
            then.status(200);
        });
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.pause();

        mock.assert();
    }

    #[test]
    fn test_resume() {
        init_logger!();
        let server = MockServer::start();
        let mock = server.mock(move |when, then| {
            when.method(GET)
                .path(STATUS_URI)
                .query_param(COMMAND_NAME_PARAM, COMMAND_PLAY_PAUSE);
            then.status(200);
        });
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.resume();

        mock.assert();
    }

    #[test]
    fn test_seek() {
        init_logger!();
        let server = MockServer::start();
        let mock = server.mock(move |when, then| {
            when.method(GET)
                .path(STATUS_URI)
                .query_param(COMMAND_NAME_PARAM, COMMAND_SEEK)
                .query_param(COMMAND_VALUE_PARAM, "12");
            then.status(200);
        });
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.seek(12000);

        mock.assert();
    }

    #[test]
    fn test_seek_time_invalid() {
        init_logger!();
        let server = MockServer::start();
        let mock = server.mock(move |when, then| {
            when.method(GET)
                .path(STATUS_URI)
                .query_param(COMMAND_NAME_PARAM, COMMAND_SEEK)
                .query_param(COMMAND_VALUE_PARAM, "0");
            then.status(200);
        });
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.seek(800);

        mock.assert();
    }
}
