use std::borrow::Cow;
use std::fmt::Debug;

use log::debug;
#[cfg(test)]
use mockall::automock;
use rust_cast::channels::media::{ResumeState, StatusEntry};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use rust_cast::channels::{media, receiver};
use rust_cast::{CastDevice, ChannelMessage};
use serde::Serialize;

use crate::chromecast;
use crate::chromecast::ChromecastError;

pub(crate) const DEFAULT_RECEIVER: &str = "receiver-0";

/// A trait representing a Chromecast device with casting capabilities.
///
/// This trait defines methods for interacting with a Chromecast device, such as connecting,
/// launching apps, broadcasting messages, controlling media playback, and retrieving status.
///
/// Implementors of this trait can provide custom implementations for casting devices.
#[cfg_attr(test, automock)]
pub trait FxCastDevice: Debug + Send + Sync {
    /// Connects to the Chromecast device with the given receiver.
    fn connect<S: Into<Cow<'static, str>> + 'static>(&self, receiver: S) -> chromecast::Result<()>;

    /// Sends a ping message to the Chromecast device.
    fn ping(&self) -> chromecast::Result<()>;

    /// Sends a pong message to the Chromecast device.
    fn pong(&self) -> chromecast::Result<()>;

    /// Launches the specified app on the Chromecast device.
    fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application>;

    /// Broadcasts a message to the Chromecast device.
    fn broadcast_message<M: Serialize + 'static>(
        &self,
        namespace: &str,
        message: &M,
    ) -> chromecast::Result<()>;

    /// Stops the app with the given session ID on the Chromecast device.
    fn stop_app<S: Into<Cow<'static, str>> + 'static>(
        &self,
        session_id: S,
    ) -> chromecast::Result<()>;

    /// Pauses media playback on the Chromecast device.
    fn pause<S: Into<Cow<'static, str>> + 'static>(
        &self,
        destination: S,
        media_session_id: i32,
    ) -> chromecast::Result<StatusEntry>;

    /// Resumes media playback on the Chromecast device.
    fn play<S: Into<Cow<'static, str>> + 'static>(
        &self,
        destination: S,
        media_session_id: i32,
    ) -> chromecast::Result<StatusEntry>;

    /// Seeks to a specified time position in the media playback on the Chromecast device.
    fn seek<S: Into<Cow<'static, str>> + 'static>(
        &self,
        destination: S,
        media_session_id: i32,
        current_time: Option<f32>,
        resume_state: Option<ResumeState>,
    ) -> chromecast::Result<StatusEntry>;

    /// Retrieves the status of the Chromecast device.
    fn media_status<S: Into<Cow<'static, str>> + 'static>(
        &self,
        destination: S,
        media_session_id: Option<i32>,
    ) -> chromecast::Result<media::Status>;

    /// Retrieves the status of the cast device.
    fn device_status(&self) -> chromecast::Result<receiver::Status>;

    /// Receives messages from the Chromecast device.
    fn receive(&self) -> chromecast::Result<ChannelMessage>;
}

