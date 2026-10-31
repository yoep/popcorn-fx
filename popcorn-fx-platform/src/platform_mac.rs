use crate::platform::SystemPlatform;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use log::{debug, error, info, trace, warn};
use std::ffi::c_int;
use std::sync::Mutex;

const KIOPMASSERTIONLEVEL_ON: u32 = 255;
const ASSERTION_TYPE_DISPLAY_SLEEP: &str = "PreventUserIdleDisplaySleep";
const ASSERTION_REASON: &str = "Media playback application is active";
const KIORETURN_SUCCESS: c_int = 0;

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    #[allow(non_snake_case)]
    fn IOPMAssertionCreateWithName(
        AssertionType: CFStringRef,
        AssertionLevel: u32,
        AssertionName: CFStringRef,
        AssertionID: *mut u32,
    ) -> c_int;

    #[allow(non_snake_case)]
    fn IOPMAssertionRelease(AssertionID: u32) -> c_int;
}

/// MacOS specific platform instructions.
#[derive(Debug, Default)]
pub struct PlatformMac {
    /// The active power assertion preventing the display from sleeping.
    assertion_id: Mutex<Option<u32>>,
}

impl SystemPlatform for PlatformMac {
    fn disable_screensaver(&self) -> bool {
        let mut guard = match self.assertion_id.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire the IOPMAssertion lock, {}", e);
                return false;
            }
        };
        if guard.is_some() {
            trace!("Screensaver is already disabled");
            return true;
        }

        let assertion_type = CFString::new(ASSERTION_TYPE_DISPLAY_SLEEP);
        let reason = CFString::new(ASSERTION_REASON);
        let mut id: u32 = 0;

        let result = unsafe {
            IOPMAssertionCreateWithName(
                assertion_type.as_concrete_TypeRef(),
                KIOPMASSERTIONLEVEL_ON,
                reason.as_concrete_TypeRef(),
                &mut id,
            )
        };

        if result == KIORETURN_SUCCESS {
            *guard = Some(id);
            info!("Screensaver has been disabled (assertion {})", id);
            true
        } else {
            warn!("Failed to create IOPMAssertion, IOReturn {}", result);
            false
        }
    }

    fn enable_screensaver(&self) -> bool {
        let mut guard = match self.assertion_id.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire the IOPMAssertion lock, {}", e);
                return false;
            }
        };

        match guard.take() {
            None => {
                debug!("Screensaver not disabled, not releasing any IOPMAssertion");
                true
            }
            Some(id) => {
                let result = unsafe { IOPMAssertionRelease(id) };
                if result == KIORETURN_SUCCESS {
                    info!("Screensaver has been enabled (assertion {} released)", id);
                    true
                } else {
                    warn!(
                        "Failed to release IOPMAssertion {}, IOReturn {}",
                        id, result
                    );
                    *guard = Some(id);
                    false
                }
            }
        }
    }

    fn window_handle(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
}

impl Drop for PlatformMac {
    fn drop(&mut self) {
        self.enable_screensaver();
    }
}

#[cfg(test)]
mod test {
    use crate::platform::SystemPlatform;

    use super::PlatformMac;

    #[test]
    fn disable_screensaver_macos_should_return_true() {
        init_logger!();
        let platform = PlatformMac::default();

        assert_eq!(true, platform.disable_screensaver());
    }

    #[test]
    fn enable_screensaver_macos_should_return_true() {
        init_logger!();
        let platform = PlatformMac::default();

        assert_eq!(
            true,
            platform.disable_screensaver(),
            "Failed to disable the screensaver first"
        );
        assert_eq!(true, platform.enable_screensaver());
    }

    #[test]
    fn test_window_handle() {
        init_logger!();
        let platform = PlatformMac::default();

        assert_eq!(None, platform.window_handle())
    }
}
