#[cfg(target_os = "windows")]
use crate::popcorn::fx::platform::platform_win::PlatformWin;

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
fn new_platform() -> Box<dyn Platform> {
    return Box::new(PlatformWin::new())
}