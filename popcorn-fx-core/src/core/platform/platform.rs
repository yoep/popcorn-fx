use std::fmt::Debug;

use derive_more::Display;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::CoreCallback;
use crate::core::playback::MediaNotificationEvent;

/// The platform event specific callback type.
pub type PlatformCallback = CoreCallback<PlatformEvent>;

/// The platform system specific functions trait.
/// This trait defines actions which should be performed on the current platform.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait Platform: Debug + Send + Sync {
    /// Disable the screensaver on the current platform
    /// It returns `true` if the screensaver was disabled with success, else `false`.
    fn disable_screensaver(&self) -> bool;

    /// Enable the screensaver on the current platform
    /// It returns `true` if the screensaver was enabled with success, else `false`.
    fn enable_screensaver(&self) -> bool;

    /// Notify the system that a new media playback has been started.
    fn notify_media_event(&self, notification: MediaNotificationEvent);

    /// Register a new callback listener for the [PlatformEvent]'s.
    fn register(&self, callback: PlatformCallback);
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
    #[display(fmt = "Toggle the media playback state")]
    TogglePlaybackState,
    #[display(fmt = "Forward the current media playback time")]
    ForwardMedia,
    #[display(fmt = "Rewind the current media playback time")]
    RewindMedia,
}

/// PlatformInfo defines the info of the current platform
#[derive(Debug, Clone, Display, PartialEq)]
#[display(fmt = "platform_type: {}, arch: {}", platform_type, arch)]
pub struct PlatformInfo {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: String,
}

/// The platform type
#[repr(i32)]
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
