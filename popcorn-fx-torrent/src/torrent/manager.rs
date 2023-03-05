use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error};
use tokio::sync::Mutex;

use popcorn_fx_core::core::config::ApplicationConfig;
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{TorrentInfo, TorrentManager, TorrentManagerCallback, TorrentManagerState};

/// The default torrent manager of the application.
/// It currently only cleans the torrent directory if needed.
/// No actual torrent implementation is available.
pub struct DefaultTorrentManager {
    /// The settings of the application
    settings: Arc<Mutex<ApplicationConfig>>,
}

impl DefaultTorrentManager {
    pub fn new(settings: &Arc<Mutex<ApplicationConfig>>) -> Self {
        Self {
            settings: settings.clone(),
        }
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

impl Drop for DefaultTorrentManager {
    fn drop(&mut self) {
        let mutex = self.settings.blocking_lock();
        let settings = mutex.settings.torrent();

        if settings.auto_cleaning_enabled {
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

    use popcorn_fx_core::core::config::{PopcornProperties, PopcornSettings, TorrentSettings};
    use popcorn_fx_core::core::storage::Storage;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_drop_should_clean_directory() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: Default::default(),
            settings: PopcornSettings {
                subtitle_settings: Default::default(),
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: TorrentSettings {
                    directory: PathBuf::from(temp_path),
                    auto_cleaning_enabled: true,
                    connections_limit: 0,
                    download_rate_limit: 0,
                    upload_rate_limit: 0,
                },
                playback_settings: Default::default(),
            },
            callbacks: Default::default(),
        }));
        copy_test_file(temp_path, "debian.torrent", None);
        let manager = DefaultTorrentManager::new(&settings);

        drop(manager);

        assert_eq!(true, temp_dir.path().read_dir().unwrap().next().is_none())
    }
}