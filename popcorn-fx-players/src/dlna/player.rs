use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, trace};
use rupnp::{Device, Service};
use tokio::select;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use xml::escape::escape_str_attribute;

use popcorn_fx_core::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
use popcorn_fx_core::core::subtitles::model::SubtitleType;
use popcorn_fx_core::core::subtitles::SubtitleServer;
use popcorn_fx_core::core::utils::time::{
    parse_millis_from_time, parse_str_from_time, parse_time_from_millis, parse_time_from_str,
};

use crate::dlna;
use crate::dlna::models::{PositionInfo, TransportInfo, UpnpEvent};

const DLNA_GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-dlna-icon.png");
const DLNA_PLAYER_DESCRIPTION: &str = "DLNA Player";
const UPNP_PLAYER_SUBTITLE_TYPE: &str = "srt";
const UPNP_PLAYER_SUBTITLE_FORMAT: SubtitleType = SubtitleType::Srt;
const UPNP_PLAYER_PLAY_PAYLOAD: &str = r#"
    <InstanceID>0</InstanceID>
    <Speed>1</Speed>
"#;
const UPNP_PLAYER_PAUSE_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;
const UPNP_PLAYER_STOP_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;
const UPNP_PLAYER_POSITION_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;
const UPNP_PLAYER_TRANSPORT_INFO_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;
const UPNP_PLAYER_VOLUME_PAYLOAD: &str = r#"
    <InstanceID>0</InstanceID>
    <Channel>Master</Channel>
"#;

/// Represents a DLNA/UPnP player that supports devices such as TVs for remote media playback.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct DlnaPlayer {
    inner: Arc<InnerPlayer>,
}

