use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use libc::c_int;
use log::{debug, warn};

use crate::popcorn::fx::platform::platform::Platform;

const KIOPMASSERTIONLEVEL_ON: u32 = 255;
const KIOPMASSERTIONLEVEL_OFF: u32 = 0;

#[link(name = "IOKit", kind = "framework")]
extern {
    #[allow(non_snake_case)]
    fn IOPMAssertionCreateWithName(AssertionType: CFStringRef, AssertionLevel: u32, AssertionName: CFStringRef, AssertionID: *mut u32) -> c_int;
}

pub struct PlatformMac {}

impl PlatformMac {
    pub fn new() -> PlatformMac {
        return PlatformMac {};
    }

    fn call_io_assertion(&mut self, assertion_level: u32) -> bool {
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
    fn disable_screensaver(&mut self) -> bool {
        self.call_io_assertion(KIOPMASSERTIONLEVEL_ON)
    }

    fn enable_screensaver(&mut self) -> bool {
        self.call_io_assertion(KIOPMASSERTIONLEVEL_OFF)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn disable_screensaver_macos_should_return_true() {
        let mut platform = PlatformMac::new();

        assert_eq!(true, platform.disable_screensaver());
    }

    #[test]
    fn enable_screensaver_macos_should_return_true() {
        let mut platform = PlatformMac::new();

        assert_eq!(true, platform.disable_screensaver(), "Failed to disable the screensaver first");
        assert_eq!(true, platform.enable_screensaver());
    }
}