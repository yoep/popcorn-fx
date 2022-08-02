use std::path::Path;
use std::sync::Once;

use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;

use crate::platform_c::PlatformC;
use crate::platform_info_c::PlatformInfoC;
use crate::popcorn::fx::platform::platform::{new_platform, Platform};
use crate::popcorn::fx::platform::platform_info::{PlatformInfo, PlatformType};

pub mod popcorn;
mod platform_info_c;
mod platform_c;

static INIT: Once = Once::new();

const LOG_FILENAME: &str = "log4.yml";
const LOG_FORMAT: &str = "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l}):>5.5} {I} --- [{T:>15.15}] {M} : {m}{n}";
const CONSOLE_APPENDER: &str = "stdout";

#[no_mangle]
pub extern "C" fn init() {
    INIT.call_once(|| {
        if Path::new(LOG_FILENAME).exists() {
           log4rs::init_file(LOG_FILENAME, Default::default()).unwrap();
        } else {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build(CONSOLE_APPENDER, Box::new(ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new(LOG_FORMAT)))
                    .build())))
                .build(Root::builder().appender(CONSOLE_APPENDER).build(LevelFilter::Info))
                .unwrap())
                .unwrap();
        }
    })
}

/// Retrieve the platform information.
#[no_mangle]
pub extern "C" fn platform_info() -> PlatformInfoC {
    PlatformInfoC::from(PlatformInfo::new())
}

/// Retrieve the platform instance.
#[no_mangle]
pub extern "C" fn new_platform_c() -> Box<PlatformC> {
    Box::new(PlatformC::new())
}

/// Disable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn disable_screensaver(mut platform: Box<PlatformC>) {
    platform.disable_screensaver();
}

/// Enable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn enable_screensaver(mut platform: Box<PlatformC>) {
    platform.enable_screensaver();
}