impl DlnaPlayer {
    /// Creates a new DLNA player instance for the give UPnP [Device] and [Service].
    ///
    /// # Example
    ///
    /// Create a new player with the device and service provided by the UPnP discovery.
    ///
    /// ```rust,no_run
    /// use rupnp::Device;
    /// use ssdp_client::URN;
    /// use popcorn_fx_core::core::subtitles::SubtitleServer;
    /// use popcorn_fx_players::dlna::DlnaPlayer;
    /// use std::sync::Arc;
    ///
    /// async fn example(subtitle_server: Arc<SubtitleServer>) {
    ///     let uri = "upnp://237.84.2.178:1234".parse().unwrap();
    ///     let service_uri = URN::service("schemas-upnp-org", "AVTransport", 1);
    ///     let device = Device::from_url(uri).await.unwrap();
    ///     let service = device.find_service(&service_uri).unwrap().clone();
    ///
    ///     let player = DlnaPlayer::new(device, service, subtitle_server);
    /// }
    /// ```
    pub fn new(device: Device, service: Service, subtitle_server: Arc<SubtitleServer>) -> Self {
        let name = device.friendly_name().to_string();
        let id = format!("[{}]{}", device.device_type(), name);
        let (event_sender, event_receiver) = unbounded_channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let instance = Arc::new(InnerPlayer {
            id,
            device,
            service,
            event_sender,
            request: Default::default(),
            playback_state: Default::default(),
            subtitle_server,
            event_poller_activated: Default::default(),
            command_sender,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let inner_main = instance.clone();
        tokio::spawn(async move {
            inner_main.start(command_receiver, event_receiver).await;
        });

        Self { inner: instance }
    }
}

#[async_trait]
impl Player for DlnaPlayer {
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

    async fn request(&self) -> Option<PlayRequest> {
        self.inner.request().await.clone()
    }

    async fn current_volume(&self) -> Option<u32> {
        // TODO
        None
    }

    async fn play(&self, request: PlayRequest) {
        self.inner.play(request).await
    }

    async fn pause(&self) {
        self.inner.send_command(DlnaPlayerCommand::Pause)
    }

    async fn resume(&self) {
        self.inner.send_command(DlnaPlayerCommand::Resume)
    }

    async fn seek(&self, time: u64) {
        self.inner.send_command(DlnaPlayerCommand::Seek(time))
    }

    async fn stop(&self) {
        self.inner.send_command(DlnaPlayerCommand::Stop)
    }
}

impl Callback<PlayerEvent> for DlnaPlayer {
    fn subscribe(&self) -> Subscription<PlayerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for DlnaPlayer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, PartialEq)]
enum DlnaPlayerCommand {
    Pause,
    Resume,
    Seek(u64),
    Stop,
}

#[derive(Debug, Display)]
#[display(fmt = "{}", id)]
struct InnerPlayer {
    id: String,
    device: Device,
    service: Service,
    event_sender: UnboundedSender<UpnpEvent>,
    request: Mutex<Option<PlayRequest>>,
    playback_state: Mutex<PlaybackState>,
    subtitle_server: Arc<SubtitleServer>,
    event_poller_activated: Mutex<bool>,
    command_sender: UnboundedSender<DlnaPlayerCommand>,
    callbacks: MultiThreadedCallback<PlayerEvent>,
    cancellation_token: CancellationToken,
}

impl InnerPlayer {
    async fn start(
        &self,
        mut command_receiver: UnboundedReceiver<DlnaPlayerCommand>,
        mut receiver: UnboundedReceiver<UpnpEvent>,
    ) {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                Some(event) = receiver.recv() => self.handle_event(event).await,
                _ = interval.tick() => {
                    if self.should_poll_event_info().await {
                        self.poll_event_info().await
                    }
                },
            }
        }
        self.stop().await;
        debug!("UPnP event stream listener stopped");
    }

    async fn handle_command(&self, command: DlnaPlayerCommand) {
        match command {
            DlnaPlayerCommand::Pause => self.pause().await,
            DlnaPlayerCommand::Resume => self.resume().await,
            DlnaPlayerCommand::Seek(time) => self.seek(time).await,
            DlnaPlayerCommand::Stop => self.stop().await,
        }
    }

    async fn handle_event(&self, event: UpnpEvent) {
        match event {
            UpnpEvent::Time(e) => self.handle_time_event(e).await,
            UpnpEvent::State(e) => self.handle_state_event(e).await,
        }
    }

    async fn handle_subtitle(&self, request: &PlayRequest) -> (String, String) {
        let mut subtitle_attributes = String::new();
        let mut video_resource_attributes = String::new();

        if let Some(subtitle) = request.subtitle().subtitle.as_ref() {
            trace!(
                "Trying to serve DLNA subtitle {} for {}",
                subtitle.file(),
                request.url()
            );
            match self
                .subtitle_server
                .serve(subtitle.clone(), UPNP_PLAYER_SUBTITLE_FORMAT)
                .await
            {
                Ok(subtitle_url) => {
                    debug!("Serving DLNA subtitle at {}", subtitle_url);
                    subtitle_attributes = format!(
                        r#"<res protocolInfo="http-get:*:text/{subtitle_type}:*">{subtitle_uri}</res>
                           <res protocolInfo="http-get:*:smi/caption:*">{subtitle_uri}</res>
                           <sec:CaptionInfoEx sec:type="{subtitle_type}">{subtitle_uri}</sec:CaptionInfoEx>
                           <sec:CaptionInfo sec:type="{subtitle_type}">{subtitle_uri}</sec:CaptionInfo>"#,
                        subtitle_type = UPNP_PLAYER_SUBTITLE_TYPE,
                        subtitle_uri = subtitle_url,
                    );
                    video_resource_attributes = format!(
                        r#"xmlns:pv="http://www.pv.com/pvns/" pv:subtitleFileUri="{uri_sub}" pv:subtitleFileType="{subtitle_type}""#,
                        subtitle_type = UPNP_PLAYER_SUBTITLE_TYPE,
                        uri_sub = subtitle_url,
                    )
                }
                Err(e) => error!("Failed to serve DLNA subtitle, {}", e),
            }
        }

        (subtitle_attributes, video_resource_attributes)
    }

    async fn update_state(&self, state: PlayerState) {
        {
            let mut mutex = self.playback_state.lock().await;
            if mutex.state != state {
                mutex.state = state.clone();
            } else {
                return;
            }
        }

        self.callbacks.invoke(PlayerEvent::StateChanged(state));
    }

    async fn update_event_poller_state(&self, running: bool) {
        trace!(
            "Updating UPnP event poller state to {} for {}",
            running,
            self.device.url()
        );
        *self.event_poller_activated.lock().await = running;
    }

    async fn execute_action(
        &self,
        action: &str,
        payload: &str,
    ) -> dlna::Result<HashMap<String, String>> {
        trace!("Executing UPnP {} command with payload {}", action, payload);
        match self
            .service
            .action(self.device.url(), action, payload)
            .await
            .map(|e| {
                trace!("Received command {} response: {:?}", action, e);
                e
            }) {
            Ok(e) => Ok(e),
            Err(e) => {
                error!("Failed to execute {} UPnP action, {}", action, e);
                self.update_state(PlayerState::Error).await;
                Err(dlna::DlnaError::ServiceCommand)
            }
        }
    }

    /// Check if the event information should be polled from the player.
    ///
    /// # Returns
    ///
    /// It returns `true` if the event information should be polled, `false` otherwise.
    async fn should_poll_event_info(&self) -> bool {
        *self.event_poller_activated.lock().await
    }

    /// Poll the event information from the player.
    async fn poll_event_info(&self) {
        if let Ok(info) = self
            .execute_action("GetPositionInfo", UPNP_PLAYER_POSITION_PAYLOAD)
            .await
        {
            trace!("Received UPnP position info: {:?}", info);
            let event = UpnpEvent::Time(PositionInfo::from(info));
            if let Err(e) = self.event_sender.send(event) {
                self.handle_poll_event_error(e).await;
            }
        }
        if let Ok(info) = self
            .execute_action("GetTransportInfo", UPNP_PLAYER_TRANSPORT_INFO_PAYLOAD)
            .await
        {
            trace!("Received UPnP transport info: {:?}", info);
            let event = UpnpEvent::State(TransportInfo::from(info));
            if let Err(e) = self.event_sender.send(event) {
                self.handle_poll_event_error(e).await;
            }
        }
    }

    async fn handle_poll_event_error(&self, e: SendError<UpnpEvent>) {
        error!("Failed to send poll event information, {}", e);
        let mut mutex = self.event_poller_activated.lock().await;
        *mutex = false;
    }

    async fn handle_time_event(&self, event: PositionInfo) {
        let mut mutex = self.playback_state.lock().await;

        if let Ok(duration) = parse_time_from_str(event.track_duration.as_str()) {
            let duration = parse_millis_from_time(&duration);

            if mutex.duration != duration {
                mutex.duration = duration;
                self.callbacks
                    .invoke(PlayerEvent::DurationChanged(duration));
            }
        }

        if let Ok(time) = parse_time_from_str(event.rel_time.as_str()) {
            let time = parse_millis_from_time(&time);

            if mutex.time != time {
                mutex.time = time;
                self.callbacks.invoke(PlayerEvent::TimeChanged(time));
            }
        }
    }

    async fn handle_state_event(&self, event: TransportInfo) {
        let current_state = self.playback_state.lock().await.state.clone();
        let player_state = PlayerState::from(&event.current_transport_state);

        if current_state != player_state {
            self.update_state(player_state.clone()).await;
            self.callbacks
                .invoke(PlayerEvent::StateChanged(player_state));
        }
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn name(&self) -> &str {
        self.device.friendly_name()
    }

    fn description(&self) -> &str {
        DLNA_PLAYER_DESCRIPTION
    }

    fn graphic_resource(&self) -> Vec<u8> {
        DLNA_GRAPHIC_RESOURCE.to_vec()
    }

    async fn state(&self) -> PlayerState {
        let mutex = self.playback_state.lock().await;
        mutex.state.clone()
    }

    async fn request(&self) -> Option<PlayRequest> {
        self.request.lock().await.clone()
    }

    async fn play(&self, request: PlayRequest) {
        trace!("Starting DLNA playback for {:?}", request);
        let extension = PathBuf::from(request.url())
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or("mpeg".to_string());

        // process the playback subtitle information
        let (subtitle_attributes, video_resource_attributes) = self.handle_subtitle(&request).await;

        let video_resource = format!(
            r#"<res protocolInfo="http-get:*:video/{video_type}:DLNA.ORG_OP=01;DLNA.ORG_FLAGS=01100000000000000000000000000000" {video_attributes}>{video_uri}</res>"#,
            video_type = extension,
            video_uri = request.url(),
            video_attributes = video_resource_attributes,
        );
        let metadata = escape_str_attribute(
            format!(
                r#"<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
               xmlns:dc="http://purl.org/dc/elements/1.1/"
               xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/"
               xmlns:dlna="urn:schemas-dlna-org:device-1-0">
            <item id="0" parentID="-1" restricted="0">
                <dc:title>{title}</dc:title>
                {video_resource}
                {subtitle_attributes}
                <upnp:class>object.item.videoItem.movie</upnp:class>
            </item>
        </DIDL-Lite>"#,
                title = request.title(),
                video_resource = video_resource,
                subtitle_attributes = subtitle_attributes,
            )
            .as_str(),
        )
        .to_string();
        let initialize_payload = format!(
            r#"
            <InstanceID xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="ui4">0</InstanceID>
            <CurrentURI xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="string">{}</CurrentURI>
            <CurrentURIMetaData xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="string">{}</CurrentURIMetaData>
        "#,
            request.url(),
            metadata
        ).trim().to_string();

        trace!("Initializing DLNA playback with {:?}", initialize_payload);
        if let Err(e) = self
            .service
            .action(self.device.url(), "SetAVTransportURI", &initialize_payload)
            .await
        {
            error!("Failed to initialize UPnP playback, {}", e);
            self.update_state(PlayerState::Error).await;
            return;
        }

        trace!("Starting DLNA playback");
        self.resume().await;
        self.update_event_poller_state(true).await;

        debug!("DLNA playback has been started for {:?}", request);
        self.update_state(PlayerState::Buffering).await;

        if let Some(auto_resume) = request.auto_resume_timestamp() {
            trace!("Auto resuming DLNA playback at {}", auto_resume);
            self.seek(auto_resume).await;
        }

        {
            trace!("Updating DLNA player request to {:?}", request);
            let mut mutex = self.request.lock().await;
            *mutex = Some(request)
        }
    }

    async fn pause(&self) {
        let _ = self
            .execute_action("Pause", UPNP_PLAYER_PAUSE_PAYLOAD)
            .await;
    }

    async fn resume(&self) {
        let _ = self.execute_action("Play", UPNP_PLAYER_PLAY_PAYLOAD).await;
    }

    async fn seek(&self, time: u64) {
        let time = parse_time_from_millis(time);
        let time_str = parse_str_from_time(&time);
        let _ = self
            .execute_action(
                "Seek",
                format!(
                    r#"
                <InstanceID>0</InstanceID>
                <Unit>REL_TIME</Unit>
                <Target>{}</Target>
            "#,
                    time_str
                )
                .as_str(),
            )
            .await;
    }

    async fn stop(&self) {
        let _ = self.execute_action("Stop", UPNP_PLAYER_STOP_PAYLOAD).await;
        self.update_event_poller_state(false).await;
    }

    fn send_command(&self, command: DlnaPlayerCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Dlna player failed to send command, {}", e);
        }
    }
}

