use std::borrow::Cow;
use std::fmt::Debug;

use log::debug;
#[cfg(test)]
use mockall::automock;
use rust_cast::CastDevice;
use rust_cast::channels::media::{ResumeState, Status, StatusEntry};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use serde::Serialize;

use crate::chromecast;
use crate::chromecast::ChromecastError;

#[cfg_attr(test, automock)]
pub trait FxCastDevice: Debug + Send + Sync {
    fn connect<S: Into<Cow<'static, str>> + 'static>(&self, receiver: S) -> chromecast::Result<()>;

    fn ping(&self) -> chromecast::Result<()>;

    fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application>;

    fn broadcast_message<M: Serialize + 'static>(&self, namespace: &str, message: &M) -> chromecast::Result<()>;

    fn stop_app<S: Into<Cow<'static, str>> + 'static>(&self, session_id: S) -> chromecast::Result<()>;

    fn pause<S: Into<Cow<'static, str>> + 'static>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry>;

    fn play<S: Into<Cow<'static, str>> + 'static>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry>;

    fn seek<S: Into<Cow<'static, str>> + 'static>(&self, destination: S, media_session_id: i32, current_time: Option<f32>, resume_state: Option<ResumeState>) -> chromecast::Result<StatusEntry>;

    fn status<S: Into<Cow<'static, str>> + 'static>(&self, destination: S, media_session_id: Option<i32>) -> chromecast::Result<Status>;
}

pub struct DefaultCastDevice(CastDevice<'static>);

impl DefaultCastDevice {
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
        self.0.connection.connect(receiver)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn ping(&self) -> chromecast::Result<()> {
        self.0.heartbeat.ping()
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn launch_app(&self, app: &CastDeviceApp) -> chromecast::Result<Application> {
        self.0.receiver.launch_app(app)
            .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
    }

    fn broadcast_message<M: Serialize>(&self, namespace: &str, message: &M) -> chromecast::Result<()> {
        self.0.receiver.broadcast_message(namespace, message)
            .map_err(|e| ChromecastError::AppInitializationFailed(e.to_string()))
    }

    fn stop_app<S: Into<Cow<'static, str>>>(&self, session_id: S) -> chromecast::Result<()> {
        self.0.receiver.stop_app(session_id)
            .map_err(|e| ChromecastError::AppTerminationFailed(e.to_string()))
    }

    fn pause<S: Into<Cow<'static, str>>>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry> {
        self.0.media.pause(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn play<S: Into<Cow<'static, str>>>(&self, destination: S, media_session_id: i32) -> chromecast::Result<StatusEntry> {
        self.0.media.play(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn seek<S: Into<Cow<'static, str>>>(&self, destination: S, media_session_id: i32, current_time: Option<f32>, resume_state: Option<ResumeState>) -> chromecast::Result<StatusEntry> {
        self.0.media.seek(destination, media_session_id, current_time, resume_state)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }

    fn status<S: Into<Cow<'static, str>>>(&self, destination: S, media_session_id: Option<i32>) -> chromecast::Result<Status> {
        self.0.media.get_status(destination, media_session_id)
            .map_err(|e| ChromecastError::Connection(e.to_string()))
    }
}

impl Debug for DefaultCastDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultCastDevice")
            .finish()
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
        
        let result = DefaultCastDevice::new(addr.to_string(), port);
        
        assert!(result.is_ok(), "expected the device to have been create, {:?}", result);
    }
    
    #[test]
    fn test_default_cast_device_connect() {
        init_logger();
        let test_instance = TestInstance::new_mdns();
        let addr = test_instance.addr.ip();
        let port = test_instance.addr.port();
        let device = DefaultCastDevice::new(addr.to_string(), port).unwrap();

        let _ = device.connect("receiver-0");
    }
}

