use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace, warn};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::env::consts::{ARCH, OS};
use std::fmt;
use std::fmt::Debug;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use popcorn_fx_core::core::platform::{
    Platform, PlatformData, PlatformEvent, PlatformInfo, PlatformType,
};
use popcorn_fx_core::core::playback::{MediaInfo, MediaNotificationEvent};

#[cfg(target_os = "linux")]
use crate::platform::platform_linux::PlatformLinux;
#[cfg(target_os = "macos")]
use crate::platform::platform_mac::PlatformMac;
#[cfg(target_os = "windows")]
use crate::platform::platform_win::PlatformWin;

const DBUS_NAME: &str = "popcorn_time.media";
const DISPLAY_NAME: &str = "Popcorn Time";

/// The os system specific actions.
pub trait SystemPlatform: Debug + Send + Sync {
    /// Disable the screensaver on the current platform
    /// It returns `true` if the screensaver was disabled with success, else `false`.
    fn disable_screensaver(&self) -> bool;

    /// Enable the screensaver on the current platform
    /// It returns `true` if the screensaver was enabled with success, else `false`.
    fn enable_screensaver(&self) -> bool;

    /// Retrieve the handle of the window for the platform.
    fn window_handle(&self) -> Option<*mut std::ffi::c_void>;
}

/// The `DefaultPlatform` struct represents the [PlatformData], which contains a reference to a
/// platform and platform information.
pub struct DefaultPlatform {
    platform: Arc<Box<dyn SystemPlatform>>,
    controls: Mutex<Option<MediaControls>>,
    callbacks: MultiThreadedCallback<PlatformEvent>,
}

impl DefaultPlatform {
    fn create_controls(&self) -> Option<MediaControls> {
        let platform_config = self.create_controls_config();

        #[cfg(target_os = "windows")]
        if platform_config.hwnd == None {
            warn!("No window handle present for Windows DefaultPlatform::create_controls, preventing thread panic");
            return None;
        }

        trace!("Creating system media control with {:?}", platform_config);
        match MediaControls::new(platform_config) {
            Ok(mut controls) => {
                debug!("System media controls have been created");
                // attach the media controls events of the system to our known callbacks
                let callbacks = self.callbacks.clone();
                match controls.attach(move |event: MediaControlEvent| {
                    Self::handle_media_event(event, &callbacks)
                }) {
                    Ok(_) => debug!("System media controls attached"),
                    Err(e) => error!("Failed to attach system media controls, {:?}", e),
                };

                Some(controls)
            }
            Err(e) => {
                error!("Failed to create system media controls, {:?}", e);
                None
            }
        }
    }

