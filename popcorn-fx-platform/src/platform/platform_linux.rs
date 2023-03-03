use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use log::warn;
use tokio::sync::Mutex;
use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::rust_connection::RustConnection;

use popcorn_fx_core::core::platform::Platform;

/// The linux platform specific implementation
#[derive(Debug)]
pub struct PlatformLinux {
    /// The connection to the X11 server
    connection: RustConnection,
    /// Indicates if the screen is being kept alive or not
    keep_alive: Arc<Mutex<bool>>,
    runtime: tokio::runtime::Runtime,
}

impl Platform for PlatformLinux {
    fn disable_screensaver(&self) -> bool {
        let mut keep_alive = self.keep_alive.blocking_lock();
        *keep_alive = true;
        drop(keep_alive);

        warn!("disable_screensaver has not been implemented for Linux");
        true
    }

    fn enable_screensaver(&self) -> bool {
        let mut keep_alive = self.keep_alive.blocking_lock();
        *keep_alive = false;
        true
    }
}

impl Default for PlatformLinux {
    fn default() -> Self {
        let (conn, _screen_num) = x11rb::connect(None).unwrap();

        Self {
            connection: conn,
            keep_alive: Default::default(),
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_disable_screensaver() {
        let platform = PlatformLinux::default();

        let result = platform.disable_screensaver();

        assert_eq!(true, result)
    }

    #[test]
    fn test_enable_screensaver() {
        let platform = PlatformLinux::default();

        let result = platform.enable_screensaver();

        assert_eq!(true, result)
    }
}