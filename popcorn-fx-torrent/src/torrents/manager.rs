use std::fmt::{Debug, Formatter};
use std::fs;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::torrents::peers::extensions::metadata::MetadataExtension;
use crate::torrents::peers::extensions::Extension;
use crate::torrents::{errors, DefaultSession, Session, TorrentInfoFile};
use popcorn_fx_core::core::config::{ApplicationConfig, CleaningMode, TorrentSettings};
use popcorn_fx_core::core::events::{Event, EventPublisher, PlayerStoppedEvent};
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::torrents::{
    Torrent, TorrentFileInfo, TorrentHealth, TorrentInfo, TorrentManager, TorrentManagerCallback,
    TorrentManagerState, TorrentWrapper,
};
use popcorn_fx_core::core::{block_in_place, events, torrents};

const CLEANUP_WATCH_THRESHOLD: f64 = 85f64;
const CLEANUP_AFTER: fn() -> Duration = || Duration::days(10);

/// A callback function type for resolving torrents.
///
/// The function takes a `TorrentFileInfo` struct, a `String` representing the torrent directory,
/// and a `bool` indicating whether auto-start download is enabled. It returns a `TorrentWrapper`.
/// It must be `Send` and `Sync` to support concurrent execution.
pub type ResolveTorrentCallback =
    Box<dyn Fn(&TorrentFileInfo, &str, bool) -> TorrentWrapper + Send + Sync>;

/// A callback function signature for canceling a torrent operation.
///
/// This type represents a callback function signature that takes a `String` argument. It can be used to define
/// functions that cancel torrent-related operations, where the `String` argument may contain additional information
/// or identifiers related to the operation to be canceled.
///
/// The callback function can be used to invoke cancellation logic, typically to stop and clean up torrent-related tasks or processes.
pub type CancelTorrentCallback = Box<dyn Fn(String) + Send + Sync>;

/// The default torrent manager of the application.
/// It currently only cleans the torrent directory if needed.
/// No actual torrent implementation is available.
#[derive(Debug)]
pub struct DefaultTorrentManager {
    inner: Arc<InnerTorrentManager>,
}

impl DefaultTorrentManager {
    pub fn new(
        settings: Arc<ApplicationConfig>,
        event_publisher: Arc<EventPublisher>,
        runtime: Arc<Runtime>,
    ) -> popcorn_fx_core::core::torrents::Result<Self> {
        let extensions: Vec<Box<dyn Extension>> = vec![Box::new(MetadataExtension::new())];
        let session: Box<dyn Session> = block_in_place(DefaultSession::new(extensions, runtime))
            .map(|e| Box::new(e))
            .map_err(|e| torrents::TorrentError::TorrentError(e.to_string()))?;

        let instance = Self {
            inner: Arc::new(InnerTorrentManager {
                settings,
                session,
                torrents: Default::default(),
                resolve_torrent_callback: Mutex::new(Box::new(|_, _, _| {
                    panic!("No torrent resolver configured")
                })),
                cancel_torrent_callback: Mutex::new(Box::new(|_| {
                    panic!("No cancel torrent callback configured")
                })),
            }),
        };

        let cloned_instance = instance.inner.clone();
        event_publisher.register(
            Box::new(move |event| {
                if let Event::PlayerStopped(e) = &event {
                    cloned_instance.on_player_stopped(e.clone());
                }

                Some(event)
            }),
            events::DEFAULT_ORDER - 10,
        );

        Ok(instance)
    }

    pub fn register_resolve_callback(&self, callback: ResolveTorrentCallback) {
        trace!("Updating torrent resolve callback");
        let mut guard = block_in_place(self.inner.resolve_torrent_callback.lock());
        *guard = callback;
        info!("Updated torrent resolve callback");
    }

