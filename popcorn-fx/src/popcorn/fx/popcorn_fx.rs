use std::env;
use std::str::FromStr;
use std::sync::{Arc, Once};

use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

use popcorn_fx_core::core::config::Application;
use popcorn_fx_core::core::media::favorites::{DefaultFavoriteService, FavoriteService};
use popcorn_fx_core::core::media::providers::{FavoritesProvider, MediaProvider, MovieProvider, ProviderManager, ShowProvider};
use popcorn_fx_core::core::media::resume::{AutoResumeService, DefaultAutoResumeService};
use popcorn_fx_core::core::media::watched::{DefaultWatchedService, WatchedService};
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::subtitles::{SubtitleManager, SubtitleProvider, SubtitleServer};
use popcorn_fx_core::core::torrent::TorrentStreamServer;
use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
use popcorn_fx_platform::popcorn::fx::platform::platform::{PlatformService, PlatformServiceImpl};
use popcorn_fx_torrent_stream::torrent::stream::DefaultTorrentStreamServer;

static INIT: Once = Once::new();

const LOG_FILENAME: &str = "log4.yml";
const LOG_FORMAT: &str = "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l:>5.5})} {I} --- [{T:>15.15}] {M} : {m}{n}";
const CONSOLE_APPENDER: &str = "stdout";

/// The [PopcornFX] application instance.
/// This is the main entry into the FX application and manages all known data.
///
/// A simple instance with default values can be retrieved by as follows.
/// ```no_run
/// let instance = PopcornFX::default();
/// ```
/// This instance will have initialize the log4rs logger.
///
#[repr(C)]
pub struct PopcornFX {
    settings: Arc<Application>,
    subtitle_service: Arc<Box<dyn SubtitleProvider>>,
    subtitle_server: Arc<SubtitleServer>,
    subtitle_manager: Arc<SubtitleManager>,
    platform_service: Box<dyn PlatformService>,
    favorites_service: Arc<Box<dyn FavoriteService>>,
    watched_service: Arc<Box<dyn WatchedService>>,
    torrent_stream_server: Arc<Box<dyn TorrentStreamServer>>,
    auto_resume_service: Arc<Box<dyn AutoResumeService>>,
    providers: ProviderManager,
    storage: Arc<Storage>,
}

impl PopcornFX {
    /// The platform service of the popcorn FX instance.
    pub fn subtitle_provider(&mut self) -> &mut Arc<Box<dyn SubtitleProvider>> {
        &mut self.subtitle_service
    }

    /// Retrieve the subtitle server instance.
    pub fn subtitle_server(&mut self) -> &mut Arc<SubtitleServer> {
        &mut self.subtitle_server
    }

    /// Retrieve the subtitle manager instance.
    pub fn subtitle_manager(&mut self) -> &mut Arc<SubtitleManager> {
        &mut self.subtitle_manager
    }

    /// The platform service of the popcorn FX instance.
    pub fn platform_service(&mut self) -> &mut Box<dyn PlatformService> {
        &mut self.platform_service
    }

    /// The available [popcorn_fx_core::core::media::Media] providers of the [PopcornFX].
    pub fn providers(&mut self) -> &mut ProviderManager {
        &mut self.providers
    }

    /// The favorite service of [PopcornFX] which handles all liked items and actions.
    pub fn favorite_service(&mut self) -> &Arc<Box<dyn FavoriteService>> {
        &self.favorites_service
    }

    /// The watched service of [PopcornFX] which handles all watched items and actions.
    pub fn watched_service(&mut self) -> &Arc<Box<dyn WatchedService>> {
        &self.watched_service
    }

    /// The torrent stream server which handles the video streams.
    pub fn torrent_stream_server(&mut self) -> &Arc<Box<dyn TorrentStreamServer>> {
        &self.torrent_stream_server
    }

    /// The auto-resume service which handles the resume timestamps of videos.
    pub fn auto_resume_service(&mut self) -> &Arc<Box<dyn AutoResumeService>> {
        &self.auto_resume_service
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

    fn default_providers(settings: &Arc<Application>, favorites: &Arc<Box<dyn FavoriteService>>, watched: &Arc<Box<dyn WatchedService>>) -> ProviderManager {
        let movie_provider: Arc<Box<dyn MediaProvider>> = Arc::new(Box::new(MovieProvider::new(&settings)));
        let show_provider: Arc<Box<dyn MediaProvider>> = Arc::new(Box::new(ShowProvider::new(&settings)));
        let favorites: Arc<Box<dyn MediaProvider>> = Arc::new(Box::new(FavoritesProvider::new(favorites.clone(), watched.clone(), vec![
            &movie_provider,
            &show_provider,
        ])));

        ProviderManager::with_providers(vec![
            movie_provider,
            show_provider,
            favorites,
        ])
    }
}

impl Default for PopcornFX {
    fn default() -> Self {
        Self::initialize_logger();
        let storage = Arc::new(Storage::new());
        let settings = Arc::new(Application::new_auto(&storage));
        let subtitle_service: Arc<Box<dyn SubtitleProvider>> = Arc::new(Box::new(OpensubtitlesProvider::new(&settings)));
        let subtitle_server = Arc::new(SubtitleServer::new(&subtitle_service));
        let subtitle_manager = Arc::new(SubtitleManager::default());
        let platform_service = Box::new(PlatformServiceImpl::new());
        let favorites_service = Arc::new(Box::new(DefaultFavoriteService::new(&storage)) as Box<dyn FavoriteService>);
        let watched_service = Arc::new(Box::new(DefaultWatchedService::new(&storage)) as Box<dyn WatchedService>);
        let providers = Self::default_providers(&settings, &favorites_service, &watched_service);
        let torrent_stream_server = Arc::new(Box::new(DefaultTorrentStreamServer::default()) as Box<dyn TorrentStreamServer>);
        let auto_resume_service = Arc::new(Box::new(DefaultAutoResumeService::new(&storage)) as Box<dyn AutoResumeService>);

        Self {
            settings,
            subtitle_service,
            subtitle_server,
            subtitle_manager,
            platform_service,
            favorites_service,
            watched_service,
            torrent_stream_server,
            auto_resume_service,
            providers,
            storage,
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;

    use super::*;

    #[test]
    fn test_popcorn_fx_new() {
        let mut popcorn_fx = PopcornFX::default();

        let _ = popcorn_fx.platform_service().platform_info();
        let _ = popcorn_fx.subtitle_server();

        let preferred_language = popcorn_fx.subtitle_manager().preferred_language();
        let preferred_subtitle = popcorn_fx.subtitle_manager().preferred_subtitle();

        assert_eq!(SubtitleLanguage::None, preferred_language);
        assert_eq!(None, preferred_subtitle);
    }

    #[test]
    fn test_popcorn_fx_favorite() {
        let id = "tt00000021544";
        let mut popcorn_fx = PopcornFX::default();

        let result = popcorn_fx.favorite_service().is_liked(id);

        assert_eq!(false, result)
    }

    #[test]
    fn test_popcorn_fx_auto_resume() {
        let filename = "something-totally_random123qwe.mp4";
        let mut popcorn_fx = PopcornFX::default();

        let result = popcorn_fx.auto_resume_service().resume_timestamp(None, Some(filename));

        assert_eq!(None, result)
    }
}