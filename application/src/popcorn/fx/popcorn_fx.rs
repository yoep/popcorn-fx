use std::path::Path;
use std::sync::Once;

use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

static INIT: Once = Once::new();

const LOG_FILENAME: &str = "log4.yml";
const LOG_FORMAT: &str = "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l}):>5.5} {I} --- [{T:>15.15}] {M} : {m}{n}";
const CONSOLE_APPENDER: &str = "stdout";

/// The Popcorn FX struct contains the main controller logic of popcorn.
pub struct PopcornFX {}

impl PopcornFX {
    /// Initialize a new popcorn FX instance.
    pub fn new() -> PopcornFX {
        let instance = PopcornFX {};
        instance.initialize_logger();
        return instance;
    }

    /// Initialize the logger
    fn initialize_logger(&self) {
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
            info!("Logger has been initialized")
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_initialize_logger_only_once() {
        let popcorn_fx = PopcornFX::new();

        popcorn_fx.initialize_logger();
        // the second call should not crash the application
        popcorn_fx.initialize_logger();
    }
}