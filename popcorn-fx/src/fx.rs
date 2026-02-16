use clap::Parser;
use derive_more::Display;
use directories::{BaseDirs, UserDirs};
use log::{debug, error, info, trace, warn, LevelFilter};
use popcorn_fx_core::core::cache::CacheManager;
use popcorn_fx_core::core::config::{ApplicationConfig, PopcornProperties};
use popcorn_fx_core::core::event::EventPublisher;
use popcorn_fx_core::core::images::{DefaultImageLoader, ImageLoader};
use popcorn_fx_core::core::loader::{
    AutoResumeLoadingStrategy, DefaultMediaLoader, LoadingStrategy, MediaLoader,
    MediaTorrentUrlLoadingStrategy, PlayerLoadingStrategy, SubtitlesLoadingStrategy,
    TorrentDetailsLoadingStrategy, TorrentInfoLoadingStrategy, TorrentLoadingStrategy,
    TorrentStreamLoadingStrategy,
};
use popcorn_fx_core::core::media::favorites::{
    FXFavoriteService, FavoriteCacheUpdater, FavoriteService,
};
use popcorn_fx_core::core::media::providers::enhancers::ThumbEnhancer;
use popcorn_fx_core::core::media::providers::{
    FavoritesProvider, MovieProvider, ProviderManager, ShowProvider,
};
use popcorn_fx_core::core::media::resume::{AutoResumeService, DefaultAutoResumeService};
use popcorn_fx_core::core::media::tracking::{SyncMediaTracking, TrackingProvider};
use popcorn_fx_core::core::media::watched::{DefaultWatchedService, WatchedService};
use popcorn_fx_core::core::platform::PlatformData;
use popcorn_fx_core::core::playback::PlaybackControls;
use popcorn_fx_core::core::players::{DefaultPlayerManager, PlayerManager};
use popcorn_fx_core::core::playlist::PlaylistManager;
use popcorn_fx_core::core::subtitles::model::SubtitleType;
use popcorn_fx_core::core::subtitles::parsers::{SrtParser, VttParser};
use popcorn_fx_core::core::subtitles::{
    DefaultSubtitleManager, SubtitleManager, SubtitleProvider, SubtitleServer,
};
use popcorn_fx_core::core::torrents::collection::TorrentCollection;
use popcorn_fx_core::core::torrents::stream::FXTorrentStreamServer;
use popcorn_fx_core::core::torrents::{FxTorrentManager, TorrentManager, TorrentStreamServer};
use popcorn_fx_core::core::updater::Updater;
use popcorn_fx_logging::FxLogger;
use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
use popcorn_fx_platform::platform::DefaultPlatform;
use popcorn_fx_players::chromecast::ChromecastDiscovery;
use popcorn_fx_players::dlna::DlnaDiscovery;
use popcorn_fx_players::vlc::VlcDiscovery;
use popcorn_fx_players::Discovery;
use popcorn_fx_trakt::trakt::TraktProvider;
use std::env;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

const LOG_FILENAME: &str = "log4.yml";
const LOG_FILE_DIRECTORY: &str = "logs";
const LOG_FILE_NAME: &str = "popcorn-time.log";
const DEFAULT_APP_DIRECTORY: fn() -> String = || {
    UserDirs::new()
        .map(|e| PathBuf::from(e.home_dir()))
        .map(|e| e.join(".popcorn-time"))
        .map(|e| e.to_str().expect("expected a valid home path").to_string())
        .expect("expected a home directory to exist")
};
const DEFAULT_DATA_DIRECTORY: fn() -> String = || {
    BaseDirs::new()
        .map(|e| PathBuf::from(e.data_dir()))
        .map(|e| e.join("popcorn-fx"))
        .map(|e| e.to_str().expect("expected a valid data path").to_string())
        .expect("expected a data directory to exist")
};

