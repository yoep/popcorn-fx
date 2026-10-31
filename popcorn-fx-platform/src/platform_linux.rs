use log::{debug, error, info, trace, warn};
use popcorn_fx_core::core::platform;
use popcorn_fx_core::core::platform::PlatformError;
use std::sync::Mutex;
use x11rb::connection::RequestConnection;
use x11rb::protocol::dpms::{self, ConnectionExt as DpmsConnectionExt};
use x11rb::protocol::xproto::{Blanking, ConnectionExt as XprotoConnectionExt, Exposures};
use x11rb::rust_connection::{ConnectionError, RustConnection};

use crate::platform::SystemPlatform;

/// The X11 power settings which were active before the screensaver was disabled.
#[derive(Debug, Clone, Copy)]
struct SavedState {
    timeout: i16,
    interval: i16,
    prefer_blanking: Blanking,
    allow_exposures: Exposures,
    /// Indicates if DPMS was enabled, `None` if the extension isn't available.
    dpms_enabled: Option<bool>,
}

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {
    /// The X11 server connection
    conn: Option<RustConnection>,
    /// The settings to restore when the screensaver is enabled again, `Some` while disabled.
    saved: Mutex<Option<SavedState>>,
}

impl PlatformLinux {
    fn has_dpms(conn: &RustConnection) -> bool {
        matches!(
            conn.extension_information(dpms::X11_EXTENSION_NAME),
            Ok(Some(_))
        )
    }

    fn err<E: ToString>(e: E) -> PlatformError {
        PlatformError::Screensaver(e.to_string())
    }

    fn apply_disable(conn: &RustConnection) -> platform::Result<SavedState> {
        let current = conn
            .get_screen_saver()
            .map_err(Self::err)?
            .reply()
            .map_err(Self::err)?;

        let dpms_enabled = if Self::has_dpms(conn) {
            let info = conn
                .dpms_info()
                .map_err(Self::err)?
                .reply()
                .map_err(Self::err)?;
            Some(info.state)
        } else {
            warn!("DPMS extension not found, only the X11 screensaver will be disabled");
            None
        };

        let saved = SavedState {
            timeout: current.timeout as i16,
            interval: current.interval as i16,
            prefer_blanking: current.prefer_blanking,
            allow_exposures: current.allow_exposures,
            dpms_enabled,
        };

        trace!("Disabling the X11 screensaver timeout");
        conn.set_screen_saver(0, 0, Blanking::NOT_PREFERRED, Exposures::NOT_ALLOWED)
            .map_err(Self::err)?
            .check()
            .map_err(Self::err)?;

        if dpms_enabled == Some(true) {
            trace!("Disabling X11 DPMS");
            conn.dpms_disable()
                .map_err(Self::err)?
                .check()
                .map_err(Self::err)?;
        }

        Ok(saved)
    }

    fn apply_restore(conn: &RustConnection, saved: &SavedState) -> platform::Result<()> {
        conn.set_screen_saver(
            saved.timeout,
            saved.interval,
            saved.prefer_blanking,
            saved.allow_exposures,
        )
        .map_err(Self::err)?
        .check()
        .map_err(Self::err)?;

        if saved.dpms_enabled == Some(true) {
            conn.dpms_enable()
                .map_err(Self::err)?
                .check()
                .map_err(Self::err)?;
        }

        Ok(())
    }
}

impl SystemPlatform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        let Some(conn) = self.conn.as_ref() else {
            warn!("Unable to disable_screensaver, no X11 connection could be established");
            return false;
        };
        let Ok(mut guard) = self.saved.lock() else {
            error!("Failed to acquire the screensaver lock");
            return false;
        };
        if guard.is_some() {
            trace!("Screensaver is already disabled");
            return true;
        }

        match Self::apply_disable(conn) {
            Ok(saved) => {
                *guard = Some(saved);
                info!("X11 screensaver and power management have been disabled");
                true
            }
            Err(e) => {
                error!("Failed to disable the screensaver, {}", e);
                false
            }
        }
    }

    fn enable_screensaver(&self) -> bool {
        let Some(conn) = self.conn.as_ref() else {
            warn!("Unable to enable_screensaver, no X11 connection could be established");
            return false;
        };
        let Ok(mut guard) = self.saved.lock() else {
            error!("Failed to acquire the screensaver lock");
            return false;
        };
        let Some(saved) = *guard else {
            debug!("Screensaver not disabled, nothing to restore");
            return true;
        };

        match Self::apply_restore(conn, &saved) {
            Ok(_) => {
                *guard = None;
                info!("X11 screensaver and power management have been restored");
                true
            }
            Err(e) => {
                error!("Failed to restore the screensaver, {}", e);
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

        Self {
            conn,
            saved: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform_linux::PlatformLinux;
    use crate::SystemPlatform;

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
