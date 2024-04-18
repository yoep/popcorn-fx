use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace};
use rupnp::{Device, Service};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;
use tokio::time;
use tokio_util::sync::CancellationToken;
use xml::escape::escape_str_attribute;

use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};
use popcorn_fx_core::core::utils::time::{parse_millis_from_time, parse_str_from_time, parse_time_from_millis, parse_time_from_str};

use crate::dlna;
use crate::dlna::models::{PositionInfo, TransportInfo, UpnpEvent};

const DLNA_GRAPHIC_RESOURCE: &[u8] = include_bytes!("../../resources/external-dlna-icon.png");
const DLNA_PLAYER_DESCRIPTION: &str = "DLNA Player";
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
    /// use ssdp_client::SearchTarget::URN;
    /// use popcorn_fx_dlna::dlna::DlnaPlayer;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let uri = "upnp://237.84.2.178:1234".parse().unwrap();
    ///     let service_uri =  URN::service("schemas-upnp-org", "AVTransport", 1);
    ///     let device = Device::from_url(uri).await.unwrap();
    ///     let service = device.find_service(service_uri).unwrap().clone();
    ///
    ///     let player = DlnaPlayer::new(device, service); 
    /// }
    /// ```
    pub fn new(device: Device, service: Service) -> Self {
        let name = device.friendly_name().to_string();
        let id = format!(
            "[{}]{}",
            device.device_type(),
            name);
        let (tx, mut rx) = channel(10);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name(format!("dlna-{}", name))
            .build()
            .expect("expected a new runtime");
        let instance = Arc::new(InnerPlayer {
            id,
            device,
            service,
            event_sender: tx,
            request: Default::default(),
            playback_state: Default::default(),
            callbacks: Default::default(),
            event_poller_activated: Default::default(),
            cancellation_token: Default::default(),
            runtime,
        });

        let inner_instance = instance.clone();
        instance.runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = inner_instance.cancellation_token.cancelled() => break,
                    result = rx.recv() => {
                        if let Some(event) = result {
                            match event {
                                UpnpEvent::Time(e) => inner_instance.handle_time_event(e).await,
                                UpnpEvent::State(e) => inner_instance.handle_state_event(e).await, 
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            debug!("UPnP event stream listener stopped");
        });
        let inner_instance = instance.clone();
        instance.runtime.spawn(async move {
            loop {
                if inner_instance.cancellation_token.is_cancelled() {
                    break;
                }

                if *inner_instance.event_poller_activated.lock().await {
                    inner_instance.poll_event_info().await;
                }

                time::sleep(Duration::from_secs(1)).await;
            }
            debug!("UPnP main event poller stopped");
        });

        Self {
            inner: instance
        }
    }
}

impl Callbacks<PlayerEvent> for DlnaPlayer {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
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