/// The result type for popcorn fx main operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The options for the [PopcornFX] instance.
#[derive(Debug, Clone, Display, Parser)]
#[command(name = "popcorn-fx")]
#[display("app_directory: {:?}", app_directory)]
pub struct PopcornFxArgs {
    /// The directory containing the application files.
    /// This directory is also referred to as the `storage_directory` or `storage_path` within the application.
    #[arg(long, default_value_t = DEFAULT_APP_DIRECTORY())]
    pub app_directory: String,
    /// The directory containing the application data files.
    /// This directory is also referred to as the `runtime_directory` within the application.
    #[arg(long, default_value_t = DEFAULT_DATA_DIRECTORY())]
    pub data_directory: String,
    /// Disable the default `log4rs` logger for popcorn FX.
    /// This allows you to bring your own logger for the instance which should support [log].
    #[arg(long, global = true, default_value_t = false)]
    pub disable_logger: bool,
    /// Disable the mouse within the application.
    #[arg(long, default_value_t = false)]
    pub disable_mouse: bool,
    /// Enable the youtube video player.
    #[arg(long, default_value_t = true)]
    pub enable_youtube_video_player: bool,
    /// Enable the FX embedded video player.
    #[arg(long, default_value_t = true)]
    pub enable_fx_video_player: bool,
    /// Enable the VLC video player.
    #[arg(long, default_value_t = true)]
    pub enable_vlc_video_player: bool,
    /// Indicates if the TV mode is enabled of the application.
    #[arg(long, default_value_t = false)]
    pub tv: bool,
    /// Indicates if the application should be maximized on startup.
    #[arg(long, default_value_t = false)]
    pub maximized: bool,
    /// Indicates if the application should be started in kiosk mode.
    #[arg(long, default_value_t = false)]
    pub kiosk: bool,
    /// Indicates if insecure TLS connections are allowed
    #[arg(long, default_value_t = false)]
    pub insecure: bool,
    /// The properties of the application which are constant during the lifecycle of [PopcornFX]
    #[arg(skip = PopcornProperties::new_auto())]
    pub properties: PopcornProperties,
}

impl Default for PopcornFxArgs {
    fn default() -> Self {
        Self {
            app_directory: DEFAULT_APP_DIRECTORY(),
            data_directory: DEFAULT_DATA_DIRECTORY(),
            disable_logger: false,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            properties: PopcornProperties::new_auto(),
        }
    }
}

/// The Popcorn FX errors.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("failed to initialize a new instance, {0}")]
    Initialization(String),
}

/// The [PopcornFX] application instance.
/// This is the main entry into the FX application and manages all known data.
///
/// # Examples
///
/// Create a simple instance with default values.
/// This instance will have the [log4rs] loggers initialized.
/// ```no_run
/// use popcorn_fx::PopcornFX;
/// let instance = PopcornFX::default();
/// ```
#[repr(C)]
pub struct PopcornFX {
    auto_resume_service: Arc<dyn AutoResumeService>,
    cache_manager: CacheManager,
    event_publisher: EventPublisher,
    favorite_cache_updater: Arc<FavoriteCacheUpdater>,
    favorites_service: Arc<dyn FavoriteService>,
    image_loader: Arc<dyn ImageLoader>,
    media_loader: Arc<dyn MediaLoader>,
    platform: Arc<Box<dyn PlatformData>>,
    playback_controls: PlaybackControls,
    player_discovery_services: Vec<Arc<Box<dyn Discovery>>>,
    player_manager: Arc<Box<dyn PlayerManager>>,
    playlist_manager: PlaylistManager,
    providers: Arc<ProviderManager>,
    settings: ApplicationConfig,
    subtitle_manager: Arc<Box<dyn SubtitleManager>>,
    subtitle_provider: Arc<dyn SubtitleProvider>,
    subtitle_server: Arc<SubtitleServer>,
    torrent_collection: TorrentCollection,
    torrent_manager: Arc<dyn TorrentManager>,
    torrent_stream_server: Arc<dyn TorrentStreamServer>,
    tracking_provider: Arc<dyn TrackingProvider>,
    tracking_sync: Arc<SyncMediaTracking>,
    updater: Arc<Updater>,
    watched_service: Arc<dyn WatchedService>,
    /// The options that were used to create this instance
    opts: PopcornFxArgs,
}

impl PopcornFX {
    /// Create a new Popcorn FX instance with the given [PopcornFxArgs].
    pub async fn new(args: PopcornFxArgs) -> Result<Self> {
        Self::try_new(args).await
    }

    /// Try to create a new Popcorn FX instance within an async context.
    async fn try_new(args: PopcornFxArgs) -> Result<Self> {
        // check if we need to enable the logger
        if !args.disable_logger {
            Self::initialize_logger(&args)?;
        }
        if args.insecure {
            warn!("INSECURE CONNECTIONS ARE ENABLED");
        }

        trace!("Registering default crypto provider");
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| {
                Error::Initialization("failed to initialize crypto provider".to_string())
            })?;

