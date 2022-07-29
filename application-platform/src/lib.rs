use crate::popcorn::fx::platform::platform_info::PlatformInfo;

pub mod popcorn;

/// Retrieve the platform information.
#[no_mangle]
pub extern "C" fn platform_info() -> PlatformInfo {
    PlatformInfo::new()
}