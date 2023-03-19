use log::{debug, error, info, trace};
use x11rb::connection::RequestConnection;
use x11rb::protocol::dpms::{ConnectionExt as DpmsConnectionExt, DPMSMode};
use x11rb::protocol::xproto::{Blanking, ConnectionExt as ScreensaverConnectionExt, Exposures};
use x11rb::rust_connection::RustConnection;

use popcorn_fx_core::core::platform;
use popcorn_fx_core::core::platform::{Platform, PlatformError};

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {
    /// The X11 server connection
    conn: RustConnection,
}

impl PlatformLinux {
    fn update_dpms_state(&self, mode: DPMSMode) -> platform::Result<()> {
        if let None = self.conn.extension_information(x11rb::protocol::dpms::X11_EXTENSION_NAME).unwrap() {
            return Err(PlatformError::Screensaver("DPMS extension not found, unable to prevent sleeping mode".to_string()));
        }

        trace!("Sending DPMS force level to X11 server");
        self.conn.dpms_force_level(mode)
            .map_err(|e| PlatformError::Screensaver(e.to_string()))
            .map(|cookie| {
                cookie.check()
                    .map(|_| {
                        debug!("X11 DPMS mode activated");
                        Ok(())
                    })
                    .map_err(|e| PlatformError::Screensaver(e.to_string()))?
            })?
    }

    fn disable_x11_screensaver(&self) -> platform::Result<()> {
        trace!("Sending screensaver attributes to X11");
        self.conn.set_screen_saver(i16::MAX, 0, Blanking::NOT_PREFERRED, Exposures::NOT_ALLOWED)
            .map_err(|e| PlatformError::Screensaver(format!("X11 connection error, {}", e)))
            .map(|cookie| {
                cookie.check()
                    .map(|_| {
                        debug!("Screensaver has been disabled");
                        Ok(())
                    })
                    .map_err(|e| PlatformError::Screensaver(e.to_string()))?
            })?
    }
}

impl Platform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        match self.update_dpms_state(DPMSMode::ON) {
            Ok(_) => {
                match self.disable_x11_screensaver() {
                    Ok(_) => {
                        info!("X11 sleep mode prevented");
                        return true;
                    }
                    Err(e) => error!("Screensaver failed, {}", e)
                }
            }
            Err(e) => error!("Power management failed, {}", e)
        }

        false
    }

    fn enable_screensaver(&self) -> bool {
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
}

impl Default for PlatformLinux {
    fn default() -> Self {
        let (conn, _) = x11rb::connect(None).unwrap();

        Self {
            conn
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_disable_screensaver() {
        init_logger();
        let platform = PlatformLinux::default();

        let result = platform.disable_screensaver();

        assert_eq!(true, result);
    }

    #[test]
    fn test_enable_screensaver() {
        let platform = PlatformLinux::default();

        let result = platform.enable_screensaver();

        assert_eq!(true, result)
    }
}