    fn create_controls_config(&self) -> PlatformConfig<'_> {
        PlatformConfig {
            dbus_name: DBUS_NAME,
            display_name: DISPLAY_NAME,
            hwnd: self.platform.window_handle(),
        }
    }

    fn on_media_info_changed(&self, controls: &mut MediaControls, info: MediaInfo) {
        let metadata = MediaMetadata {
            title: Some(&info.title),
            artist: info.subtitle.as_ref().map(|e| e.as_str()),
            // FIXME: panicked at souvlaki-0.8.2/src/platform/macos/mod.rs:319:24: null pointer dereference occurred
            // cover_url: info.thumb.as_ref().filter(|e| !e.is_empty()).map(|e| e.as_ref()),
            ..Default::default()
        };

        trace!("Notifying system of media playback {:?}", metadata);
        match controls.set_metadata(metadata) {
            Ok(_) => info!("System has been notified of the new media playback"),
            Err(e) => error!("System media notification failed, {:?}", e),
        };
    }

    fn on_playback_state_changed(&self, controls: &mut MediaControls, state: MediaPlayback) {
        let state_info = format!("{:?}", state);

        trace!("Updating system media playback state to {}", state_info);
        match controls.set_playback(state) {
            Ok(_) => debug!("System media state has changed {}", state_info),
            Err(e) => error!("System media state couldn't be updated, {:?}", e),
        }
    }

    fn internal_notify_media_event(&self, event: MediaNotificationEvent) {
        let mut mutex = futures::executor::block_on(self.controls.lock());

        // check if the controls already exist
        // if not, we'll create them first
        if mutex.is_none() {
            *mutex = self.create_controls();
        }

        if let Some(mut controls) = mutex.as_mut() {
            match &event {
                MediaNotificationEvent::StateStarting(info) => {
                    self.on_media_info_changed(&mut controls, info.clone())
                }
                MediaNotificationEvent::StatePlaying => self.on_playback_state_changed(
                    &mut controls,
                    MediaPlayback::Playing { progress: None },
                ),
                MediaNotificationEvent::StatePaused => self.on_playback_state_changed(
                    &mut controls,
                    MediaPlayback::Paused { progress: None },
                ),
                MediaNotificationEvent::StateStopped => {
                    self.on_playback_state_changed(&mut controls, MediaPlayback::Stopped)
                }
            }
        } else {
            warn!("Unable to handle the media playback notification, MediaControls not present")
        }

        if MediaNotificationEvent::StateStopped == event {
            Self::dispose_media_controls(&mut mutex);
        }
    }

    fn handle_media_event(
        event: MediaControlEvent,
        callbacks: &MultiThreadedCallback<PlatformEvent>,
    ) {
        debug!("Received system media control event {:?}", event);
        match event {
            MediaControlEvent::Play => callbacks.invoke(PlatformEvent::TogglePlaybackState),
            MediaControlEvent::Pause => callbacks.invoke(PlatformEvent::TogglePlaybackState),
            MediaControlEvent::Toggle => callbacks.invoke(PlatformEvent::TogglePlaybackState),
            MediaControlEvent::Next => callbacks.invoke(PlatformEvent::ForwardMedia),
            MediaControlEvent::Previous => callbacks.invoke(PlatformEvent::RewindMedia),
            _ => {}
        }
    }

    fn dispose_media_controls(mutex: &mut MutexGuard<Option<MediaControls>>) {
        // release the controls
        debug!("Releasing system media controls");
        if let Some(controls) = mutex.as_mut() {
            trace!("Detaching system media controls");
            match controls.detach() {
                Ok(_) => debug!("Detached system media controls"),
                Err(e) => error!("Failed to detach from system media controls, {:?}", e),
            }
        }

        trace!("Releasing system media controls");
        let _ = mutex.take();
        info!("System media controls have been released");
    }
}

impl Callback<PlatformEvent> for DefaultPlatform {
    fn subscribe(&self) -> Subscription<PlatformEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlatformEvent>) {
        self.callbacks.subscribe_with(subscriber);
    }
}

impl Platform for DefaultPlatform {
    fn disable_screensaver(&self) -> bool {
        self.platform.disable_screensaver()
    }

    fn enable_screensaver(&self) -> bool {
        self.platform.enable_screensaver()
    }

    fn notify_media_event(&self, event: MediaNotificationEvent) {
        trace!("Received platform media notification {:?}", event);
        if let Err(e) = catch_unwind(AssertUnwindSafe(|| self.internal_notify_media_event(event))) {
            error!("Failed to notify media event, {:?}", e);
        }
    }
}

impl PlatformData for DefaultPlatform {
    fn info(&self) -> PlatformInfo {
        trace!("Retrieving system information");
        let platform_type = match OS {
            "windows" => PlatformType::Windows,
            "macos" => PlatformType::MacOs,
            _ => PlatformType::Linux,
        };
        let arch = String::from(ARCH);

        PlatformInfo {
            platform_type,
            arch,
        }
    }
}

impl Default for DefaultPlatform {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let platform = Box::new(PlatformWin::default());
        #[cfg(target_os = "macos")]
        let platform = Box::new(PlatformMac::default());
        #[cfg(target_os = "linux")]
        let platform = Box::new(PlatformLinux::default());

        Self {
            platform: Arc::new(platform),
            controls: Default::default(),
            callbacks: MultiThreadedCallback::new(),
        }
    }
}

impl Debug for DefaultPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultPlatform")
            .field("platform", &self.platform)
            .finish()
    }
}

