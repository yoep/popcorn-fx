use log::trace;

use crate::popcorn::fx::platform::platform_info::PlatformInfo;
#[cfg(target_os = "linux")]
use crate::popcorn::fx::platform::platform_linux::PlatformLinux;
#[cfg(target_os = "macos")]
use crate::popcorn::fx::platform::platform_mac::PlatformMac;
#[cfg(target_os = "windows")]
use crate::popcorn::fx::platform::platform_win::PlatformWin;

/// Platform defines native system functions
pub trait Platform {
    /// Disable the screensaver on the current platform
    /// It returns `true` if the screensaver was disabled with success, else `false`.
    fn disable_screensaver(&mut self) -> bool;

    /// Enable the screensaver on the current platform
    /// It returns `true` if the screensaver was enabled with success, else `false`.
    fn enable_screensaver(&mut self) -> bool;
}

pub trait PlatformService : Platform {
    /// Retrieve the platform info of the current system.
    fn platform_info(&self) -> &PlatformInfo;
}

/// Initialize a new platform
#[cfg(target_os = "windows")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformWin::new())
}

/// Initialize a new platform
#[cfg(target_os = "macos")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformMac::new())
}

/// Initialize a new platform
#[cfg(target_os = "linux")]
pub fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformLinux::new());
}

/// A basic implementation of the platform service which handles most system actions and information.
pub struct PlatformServiceImpl {
    platform: Box<dyn Platform>,
    platform_info: PlatformInfo,
}

impl PlatformServiceImpl {
    pub fn new() -> Self {
        Self {
            platform: new_platform(),
            platform_info: PlatformInfo::new()
        }
    }
}

impl Platform for PlatformServiceImpl {
    fn disable_screensaver(&mut self) -> bool {
        self.platform.disable_screensaver()
    }

    fn enable_screensaver(&mut self) -> bool {
        self.platform.enable_screensaver()
    }
}

impl PlatformService for PlatformServiceImpl {
    fn platform_info(&self) -> &PlatformInfo {
        trace!("Retrieving system information");
        &self.platform_info
    }
}

#[cfg(test)]
mod test{
    use crate::popcorn::fx::platform::platform::new_platform;

    #[test]
    fn test_new_platform_should_return_platform() {
        new_platform();
    }
}