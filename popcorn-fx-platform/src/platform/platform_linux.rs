use popcorn_fx_core::core::platform::Platform;

use crate::platform::Platform;

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {

}

impl Platform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        false
    }

    fn enable_screensaver(&self) -> bool {
        false
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