/// Represents the playback state information of a DLNA/UPnP media player.
#[derive(Debug, Clone, PartialEq)]
struct PlaybackState {
    /// The current playback time in millis.
    pub time: u64,
    /// The total duration of the media being played in millis.
    pub duration: u64,
    /// The state of the player.
    pub state: PlayerState,
}

impl Default for PlaybackState {
    /// Creates a default `PlaybackState` instance with zero time, zero duration, and `PlayerState::Ready`.
    fn default() -> Self {
        Self {
            time: 0,
            duration: 0,
            state: PlayerState::Ready,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use httpmock::Method::{GET, POST};
    use httpmock::{Mock, MockServer};
    use popcorn_fx_core::core::subtitles::MockSubtitleProvider;
    use popcorn_fx_core::{init_logger, recv_timeout};

    use super::*;
    use crate::dlna::tests::DEFAULT_SSDP_DESCRIPTION_RESPONSE;
    use crate::dlna::AV_TRANSPORT;
    use crate::tests::wait_for_hit;

    const RESPONSE_GET_POSITION: &str = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
                    s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
          <s:Body>
            <u:GetPositionInfoResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
              <Track>1</Track>
              <TrackDuration>00:05:32</TrackDuration>
              <TrackMetaData>
                &lt;DIDL-Lite xmlns:dc=&quot;http://purl.org/dc/elements/1.1/&quot; 
                            xmlns:upnp=&quot;urn:schemas-upnp-org:metadata-1-0/upnp/&quot; 
                            xmlns=&quot;urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/&quot;&gt;
                  &lt;item id=&quot;0&quot; parentID=&quot;0&quot; restricted=&quot;1&quot;&gt;
                    &lt;dc:title&gt;Example Track&lt;/dc:title&gt;
                    &lt;upnp:artist&gt;Artist Name&lt;/upnp:artist&gt;
                    &lt;upnp:albumArtURI&gt;http://example.com/albumart.jpg&lt;/upnp:albumArtURI&gt;
                  &lt;/item&gt;
                &lt;/DIDL-Lite&gt;
              </TrackMetaData>
              <TrackURI>http://example.com/example.mp3</TrackURI>
              <RelTime>00:02:15</RelTime>
              <AbsTime>NOT_IMPLEMENTED</AbsTime>
              <RelCount>214</RelCount>
              <AbsCount>NOT_IMPLEMENTED</AbsCount>
            </u:GetPositionInfoResponse>
          </s:Body>
        </s:Envelope>"#;

    struct TestInstance {
        server: MockServer,
        player: Arc<DlnaPlayer>,
    }

    impl TestInstance {
        pub fn server(&self) -> &MockServer {
            &self.server
        }

        pub fn player_instance(&self) -> Arc<DlnaPlayer> {
            self.player.clone()
        }
    }

    #[tokio::test]
    async fn test_id() {
        init_logger!();
        let instance = new_test_instance().await;
        let player = instance.player_instance();

        let result = player.id();

        assert_eq!("[urn:schemas-upnp-org:device:MediaRenderer:1]test", result);
    }

    #[tokio::test]
    async fn test_name() {
        init_logger!();
        let instance = new_test_instance().await;
        let player = instance.player_instance();

        let result = player.name();

        assert_eq!("test", result);
    }

    #[tokio::test]
    async fn test_description() {
        init_logger!();
        let instance = new_test_instance().await;
        let player = instance.player_instance();

        let result = player.description();

        assert_eq!(DLNA_PLAYER_DESCRIPTION, result);
    }

    #[tokio::test]
    async fn test_graphic_resource() {
        init_logger!();
        let instance = new_test_instance().await;
        let player = instance.player_instance();

        let result = player.graphic_resource();

        assert_eq!(DLNA_GRAPHIC_RESOURCE.to_vec(), result);
    }

    #[tokio::test]
    async fn test_state() {
        init_logger!();
        let instance = new_test_instance().await;
        let player = instance.player_instance();

        let result = player.state().await;

        assert_eq!(PlayerState::Ready, result);
    }

    #[tokio::test]
    async fn test_play() {
        init_logger!();
        let request = PlayRequest::builder()
            .url("http://localhost/my-video.mp4")
            .title("FooBar")
            .subtitles_enabled(true)
            .build();
        let instance = new_test_instance().await;
        let init_mock = create_init_mock(&instance);
        let play_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#Play\"")
                .body_contains(UPNP_PLAYER_PLAY_PAYLOAD);
            then.status(200)
                .body(r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <u:PlayResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"/>
                    </s:Body>
                </s:Envelope>"#);
        });
        let player = instance.player_instance();

        player.play(request).await;

        let result = player.state().await;
        assert_eq!(PlayerState::Buffering, result);
        assert_eq!(
            true,
            *player.inner.event_poller_activated.lock().await,
            "expected the event poller to have been activated"
        );
        init_mock.assert();
        play_mock.assert();
    }

    #[tokio::test]
    async fn test_pause() {
        init_logger!();
        let instance = new_test_instance().await;
        let pause_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#Pause\"");
            then.status(200)
                .body(r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <u:PauseResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                            <InstanceID>0</InstanceID>
                        </u:PauseResponse>
                    </s:Body>
                </s:Envelope>"#);
        });
        let player = instance.player_instance();

        player.pause().await;

        wait_for_hit(&pause_mock).await;
        pause_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_resume() {
        init_logger!();
        let instance = new_test_instance().await;
        let resume_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#Play\"");
            then.status(200)
                .body(r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <u:PlayResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                            <InstanceID>0</InstanceID>
                        </u:PlayResponse>
                    </s:Body>
                </s:Envelope>"#);
        });
        let player = instance.player_instance();

        player.resume().await;

        wait_for_hit(&resume_mock).await;
        resume_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_seek() {
        init_logger!();
        let instance = new_test_instance().await;
        let seek_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header(
                    "soapaction",
                    "\"urn:schemas-upnp-org:service:AVTransport:1#Seek\"",
                );
            then.status(200)
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .body(
                    r#"<?xml version="1.0"?>
                    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
                                s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                      <s:Body>
                        <u:SeekResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                          <InstanceID>0</InstanceID>
                        </u:SeekResponse>
                      </s:Body>
                    </s:Envelope>"#,
                );
        });
        let player = instance.player_instance();