    pub fn register_cancel_callback(&self, callback: CancelTorrentCallback) {
        trace!("Updating torrent cancel callback");
        let mut guard = block_in_place(self.inner.cancel_torrent_callback.lock());
        *guard = callback;
        info!("Updated torrent cancel callback");
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

    async fn info<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentInfo> {
        self.inner.info(url).await
    }

    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth> {
        self.inner.health_from_uri(url).await
    }

    async fn create(
        &self,
        file_info: &TorrentFileInfo,
        torrent_directory: &str,
        auto_download: bool,
    ) -> torrents::Result<Weak<Box<dyn Torrent>>> {
        self.inner
            .create(file_info, torrent_directory, auto_download)
            .await
    }

    fn by_handle(&self, handle: &str) -> Option<Weak<Box<dyn Torrent>>> {
        self.inner.by_handle(handle)
    }

    fn remove(&self, handle: &str) {
        self.inner.remove(handle)
    }

    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth {
        self.inner.calculate_health(seeds, leechers)
    }

    fn cleanup(&self) {
        self.inner.cleanup()
    }
}

struct InnerTorrentManager {
    /// The settings of the application
    settings: Arc<ApplicationConfig>,
    /// The underlying torrent sessions of the application
    session: Box<dyn Session>,
    #[deprecated]
    torrents: Mutex<Vec<Arc<Box<dyn Torrent>>>>,
    resolve_torrent_callback: Mutex<ResolveTorrentCallback>,
    cancel_torrent_callback: Mutex<CancelTorrentCallback>,
}

impl InnerTorrentManager {
    fn state(&self) -> TorrentManagerState {
        TorrentManagerState::Running
    }

    fn register(&self, _callback: TorrentManagerCallback) {
        todo!()
    }

    async fn info<'a>(
        &'a self,
        url: &'a str,
    ) -> popcorn_fx_core::core::torrents::Result<TorrentInfo> {
        debug!("Resolving torrent magnet url {}", url);
        self.session
            .fetch_magnet(url, std::time::Duration::from_secs(30))
            .await
            .and_then(|torrent_info| {
                trace!("Retrieved torrent info {:?}", torrent_info);
                if let Some(metadata) = torrent_info.info {
                    let directory_name = if let TorrentInfoFile::Single { .. } = &metadata.files {
                        None
                    } else {
                        Some(metadata.name.clone())
                    };

                    Ok(TorrentInfo {
                        uri: url.to_string(),
                        total_files: metadata.total_files() as u32,
                        files: metadata
                            .files()
                            .into_iter()
                            .enumerate()
                            .map(|(index, file)| {
                                let file_path = file.path.as_ref().cloned().unwrap_or(Vec::new());

                                TorrentFileInfo {
                                    filename: file_path
                                        .iter()
                                        .last()
                                        .cloned()
                                        .unwrap_or(String::new()),
                                    file_path: file_path.iter().join("/"),
                                    file_size: file.length,
                                    file_index: index,
                                }
                            })
                            .collect(),
                        name: metadata.name,
                        directory_name,
                    })
                } else {
                    debug!(
                        "Torrent info is missing it's metadata, unable to create the torrent info"
                    );
                    Err(errors::TorrentError::InvalidMetadata(format!(
                        "metadata is missing for torrent {}",
                        torrent_info.info_hash
                    )))
                }
            })
            .map_err(|e| {
                if let errors::TorrentError::Timeout = e {
                    return torrents::TorrentError::TorrentResolvingFailed(e.to_string());
                }

                torrents::TorrentError::TorrentError(e.to_string())
            })
    }

