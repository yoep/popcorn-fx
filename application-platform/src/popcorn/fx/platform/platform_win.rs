use std::io::Error;

use log::{info, trace, warn};
use windows::core::PWSTR;
use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Power::{PowerClearRequest, PowerCreateRequest, PowerRequestDisplayRequired, PowerSetRequest};
use windows::Win32::System::Threading::{POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0};

use crate::popcorn::fx::platform::platform::Platform;

/// Windows specific platform instructions
pub struct PlatformWin {
    /// The power request which has been made to the windows system
    request: Option<HANDLE>,
}

impl PlatformWin {
    /// Create a new windows platform instance.
    /// It returns the created instance.
    pub fn new() -> PlatformWin {
        return PlatformWin { request: None };
    }
}

impl Platform for PlatformWin {
    fn disable_screensaver(&mut self) -> bool {
        let mut encoded = "Popcorn FX playing media"
            .encode_utf16()
            .chain([0u16])
            .collect::<Vec<u16>>();

        let context = REASON_CONTEXT {
            Version: 0,
            Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
            Reason: REASON_CONTEXT_0 {
                SimpleReasonString: PWSTR(encoded.as_mut_ptr()),
            },
        };

        unsafe {
            trace!("Creating new windows power request");
            let request: Result<HANDLE> = PowerCreateRequest(&context);

            if request.is_ok() {
                self.request = Some(request.unwrap());

                return if PowerSetRequest(self.request.unwrap(), PowerRequestDisplayRequired).as_bool() {
                    info!("Screensaver has been disabled");
                    true
                } else {
                    warn!("Failed to disable windows screensaver, {}", Error::last_os_error().to_string());
                    false
                };
            }

            warn!("Failed to create windows power request, {}", request.err().unwrap());
            return false;
        }
    }

    fn enable_screensaver(&mut self) -> bool {
        // verify if a request was made before to disable it
        // otherwise, ignore this call
        if self.request.is_none() {
            trace!("Windows screensaver not disabled, not trying to clear power request");
            return true;
        }

        let handle = self.request.unwrap();

        unsafe {
            return if PowerClearRequest(handle, PowerRequestDisplayRequired).as_bool() {
                info!("Screensaver has been enabled");
                self.request = None;
                true
            } else {
                warn!("Failed to enabled windows screensaver");
                false
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_windows_disable_screensaver() {
        let mut platform = PlatformWin::new();

        assert_eq!(platform.disable_screensaver(), true, "Expected the screensaver to have been disabled");
    }

    #[test]
    fn test_windows_enable_screensaver() {
        let mut platform = PlatformWin::new();

        assert_eq!(platform.disable_screensaver(), true, "Expected the screensaver to have been disabled");
        assert_eq!(platform.enable_screensaver(), true, "Expected the screensaver to have been enabled");
    }
}