use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace, warn};
#[cfg(test)]
use mockall::automock;
use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::{media, receiver};
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::media::{ResumeState, StatusEntry};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use rust_cast::errors::Error;
use serde::Serialize;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::time;
use tokio_util::sync::CancellationToken;

use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};

use crate::chromecast;
use crate::chromecast::ChromecastError;

pub const DEFAULT_RECEIVER: &str = "receiver-0";

/// Represents events related to a cast device, such as messages and errors.
#[derive(Debug, Display, Clone)]
pub enum CastDeviceEvent {
    /// Indicates a received Chromecast message.
    #[display(fmt = "received Chromecast message")]
    Message(ChannelMessage),
    /// Indicates a received error message on the channel.
    #[display(fmt = "received error message channel, {}", _0)]
    Error(String),
}

/// Alias for a core callback handling cast device events.
pub type CastDeviceCallback = CoreCallback<CastDeviceEvent>;

/// Trait representing functionality for interacting with a cast device.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait FxCastDevice: Debug + Send + Sync {
    /// Connects to the specified destination.
    /// This can either be a receiver or an application.
    async fn connect<S: Into<String> + 'static + Send>(&self, destination: S) -> chromecast::Result<()>;

    /// Sends a ping to the cast device.
    async fn ping(&self) -> chromecast::Result<()>;

    /// Launches an application on the cast device.
    async fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application>;

    /// Broadcasts a message to the cast device.
    async fn broadcast_message<M: Serialize + 'static + Sync>(&self, namespace: &str, message: &M) -> chromecast::Result<()>;

    /// Stops the specified application session.
    async fn stop_app<S: Into<Cow<'static, str>> + 'static + Send>(&self, session_id: S) -> chromecast::Result<()>;

    /// Pauses playback on the cast device.
    async fn pause<S: Into<Cow<'static, str>> + 'static + Send>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry>;

    /// Resumes playback on the cast device.
    async fn play<S: Into<Cow<'static, str>> + 'static + Send>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry>;

    /// Seeks to a specified position in the media playback.
    async fn seek<S: Into<Cow<'static, str>> + 'static + Send>(&self, destination: S, media_session_id: i32, current_time: Option<f32>, resume_state: Option<ResumeState>) -> chromecast::Result<StatusEntry>;

    /// Retrieves the status of media playback.
    async fn media_status<S: Into<Cow<'static, str>> + 'static + Send>(&self, destination: S, media_session_id: Option<i32>) -> chromecast::Result<media::Status>;

    /// Retrieves the status of the cast device.
    async fn device_status(&self) -> chromecast::Result<receiver::Status>;

    /// Start the heartbeat loop on the cast device.
    /// This function ensures that the cast device maintains a connection
    /// by periodically sending heartbeat messages.
    ///
    /// # Arguments
    ///
    /// * `runtime` - The Tokio runtime to spawn the heartbeat loop on.
    fn start_heartbeat_loop(&self, runtime: &Runtime);

    /// Subscribe to receive events from the cast device.
    /// This function registers a callback to handle events emitted by the cast device.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to handle cast device events.
    ///
    /// # Returns
    ///
    /// A `CallbackHandle` that can be used to unsubscribe from the events later.
    fn subscribe(&self, callback: CastDeviceCallback) -> CallbackHandle;

    /// Unsubscribe from receiving cast device events.
    /// This function removes the callback associated with the specified handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle returned from the `subscribe` function.
    fn unsubscribe(&self, handle: CallbackHandle);
}

/// Default implementation of a cast device.
pub struct DefaultCastDevice {
    /// The address of the cast device.
    addr: String,
    /// The port of the cast device.
    port: u16,
    /// The device heartbeat in seconds.
    heartbeat_seconds: u64,
    /// The underlying cast device.
    device: Arc<RwLock<CastDevice<'static>>>,
    /// Callbacks for handling cast device events.
    callbacks: CoreCallbacks<CastDeviceEvent>,
    /// Token for canceling asynchronous tasks.
    cancellation_token: CancellationToken,
}

impl DefaultCastDevice {
    /// Creates a new `DefaultCastDevice` with the specified address, port, and runtime.
    ///
    /// This function establishes a connection to a Chromecast device at the given address and port.
    ///
    /// # Arguments
    ///
    /// * `addr` - The IP address or hostname of the Chromecast device.
    /// * `port` - The port number to connect to on the Chromecast device.
    /// * `heartbeat_seconds` - The interval in seconds between heartbeat messages.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DefaultCastDevice` if successful, or a `ChromecastError` if
    /// an error occurs during initialization or connection.
    pub async fn new(addr: String,
                     port: u16,
                     heartbeat_seconds: u64,
                     runtime: &Runtime) -> chromecast::Result<Self> {
        match Self::new_device(addr.as_str(), port) {
            Ok(device) => {
                trace!("Creating new chromecast device for {}", addr);
                let callbacks = CoreCallbacks::default();
                let device = Arc::new(RwLock::new(device));
                let cancellation_token = CancellationToken::new();

                let event_addr = addr.clone();
                let event_cancel = cancellation_token.clone();
                let event_callbacks = callbacks.clone();
                Self::execute_event_loop(event_addr, port, event_cancel, event_callbacks, runtime);

                Ok(Self {
                    addr,
                    port,
                    heartbeat_seconds,
                    device,
                    callbacks,
                    cancellation_token,
                })
            }
            Err(e) => {
                debug!("Failed to initialize Chromecast connection, {}", e);
                Err(ChromecastError::Connection(e.to_string()))
            }
        }
    }