impl Drop for DefaultPlatform {
    fn drop(&mut self) {
        self.enable_screensaver();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use mockall::mock;
    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tokio::sync::oneshot::channel;
    use tokio::time::timeout;

    mock! {
        #[derive(Debug)]
        pub DummySystemPlatform{}

        impl SystemPlatform for DummySystemPlatform {
            fn disable_screensaver(&self) -> bool;

            fn enable_screensaver(&self) -> bool;

            fn window_handle(&self) -> Option<*mut std::ffi::c_void>;
        }
    }

    #[tokio::test]
    async fn test_disable_screensaver() {
        init_logger!();
        let mut sys_platform = MockDummySystemPlatform::new();
        sys_platform.expect_disable_screensaver().returning(|| true);
        sys_platform.expect_enable_screensaver().returning(|| false);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
            callbacks: MultiThreadedCallback::new(),
        };

        assert!(
            platform.disable_screensaver(),
            "expected the screensaver to be disabled"
        )
    }

    #[tokio::test]
    async fn test_enable_screensaver() {
        init_logger!();
        let mut sys_platform = MockDummySystemPlatform::new();
        sys_platform.expect_enable_screensaver().returning(|| true);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
            callbacks: MultiThreadedCallback::new(),
        };

        assert!(
            platform.enable_screensaver(),
            "expected the screensaver to be enabled"
        )
    }

    #[tokio::test]
    async fn test_drop_default_platform() {
        init_logger!();
        let mut sys_platform = MockDummySystemPlatform::new();
        sys_platform
            .expect_enable_screensaver()
            .returning(|| true)
            .times(1);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
            callbacks: MultiThreadedCallback::new(),
        };

        drop(platform);
    }

    #[test]
    fn test_platform_info() {
        let platform = DefaultPlatform::default();
        #[cfg(target_os = "windows")]
        let platform_type = PlatformType::Windows;
        #[cfg(target_os = "linux")]
        let platform_type = PlatformType::Linux;
        #[cfg(target_os = "macos")]
        let platform_type = PlatformType::MacOs;
        #[cfg(target_arch = "x86_64")]
        let arch = "x86_64";
        #[cfg(target_arch = "aarch64")]
        let arch = "aarch64";
        #[cfg(target_arch = "arm")]
        let arch = "arm";

        let result = platform.info();

        assert_eq!(platform_type, result.platform_type);
        assert_eq!(arch.to_string(), result.arch);
    }

    #[test]
    fn test_platform_notify_media_event() {
        init_logger!();
        let platform = DefaultPlatform::default();

        // notify the system about a new media playback
        // this will however do nothing as we have no actual playback going on
        platform.notify_media_event(MediaNotificationEvent::StateStarting(MediaInfo {
            title: "Lorem".to_string(),
            subtitle: None,
            thumb: None,
        }));
        // verify that the other events don't crash the program
        // when no controls are present
        platform.notify_media_event(MediaNotificationEvent::StatePaused);
    }

    #[tokio::test]
    async fn test_handle_media_play_event() {
        let (tx, rx) = channel();
        let callbacks = MultiThreadedCallback::new();
        let event = MediaControlEvent::Play;

        let mut receiver = callbacks.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send(event).unwrap();
                break;
            }
        });

        DefaultPlatform::handle_media_event(event, &callbacks.clone());

        let result = timeout(Duration::from_millis(100), rx)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&PlatformEvent::TogglePlaybackState, &*result);
    }

    #[tokio::test]
    async fn test_handle_media_pause_event() {
        let (tx, rx) = channel();
        let callbacks = MultiThreadedCallback::new();
        let event = MediaControlEvent::Pause;

        let mut receiver = callbacks.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send(event).unwrap();
                break;
            }
        });

        DefaultPlatform::handle_media_event(event, &callbacks.clone());

        let result = timeout(Duration::from_millis(100), rx)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&PlatformEvent::TogglePlaybackState, &*result);
    }

    #[tokio::test]
    async fn test_handle_media_next_event() {
        let (tx, rx) = channel();
        let callbacks = MultiThreadedCallback::new();
        let event = MediaControlEvent::Next;

        let mut receiver = callbacks.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send(event).unwrap();
                break;
            }
        });

        DefaultPlatform::handle_media_event(event, &callbacks.clone());

        let result = timeout(Duration::from_millis(100), rx)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&PlatformEvent::ForwardMedia, &*result);
    }
}