    fn state(&self) -> PlayerState {
        self.inner.state()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        self.inner.request()
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        self.inner.play(request).await
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
        self.inner.stop()
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", id)]
struct InnerPlayer {
    id: String,
    device: Device,
    service: Service,
    event_sender: Sender<UpnpEvent>,
    request: Mutex<Option<Arc<Box<dyn PlayRequest>>>>,
    playback_state: Mutex<PlaybackState>,
    callbacks: CoreCallbacks<PlayerEvent>,
    event_poller_activated: Mutex<bool>,
    cancellation_token: CancellationToken,
    runtime: Runtime,
}

impl InnerPlayer {
    fn update_state(&self, state: PlayerState) {
        block_in_place(self.update_state_async(state))
    }

    async fn update_state_async(&self, state: PlayerState) {
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

    async fn start_event_poller(&self) {
        trace!("Starting UPnP event poller for {}", self.device.url());
        {
            let mut mutex = self.event_poller_activated.lock().await;
            *mutex = true;
        }
    }

    async fn stop_event_poller(&self) {
        let mut mutex = self.event_poller_activated.lock().await;
        *mutex = false;
    }

    async fn execute_action(&self, action: &str, payload: &str) -> dlna::Result<HashMap<String, String>> {
        trace!("Executing UPnP {} command with payload {}", action, payload);
        self.service.action(self.device.url(), action, payload).await
            .map(|e| {
                trace!("Received command {} response: {:?}", action, e);
                e
            })
            .map_err(|e| {
                error!("Failed to execute {} UPnP action, {}", action, e);
                self.update_state(PlayerState::Error);
                dlna::DlnaError::ServiceCommand
            })
    }

    async fn poll_event_info(&self) {
        if let Ok(info) = self.execute_action("GetPositionInfo", UPNP_PLAYER_POSITION_PAYLOAD).await {
            trace!("Received UPnP position info: {:?}", info);
            let event = UpnpEvent::Time(PositionInfo::from(info));
            if let Err(e) = self.event_sender.send(event).await {
                self.handle_poll_event_error(e).await;
            }
        }
        if let Ok(info) = self.execute_action("GetTransportInfo", UPNP_PLAYER_TRANSPORT_INFO_PAYLOAD).await {
            trace!("Received UPnP transport info: {:?}", info);
            let event = UpnpEvent::State(TransportInfo::from(info));
            if let Err(e) = self.event_sender.send(event).await {
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
                self.callbacks.invoke(PlayerEvent::DurationChanged(duration));
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
            self.update_state_async(player_state.clone()).await;
            self.callbacks.invoke(PlayerEvent::StateChanged(player_state));
        }
    }
}

impl Callbacks<PlayerEvent> for InnerPlayer {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl Player for InnerPlayer {
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

    fn state(&self) -> PlayerState {
        let mutex = block_in_place(self.playback_state.lock());
        mutex.state.clone()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = block_in_place(self.request.lock());
        mutex.as_ref()
            .map(|e| Arc::downgrade(e))
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        trace!("Starting DLNA playback for {:?}", request);
        if request.subtitles_enabled() {
            // todo
        }

        let metadata = escape_str_attribute(
            format!(r#"<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
               xmlns:dc="http://purl.org/dc/elements/1.1/"
               xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/"
               xmlns:dlna="urn:schemas-dlna-org:device-1-0">
            <item id="0" parentID="-1" restricted="0">
                <dc:title>{title}</dc:title>
                <res>{video_uri}</res>
                <upnp:class>object.item.videoItem.movie</upnp:class>
            </item>
        </DIDL-Lite>"#,
                    title = request.title(),
                    video_uri = request.url()).as_str())
            .to_string();
        let initialize_payload = format!(r#"
            <InstanceID xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="ui4">0</InstanceID>
            <CurrentURI xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="string">{}</CurrentURI>
            <CurrentURIMetaData xmlns:dt="urn:schemas-microsoft-com:datatypes" dt:dt="string">{}</CurrentURIMetaData>
        "#, request.url(), metadata);

        trace!("Initializing DLNA playback with {:?}", initialize_payload);
        if let Err(e) =
            self.service.action(self.device.url(), "SetAVTransportURI", &initialize_payload).await {
            error!("Failed to initialize UPnP playback, {}", e);
            self.update_state_async(PlayerState::Error).await;
            return;
        }

        trace!("Starting DLNA playback");
        self.resume();
        self.start_event_poller().await;

        debug!("DLNA playback has been started for {:?}", request);
        self.update_state_async(PlayerState::Buffering).await;

        {
            trace!("Updating DLNA player request to {:?}", request);
            let mut mutex = self.request.lock().await;
            *mutex = Some(Arc::new(request))
        }
    }

    fn pause(&self) {
        block_in_place(async {
            let _ = self.execute_action("Pause", UPNP_PLAYER_PAUSE_PAYLOAD).await;
        })
    }

    fn resume(&self) {
        block_in_place(async {
            let _ = self.execute_action("Play", UPNP_PLAYER_PLAY_PAYLOAD).await;
        })
    }

    fn seek(&self, time: u64) {
        let time = parse_time_from_millis(time);
        let time_str = parse_str_from_time(&time);
        block_in_place(async {
            let _ = self.execute_action("Seek", format!(r#"
                <InstanceID>0</InstanceID>
                <Unit>REL_TIME</Unit>
                <Target>{}</Target>
            "#, time_str).as_str()).await;
        })
    }

    fn stop(&self) {
        block_in_place(async {
            let _ = self.execute_action("Stop", UPNP_PLAYER_STOP_PAYLOAD).await;
            self.stop_event_poller().await;
        })
    }
}

impl Drop for InnerPlayer {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlaybackState {
    pub time: u64,
    pub duration: u64,
    pub state: PlayerState,
}

impl Default for PlaybackState {
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
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use httpmock::{Mock, MockServer};
    use httpmock::Method::{GET, POST};
    use tokio::runtime::Runtime;

    use popcorn_fx_core::core::players::PlayUrlRequestBuilder;
    use popcorn_fx_core::testing::init_logger;

    use crate::dlna::AV_TRANSPORT;
    use crate::tests::DEFAULT_SSDP_DESCRIPTION_RESPONSE;

    use super::*;

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
        runtime: Arc<Runtime>,
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
    
    #[test]
    fn test_id() {
        init_logger();
        let instance = new_test_instance();
        let player = instance.player_instance();
        
        let result = player.id();
        
        assert_eq!("[urn:schemas-upnp-org:device:MediaRenderer:1]test", result);
    }
    
    #[test]
    fn test_name() {
        init_logger();
        let instance = new_test_instance();
        let player = instance.player_instance();
        
        let result = player.name();
        
        assert_eq!("test", result);
    }
    
    #[test]
    fn test_description() {
        init_logger();
        let instance = new_test_instance();
        let player = instance.player_instance();
        
        let result = player.description();
        
        assert_eq!(DLNA_PLAYER_DESCRIPTION, result);
    }

    #[test]
    fn test_play() {
        init_logger();
        let request = Box::new(PlayUrlRequestBuilder::builder()
            .url("http://localhost/my-video.mp4")
            .title("FooBar")
            .subtitles_enabled(true)
            .build());
        let instance = new_test_instance();
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

        instance.runtime.block_on(player.play(request));

        assert_eq!(PlayerState::Buffering, player.state());
        assert_eq!(true, *block_in_place(player.inner.event_poller_activated.lock()), "expected the event poller to have been activated");
        init_mock.assert();
        play_mock.assert();
    }

    #[test]
    fn test_pause() {
        init_logger();
        let instance = new_test_instance();
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

        player.pause();

        pause_mock.assert();
    }

    #[test]
    fn test_resume() {
        init_logger();
        let instance = new_test_instance();
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

        player.resume();

        resume_mock.assert();
    }
    
    #[test]
    fn test_seek() {
        init_logger();
        let instance = new_test_instance();
        let seek_mock = instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#Seek\"");
            then.status(200)
                .header("content-type", "text/xml; charset=\"utf-8\"")
                .body(r#"<?xml version="1.0"?>
                    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
                                s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                      <s:Body>
                        <u:SeekResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                          <InstanceID>0</InstanceID>
                        </u:SeekResponse>
                      </s:Body>
                    </s:Envelope>"#);
        });
        let player = instance.player_instance();
        
        player.seek(14000);

        seek_mock.assert();
    }

    #[test]
    fn test_stop() {
        init_logger();
        let instance = new_test_instance();
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

        player.stop();

        let result = block_in_place(player.inner.event_poller_activated.lock());
        assert_eq!(false, *result, "expected event poller to have been cancelled");
        stop_mock.assert();
    }

    #[test]
    fn test_poll_event_info_position_info() {
        init_logger();
        let instance = new_test_instance();
        let _ = create_init_mock(&instance);
        instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"");
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(RESPONSE_GET_POSITION);
        });
        let (tx_duration, rx_duration) = channel();
        let (tx_time, rx_time) = channel();
        let player = instance.player_instance();

        player.add(Box::new(move |event| {
            match &event {
                PlayerEvent::DurationChanged(_) => tx_duration.send(event).unwrap(),
                PlayerEvent::TimeChanged(_) => tx_time.send(event).unwrap(),
                _ => {}
            }
        }));
        player.inner.runtime.block_on(player.inner.poll_event_info());

        let result = rx_duration.recv_timeout(Duration::from_millis(200)).unwrap();
        if let PlayerEvent::DurationChanged(duration) = result {
            assert_eq!(332000, duration);
        } else {
            assert!(false, "expected PlayerEvent::DurationChanged, but got {:?} instead", result);
        }

        let result = rx_time.recv_timeout(Duration::from_millis(200)).unwrap();
        if let PlayerEvent::TimeChanged(time) = result {
            assert_eq!(135000, time);
        } else {
            assert!(false, "expected PlayerEvent::TimeChanged, but got {:?} instead", result);
        }
    }

    #[test]
    fn test_poll_event_info_transport_info() {
        init_logger();
        let instance = new_test_instance();
        let _ = create_init_mock(&instance);
        instance.server().mock(|when, then| {
            when.method(POST)
                .path("/AVTransport/control")
                .header("soapaction", "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"");
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
        let (tx, rx) = channel();
        let player = instance.player_instance();

        player.add(Box::new(move |event| {
            if let PlayerEvent::StateChanged(_) = &event {
                tx.send(event).unwrap();
            }
        }));
        player.inner.runtime.block_on(player.inner.poll_event_info());

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        if let PlayerEvent::StateChanged(state) = result {
            assert_eq!(PlayerState::Playing, state);
        } else {
            assert!(false, "expected PlayerEvent::StateChanged, but got {:?} instead", result);
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

    fn new_test_instance() -> TestInstance {
        let runtime = Arc::new(Runtime::new().unwrap());
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/description.xml");
            then.status(200)
                .header("Content-Type", "text/xml; charset=\"utf-8\"")
                .body(DEFAULT_SSDP_DESCRIPTION_RESPONSE);
        });
        let addr = format!("http://{}/description.xml", server.address());
        let device = runtime.block_on(Device::from_url(addr.parse().unwrap())).unwrap();
        let service = device.find_service(&AV_TRANSPORT).cloned().unwrap();
        let player = Arc::new(DlnaPlayer::new(device, service));

        TestInstance {
            runtime,
            server,
            player,
        }
    }
}