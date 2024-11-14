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
use log::{debug, error, info, trace, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder, Error, Response};
use serde_xml_rs::from_str;
use thiserror::Error;
use tokio::runtime;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use url::Url;

use popcorn_fx_core::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider};
use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
};

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
    cancel_token: Mutex<Option<CancellationToken>>,
}

impl VlcPlayer {
    pub fn builder() -> VlcPlayerBuilder {
        VlcPlayerBuilder::builder()
    }
}

impl Callbacks<PlayerEvent> for VlcPlayer {
    fn add_callback(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.inner.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.inner.remove_callback(handle)
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

    fn state(&self) -> PlayerState {
        self.inner.state()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        self.inner.request()
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        self.inner.play(request).await;
        let cancel_token = CancellationToken::new();

        {
            trace!("Creating new cancellation token");
            let mut mutex = self.cancel_token.lock().await;
            *mutex = Some(cancel_token.clone());
        }

        let inner_timer = self.inner.clone();
        self.inner.runtime.spawn(async move {
            // initial delay for startup
            sleep(Duration::from_secs(3)).await;

            while !cancel_token.is_cancelled() {
                if !inner_timer.check_status().await {
                    cancel_token.cancel()
                }

                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    fn pause(&self) {
        self.inner.pause()
    }

    fn resume(&self) {
        self.inner.resume()
    }

    fn seek(&self, time: u64) {
        self.inner.seek(time)
    }

    fn stop(&self) {
        debug!("Stopping external VLC player with status listener cancellation");
        {
            let mut mutex = block_in_place(self.cancel_token.lock());
            if let Some(cancel_token) = mutex.take() {
                cancel_token.cancel();
            }
        }

        self.inner.stop()
    }
}

impl Drop for VlcPlayer {
    fn drop(&mut self) {
        self.stop()
    }
}

/// Builder for creating new [VlcPlayer] instances.
///
/// # Example
///
/// ```rust
/// use tokio::runtime::Runtime;
/// use popcorn_fx_players::vlc::VlcPlayer;
///
/// let shared_runtime = Runtime::new().unwrap();
/// VlcPlayer::builder()
///     .runtime(shared_runtime)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct VlcPlayerBuilder {
    subtitle_manager: Option<Arc<Box<dyn SubtitleManager>>>,
    subtitle_provider: Option<Arc<Box<dyn SubtitleProvider>>>,
    password: Option<String>,
    address: Option<SocketAddr>,
    runtime: Option<Runtime>,
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

    /// Sets the runtime for the VLC player.
    pub fn runtime(mut self, runtime: Runtime) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Builds the `VlcPlayer` instance.
    pub fn build(self) -> VlcPlayer {
        let runtime = Arc::new(self.runtime.unwrap_or_else(|| {
            runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(1)
                .thread_name("vlc")
                .build()
                .expect("expected a new runtime")
        }));
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

        VlcPlayer {
            inner: Arc::new(InnerVlcPlayer {
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
                callbacks: Default::default(),
                runtime,
                subtitle_manager: self
                    .subtitle_manager
                    .expect("expected the subtitle_manager to have been set"),
                subtitle_provider: self
                    .subtitle_provider
                    .expect("expected the subtitle_provider to have been set"),
            }),
            cancel_token: Default::default(),
        }
    }
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
    callbacks: CoreCallbacks<PlayerEvent>,
    runtime: Arc<Runtime>,
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<Box<dyn SubtitleProvider>>,
}

impl InnerVlcPlayer {
    fn build_uri(&self, params: Vec<(&str, &str)>) -> Url {
        Url::parse_with_params(
            format!("http://{}:{}{}", VLC_HOST, self.socket.port(), STATUS_URI).as_str(),
            params,
        )
        .expect("expected a valid uri to have been created")
    }

    async fn check_status(&self) -> bool {
        trace!("Checking external VLC player status for {:?}", self);
        return match self.retrieve_status().await {
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
                true
            }
            Err(e) => {
                warn!("Failed to retrieve VLC status, {}", e);
                self.stop();
                false
            }
        };
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
}

impl Callbacks<PlayerEvent> for InnerVlcPlayer {
    fn add_callback(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.callbacks.remove_callback(handle)
    }
}

#[async_trait]
impl Player for InnerVlcPlayer {
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

    fn state(&self) -> PlayerState {
        block_in_place(self.state.lock()).clone()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = block_in_place(self.request.lock());
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

    fn pause(&self) {
        block_in_place(self.execute_command(VlcCommand::builder().name(COMMAND_PLAY_PAUSE).build()))
    }

    fn resume(&self) {
        block_in_place(self.execute_command(VlcCommand::builder().name(COMMAND_PLAY_PAUSE).build()))
    }

    fn seek(&self, mut time: u64) {
        if time <= 999 {
            warn!("Invalid seek time {} given", time);
            time = 0;
        } else {
            time = time / 1000;
        }

        block_in_place(
            self.execute_command(VlcCommand::builder().name(COMMAND_SEEK).value(time).build()),
        )
    }

    fn stop(&self) {
        debug!("Stopping external VLC player");
        block_in_place(self.execute_command(VlcCommand::builder().name(COMMAND_STOP).build()));

        {
            let mut mutex = block_in_place(self.process.lock());
            if let Some(mut process) = mutex.take() {
                if let Err(err) = process.kill() {
                    warn!("Failed to stop VLC process, {}", err);
                }
            }
        }

        self.callbacks
            .invoke(PlayerEvent::StateChanged(PlayerState::Stopped));
    }
}

impl Drop for InnerVlcPlayer {
    fn drop(&mut self) {
        self.stop()
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
    use std::sync::mpsc::channel;

    use httpmock::Method::GET;
    use httpmock::MockServer;

    use popcorn_fx_core::core::players::{MockPlayRequest, PlaySubtitleRequest};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
    use popcorn_fx_core::core::subtitles::{MockSubtitleProvider, SubtitlePreference};
    use popcorn_fx_core::testing::{init_logger, MockSubtitleManager};

    use super::*;

    #[test]
    fn test_id() {
        init_logger();
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
        init_logger();
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
        init_logger();
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
        init_logger();
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

    #[test]
    fn test_state() {
        init_logger();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .build();

        assert_eq!(PlayerState::Unknown, player.state());
    }

    #[test]
    fn test_play() {
        init_logger();
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

        block_in_place(player.play(Box::new(request)));

        let result = block_in_place(player.inner.process.lock());
        assert!(
            result.is_some(),
            "expected the VLC process to have been spawned"
        );

        let result = player
            .request()
            .and_then(|e| e.upgrade())
            .expect("expected the request to have been stored");
        assert_eq!(title.to_string(), result.title());
    }

    #[test]
    fn test_stop() {
        init_logger();
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

        block_in_place(player.play(Box::new(request)));
        player.stop();

        let result = block_in_place(player.inner.process.lock());
        assert!(
            result.is_none(),
            "expected the VLC process to have been killed"
        );

        mock.assert();
    }

    #[test]
    fn test_check_status() {
        init_logger();
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
        let (tx_status, rx_status) = channel();
        let (tx_time, rx_time) = channel();
        let (tx_duration, rx_duration) = channel();
        let manager = MockSubtitleManager::new();
        let provider = MockSubtitleProvider::new();
        let player = VlcPlayer::builder()
            .subtitle_manager(Arc::new(Box::new(manager)))
            .subtitle_provider(Arc::new(Box::new(provider)))
            .address(server.address().clone())
            .build();

        player.add_callback(Box::new(move |event| match event {
            PlayerEvent::DurationChanged(e) => tx_duration.send(e).unwrap(),
            PlayerEvent::TimeChanged(e) => tx_time.send(e).unwrap(),
            PlayerEvent::StateChanged(e) => tx_status.send(e).unwrap(),
            _ => {}
        }));

        let result = block_in_place(player.inner.check_status());
        assert!(result, "expected the status to have been retrieved");

        let result = rx_status.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(PlayerState::Playing, result);

        let result = rx_time.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(1000u64, result);

        let result = rx_duration
            .recv_timeout(Duration::from_millis(200))
            .unwrap();
        assert_eq!(6300000u64, result);
    }

    #[test]
    fn test_pause() {
        init_logger();
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
        init_logger();
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
        init_logger();
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
        init_logger();
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
