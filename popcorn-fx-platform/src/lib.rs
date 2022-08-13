use crate::platform_info_c::PlatformInfoC;
use crate::popcorn::fx::platform::platform::PlatformService;

pub mod popcorn;
pub mod platform_info_c;

#[no_mangle]
pub extern "C" fn enable_screensaver(platform: &mut Box<dyn PlatformService>) {
    platform.enable_screensaver();
}

#[no_mangle]
pub extern "C" fn disable_screensaver(platform: &mut Box<dyn PlatformService>) {
    platform.disable_screensaver();
}

#[no_mangle]
pub extern "C" fn platform_info(platform: &mut Box<dyn PlatformService>) -> PlatformInfoC {
    PlatformInfoC::from(platform.platform_info())
}