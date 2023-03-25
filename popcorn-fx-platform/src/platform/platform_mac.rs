use std::os::raw::c_int;

use log::{debug, info, warn};

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use popcorn_fx_core::core::platform::Platform;

const KIOPMASSERTIONLEVEL_ON: u32 = 255;
const KIOPMASSERTIONLEVEL_OFF: u32 = 0;

#[link(name = "IOKit", kind = "framework")]
extern {
    #[allow(non_snake_case)]
    fn IOPMAssertionCreateWithName(AssertionType: CFStringRef, AssertionLevel: u32, AssertionName: CFStringRef, AssertionID: *mut u32) -> c_int;
}

#[derive(Debug)]
pub struct PlatformMac {}

impl PlatformMac {
    pub fn new() -> PlatformMac {
        return PlatformMac {};
    }

    fn call_io_assertion(&self, assertion_level: u32) -> bool {
        let prevent_sleep = CFString::new("PreventUserIdleSystemSleep");
        let reason = CFString::new("Media playback application is active");
        #[allow(unused_mut)]
            let mut id = Box::new(u32::MIN);

        unsafe {
            debug!("Calling IOPMAssertion on macos with assertion level {}", assertion_level);
            let result = IOPMAssertionCreateWithName(prevent_sleep.as_concrete_TypeRef(), assertion_level, reason.as_concrete_TypeRef(), Box::into_raw(id));

            if result == 0 {
                debug!("IOPMAssertion succeeded");
                return true;
            }
        }

        warn!("Failed to invoke IOPMAssertion");
        return false;
    }
}

impl Platform for PlatformMac {
    fn disable_screensaver(&self) -> bool {
        let result = self.call_io_assertion(KIOPMASSERTIONLEVEL_ON);
        info!("Disable screensaver returned state {}", result);
        result
    }

    fn enable_screensaver(&self) -> bool {
        let result = self.call_io_assertion(KIOPMASSERTIONLEVEL_OFF);
        info!("Enable screensaver returned state {}", result);
        result
    }

    fn notify_media_event(&self, _: MediaNotification) {
        // no-op
    }

    fn window_handle(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::platform::Platform;
    use popcorn_fx_core::testing::init_logger;

    use super::PlatformMac;

    #[test]
    fn disable_screensaver_macos_should_return_true() {
        init_logger();
        let mut platform = PlatformMac::new();

        assert_eq!(true, platform.disable_screensaver());
    }

    #[test]
    fn enable_screensaver_macos_should_return_true() {
        init_logger();
        let mut platform = PlatformMac::new();

        assert_eq!(true, platform.disable_screensaver(), "Failed to disable the screensaver first");
        assert_eq!(true, platform.enable_screensaver());
    }

    #[test]
    fn test_window_handle() {
        init_logger();
        let platform = PlatformLinux::default();

        assert_eq!(None, platform.window_handle())
    }
}