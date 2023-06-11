use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, trace};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use popcorn_fx_core::core::{block_in_place, events, torrent};
use popcorn_fx_core::core::config::{ApplicationConfig, CleaningMode};
use popcorn_fx_core::core::events::{Event, EventPublisher, Order, PlayerStoppedEvent};
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::torrent::{TorrentInfo, TorrentManager, TorrentManagerCallback, TorrentManagerState};

/// The default torrent manager of the application.
/// It currently only cleans the torrent directory if needed.
/// No actual torrent implementation is available.
#[derive(Debug)]
pub struct DefaultTorrentManager {
    inner: Arc<InnerTorrentManager>,
}

impl DefaultTorrentManager {
    pub fn new(settings: Arc<Mutex<ApplicationConfig>>, event_publisher: Arc<EventPublisher>, runtime: Arc<Runtime>) -> Self {
        let instance = Self {
            inner: Arc::new(InnerTorrentManager {
                settings,
                runtime,
            }),
        };

        let cloned_instance = instance.inner.clone();
        event_publisher.register(Box::new(move |event| {
            if let Event::PlayerStopped(e) = &event {
                cloned_instance.on_player_stopped(e.clone());
            }

            Some(event)
        }), events::LOWEST_ORDER);

        instance
    }
}

#[async_trait]
impl TorrentManager for DefaultTorrentManager {
    fn state(&self) -> TorrentManagerState {
        todo!()
    }

    fn register(&self, _callback: TorrentManagerCallback) {
        todo!()
    }

    async fn info<'a>(&'a self, _url: &'a str) -> torrent::Result<TorrentInfo> {
        todo!()
    }
}

#[derive(Debug)]
struct InnerTorrentManager {
    /// The settings of the application
    settings: Arc<Mutex<ApplicationConfig>>,
    runtime: Arc<Runtime>,
}

impl InnerTorrentManager {
    fn on_player_stopped(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        let config = block_in_place(self.settings.lock());
        let torrent_settings = &config.settings.torrent_settings;

        if torrent_settings.cleaning_mode == CleaningMode::Watched {
            debug!("Handling player stopped event for {:?}", event);
        }
    }
}

impl Drop for InnerTorrentManager {
    fn drop(&mut self) {
        let mutex = block_in_place(self.settings.lock());
        let settings = mutex.settings.torrent();

        if settings.cleaning_mode == CleaningMode::OnShutdown {
            debug!("Cleaning torrent directory {:?}", settings.directory);
            if let Err(e) = Storage::clean_directory(settings.directory()) {
                error!("Failed to clean torrent directory, {}", e)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use popcorn_fx_core::core::config::{PopcornSettings, TorrentSettings};
    use popcorn_fx_core::core::storage::Storage;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_on_player_stopped() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let output_path = copy_test_file(temp_path, "example.mp4", None);
        let settings = default_config(temp_path, CleaningMode::Watched);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultTorrentManager::new(settings, event_publisher.clone(), Arc::new(Runtime::new().unwrap()));
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
                url: "http://localhost:8081/example.mp4".to_string(),
                media: None,
                time: None,
                duration: None,
            }));
        });


        assert_eq!(false, PathBuf::from(output_path).exists())
    }

    #[test]
    fn test_drop_cleaning_disabled() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Off);
        let filepath = copy_test_file(temp_path, "debian.torrent", None);
        let manager = DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()), Arc::new(Runtime::new().unwrap()));

        drop(manager);

        assert_eq!(true, PathBuf::from(filepath).exists())
    }

    #[test]
    fn test_drop_should_clean_directory() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::OnShutdown);
        copy_test_file(temp_path, "debian.torrent", None);
        let manager = DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()), Arc::new(Runtime::new().unwrap()));

        drop(manager);

        assert_eq!(true, temp_dir.path().read_dir().unwrap().next().is_none())
    }

    fn default_config(temp_path: &str, cleaning_mode: CleaningMode) -> Arc<Mutex<ApplicationConfig>> {
        Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: PopcornSettings {
                subtitle_settings: Default::default(),
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: TorrentSettings {
                    directory: PathBuf::from(temp_path),
                    cleaning_mode,
                    connections_limit: 0,
                    download_rate_limit: 0,
                    upload_rate_limit: 0,
                },
                playback_settings: Default::default(),
            },
            callbacks: Default::default(),
        }))
    }
}