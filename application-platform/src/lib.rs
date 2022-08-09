use crate::platform_c::PlatformC;
use crate::platform_info_c::PlatformInfoC;
use crate::popcorn::fx::platform::platform::{new_platform, Platform};
use crate::popcorn::fx::platform::platform_info::{PlatformInfo, PlatformType};

pub mod popcorn;
mod platform_info_c;
mod platform_c;

/// Retrieve the platform information.
#[no_mangle]
pub extern "C" fn platform_info() -> PlatformInfoC {
    PlatformInfoC::from(PlatformInfo::new())
}

/// Retrieve the platform instance.
/// The caller will become the owner of the instance and is responsible for the memory management of it.
/// The instance can be safely deleted by using [delete_platform_c()]
#[no_mangle]
pub extern "C" fn new_platform_c() -> Box<PlatformC> {
    Box::new(PlatformC::new())
}

/// Drop the platform instance.
#[no_mangle]
pub extern "C" fn delete_platform_c(platform: Box<PlatformC>) {
    drop(platform);
}

/// Disable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn disable_screensaver(platform: &mut PlatformC) {
    platform.disable_screensaver();
}

/// Enable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn enable_screensaver(platform: &mut PlatformC) {
    platform.enable_screensaver();
}