    async fn health_from_uri<'a>(
        &'a self,
        url: &'a str,
    ) -> popcorn_fx_core::core::torrents::Result<TorrentHealth> {
        trace!("Retrieving torrent health from magnet link {}", url);
        self.session
            .torrent_health_from_uri(url)
            .await
            .map_err(|e| torrents::TorrentError::TorrentError(e.to_string()))
    }

    async fn create(
        &self,
        file_info: &TorrentFileInfo,
        torrent_directory: &str,
        auto_download: bool,
    ) -> torrents::Result<Weak<Box<dyn Torrent>>> {
        debug!("Resolving torrent info {:?}", file_info);
        let torrent_wrapper: TorrentWrapper;

        {
            let callback = block_in_place(self.resolve_torrent_callback.lock());
            torrent_wrapper = callback(file_info, torrent_directory, auto_download);
        }

        trace!("Received resolved torrent {:?}", torrent_wrapper);
        let wrapper = Arc::new(Box::new(torrent_wrapper) as Box<dyn Torrent>);
        let handle = wrapper.handle();

        if self.by_handle(handle).is_none() {
            let mut mutex = block_in_place(self.torrents.lock());
            debug!("Adding torrent with handle {}", handle);
            mutex.push(wrapper.clone());
        } else {
            warn!(
                "Duplicate handle {} detected, unable to add torrent",
                handle
            );
        }

        Ok(Arc::downgrade(&wrapper))
    }

    fn by_handle(&self, handle: &str) -> Option<Weak<Box<dyn Torrent>>> {
        let mutex = block_in_place(self.torrents.lock());
        mutex
            .iter()
            .find(|e| e.handle() == handle)
            .map(|e| Arc::downgrade(e))
    }

    fn remove(&self, handle: &str) {
        let mut mutex = block_in_place(self.torrents.lock());
        let position = mutex.iter().position(|e| e.handle() == handle);

        if let Some(position) = position {
            debug!("Removing torrent with handle {}", handle);
            let torrent = mutex.remove(position);
            drop(mutex);

            let mutex = block_in_place(self.cancel_torrent_callback.lock());
            mutex(torrent.handle().to_string());
        }
    }

    fn on_player_stopped(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        let settings = self.settings.user_settings();
        let torrent_settings = &settings.torrent_settings;

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
                                    error!(
                                        "Failed to remove media file \"{}\", {}",
                                        absolute_filepath, e
                                    )
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

    fn find_by_filename(&self, filename: &str) -> Option<Arc<Box<dyn Torrent>>> {
        let torrents = block_in_place(self.torrents.lock());

        trace!("Searching for \"{}\" in {:?}", filename, *torrents);
        torrents
            .iter()
            .find(|e| {
                let absolute_path = e
                    .file()
                    .to_str()
                    .map(|e| e.to_string())
                    .expect("expected the torrent to have a valid filepath");
                absolute_path.contains(filename)
            })
            .map(|e| e.clone())
    }

    fn remove_by_filename(&self, filename: &str) {
        let mut torrents = block_in_place(self.torrents.lock());
        let position = torrents.iter().position(|e| {
            let absolute_path = e
                .file()
                .to_str()
                .map(|e| e.to_string())
                .expect("expected the torrent to have a valid filepath");
            absolute_path.contains(filename)
        });

        if let Some(position) = position {
            let torrent = torrents.remove(position);
            debug!("Removed torrent {:?}", torrent)
        } else {
            warn!(
                "Unable to remove torrent with filename {}, torrent not found",
                filename
            )
        }
    }

    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth {
        TorrentHealth::from(seeds, leechers)
    }

    fn cleanup(&self) {
        let settings = self.settings.user_settings();
        let settings = settings.torrent();
        Self::clean_directory(settings);
    }

    fn clean_directory(settings: &TorrentSettings) {
        debug!(
            "Cleaning torrent directory {}",
            settings.directory().to_str().unwrap()
        );
        if let Err(e) = Storage::clean_directory(settings.directory()) {
            error!("Failed to clean torrent directory, {}", e)
        }
    }

    fn clean_directory_after(settings: &TorrentSettings) {
        let cleanup_after = CLEANUP_AFTER();
        debug!("Cleaning torrents older than {}", cleanup_after);
        for entry in settings
            .directory
            .read_dir()
            .expect("expected the directory to be readable")
        {
            match entry {
                Ok(filepath) => match filepath.metadata() {
                    Ok(meta) => {
                        let absolute_path = filepath.path().to_str().unwrap().to_string();
                        if let Ok(last_modified) = meta.modified() {
                            let last_modified = DateTime::from(last_modified);
                            trace!(
                                "Torrent path {} has last been modified at {}",
                                absolute_path,
                                last_modified
                            );
                            if Local::now() - last_modified >= cleanup_after {
                                match Storage::delete(filepath.path()) {
                                    Ok(_) => {
                                        debug!("Torrent path {} has been removed", absolute_path)
                                    }
                                    Err(e) => error!(
                                        "Failed to remove torrent path {}, {}",
                                        absolute_path, e
                                    ),
                                }
                            }
                        };
                    }
                    Err(e) => warn!("Unable to read entry data, {}", e),
                },
                Err(e) => warn!("File entry is invalid, {}", e),
            }
        }
    }
}

