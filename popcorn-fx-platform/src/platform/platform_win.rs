use crate::platform::SystemPlatform;
use log::{error, info, trace, warn};
use std::sync::Mutex;
use windows::core::Result;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Power::{
    PowerClearRequest, PowerCreateRequest, PowerRequestDisplayRequired, PowerSetRequest,
};
use windows::Win32::System::Threading::{
    POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0,
};

const WINDOW_NAME: &str = "Popcorn Time";

/// Windows specific platform instructions
#[derive(Debug)]
pub struct PlatformWin {
    /// The power request which has been made to the windows system
    screensaver_request: Mutex<Option<HANDLE>>,
}

impl SystemPlatform for PlatformWin {
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

            match request {
                Err(ex) => {
                    warn!("Failed to disable windows screensaver, {}", ex);
                    false
                }
                Ok(handle) => {
                    trace!("Storing windows screensaver handle");
                    match self.screensaver_request.lock() {
                        Ok(mut mutex) => {
                            *mutex = Some(handle);

                            if PowerSetRequest(handle, PowerRequestDisplayRequired).is_ok() {
                                info!("Screensaver has been disabled");
                                return true;
                            }
                        }
                        Err(e) => error!("Failed to acquire windows screensaver lock, {}", e),
                    }

                    false
                }
            }
        }
    }

    fn enable_screensaver(&self) -> bool {
        // verify if a request was made before to disable it
        // otherwise, ignore this call
        if let Ok(mut mutex) = self.screensaver_request.lock() {
            if let Some(handle) = *mutex {
                if unsafe { PowerClearRequest(handle, PowerRequestDisplayRequired).is_ok() } {
                    info!("Screensaver has been enabled");
                    *mutex = None;
                    return true;
                } else {
                    warn!("Failed to enabled windows screensaver");
                }
            } else {
                trace!("Windows screensaver not disabled, not trying to clear power request");
                return true;
            }
        }

        false
    }

    fn window_handle(&self) -> Option<*mut core::ffi::c_void> {
        let mut encoded_name = WINDOW_NAME
            .encode_utf16()
            .chain([0u16])
            .collect::<Vec<u16>>();

        trace!("Retrieving window handle");
        let handle = unsafe {
            windows::Win32::UI::WindowsAndMessaging::FindWindowW(
                PCWSTR::null(),
                PCWSTR(encoded_name.as_mut_ptr()),
            )
        };

        match handle {
            Ok(handle) => {
                if handle.is_invalid() {
                    warn!("Failed to find window handle");
                    None
                } else {
                    Some(handle.0)
                }
            }
            Err(err) => {
                warn!("Failed to retrieve window handle, {}", err);
                None
            }
        }
    }
}

impl Default for PlatformWin {
    fn default() -> Self {
        Self {
            screensaver_request: Mutex::new(None),
        }
    }
}

unsafe impl Send for PlatformWin {}
unsafe impl Sync for PlatformWin {}

#[cfg(test)]
mod test {
    use popcorn_fx_core::init_logger;

    use super::*;

    #[test]
    fn test_windows_disable_screensaver() {
        let platform = PlatformWin::default();

        assert_eq!(
            platform.disable_screensaver(),
            true,
            "Expected the screensaver to have been disabled"
        );
    }

    #[test]
    fn test_windows_enable_screensaver() {
        let platform = PlatformWin::default();

        assert_eq!(
            platform.disable_screensaver(),
            true,
            "Expected the screensaver to have been disabled"
        );
        assert_eq!(
            platform.enable_screensaver(),
            true,
            "Expected the screensaver to have been enabled"
        );
    }

    #[test]
    fn test_window_handle() {
        init_logger!();
        let platform = PlatformWin::default();

        let handle = platform.window_handle();
        info!("Retrieved window handle {:?}", handle);
    }
}