        info!("Creating new popcorn fx instance with {:?}", args);
        let app_directory_path = args.app_directory.as_str();

        let event_publisher = EventPublisher::default();
        let settings = ApplicationConfig::builder()
            .storage(app_directory_path)
            .properties(args.properties.clone())
            .build();
        let cache_manager = CacheManager::new(app_directory_path);
        let subtitle_provider: Arc<dyn SubtitleProvider> = Arc::new(
            OpensubtitlesProvider::builder()
                .settings(settings.clone())
                .with_parser(SubtitleType::Srt, Box::new(SrtParser::default()))
                .with_parser(SubtitleType::Vtt, Box::new(VttParser::default()))
                .insecure(args.insecure)
                .build(),
        );
        let subtitle_server = Arc::new(
            SubtitleServer::new(subtitle_provider.clone())
                .await
                .map_err(|e| Error::Initialization(e.to_string()))?,
        );
        let subtitle_manager = Arc::new(Box::new(
            DefaultSubtitleManager::new(settings.clone()).await,
        ) as Box<dyn SubtitleManager>);
        let platform = Arc::new(Box::new(DefaultPlatform::default()) as Box<dyn PlatformData>);
        let favorites_service =
            Arc::new(FXFavoriteService::new(app_directory_path)) as Arc<dyn FavoriteService>;
        let watched_service = Arc::new(DefaultWatchedService::new(
            app_directory_path,
            event_publisher.clone(),
        )) as Arc<dyn WatchedService>;
        let providers = Arc::new(
            Self::default_providers(
                &settings,
                &args,
                &cache_manager,
                &favorites_service,
                &watched_service,
            )
            .await,
        );
        let torrent_manager = Arc::new(
            FxTorrentManager::new(
                Duration::from_hours(10 * 24),
                settings.clone(),
                event_publisher.clone(),
            )
            .await
            .map_err(|e| Error::Initialization(e.to_string()))?,
        ) as Arc<dyn TorrentManager>;
        let torrent_stream_server = Arc::new(
            FXTorrentStreamServer::new()
                .await
                .map_err(|e| Error::Initialization(e.to_string()))?,
        ) as Arc<dyn TorrentStreamServer>;
        let torrent_collection = TorrentCollection::new(app_directory_path);
        let auto_resume_service = Arc::new(
            DefaultAutoResumeService::builder()
                .storage_directory(app_directory_path)
                .event_publisher(event_publisher.clone())
                .build(),
        ) as Arc<dyn AutoResumeService>;
        let favorite_cache_updater = Arc::new(
            FavoriteCacheUpdater::builder()
                .favorite_service(favorites_service.clone())
                .provider_manager(providers.clone())
                .build(),
        );
        let app_updater = Arc::new(
            Updater::builder()
                .settings(settings.clone())
                .platform(platform.clone())
                .insecure(args.insecure)
                .data_path(args.data_directory.as_str())
                .build(),
        );
        let playback_controls = PlaybackControls::builder()
            .platform(platform.clone())
            .event_publisher(event_publisher.clone())
            .build();
        let image_loader =
            Arc::new(DefaultImageLoader::new(cache_manager.clone())) as Arc<dyn ImageLoader>;
        let player_manager = Arc::new(Box::new(DefaultPlayerManager::new(
            event_publisher.clone(),
            torrent_manager.clone(),
            torrent_stream_server.clone(),
        )) as Box<dyn PlayerManager>);
        let loading_chain: Vec<Box<dyn LoadingStrategy>> = vec![
            Box::new(MediaTorrentUrlLoadingStrategy::new()),
            Box::new(TorrentInfoLoadingStrategy::new(torrent_manager.clone())),
            Box::new(AutoResumeLoadingStrategy::new(auto_resume_service.clone())),
            Box::new(SubtitlesLoadingStrategy::new(
                subtitle_provider.clone(),
                subtitle_manager.clone(),
            )),
            Box::new(TorrentLoadingStrategy::new(
                torrent_manager.clone(),
                settings.clone(),
            )),
            Box::new(TorrentStreamLoadingStrategy::new(
                torrent_manager.clone(),
                torrent_stream_server.clone(),
            )),
            Box::new(TorrentDetailsLoadingStrategy::new(
                event_publisher.clone(),
                torrent_manager.clone(),
            )),
            Box::new(PlayerLoadingStrategy::new(player_manager.clone())),
        ];
        let media_loader = Arc::new(DefaultMediaLoader::new(loading_chain)) as Arc<dyn MediaLoader>;
        let playlist_manager = PlaylistManager::new(
            player_manager.clone(),
            event_publisher.clone(),
            media_loader.clone(),
        );
        let tracking_provider =
            Arc::new(TraktProvider::new(settings.clone()).unwrap()) as Arc<dyn TrackingProvider>;
        let tracking_sync = Arc::new(
            SyncMediaTracking::builder()
                .config(settings.clone())
                .tracking_provider(tracking_provider.clone())
                .watched_service(watched_service.clone())
                .build(),
        );
        let player_discovery_services: Vec<Arc<Box<dyn Discovery>>> = vec![
            Arc::new(Box::new(
                ChromecastDiscovery::builder()
                    .player_manager(player_manager.clone())
                    .subtitle_server(subtitle_server.clone())
                    .build(),
            ) as Box<dyn Discovery>),
            Arc::new(Box::new(
                DlnaDiscovery::builder()
                    .player_manager(player_manager.clone())
                    .subtitle_server(subtitle_server.clone())
                    .build(),
            )),
            Arc::new(Box::new(VlcDiscovery::new(
                subtitle_manager.clone(),
                subtitle_provider.clone(),
                player_manager.clone(),
            ))),
        ];

