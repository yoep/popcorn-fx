use std::fmt;
use std::fmt::Debug;
use std::os::raw::c_void;
use std::sync::Arc;

use log::{debug, error, trace};
use souvlaki::{MediaControls, PlatformConfig};
use tokio::sync::Mutex;

use popcorn_fx_core::core::platform::{Platform, PlatformData, PlatformInfo, PlatformType};

#[cfg(target_os = "linux")]
use crate::platform::platform_linux::PlatformLinux;
#[cfg(target_os = "macos")]
use crate::platform::platform_mac::PlatformMac;
#[cfg(target_os = "windows")]
use crate::platform::platform_win::PlatformWin;

#[cfg(target_arch = "x86_64")]
const X64: &str = "x86-64";
#[cfg(target_arch = "arm")]
const ARM: &str = "arm";
#[cfg(target_arch = "aarch64")]
const ARCH64: &str = "aarch64";

const DBUS_NAME: &str = ":popcorn_time.media";
const DISPLAY_NAME: &str = "Popcorn Time";

/// Initialize a new platform
#[cfg(target_os = "windows")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformWin::default());
}

/// Initialize a new platform
#[cfg(target_os = "macos")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformMac::new());
}

/// Initialize a new platform
#[cfg(target_os = "linux")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformLinux::default());
}

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
pub fn platform_info() -> PlatformInfo {
    trace!("Retrieving windows platform info");
    PlatformInfo {
        platform_type: PlatformType::Windows,
        arch: String::from(X64),
    }
}

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
pub fn platform_info() -> PlatformInfo {
    trace!("Retrieving macos platform info");
    PlatformInfo {
        platform_type: PlatformType::MacOs,
        arch: String::from(X64),
    }
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
pub fn platform_info() -> PlatformInfo {
    trace!("Retrieving linux platform info");
    PlatformInfo {
        platform_type: PlatformType::Linux,
        arch: String::from(X64),
    }
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "aarch64")]
pub fn platform_info() -> PlatformInfo {
    trace!("Retrieving linux platform info");
    PlatformInfo {
        platform_type: PlatformType::Linux,
        arch: String::from(ARCH64.to_string()),
    }
}

#[cfg(target_os = "linux")]
#[cfg(target_arch = "arm")]
pub fn platform_info() -> PlatformInfo {
    trace!("Retrieving linux platform info");
    PlatformInfo {
        platform_type: PlatformType::Linux,
        arch: String::from(ARM.to_string()),
    }
}

/// The `DefaultPlatform` struct represents the [PlatformData], which contains a reference to a
/// platform and platform information.
pub struct DefaultPlatform {
    pub platform: Arc<Box<dyn Platform>>,
    pub platform_info: PlatformInfo,
    pub controls: Mutex<Option<MediaControls>>,
}

impl DefaultPlatform {
    fn create_controls(&self) -> Option<MediaControls> {
        match MediaControls::new(self.create_controls_config()) {
            Ok(e) => {
                debug!("Media controls have been created");
                Some(e)
            }
            Err(e) => {
                error!("Failed to create media controls, {:?}", e);
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
}

impl Platform for DefaultPlatform {
    fn disable_screensaver(&self) -> bool {
        self.platform.disable_screensaver()
    }

    fn enable_screensaver(&self) -> bool {
        self.platform.enable_screensaver()
    }

    fn window_handle(&self) -> Option<*mut c_void> {
        None
    }
}

impl PlatformData for DefaultPlatform {
    fn info(&self) -> &PlatformInfo {
        trace!("Retrieving system information");
        &self.platform_info
    }
}

impl Default for DefaultPlatform {
    fn default() -> Self {
        Self {
            platform: Arc::new(new_platform()),
            platform_info: platform_info(),
            controls: Default::default(),
        }
    }
}

impl Debug for DefaultPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultPlatform")
            .field("platform", &self.platform)
            .field("platform_info", &self.platform_info)
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

    #[test]
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    fn test_new_platform_should_return_platform() {
        let platform = new_platform();

        assert!(platform.enable_screensaver(), "expected the screensaver to have been enabled")
    }

    #[test]
    fn test_default_platform() {
        let platform = DefaultPlatform::default();
        let expected_result = platform_info();

        let result = platform.info();

        assert_eq!(&expected_result, result)
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_platform_info() {
        let info = platform_info();

        assert!(matches!(info.platform_type, PlatformType::Windows));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_platform_info() {
        let info = platform_info();

        assert!(matches!(info.platform_type, PlatformType::Linux));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_platform_info() {
        let info = platform_info();

        assert!(matches!(info.platform_type, PlatformType::MacOs));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_platform_info_new_should_return_x64_info() {
        let info = platform_info();

        assert_eq!(X64, String::from(info.arch))
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_platform_info_new_should_return_aarch64_info() {
        let info = platform_info();

        assert_eq!(ARCH64, String::from(info.arch))
    }

    #[test]
    #[cfg(target_arch = "arm")]
    fn test_platform_info_new_should_return_arm_info() {
        let info = platform_info();

        assert_eq!(ARM, String::from(info.arch))
    }
}