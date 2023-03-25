use log::{debug, info, trace, warn};
use tokio::sync::Mutex;
use windows::core::{PCWSTR, PWSTR};
use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Power::{PowerClearRequest, PowerCreateRequest, PowerRequestDisplayRequired, PowerSetRequest};
use windows::Win32::System::Threading::{POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0};

use popcorn_fx_core::core::platform::Platform;

const WINDOW_NAME: &str = "Popcorn Time";

/// Windows specific platform instructions
#[derive(Debug)]
pub struct PlatformWin {
    /// The power request which has been made to the windows system
    screensaver_request: Mutex<Option<HANDLE>>,
}

impl Platform for PlatformWin {
    fn disable_screensaver(&self) -> bool {
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

            return match request {
                Err(ex) => {
                    warn!("Failed to disable windows screensaver, {}", ex);
                    false
                }
                Ok(handle) => {
                    trace!("Storing windows screensaver handle");
                    let mut mutex = self.screensaver_request.blocking_lock();
                    *mutex = Some(handle);

                    if PowerSetRequest(handle, PowerRequestDisplayRequired).as_bool() {
                        info!("Screensaver has been disabled");
                        true
                    } else {
                        false
                    }
                }
            };
        }
    }

    fn enable_screensaver(&self) -> bool {
        // verify if a request was made before to disable it
        // otherwise, ignore this call
        let mut mutex = self.screensaver_request.blocking_lock();

        if let Some(handle) = *mutex {
            if unsafe { PowerClearRequest(handle, PowerRequestDisplayRequired).as_bool() } {
                info!("Screensaver has been enabled");
                *mutex = None;
                true
            } else {
                warn!("Failed to enabled windows screensaver");
                false
            }
        } else {
            trace!("Windows screensaver not disabled, not trying to clear power request");
            true
        }
    }

    fn window_handle(&self) -> Option<*mut std::ffi::c_void> {
        let mut encoded_name = WINDOW_NAME
            .encode_utf16()
            .chain([0u16])
            .collect::<Vec<u16>>();

        trace!("Retrieving window handle");
        let handle = unsafe { windows::Win32::UI::WindowsAndMessaging::FindWindowW(PCWSTR::null(), PCWSTR(encoded_name.as_mut_ptr())) };

        if handle.0 == 0 {
            debug!("Window handle couldn't be found");
            None
        } else {
            trace!("Converting window handle {:?} to std::ffi::c_void", handle);
            Some(handle.0 as *mut std::ffi::c_void)
        }
    }
}

impl Default for PlatformWin {
    fn default() -> Self {
        Self {
            screensaver_request: Mutex::new(None)
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_windows_disable_screensaver() {
        let platform = PlatformWin::default();

        assert_eq!(platform.disable_screensaver(), true, "Expected the screensaver to have been disabled");
    }

    #[test]
    fn test_windows_enable_screensaver() {
        let platform = PlatformWin::default();

        assert_eq!(platform.disable_screensaver(), true, "Expected the screensaver to have been disabled");
        assert_eq!(platform.enable_screensaver(), true, "Expected the screensaver to have been enabled");
    }

    #[test]
    fn test_window_handle() {
        init_logger();
        let platform = PlatformWin::default();

        let handle = platform.window_handle();
        info!("Retrieved window handle {:?}", handle);
    }
}