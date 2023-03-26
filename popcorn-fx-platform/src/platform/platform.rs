use std::env::consts::{ARCH, OS};
use std::fmt;
use std::fmt::Debug;
use std::os::raw::c_void;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use tokio::sync::Mutex;

use popcorn_fx_core::core::platform::{Platform, PlatformData, PlatformInfo, PlatformType};
use popcorn_fx_core::core::playback::{MediaInfo, MediaNotificationEvent};

#[cfg(target_os = "linux")]
use crate::platform::platform_linux::PlatformLinux;
#[cfg(target_os = "macos")]
use crate::platform::platform_mac::PlatformMac;
#[cfg(target_os = "windows")]
use crate::platform::platform_win::PlatformWin;

const DBUS_NAME: &str = ":popcorn_time.media";
const DISPLAY_NAME: &str = "Popcorn Time";

/// The `DefaultPlatform` struct represents the [PlatformData], which contains a reference to a
/// platform and platform information.
pub struct DefaultPlatform {
    pub platform: Arc<Box<dyn Platform>>,
    pub controls: Mutex<Option<MediaControls>>,
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
            Ok(e) => {
                debug!("System media controls have been created");
                Some(e)
            }
            Err(e) => {
                error!("Failed to create system media controls, {:?}", e);
                None
            }
        }
    }

    fn create_controls_config(&self) -> PlatformConfig {
        PlatformConfig {
            dbus_name: DBUS_NAME,
            display_name: DISPLAY_NAME,
            hwnd: self.platform.window_handle(),
        }
    }

    fn on_media_info_changed(&self, controls: &mut MediaControls, info: MediaInfo) {
        let metadata = MediaMetadata {
            title: Some(&info.title),
            artist: info.show_name.as_ref().map(|e| e.as_str()),
            cover_url: info.thumb.as_ref().map(|e| e.as_ref()),
            ..Default::default()
        };

        // this always needs to be done before calling `controls.set_metadata`
        match controls.attach(|event: MediaControlEvent| info!("Received media controle event {:?}", event)) {
            Ok(_) => debug!("System media controls attached"),
            Err(e) => error!("Failed to attach system media controls, {:?}", e)
        };

        trace!("Notifying system of media playback {:?}", metadata);
        match controls.set_metadata(metadata) {
            Ok(_) => info!("System has been notified of the new media playback"),
            Err(e) => error!("System media notification failed, {:?}", e)
        };
    }

    fn on_playback_state_changed(&self, controls: &mut MediaControls, state: MediaPlayback) {
        let state_info = format!("{:?}", state);

        trace!("Updating system media playback state to {}", state_info);
        match controls.set_playback(state) {
            Ok(_) => debug!("System media state has changed {}", state_info),
            Err(e) => error!("System media state couldn't be updated, {:?}", e)
        }
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
        let mut mutex = futures::executor::block_on(self.controls.lock());

        // check if the controls already exist
        // if not, we'll create them first
        if mutex.is_none() {
            *mutex = self.create_controls();
        }

        if let Some(mut controls) = mutex.as_mut() {
            match event {
                MediaNotificationEvent::PlaybackStarted(info) => self.on_media_info_changed(&mut controls, info),
                MediaNotificationEvent::StatePlaying => self.on_playback_state_changed(&mut controls, MediaPlayback::Playing { progress: None }),
                MediaNotificationEvent::StatePaused => self.on_playback_state_changed(&mut controls, MediaPlayback::Paused { progress: None }),
                MediaNotificationEvent::StateStopped => {
                    self.on_playback_state_changed(&mut controls, MediaPlayback::Stopped);
                    // release the controls
                    debug!("Releasing system media controls");
                    *mutex = None;
                }
            }
        } else {
            warn!("Unable to handle the media playback notification, MediaControls not present")
        }
    }

    fn window_handle(&self) -> Option<*mut c_void> {
        None
    }
}

impl PlatformData for DefaultPlatform {
    fn info(&self) -> PlatformInfo {
        trace!("Retrieving system information");
        let platform_type = match OS {
            "windows" => PlatformType::Windows,
            "macos" => PlatformType::MacOs,
            _ => PlatformType::Linux
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
    use popcorn_fx_core::core::platform::MockDummyPlatform;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_disable_screensaver() {
        init_logger();
        let mut sys_platform = MockDummyPlatform::new();
        sys_platform.expect_disable_screensaver()
            .returning(|| true);
        sys_platform.expect_enable_screensaver()
            .returning(|| false);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
        };

        assert!(platform.disable_screensaver(), "expected the screensaver to be disabled")
    }

    #[test]
    fn test_enable_screensaver() {
        init_logger();
        let mut sys_platform = MockDummyPlatform::new();
        sys_platform.expect_enable_screensaver()
            .returning(|| true);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
        };

        assert!(platform.enable_screensaver(), "expected the screensaver to be enabled")
    }

    #[test]
    fn test_drop_default_platform() {
        init_logger();
        let mut sys_platform = MockDummyPlatform::new();
        sys_platform.expect_enable_screensaver()
            .returning(|| true)
            .times(1);
        let platform = DefaultPlatform {
            platform: Arc::new(Box::new(sys_platform)),
            controls: Default::default(),
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
        init_logger();
        let platform = DefaultPlatform::default();

        // notify the system about a new media playback
        // this will however do nothing as we have no actual playback going on
        platform.notify_media_event(MediaNotificationEvent::PlaybackStarted(MediaInfo {
            title: "Lorem".to_string(),
            show_name: None,
            thumb: None,
        }));
        // verify that the other events don't crash the program
        // when no controls are present
        platform.notify_media_event(MediaNotificationEvent::StatePaused);
    }
}