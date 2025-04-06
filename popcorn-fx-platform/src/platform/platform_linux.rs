use log::{debug, error, info, trace, warn};

use popcorn_fx_core::core::platform;
use popcorn_fx_core::core::platform::PlatformError;
use x11rb::connection::RequestConnection;
use x11rb::protocol::dpms::{ConnectionExt as DpmsConnectionExt, DPMSMode};
use x11rb::protocol::xproto::{Blanking, ConnectionExt as ScreensaverConnectionExt, Exposures};
use x11rb::rust_connection::{ConnectionError, RustConnection};

use crate::platform::SystemPlatform;

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {
    /// The X11 server connection
    conn: Option<RustConnection>,
}

impl PlatformLinux {
    fn update_dpms_state(&self, mode: DPMSMode) -> platform::Result<()> {
        let conn = self.conn.as_ref().unwrap();
        if let None = conn
            .extension_information(x11rb::protocol::dpms::X11_EXTENSION_NAME)
            .unwrap()
        {
            return Err(PlatformError::Screensaver(
                "DPMS extension not found, unable to prevent sleeping mode".to_string(),
            ));
        }

        trace!("Sending DPMS force level to X11 server");
        conn.dpms_force_level(mode)
            .map_err(|e| PlatformError::Screensaver(e.to_string()))
            .map(|cookie| {
                cookie
                    .check()
                    .map(|_| {
                        debug!("X11 DPMS mode activated");
                        Ok(())
                    })
                    .map_err(|e| PlatformError::Screensaver(e.to_string()))?
            })?
    }

    fn disable_x11_screensaver(&self) -> platform::Result<()> {
        let conn = self.conn.as_ref().unwrap();

        trace!("Sending screensaver attributes to X11");
        conn.set_screen_saver(i16::MAX, 0, Blanking::NOT_PREFERRED, Exposures::NOT_ALLOWED)
            .map_err(|e| PlatformError::Screensaver(format!("X11 connection error, {}", e)))
            .map(|cookie| {
                cookie
                    .check()
                    .map(|_| {
                        debug!("Screensaver has been disabled");
                        Ok(())
                    })
                    .map_err(|e| PlatformError::Screensaver(e.to_string()))?
            })?
    }
}

impl SystemPlatform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        if self.conn.is_none() {
            warn!("Unable to disable_screensaver, no X11 connection could be established");
            return false;
        }

        match self.update_dpms_state(DPMSMode::ON) {
            Ok(_) => match self.disable_x11_screensaver() {
                Ok(_) => {
                    info!("X11 screensaver mode deactivated");
                    return true;
                }
                Err(e) => error!("Screensaver failed, {}", e),
            },
            Err(e) => error!("Power management failed, {}", e),
        }

        false
    }

    fn enable_screensaver(&self) -> bool {
        if self.conn.is_none() {
            warn!("Unable to enable_screensaver, no X11 connection could be established");
            return false;
        }

        match self.update_dpms_state(DPMSMode::OFF) {
            Ok(_) => {
                debug!("Power management has been enabled");
                true
            }
            Err(e) => {
                error!("Failed to enabled power management, {}", e);
                false
            }
        }
    }

    fn window_handle(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
}

impl Default for PlatformLinux {
    fn default() -> Self {
        let conn = x11rb::connect(None)
            .map(|(conn, _)| {
                debug!("X11 connection has been established");
                Some(conn)
            })
            .or_else(|e| {
                error!("Failed to open X11 connection, {}", e);
                Ok::<Option<RustConnection>, ConnectionError>(None)
            })
            .unwrap();

        Self { conn }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::init_logger;

    use crate::platform::platform_linux::PlatformLinux;
    use crate::platform::SystemPlatform;

    /* NOTE: Github actions is unable to activate the DPMS and XScreenSaver within xvfb */
    /* thereby actually verifying the results of the actions is useless as they will always fail within the CI */

    #[test]
    fn test_disable_screensaver() {
        init_logger!();
        let platform = PlatformLinux::default();

        let _ = platform.disable_screensaver();
    }

    #[test]
    fn test_enable_screensaver() {
        let platform = PlatformLinux::default();

        let _ = platform.enable_screensaver();
    }

    #[test]
    fn test_window_handle() {
        let platform = PlatformLinux::default();

        assert_eq!(None, platform.window_handle())
    }
}