        // Try to disable the OS screensaver while the application is running without blocking
        // the application instance creation.
        // The screensaver will be automatically enabled when the platform instance is dropped
        let platform_async = platform.clone();
        tokio::spawn(async move {
            if platform_async.disable_screensaver() {
                info!("Operating System screensaver has been disabled");
            } else {
                error!("Failed to disable Operating System screensaver");
            }
        });

        Ok(Self {
            auto_resume_service,
            cache_manager,
            event_publisher,
            favorite_cache_updater,
            favorites_service,
            image_loader,
            media_loader,
            platform,
            playback_controls,
            player_manager,
            playlist_manager,
            providers,
            settings,
            subtitle_manager,
            subtitle_provider,
            subtitle_server,
            torrent_collection,
            torrent_manager,
            torrent_stream_server,
            tracking_provider,
            tracking_sync,
            updater: app_updater,
            watched_service,
            player_discovery_services,
            opts: args,
        })
    }

    /// Get the settings of the popcorn FX instance.
    pub fn settings(&self) -> &ApplicationConfig {
        &self.settings
    }

    /// The platform service of the popcorn FX instance.
    pub fn subtitle_provider(&self) -> &Arc<dyn SubtitleProvider> {
        &self.subtitle_provider
    }

    /// Retrieve the subtitle server instance.
    pub fn subtitle_server(&mut self) -> &mut Arc<SubtitleServer> {
        &mut self.subtitle_server
    }

    /// Retrieve the subtitle manager instance.
    pub fn subtitle_manager(&self) -> &Arc<Box<dyn SubtitleManager>> {
        &self.subtitle_manager
    }

    /// The system platform on which the Popcorn FX instance is running.
    pub fn platform(&mut self) -> &Arc<Box<dyn PlatformData>> {
        &self.platform
    }

    /// The available [popcorn_fx_core::core::media::Media] providers of the [PopcornFX].
    pub fn providers(&self) -> &Arc<ProviderManager> {
        &self.providers
    }

    /// The favorite service of [PopcornFX] which handles all liked items and actions.
    pub fn favorite_service(&self) -> &Arc<dyn FavoriteService> {
        &self.favorites_service
    }

    /// The watched service of [PopcornFX] which handles all watched items and actions.
    pub fn watched_service(&self) -> &Arc<dyn WatchedService> {
        &self.watched_service
    }

    /// The torrent manager to create, manage and delete torrents.
    pub fn torrent_manager(&self) -> &Arc<dyn TorrentManager> {
        &self.torrent_manager
    }

    /// The torrent stream server which handles the video streams.
    pub fn torrent_stream_server(&self) -> &Arc<dyn TorrentStreamServer> {
        &self.torrent_stream_server
    }

    /// The torrent collection that stores magnet uri info.
    pub fn torrent_collection(&self) -> &TorrentCollection {
        &self.torrent_collection
    }

    /// The auto-resume service which handles the resume timestamps of videos.
    pub fn auto_resume_service(&mut self) -> &Arc<dyn AutoResumeService> {
        &self.auto_resume_service
    }

    /// The application updater
    pub fn updater(&self) -> &Arc<Updater> {
        &self.updater
    }

    /// The playback controls handler of the system.
    pub fn playback_controls(&self) -> &PlaybackControls {
        &self.playback_controls
    }

    /// The image loader of the Popcorn FX application.
    pub fn image_loader(&self) -> &Arc<dyn ImageLoader> {
        &self.image_loader
    }

    /// Reload the settings of this instance.
    /// This will read the settings from the storage and notify all subscribers of new changes.
    pub fn reload_settings(&mut self) {
        self.settings.reload()
    }

    /// Retrieve the event publisher of the FX instance.
    pub fn event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    /// Retrieve the player manager of the FX instance.
    pub fn player_manager(&self) -> &Arc<Box<dyn PlayerManager>> {
        &self.player_manager
    }

    /// Retrieve the playlist manager of the FX instance.
    pub fn playlist_manager(&self) -> &PlaylistManager {
        &self.playlist_manager
    }

    /// Retrieve the media loader of the FX instance.
    pub fn media_loader(&self) -> &Arc<dyn MediaLoader> {
        &self.media_loader
    }

    /// Retrieve the tracking provider of the FX instance.
    pub fn tracking_provider(&self) -> &Arc<dyn TrackingProvider> {
        &self.tracking_provider
    }

    /// Retrieve the tracking synchronizer of the FX instance.
    pub fn tracking_sync(&self) -> &Arc<SyncMediaTracking> {
        &self.tracking_sync
    }

    /// Retrieve the option that were used to create this instance.
    /// It returns a read-only reference to the options as they can't be changed anymore during the runtime.
    pub fn opts(&self) -> &PopcornFxArgs {
        &self.opts
    }

    /// Start the discovery of external players such as VLC and DLNA servers.
    /// This will start new threads in the background for handling the discovery processes.
    pub fn start_discovery_external_players(&self, _interval_in_seconds: u32) {
        let player_discovery_services = self.player_discovery_services.clone();
        tokio::spawn(async move {
            debug!(
                "Discovering new player from {} discovery services",
                player_discovery_services.len()
            );
            for service in player_discovery_services {
                if let Err(e) = service.start_discovery().await {
                    error!("Failed to start {}, {}", service, e);
                }
            }
        });
    }

    fn initialize_logger(args: &PopcornFxArgs) -> Result<()> {
        let root_level = env::var("LOG_LEVEL")
            .ok()
            .and_then(|e| LevelFilter::from_str(e.as_str()).ok())
            .unwrap_or(LevelFilter::Info);
        let config_path = env::current_dir()
            .ok()
            .map(|e| e.join(LOG_FILENAME))
            .filter(|e| e.exists());
        let log_path = PathBuf::from(args.app_directory.clone())
            .join(LOG_FILE_DIRECTORY)
            .join(LOG_FILE_NAME);
        let mut builder = FxLogger::builder();

        if let Some(path) = config_path {
            builder.config_path(path);
        }

        for (logger, logging) in args.properties.loggers.iter() {
            builder.logger(
                logger,
                LevelFilter::from_str(logging.level.as_str()).unwrap_or(LevelFilter::Info),
            );
        }

        builder
            .root_level(root_level)
            .log_path(log_path)
            .build()
            .map_err(|e| Error::Initialization(e.to_string()))?;
        Ok(())
    }

    async fn default_providers(
        settings: &ApplicationConfig,
        args: &PopcornFxArgs,
        cache_manager: &CacheManager,
        favorites: &Arc<dyn FavoriteService>,
        watched: &Arc<dyn WatchedService>,
    ) -> ProviderManager {
        let movie_provider =
            Box::new(MovieProvider::new(settings, cache_manager.clone(), args.insecure).await);
        let show_provider =
            Box::new(ShowProvider::new(settings, cache_manager.clone(), args.insecure).await);
        let favorites_provider =
            Box::new(FavoritesProvider::new(favorites.clone(), watched.clone()));
        let thumb_enhancer = Box::new(ThumbEnhancer::new(
            settings
                .properties()
                .enhancers
                .get("tvdb")
                .expect("expected the tvdb properties to be present")
                .clone(),
            cache_manager.clone(),
        ));

        ProviderManager::builder()
            .with_provider(movie_provider.clone())
            .with_provider(show_provider.clone())
            .with_provider(favorites_provider)
            .with_details_provider(movie_provider)
            .with_details_provider(show_provider)
            .with_enhancer(thumb_enhancer)
            .build()
    }
}