/// A default implementation of the `FxCastDevice` trait using a concrete `CastDevice` instance.
pub struct DefaultCastDevice(CastDevice<'static>);

impl DefaultCastDevice {
    /// Creates a new `DefaultCastDevice` instance with the specified address and port.
    ///
    /// # Arguments
    ///
    /// * `address` - The IP address or hostname of the Chromecast device.
    /// * `port` - The port number to connect to on the Chromecast device.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DefaultCastDevice` if successful, or a `ChromecastError` if
    /// an error occurs during initialization or connection.
    pub fn new(address: String, port: u16) -> chromecast::Result<Self> {
        match CastDevice::connect_without_host_verification(address.clone(), port) {
            Ok(device) => Ok(Self { 0: device }),
            Err(e) => {
                debug!("Failed to initialize Chromecast connection, {}", e);
                Err(ChromecastError::Connection(e.to_string()))
            }
        }
    }
}

impl FxCastDevice for DefaultCastDevice {
    fn connect<S: Into<Cow<'static, str>>>(&self, receiver: S) -> chromecast::Result<()> {
        self.0
            .connection
            .connect(receiver)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn ping(&self) -> chromecast::Result<()> {
        self.0
            .heartbeat
            .ping()
            .map_err(|e| ChromecastError::Heartbeat(e.to_string()))
    }

    fn pong(&self) -> chromecast::Result<()> {
        self.0
            .heartbeat
            .pong()
            .map_err(|e| ChromecastError::Heartbeat(e.to_string()))
    }

    fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application> {
        self.0
            .receiver
            .launch_app(app)
            .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
    }

    fn broadcast_message<M: Serialize>(
        &self,
        namespace: &str,
        message: &M,
    ) -> chromecast::Result<()> {
        self.0
            .receiver
            .broadcast_message(namespace, message)
            .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
    }

    fn stop_app<S: Into<Cow<'static, str>>>(&self, session_id: S) -> chromecast::Result<()> {
        self.0
            .receiver
            .stop_app(session_id)
            .map_err(|e| ChromecastError::AppTerminationFailed(e.to_string()))
    }

    fn pause<S: Into<Cow<'static, str>>>(
        &self,
        destination: S,
        media_session_id: i32,
    ) -> chromecast::Result<StatusEntry> {
        self.0
            .media
            .pause(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn play<S: Into<Cow<'static, str>>>(
        &self,
        destination: S,
        media_session_id: i32,
    ) -> chromecast::Result<StatusEntry> {
        self.0
            .media
            .play(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn seek<S: Into<Cow<'static, str>>>(
        &self,
        destination: S,
        media_session_id: i32,
        current_time: Option<f32>,
        resume_state: Option<ResumeState>,
    ) -> chromecast::Result<StatusEntry> {
        self.0
            .media
            .seek(destination, media_session_id, current_time, resume_state)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn media_status<S: Into<Cow<'static, str>>>(
        &self,
        destination: S,
        media_session_id: Option<i32>,
    ) -> chromecast::Result<media::Status> {
        self.0
            .media
            .get_status(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn device_status(&self) -> chromecast::Result<receiver::Status> {
        self.0
            .receiver
            .get_status()
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn receive(&self) -> chromecast::Result<ChannelMessage> {
        self.0
            .receive()
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }
}

impl Debug for DefaultCastDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultCastDevice").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromecast::tests::TestInstance;
    use popcorn_fx_core::init_logger;

    #[tokio::test]
    async fn test_default_cast_device_new() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();

        let result = DefaultCastDevice::new(addr.to_string(), port);

        assert!(
            result.is_ok(),
            "expected the device to have been create, {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_default_cast_device_connect() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.connect(DEFAULT_RECEIVER);
    }

    #[tokio::test]
    async fn test_default_cast_device_ping() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.ping();
    }

    #[ignore]
    #[tokio::test]
    async fn test_default_cast_device_launch() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.launch_app(&CastDeviceApp::DefaultMediaReceiver);
    }

    #[tokio::test]
    async fn test_default_cast_device_broadcast() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ =
            device.broadcast_message("urn:x-cast:BroadcastExample", &"ExampleMessage".to_string());
    }

    #[tokio::test]
    async fn test_pong() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let result = device.pong();

        assert!(
            result.is_ok(),
            "expected pong to succeed, but got {:?} instead",
            result
        );
    }

    #[ignore]
    #[tokio::test]
    async fn test_default_cast_device_play() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.play(DEFAULT_RECEIVER, 13);
    }

    #[ignore]
    #[tokio::test]
    async fn test_default_cast_device_media_status() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let mdns = test_instance.mdns().unwrap();
        let addr = mdns.addr.ip();
        let port = mdns.addr.port();
        mdns.add_response(
            "urn:x-cast:com.google.cast.media",
            r#"
        {
            "requestId":1,
            "type": "MEDIA_STATUS",
            "status":[
                {
                    "mediaSessionId":1,
                    "playerState":"PLAYING",
                    "playbackRate":1.0,
                    "supportedMediaCommands":2300
                }
            ]
        }
        "#,
        )
        .await;
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.media_status(DEFAULT_RECEIVER, None);
    }

    #[ignore]
    #[tokio::test]
    async fn test_default_cast_device_status() {
        init_logger!();
        let test_instance = TestInstance::new_mdns().await;
        let addr = test_instance.mdns().unwrap().addr.ip();
        let port = test_instance.mdns().unwrap().addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.device_status();
    }
}