        player.seek(14000).await;

        wait_for_hit(&seek_mock).await;
        seek_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_stop() {
        init_logger!();
        let instance = new_test_instance().await;
        let stop_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#Stop\"");
            then.status(200)
                .body(r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <u:StopResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                            <StopInstanceID>0</StopInstanceID>
                        </u:StopResponse>
                    </s:Body>
                </s:Envelope>"#);
        });
        let player = instance.player_instance();

        player.stop().await;

        let result = player.inner.event_poller_activated.lock().await;
        assert_eq!(
            false, *result,
            "expected event poller to have been cancelled"
        );

        wait_for_hit(&stop_mock).await;
        stop_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_poll_event_info_position_info() {
        init_logger!();
        let instance = new_test_instance().await;
        let _ = create_init_mock(&instance);
        instance.server().mock(|when, then| {
            when.method(POST).path("/AVTransport/control").header(
                "soapaction",
                "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"",
            );
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(RESPONSE_GET_POSITION);
        });
        let (tx_duration, mut rx_duration) = unbounded_channel();
        let (tx_time, mut rx_time) = unbounded_channel();
        let player = instance.player_instance();

        let mut receiver = player.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    PlayerEvent::DurationChanged(_) => tx_duration.send((*event).clone()).unwrap(),
                    PlayerEvent::TimeChanged(_) => tx_time.send((*event).clone()).unwrap(),
                    _ => {}
                }
            }
        });

        player.inner.poll_event_info().await;

        let result = recv_timeout!(&mut rx_duration, Duration::from_millis(200));
        if let PlayerEvent::DurationChanged(duration) = result {
            assert_eq!(332000, duration);
        } else {
            assert!(
                false,
                "expected PlayerEvent::DurationChanged, but got {:?} instead",
                result
            );
        }

        let result = recv_timeout!(&mut rx_time, Duration::from_millis(200));
        if let PlayerEvent::TimeChanged(time) = result {
            assert_eq!(135000, time);
        } else {
            assert!(
                false,
                "expected PlayerEvent::TimeChanged, but got {:?} instead",
                result
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_poll_event_info_transport_info() {
        init_logger!();
        let instance = new_test_instance().await;
        let _ = create_init_mock(&instance);
        instance.server().mock(|when, then| {
            when.method(POST).path("/AVTransport/control").header(
                "soapaction",
                "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"",
            );
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(RESPONSE_GET_POSITION);
        });
        instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#GetTransportInfo\"");
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(r#"<?xml version="1.0"?>
                    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
                                s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                      <s:Body>
                        <u:GetTransportInfoResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                          <CurrentTransportState>PLAYING</CurrentTransportState>
                          <CurrentTransportStatus>OK</CurrentTransportStatus>
                          <CurrentSpeed>1</CurrentSpeed>
                        </u:GetTransportInfoResponse>
                      </s:Body>
                    </s:Envelope>"#);
        });
        let (tx, mut rx) = unbounded_channel();
        let player = instance.player_instance();

        let mut receiver = player.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match &*event {
                    PlayerEvent::StateChanged(_) => tx.send((*event).clone()).unwrap(),
                    _ => {}
                }
            }
        });
        player.inner.poll_event_info().await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(250));
        if let PlayerEvent::StateChanged(state) = result {
            assert_eq!(PlayerState::Playing, state);
        } else {
            assert!(
                false,
                "expected PlayerEvent::StateChanged, but got {:?} instead",
                result
            );
        }
    }

    fn create_init_mock(instance: &TestInstance) -> Mock {
        instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#SetAVTransportURI\"")
                .body_contains(r#"<InstanceID xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="ui4">0</InstanceID>"#);
            then.status(200)
                .body(r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
                    <s:Body>
                        <u:SetAVTransportURIResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"/>
                    </s:Body>
                </s:Envelope>"#);
        })
    }

    async fn new_test_instance() -> TestInstance {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/description.xml");
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(DEFAULT_SSDP_DESCRIPTION_RESPONSE);
        });
        let addr = format!("http://{}/description.xml", server.address());
        let device = Device::from_url(addr.parse().unwrap()).await.unwrap();
        let service = device.find_service(&AV_TRANSPORT).cloned().unwrap();
        let subtitle_provider = MockSubtitleProvider::new();
        let subtitle_server = Arc::new(SubtitleServer::new(Arc::new(Box::new(subtitle_provider))));
        let player = Arc::new(DlnaPlayer::new(device, service, subtitle_server));

        TestInstance { server, player }
    }
}