impl Debug for InnerTorrentManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerTorrentManager")
            .field("settings", &self.settings)
            .field("torrents", &self.torrents)
            .finish()
    }
}

impl Drop for InnerTorrentManager {
    fn drop(&mut self) {
        let settings = self.settings.user_settings();
        let settings = settings.torrent();

        match settings.cleaning_mode {
            CleaningMode::OnShutdown => Self::clean_directory(settings),
            CleaningMode::Watched => Self::clean_directory_after(settings),
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::config::{PopcornSettings, TorrentSettings};
    use popcorn_fx_core::core::torrents::TorrentState;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};
    use std::fs::{File, FileTimes};
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn test_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = default_config(temp_path, CleaningMode::Off);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager =
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime).unwrap();

        assert_eq!(TorrentManagerState::Running, manager.state())
    }

    #[test]
    fn test_info() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let settings = default_config(temp_path, CleaningMode::Off);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager =
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime.clone()).unwrap();
        let expected_result = TorrentInfo {
            uri: uri.to_string(),
            name: "debian-12.4.0-amd64-DVD-1.iso".to_string(),
            directory_name: None,
            total_files: 1,
            files: vec![TorrentFileInfo {
                filename: "debian-12.4.0-amd64-DVD-1.iso".to_string(),
                file_path: "debian-12.4.0-amd64-DVD-1.iso".to_string(),
                file_size: 3994091520,
                file_index: 0,
            }],
        };

        let result = runtime
            .block_on(manager.info(uri))
            .expect("expected the torrent info to have been returned");

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_on_player_stopped() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let magnet_uri = "magnet:?ExampleMagnetUri";
        let filename = "torrents/lorem ipsum=[dolor].mp4";
        let filepath = PathBuf::from(temp_path).join(filename);
        let torrent_info = TorrentInfo {
            uri: String::new(),
            name: filename.to_string(),
            directory_name: None,
            total_files: 1,
            files: vec![TorrentFileInfo {
                filename: "lorem ipsum=[dolor].mp4".to_string(),
                file_path: filepath.to_str().unwrap().to_string(),
                file_size: 28000,
                file_index: 0,
            }],
        };
        let output_path = copy_test_file(temp_path, "example.mp4", Some(filename));
        let settings = default_config(temp_path, CleaningMode::Watched);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager =
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime).unwrap();
        let (tx, rx) = channel();

        manager.register_resolve_callback(Box::new(move |_, _, _| TorrentWrapper {
            handle: "MyHandle".to_string(),
            filepath: filepath.clone(),
            has_bytes: Mutex::new(Box::new(|_| true)),
            has_piece: Mutex::new(Box::new(|_| true)),
            total_pieces: Mutex::new(Box::new(|| 10)),
            prioritize_bytes: Mutex::new(Box::new(|_| {})),
            prioritize_pieces: Mutex::new(Box::new(|_| {})),
            sequential_mode: Mutex::new(Box::new(|| {})),
            torrent_state: Mutex::new(Box::new(|| TorrentState::Downloading)),
            callbacks: Default::default(),
        }));
        let torrent_info_callback = torrent_info.clone();

        // register the torrent information by invoking the callbacks
        match block_in_place(manager.info(magnet_uri)) {
            Ok(result) => {
                assert_eq!(torrent_info, result);

                let torrent_file_info = result
                    .largest_file()
                    .expect("expected a torrent file to have been present in the torrent info");
                let result = block_in_place(manager.create(&torrent_file_info, temp_path, true));
                assert!(
                    result.is_ok(),
                    "expected the torrent to have been created, {}",
                    result.err().unwrap()
                );
            }
            Err(e) => assert!(
                false,
                "expected the torrent info to have been returned, {}",
                e
            ),
        }

        event_publisher.register(
            Box::new(move |e| {
                tx.send(true).unwrap();
                Some(e)
            }),
            events::LOWEST_ORDER,
        );
        block_in_place(async {
            event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
                url: "http://localhost:8081/lorem%20ipsum%3D%5Bdolor%5D.mp4".to_string(),
                media: None,
                time: Some(55000),
                duration: Some(60000),
            }));
        });

        rx.recv_timeout(std::time::Duration::from_millis(200))
            .unwrap();
        assert_eq!(false, PathBuf::from(output_path).exists())
    }

    #[test]
    fn test_cleanup() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = default_config(temp_path, CleaningMode::Off);
        let filepath = copy_test_file(temp_path, "debian.torrent", Some("torrents/debian.torrent"));
        let manager =
            DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()), runtime)
                .unwrap();

        manager.cleanup();

        assert_eq!(
            false,
            PathBuf::from(filepath).exists(),
            "expected the file to have been removed"
        );
    }

    #[test]
    fn test_drop_cleaning_disabled() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = default_config(temp_path, CleaningMode::Off);
        let filepath = copy_test_file(temp_path, "debian.torrent", None);
        let manager =
            DefaultTorrentManager::new(settings, Arc::new(EventPublisher::default()), runtime)
                .unwrap();

        drop(manager);

        assert_eq!(true, PathBuf::from(filepath).exists())
    }

    #[test]
    fn test_drop_cleaning_mode_set_to_on_shutdown() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = default_config(temp_path, CleaningMode::OnShutdown);
        copy_test_file(temp_path, "debian.torrent", Some("torrents/debian.torrent"));
        let manager = DefaultTorrentManager::new(
            settings.clone(),
            Arc::new(EventPublisher::default()),
            runtime,
        )
        .unwrap();

        drop(manager);

        assert_eq!(
            true,
            settings
                .user_settings()
                .torrent_settings
                .directory
                .read_dir()
                .unwrap()
                .next()
                .is_none()
        )
    }

    #[test]
    fn test_drop_cleaning_mode_set_to_watched() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let settings = default_config(temp_path, CleaningMode::Watched);
        let _ = copy_test_file(
            temp_path,
            "debian.torrent",
            Some("torrents/my-torrent/debian.torrent"),
        );
        let manager = DefaultTorrentManager::new(
            settings.clone(),
            Arc::new(EventPublisher::default()),
            runtime,
        )
        .unwrap();
        let modified = Local::now() - Duration::days(10);

        let file =
            File::open(PathBuf::from(temp_path).join("torrents").join("my-torrent")).unwrap();
        file.set_times(
            FileTimes::new()
                .set_accessed(SystemTime::from(modified))
                .set_modified(SystemTime::from(modified)),
        )
        .unwrap();
        drop(manager);

        assert_eq!(
            true,
            settings
                .user_settings()
                .torrent_settings
                .directory
                .read_dir()
                .unwrap()
                .next()
                .is_none()
        )
    }

    fn default_config(temp_path: &str, cleaning_mode: CleaningMode) -> Arc<ApplicationConfig> {
        Arc::new(
            ApplicationConfig::builder()
                .storage(temp_path)
                .settings(PopcornSettings {
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
                    tracking_settings: Default::default(),
                })
                .build(),
        )
    }
}