    fn new_device<S: Into<String>>(addr: S, port: u16) -> chromecast::Result<CastDevice<'static>> {
        match CastDevice::connect_without_host_verification(addr.into(), port) {
            Ok(device) => {
                device.connection.connect(DEFAULT_RECEIVER)
                    .map_err(|e| ChromecastError::Connection(e.to_string()))?;
                Ok(device)
            }
            Err(e) => Err(ChromecastError::Connection(e.to_string())),
        }
    }

    async fn ping_device(device: &Arc<RwLock<CastDevice<'static>>>) -> Result<(), Error> {
        device.read().await.heartbeat.ping()
    }

    async fn disconnect(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing disconnect command");
            device.connection.disconnect(DEFAULT_RECEIVER)
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
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
            F: Future<Output=chromecast::Result<O>> + Send,
            O: Send,
    {
        match command_fn().await {
            Ok(e) => Ok(e),
            Err(e) => {
                debug!("Failed to execute Chromecast {} command, {}", self.addr, e);
                Self::reconnect(&self.device, self.addr.clone(), self.port).await?;
                command_fn().await
            }
        }
    }

    /// Try to reconnect to the Chromecast device.
    ///
    /// This asynchronous function attempts to reconnect to the Chromecast device at the specified
    /// address and port. If successful, it updates the device instance with the new connection.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the Chromecast device instance wrapped in an `Arc<RwLock<CastDevice>>`.
    /// * `addr` - The IP address or hostname of the Chromecast device.
    /// * `port` - The port number to connect to on the Chromecast device.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. If successful, returns `Ok(())`, otherwise returns
    /// a `ChromecastError` if an error occurs during reconnection.
    async fn reconnect(device: &Arc<RwLock<CastDevice<'static>>>, addr: String, port: u16) -> chromecast::Result<()> {
        trace!("Trying to reestablish connection to chromecast device for {}", addr);
        match Self::new_device(addr.as_str(), port) {
            Ok(new_device) => {
                debug!("Reestablished connection to chromecast device for {}", addr);
                let mut mutex = device.write().await;
                *mutex = new_device;
                Ok(())
            }
            Err(e) => {
                error!("Failed to reestablish connection to Chromecast device, {}", e);
                Err(ChromecastError::Connection(e.to_string()))
            }
        }
    }

    fn execute_event_loop(addr: String, port: u16, event_cancel: CancellationToken, callbacks: CoreCallbacks<CastDeviceEvent>, runtime: &Runtime) {
        runtime.spawn(async move {
            // create a new device
            let event_device = Arc::new(RwLock::new(Self::new_device(addr.as_str(), port).unwrap()));

            // start by connecting to the default receiver
            if let Err(e) = event_device.read().await.connection.connect(DEFAULT_RECEIVER) {
                error!("Failed to connect to the Chromecast {} default receiver, {}", addr, e);
                return;
            }

            debug!("Connected to the Chromecast {} default receiver, listening for messages", addr);
            loop {
                if event_cancel.is_cancelled() {
                    break;
                }

                match event_device.read().await.receive() {
                    Ok(message) => {
                        trace!("Received Chromecast {} event message {:?}", addr, message);
                        match &message {
                            ChannelMessage::Heartbeat(e) => {
                                trace!("Received Chromecast {} heartbeat {:?}", addr, e);
                                if let HeartbeatResponse::Ping = e {
                                    trace!("Sending Chromecast {} pong", addr);
                                    if let Err(e) = event_device.read().await.heartbeat.pong() {
                                        error!("Failed to send Chromecast {} pong, {}", addr, e);
                                    }
                                }
                            }
                            _ => callbacks.invoke(CastDeviceEvent::Message(message)),
                        }
                    }
                    Err(e) => {
                        error!("Failed to receive Chromecast {} event message, {}", addr, e);
                        if let Error::Io(_) = e {
                            // try to reestablish the connection
                            match Self::reconnect(&event_device, addr.clone(), port).await {
                                Ok(()) => {}
                                Err(e) => {
                                    error!("Failed to reconnect to Chromecast {} after an error, {}", addr, e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            debug!("Chromecast {} events has been stopped", addr);
        });
    }
}

impl Callbacks<CastDeviceEvent> for DefaultCastDevice {
    fn add(&self, callback: CoreCallback<CastDeviceEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
    }
}

#[async_trait]
impl FxCastDevice for DefaultCastDevice {
    async fn connect<S: Into<String> + 'static + Send>(&self, receiver: S) -> chromecast::Result<()> {
        let receiver = receiver.into();
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing connect command");
            device.connection.connect(receiver.clone())
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn ping(&self) -> chromecast::Result<()> {
        self.try_command(|| async {
            Self::ping_device(&self.device).await
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application> {
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing launch_app command");
            device.receiver.launch_app(app)
                .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
        }).await
    }

    async fn broadcast_message<M: Serialize + Sync>(&self, namespace: &str, message: &M) -> chromecast::Result<()> {
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing broadcast_message command");
            device.receiver.broadcast_message(namespace, message)
                .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
        }).await
    }

    async fn stop_app<S: Into<Cow<'static, str>> + Send>(&self, session_id: S) -> chromecast::Result<()> {
        let session_id = session_id.into();
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing stop_app command");
            device.receiver.stop_app(session_id.clone())
                .map_err(|e| ChromecastError::AppTerminationFailed(e.to_string()))
        }).await
    }

    async fn pause<S: Into<Cow<'static, str>> + Send>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry> {
        let destination = destination.into();
        self.try_command(|| async {
            self.device.read().await.media.pause(destination.clone(), media_session_id)
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn play<S: Into<Cow<'static, str>> + Send>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry> {
        let destination = destination.into();
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing play command");
            device.media.play(destination.clone(), media_session_id.clone())
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn seek<S: Into<Cow<'static, str>> + Send>(&self, destination: S, media_session_id: i32, current_time: Option<f32>, resume_state: Option<ResumeState>) -> chromecast::Result<StatusEntry> {
        let destination = destination.into();
        self.try_command(|| async {
            let device = self.device.read().await;
            trace!("Executing seek command");
            device.media.seek(destination.clone(), media_session_id, current_time.clone(), resume_state.clone())
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn media_status<S: Into<Cow<'static, str>> + Send>(&self, destination: S, media_session_id: Option<i32>) -> chromecast::Result<media::Status> {
        let destination = destination.into();
        self.try_command(|| async {
            self.device.read().await.media.get_status(destination.clone(), media_session_id.clone())
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    async fn device_status(&self) -> chromecast::Result<receiver::Status> {
        self.try_command(|| async {
            self.device.read().await.receiver.get_status()
                .map_err(|e| ChromecastError::Connection(e.to_string()))
        }).await
    }

    fn start_heartbeat_loop(&self, runtime: &Runtime) {
        let heartbeat_addr = self.addr.clone();
        let heartbeat_cancel = self.cancellation_token.clone();
        let heartbeat_device = self.device.clone();
        let heartbeat_interval = self.heartbeat_seconds.clone();

        runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = heartbeat_cancel.cancelled() => break,
                    result = Self::ping_device(&heartbeat_device) => {
                        if let Err(e) = result {
                            warn!("Failed to send Chromecast {} ping, {}", heartbeat_addr, e);
                        }
                    }
                }
                time::sleep(Duration::from_secs(heartbeat_interval)).await;
            }

            debug!("Chromecast {} heartbeat has been stopped", heartbeat_addr);
        });
    }

    fn subscribe(&self, callback: CastDeviceCallback) -> CallbackHandle {
        self.add(callback)
    }

    fn unsubscribe(&self, handle: CallbackHandle) {
        self.remove(handle)
    }
}

impl Debug for DefaultCastDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultCastDevice")
            .field("addr", &self.addr)
            .field("port", &self.port)
            .field("heartbeat_seconds", &self.heartbeat_seconds)
            .field("callbacks", &self.callbacks)
            .field("cancellation_token", &self.cancellation_token)
            .finish()
    }
}

impl Drop for DefaultCastDevice {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        self.cancellation_token.cancel();
        if let Err(e) = block_in_place(self.disconnect()) {
            error!("Failed to disconnect from Chromecast {} device, {}", self.addr, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::testing::init_logger;

    use crate::chromecast::tests::TestInstance;

    use super::*;

    #[test]
    fn test_default_cast_device_new() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();

        let result = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime));

        assert!(result.is_ok(), "expected the device to have been create, {:?}", result);
    }

    #[test]
    fn test_default_cast_device_connect() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        let _ = device.connect("receiver-0");
    }

    #[test]
    fn test_default_cast_device_ping() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        let _ = device.ping();
    }

    #[test]
    fn test_default_cast_device_launch() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        let _ = device.launch_app(&CastDeviceApp::DefaultMediaReceiver);
    }

    #[test]
    fn test_default_cast_device_broadcast() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        let _ = device.broadcast_message("urn:x-cast:BroadcastExample", &"ExampleMessage".to_string());
    }

    #[test]
    fn test_default_cast_device_play() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        let _ = device.play("receiver-0", 13);
    }

    #[test]
    fn test_default_cast_device_drop() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let runtime = Runtime::new().unwrap();
        let device = runtime.block_on(DefaultCastDevice::new(addr.to_string(), port, 10, &runtime)).unwrap();

        drop(device);
    }
}

