use crate::{new_platform, Platform};

/// The actions for the current platform.
/// This is a C wrapper around the rust struct.
#[repr(C)]
pub struct PlatformC {
    platform: Box<dyn Platform>,
}

impl PlatformC {
    pub fn new() -> PlatformC {
        PlatformC {
            platform: new_platform()
        }
    }

    /// Disable the screensaver on the current platform.
    /// It returns true when the screensaver was disabled, else false.
    pub fn disable_screensaver(&mut self) -> bool {
        self.platform.disable_screensaver()
    }

    /// Enable the screensaver on the current platform.
    /// It returns true when the screensaver was enabled, else false.
    pub fn enable_screensaver(&mut self) -> bool {
        self.platform.enable_screensaver()
    }
}