impl Debug for PopcornFX {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PopcornFX")
            .field("auto_resume_service", &self.auto_resume_service)
            .field("cache_manager", &self.cache_manager)
            .field("event_publisher", &self.event_publisher)
            .finish()
    }
}

unsafe impl Send for PopcornFX {}

unsafe impl Sync for PopcornFX {}

impl Drop for PopcornFX {
    fn drop(&mut self) {
        self.event_publisher.close();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::tests::default_args;

    use fx_callback::Callback;
    use popcorn_fx_core::core::config::{ApplicationConfigEvent, LoggingProperties};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::SubtitlePreference;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::copy_test_file;
    use std::collections::HashMap;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_popcorn_fx_new() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut popcorn_fx = PopcornFX::new(default_args(temp_path)).await.unwrap();

        let _ = popcorn_fx.platform().info();
        let _ = popcorn_fx.subtitle_server();

        let subtitle_manager = popcorn_fx.subtitle_manager().clone();
        let preference = subtitle_manager.preference().await;

        assert_eq!(
            SubtitlePreference::Language(SubtitleLanguage::None),
            preference
        );
    }

    #[tokio::test]
    async fn test_popcorn_fx_favorite() {
        init_logger!();
        let id = "tt00000021544";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let popcorn_fx = PopcornFX::new(default_args(temp_path)).await.unwrap();

        let service = popcorn_fx.favorite_service().clone();
        let result = service.is_liked(id).await;

        assert_eq!(false, result)
    }

