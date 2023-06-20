use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use log::{debug, error, info, trace, warn};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use popcorn_fx_core::core::{block_in_place, events, torrent};
use popcorn_fx_core::core::config::{ApplicationConfig, CleaningMode, TorrentSettings};
use popcorn_fx_core::core::events::{Event, EventPublisher, PlayerStoppedEvent};
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::torrent::{Torrent, TorrentInfo, TorrentManager, TorrentManagerCallback, TorrentManagerState, TorrentWrapper};

const CLEANUP_WATCH_THRESHOLD: f64 = 85 as f64;
const CLEANUP_AFTER: fn() -> Duration = || Duration::days(10);

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
                torrents: Default::default(),
            }),
        };

        let cloned_instance = instance.inner.clone();
        event_publisher.register(Box::new(move |event| {
            if let Event::PlayerStopped(e) = &event {
                cloned_instance.on_player_stopped(e.clone());
            }

            Some(event)
        }), events::LOWEST_ORDER - 10);

        instance
    }
}

#[async_trait]
impl TorrentManager for DefaultTorrentManager {
    fn state(&self) -> TorrentManagerState {
        self.inner.state()
    }

    fn register(&self, callback: TorrentManagerCallback) {
        self.inner.register(callback)
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        self.inner.info(url).await
    }

    fn add(&self, torrent: Arc<TorrentWrapper>) {
        self.inner.add(torrent)
    }
}

#[derive(Debug)]
struct InnerTorrentManager {
    /// The settings of the application
    settings: Arc<Mutex<ApplicationConfig>>,
    runtime: Arc<Runtime>,
    torrents: Mutex<Vec<Arc<TorrentWrapper>>>,
}

impl InnerTorrentManager {
    fn on_player_stopped(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        let config = block_in_place(self.settings.lock());
        let torrent_settings = &config.settings.torrent_settings;

        if torrent_settings.cleaning_mode == CleaningMode::Watched {
            debug!("Handling player stopped event for {:?}", event);
            if let Some(filename) = event.filename() {
                if let (Some(time), Some(duration)) = (&event.time, &event.duration) {
                    let percentage = (*time as f64 / *duration as f64) * 100 as f64;

                    trace!("Media {} has been watched for {:.2}", filename, percentage);
                    if percentage >= CLEANUP_WATCH_THRESHOLD {
                        debug!("Cleaning media file \"{}\"", filename);
                        if let Some(torrent) = self.find_by_filename(filename.as_str()) {
                            let filepath = torrent.file();
                            let absolute_filepath = filepath.to_str().unwrap();

                            if filepath.exists() {
                                if let Err(e) = fs::remove_file(filepath.as_path()) {
                                    error!("Failed to remove media file \"{}\", {}", absolute_filepath, e)
                                } else {
                                    info!("Media file \"{}\" has been removed", absolute_filepath);
                                }
                            } else {
                                warn!("Unable to clean {}, filename doesn't exist at the expected location", absolute_filepath)
                            }

                            self.remove_by_filename(filename.as_str());
                        } else {
                            warn!("Unable to find related torrent for \"{}\"", filename);
                        }
                    }
                }
            } else {
                warn!("Unable to handle player stopped event, no valid filename found")
            }
        }
    }

    fn find_by_filename(&self, filename: &str) -> Option<Arc<TorrentWrapper>> {
        let torrents = block_in_place(self.torrents.lock());

        trace!("Searching for \"{}\" in {:?}", filename, *torrents);
        torrents.iter()
            .find(|e| {
                let absolute_path = e.filepath.to_str().unwrap();
                absolute_path.contains(filename)
            })
            .map(|e| e.clone())
    }

    fn remove_by_filename(&self, filename: &str) {
        let mut torrents = block_in_place(self.torrents.lock());
        let position = torrents.iter()
            .position(|e| {
                let absolute_path = e.filepath.to_str().unwrap();
                absolute_path.contains(filename)
            });

        if let Some(position) = position {
            let torrent = torrents.remove(position);
            debug!("Removed torrent {:?}", torrent)
        } else {
            warn!("Unable to remove torrent with filename {}, torrent not found", filename)
        }
    }

    fn clean_directory(settings: &TorrentSettings) {
        debug!("Cleaning torrent directory {}", settings.directory().to_str().unwrap());
        if let Err(e) = Storage::clean_directory(settings.directory()) {
            error!("Failed to clean torrent directory, {}", e)
        }
    }

