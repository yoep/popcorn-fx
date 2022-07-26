use crate::popcorn::fx::platform::platform_info::PlatformInfo;

pub mod popcorn;

#[no_mangle]
pub extern "C" fn platform_info() -> PlatformInfo {
    PlatformInfo::new()
}