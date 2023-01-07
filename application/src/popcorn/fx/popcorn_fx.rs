use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Once};

use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

use popcorn_fx_core::core::config::Application;
use popcorn_fx_core::core::media::{Category, Movie};
use popcorn_fx_core::core::media::providers::{MovieProvider, Provider, ProviderManager};
use popcorn_fx_core::core::subtitles::service::SubtitleService;
use popcorn_fx_opensubtitles::opensubtitles::service::OpensubtitlesService;
use popcorn_fx_platform::popcorn::fx::platform::platform::{PlatformService, PlatformServiceImpl};

static INIT: Once = Once::new();

const LOG_FILENAME: &str = "log4.yml";
const LOG_FORMAT: &str = "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l}):>5.5} {I} --- [{T:>15.15}] {M} : {m}{n}";
const CONSOLE_APPENDER: &str = "stdout";

/// The [PopcornFX] application instance.
#[repr(C)]
pub struct PopcornFX {
    settings: Arc<Application>,
    subtitle_service: Box<dyn SubtitleService>,
    platform_service: Box<dyn PlatformService>,
    providers: ProviderManager,
}

impl PopcornFX {
    /// Initialize a new popcorn FX instance.
    pub fn new() -> Self {
        Self::initialize_logger();
        let settings = Arc::new(Application::new_auto());
        let subtitle_service = Box::new(OpensubtitlesService::new(&settings));
        let platform_service = Box::new(PlatformServiceImpl::new());
        let movie_provider: Box<dyn Provider<Movie>> = Box::new(MovieProvider::new(&settings));
        let providers = ProviderManager::with_providers(HashMap::from([
            (Category::MOVIES, movie_provider)
        ]));

        Self {
            settings,
            subtitle_service,
            platform_service,
            providers,
        }
    }

    /// The platform service of the popcorn FX instance.
    pub fn subtitle_service(&mut self) -> &mut Box<dyn SubtitleService> {
        &mut self.subtitle_service
    }

    /// The platform service of the popcorn FX instance.
    pub fn platform_service(&mut self) -> &mut Box<dyn PlatformService> {
        &mut self.platform_service
    }

    /// The available [popcorn_fx_core::core::media::Media] providers of the [PopcornFX].
    pub fn providers(&mut self) -> &mut ProviderManager {
        &mut self.providers
    }

    /// Dispose the FX instance.
    pub fn dispose(&self) {
        self.settings.save();
    }

    fn initialize_logger() {
        INIT.call_once(|| {
            let config: Config;
            let root_level = env::var("LOG_LEVEL").unwrap_or("Info".to_string());
            let log_path = env::current_dir().expect("Home directory should exist")
                .join(LOG_FILENAME);

            if log_path.exists() {
                match log4rs::config::load_config_file(log_path, Default::default()) {
                    Err(ex) => panic!("failed to initialize logger through file, {}", ex),
                    Ok(e) => config = e,
                };
            } else {
                config = Config::builder()
                    .appender(Appender::builder().build(CONSOLE_APPENDER, Box::new(ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(LOG_FORMAT)))
                        .build())))
                    .build(Root::builder()
                        .appender(CONSOLE_APPENDER)
                        .build(LevelFilter::from_str(root_level.as_str()).unwrap()))
                    .unwrap()
            }

            log4rs::init_config(config).unwrap();
            info!("Logger has been initialized");
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subtitle_service_should_return_the_subtitle_service() {
        let mut popcorn_fx = PopcornFX::new();

        let subtitle_service = popcorn_fx.subtitle_service();
        let result = subtitle_service.active_subtitle();

        assert!(result.is_none(), "Expected the subtitle service to return none")
    }
}