    #[tokio::test]
    async fn test_popcorn_fx_auto_resume() {
        init_logger!();
        let filename = "something-totally_random123qwe.mp4";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut popcorn_fx = PopcornFX::new(default_args(temp_path)).await.unwrap();

        let service = popcorn_fx.auto_resume_service().clone();
        let result = service
            .resume_timestamp(None, Some(filename.to_string()))
            .await;

        assert_eq!(None, result)
    }

    #[tokio::test]
    async fn test_fx_torrent_collection() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let popcorn_fx = PopcornFX::new(default_args(temp_path)).await.unwrap();

        let torrent_collection = popcorn_fx.torrent_collection().clone();
        let result = torrent_collection
            .is_stored("magnet:?myMostRandomAvailableAndEvenInvalidMagnet")
            .await;

        assert_eq!(false, result)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_popcorn_fx_reload_settings() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut popcorn_fx = PopcornFX::new(default_args(temp_path)).await.unwrap();
        copy_test_file(temp_path, "settings.json", None);

        let mut receiver = popcorn_fx.settings().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let ApplicationConfigEvent::Loaded = &*event {
                    tx.send((*event).clone()).unwrap()
                }
            }
        });

        popcorn_fx.reload_settings();
        let result = rx.recv_timeout(Duration::from_millis(500)).unwrap();

        match result {
            ApplicationConfigEvent::Loaded => {}
            _ => assert!(
                false,
                "expected ApplicationConfigEvent::SettingsLoaded, but got {:?} instead",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_initialize_logger() {
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let args = PopcornFxArgs {
            app_directory: temp_path.to_string(),
            data_directory: temp_path.to_string(),
            disable_logger: false,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            properties: PopcornProperties {
                loggers: HashMap::from([
                    (
                        "popcorn_fx".to_string(),
                        LoggingProperties {
                            level: "trace".to_string(),
                        },
                    ),
                    (
                        "popcorn_fx::ffi".to_string(),
                        LoggingProperties {
                            level: "invalid".to_string(),
                        },
                    ),
                ]),
                update_channel: String::new(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: Default::default(),
            },
        };

        // should not panic on the invalid level
        let _ = PopcornFX::initialize_logger(&args).expect("failed to initialize logger");
    }
}
