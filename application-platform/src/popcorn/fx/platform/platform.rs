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