use std::fmt::{Debug, Formatter};

use downcast_rs::{impl_downcast, DowncastSync};
use log::{debug, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::block_in_place;

/// A trait for managing the application screen.
///
/// The `ScreenService` trait defines methods for handling information and actions related to the application screen,
/// such as toggling fullscreen mode and checking if the screen is currently in fullscreen.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait ScreenService: Debug + DowncastSync {
    /// Check if the application is in fullscreen mode.
    ///
    /// # Returns
    ///
    /// `true` if the application is in fullscreen mode, `false` otherwise.
    fn is_fullscreen(&self) -> bool;

    /// Toggle fullscreen mode.
    fn toggle_fullscreen(&self);

    /// Set the fullscreen state of the application.
    ///
    /// # Arguments
    ///
    /// * `active_fullscreen` - `true` to activate fullscreen mode, `false` to deactivate it.
    fn fullscreen(&self, active_fullscreen: bool);
}
impl_downcast!(sync ScreenService);

/// A type representing a callback function to check if the application is in fullscreen mode.
pub type IsFullScreenCallback = Box<dyn Fn() -> bool + Send + Sync>;

/// A type representing a callback function to toggle fullscreen mode.
pub type ToggleFullScreenCallback = Box<dyn Fn() + Send + Sync>;

/// A type representing a callback function to set the fullscreen state of the application.
pub type FullscreenCallback = Box<dyn Fn(bool) + Send + Sync>;

/// A struct implementing ScreenService for managing screen-related actions and information.
pub struct DefaultScreenService {
    pub is_fullscreen_callback: Mutex<IsFullScreenCallback>,
    pub toggle_fullscreen_callback: Mutex<ToggleFullScreenCallback>,
    pub fullscreen_callback: Mutex<FullscreenCallback>,
}

impl DefaultScreenService {
    pub fn new() -> Self {
        Self {
            is_fullscreen_callback: Mutex::new(Box::new(|| {
                warn!("full_screen_callback has not been configured");
                false
            })),
            toggle_fullscreen_callback: Mutex::new(Box::new(|| {
                warn!("toggle_fullscreen_callback has not been configured");
            })),
            fullscreen_callback: Mutex::new(Box::new(|_| {
                warn!("fullscreen_callback has not been configured");
            })),
        }
    }

    pub fn register_is_fullscreen_callback(&self, callback: IsFullScreenCallback) {
        debug!("Registering new IsFullScreenCallback");
        let mut mutex = block_in_place(self.is_fullscreen_callback.lock());
        *mutex = callback;
    }

    pub fn register_toggle_fullscreen_callback(&self, callback: ToggleFullScreenCallback) {
        debug!("Registering new ToggleFullScreenCallback");
        let mut mutex = block_in_place(self.toggle_fullscreen_callback.lock());
        *mutex = callback;
    }

    pub fn register_fullscreen_callback(&self, callback: FullscreenCallback) {
        debug!("Registering new FullscreenCallback");
        let mut mutex = block_in_place(self.fullscreen_callback.lock());
        *mutex = callback;
    }
}

impl ScreenService for DefaultScreenService {
    fn is_fullscreen(&self) -> bool {
        let callback = block_in_place(self.is_fullscreen_callback.lock());
        callback()
    }

    fn toggle_fullscreen(&self) {
        let callback = block_in_place(self.toggle_fullscreen_callback.lock());
        callback()
    }

    fn fullscreen(&self, active_fullscreen: bool) {
        trace!(
            "Updating screen service fullscreen to {}",
            active_fullscreen
        );
        let callback = block_in_place(self.fullscreen_callback.lock());
        callback(active_fullscreen)
    }
}

impl Debug for DefaultScreenService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultScreenService").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::init_logger;

    use super::*;

    #[test]
    fn test_is_fullscreen() {
        init_logger!();
        let (tx, rx) = channel();
        let service = DefaultScreenService::new();

        service.register_is_fullscreen_callback(Box::new(move || {
            tx.send(()).unwrap();
            true
        }));

        let result = service.is_fullscreen();
        assert_eq!(true, result, "expected the fullscreen mode to be active");

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the callback to have been invoked");
    }

    #[test]
    fn test_toggle_fullscreen() {
        init_logger!();
        let (tx, rx) = channel();
        let service = DefaultScreenService::new();

        service.register_toggle_fullscreen_callback(Box::new(move || {
            tx.send(()).unwrap();
        }));

        service.toggle_fullscreen();

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the callback to have been invoked");
    }

    #[test]
    fn test_fullscreen() {
        init_logger!();
        let (tx, rx) = channel();
        let service = DefaultScreenService::new();

        service.register_fullscreen_callback(Box::new(move |value| {
            tx.send(value).unwrap();
        }));

        service.fullscreen(true);

        let result = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected the callback to have been invoked");
        assert_eq!(true, result);
    }
}
