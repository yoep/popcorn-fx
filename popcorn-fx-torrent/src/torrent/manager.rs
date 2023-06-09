use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error};
use tokio::sync::Mutex;

use popcorn_fx_core::core::{events, torrent};
use popcorn_fx_core::core::config::{ApplicationConfig, CleaningMode};
use popcorn_fx_core::core::events::{Event, EventPublisher, Order};
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
    pub fn new(settings: Arc<Mutex<ApplicationConfig>>, event_publisher: Arc<EventPublisher>) -> Self {
        let instance = Self {
            inner: Arc::new(InnerTorrentManager {
                settings,
            })
        };

        event_publisher.register(Box::new(|event| {
            if let Event::PlayerStopped(e) = &event {
                // TODO
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
}

impl InnerTorrentManager {
}

impl Drop for InnerTorrentManager {
    fn drop(&mut self) {
        let mutex = self.settings.blocking_lock();
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
    fn test_drop_cleaning_disabled() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Off);
        let filepath = copy_test_file(temp_path, "debian.torrent", None);
        let manager = DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()));

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
        let manager = DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()));

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