    fn clean_directory_after(settings: &TorrentSettings) {
        let cleanup_after = CLEANUP_AFTER();
        debug!("Cleaning torrents older than {}", cleanup_after);
        for entry in settings.directory.read_dir().expect("expected the directory to be readable") {
            match entry {
                Ok(filepath) => {
                    match filepath.metadata() {
                        Ok(meta) => {
                            let absolute_path = filepath.path().to_str().unwrap().to_string();
                            if let Ok(last_modified) = meta.modified() {
                                let last_modified = DateTime::from(last_modified);
                                trace!("Torrent path {} has last been modified at {}", absolute_path, last_modified);
                                if Local::now() - last_modified >= cleanup_after {
                                    match Storage::delete(filepath.path()) {
                                        Ok(_) => debug!("Torrent path {} has been removed", absolute_path),
                                        Err(e) => error!("Failed to remove torrent path {}, {}", absolute_path, e)
                                    }
                                }
                            };
                        }
                        Err(e) => warn!("Unable to read entry data, {}", e)
                    }
                }
                Err(e) => warn!("File entry is invalid, {}", e)
            }
        }
    }
}

#[async_trait]
impl TorrentManager for InnerTorrentManager {
    fn state(&self) -> TorrentManagerState {
        TorrentManagerState::Running
    }

    fn register(&self, callback: TorrentManagerCallback) {
        todo!()
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        todo!()
    }

    fn add(&self, torrent: Arc<TorrentWrapper>) {
        trace!("Adding new torrent wrapper {:?}", torrent);
        let mut torrents = self.torrents.blocking_lock();
        let info = torrent.to_string();
        torrents.push(torrent);
        debug!("Added torrent {}", info)
    }
}

impl Drop for InnerTorrentManager {
    fn drop(&mut self) {
        let mutex = block_in_place(self.settings.lock());
        let settings = mutex.settings.torrent();

        match settings.cleaning_mode {
            CleaningMode::OnShutdown => Self::clean_directory(settings),
            CleaningMode::Watched => Self::clean_directory_after(settings),
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::mpsc::channel;

    use utime::set_file_times;

    use popcorn_fx_core::core::config::{PopcornSettings, TorrentSettings};
    use popcorn_fx_core::core::storage::Storage;
    use popcorn_fx_core::core::torrent::TorrentState;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Off);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultTorrentManager::new(settings, event_publisher.clone(), Arc::new(Runtime::new().unwrap()));

        assert_eq!(TorrentManagerState::Running, manager.state())
    }

    #[test]
    fn test_on_player_stopped() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = "torrents/lorem ipsum=[dolor].mp4";
        let output_path = copy_test_file(temp_path, "example.mp4", Some(filename));
        let settings = default_config(temp_path, CleaningMode::Watched);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager = DefaultTorrentManager::new(settings, event_publisher.clone(), Arc::new(Runtime::new().unwrap()));
        let (tx, rx) = channel();

        manager.add(Arc::new(TorrentWrapper {
            filepath: PathBuf::from(temp_path).join(filename),
            has_bytes: Mutex::new(Box::new(|_| true)),
            has_piece: Mutex::new(Box::new(|_| true)),
            total_pieces: Mutex::new(Box::new(|| 10)),
            prioritize_bytes: Mutex::new(Box::new(|_| {})),
            prioritize_pieces: Mutex::new(Box::new(|_| {})),
            sequential_mode: Mutex::new(Box::new(|| {})),
            torrent_state: Mutex::new(Box::new(|| TorrentState::Downloading)),
            callbacks: Default::default(),
        }));
        event_publisher.register(Box::new(move |e| {
            tx.send(true).unwrap();
            Some(e)
        }), events::LOWEST_ORDER);
        manager.inner.runtime.block_on(async {
            event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
                url: "http://localhost:8081/lorem%20ipsum%3D%5Bdolor%5D.mp4".to_string(),
                media: None,
                time: Some(55000),
                duration: Some(60000),
            }));
        });

        rx.recv_timeout(std::time::Duration::from_millis(200)).unwrap();
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
    fn test_drop_cleaning_mode_set_to_on_shutdown() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::OnShutdown);
        copy_test_file(temp_path, "debian.torrent", Some("torrents/debian.torrent"));
        let manager = DefaultTorrentManager::new(settings.clone(), Arc::new(EventPublisher::default()), Arc::new(Runtime::new().unwrap()));

        drop(manager);

        let config = settings.blocking_lock();
        assert_eq!(true, config.settings.torrent_settings.directory.read_dir().unwrap().next().is_none())
    }

    #[test]
    fn test_drop_cleaning_mode_set_to_watched() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Watched);
        let filepath = copy_test_file(temp_path, "debian.torrent", Some("torrents/my-torrent/debian.torrent"));
        let manager = DefaultTorrentManager::new(settings.clone(), Arc::new(EventPublisher::default()), Arc::new(Runtime::new().unwrap()));
        let modified = Local::now() - Duration::days(10);

        set_file_times(PathBuf::from(temp_path).join("torrents").join("my-torrent"), modified.timestamp(), modified.timestamp()).unwrap();
        drop(manager);

        let config = settings.blocking_lock();
        assert_eq!(true, config.settings.torrent_settings.directory.read_dir().unwrap().next().is_none())
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
                    directory: PathBuf::from(temp_path).join("torrents"),
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