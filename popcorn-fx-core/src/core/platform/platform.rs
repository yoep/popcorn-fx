use crate::core::playback::MediaNotificationEvent;
use derive_more::Display;
use fx_callback::{Callback, Subscription};
#[cfg(any(test, feature = "testing"))]
pub use mock::*;
use std::fmt::Debug;

/// The platform event specific callback type.
pub type PlatformCallback = Subscription<PlatformEvent>;

/// The platform system specific functions trait.
/// This trait defines actions which should be performed on the current platform.
pub trait Platform: Debug + Callback<PlatformEvent> + Send + Sync {
    /// Disable the screensaver on the current platform
    /// It returns `true` if the screensaver was disabled with success, else `false`.
    fn disable_screensaver(&self) -> bool;

    /// Enable the screensaver on the current platform
    /// It returns `true` if the screensaver was enabled with success, else `false`.
    fn enable_screensaver(&self) -> bool;

    /// Notify the system that a new media playback has been started.
    fn notify_media_event(&self, notification: MediaNotificationEvent);
}

/// The information data of the current system platform.
pub trait PlatformData: Platform {
    /// Retrieve the platform info of the current system.
    fn info(&self) -> PlatformInfo;
}

/// The events of the system platform.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum PlatformEvent {
    /// Invoked when the play/pause state of the application needs to be toggled
    #[display("Toggle the media playback state")]
    TogglePlaybackState,
    #[display("Forward the current media playback time")]
    ForwardMedia,
    #[display("Rewind the current media playback time")]
    RewindMedia,
}

/// PlatformInfo defines the info of the current platform
#[derive(Debug, Clone, Display, PartialEq)]
#[display("platform_type: {}, arch: {}", platform_type, arch)]
pub struct PlatformInfo {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: String,
}

/// The platform type
#[derive(Debug, Clone, Display, PartialEq)]
pub enum PlatformType {
    /// The windows platform
    Windows = 0,
    /// The macos platform
    MacOs = 1,
    /// The linux platform
    Linux = 2,
}

impl PlatformType {
    /// The name of the platform type.
    pub fn name(&self) -> &str {
        match self {
            PlatformType::Windows => "windows",
            PlatformType::MacOs => "macos",
            PlatformType::Linux => "debian",
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;

    use fx_callback::Subscriber;
    use mockall::mock;

    mock! {
        #[derive(Debug, Clone)]
        pub Platform {}

        impl Platform for Platform {
            fn disable_screensaver(&self) -> bool;
            fn enable_screensaver(&self) -> bool;
            fn notify_media_event(&self, notification: MediaNotificationEvent);
        }

        impl Callback<PlatformEvent> for Platform {
            fn subscribe(&self) -> Subscription<PlatformEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<PlatformEvent>);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_platform_type_name() {
        assert_eq!("windows", PlatformType::Windows.name());
        assert_eq!("debian", PlatformType::Linux.name());
        assert_eq!("macos", PlatformType::MacOs.name());
    }
}
