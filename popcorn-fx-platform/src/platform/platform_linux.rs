use log::warn;

use popcorn_fx_core::core::platform::Platform;

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {

}

impl Platform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        warn!("disable_screensaver has not been implemented for Linux");
        true
    }

    fn enable_screensaver(&self) -> bool {
        warn!("enable_screensaver has not been implemented for Linux");
        true
    }
}

impl Default for PlatformLinux {
    fn default() -> Self {
        Self {

        }
    }
}

#[cfg(test)]
mod test {

}