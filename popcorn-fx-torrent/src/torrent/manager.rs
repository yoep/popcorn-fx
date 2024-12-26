use crate::torrent::{
    errors, torrent, DefaultSession, FileIndex, FilePriority, PieceIndex, PiecePriority, Session,
    Torrent, TorrentError, TorrentEvent, TorrentFlags, TorrentInfoFile,
};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use popcorn_fx_core::core::callback::{Callback, Subscriber, Subscription};
use popcorn_fx_core::core::config::{ApplicationConfig, CleaningMode, TorrentSettings};
use popcorn_fx_core::core::event::{Event, EventPublisher, PlayerStoppedEvent};
use popcorn_fx_core::core::storage::Storage;
use popcorn_fx_core::core::torrents::{
    DownloadStatus, TorrentFileInfo, TorrentHandle, TorrentHealth, TorrentInfo, TorrentManager,
    TorrentManagerEvent, TorrentState,
};
use popcorn_fx_core::core::{
    block_in_place_runtime, event, torrents, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::RwLock;

const CLEANUP_WATCH_THRESHOLD: f64 = 85f64;
const CLEANUP_AFTER: fn() -> Duration = || Duration::from_secs(10 * 24 * 60 * 60);

impl crate::torrent::Torrent {
    fn subscribe_and_map_event(&self, sender: Subscriber<torrents::TorrentEvent>) {
        let mut rx_event = self.subscribe();
        tokio::spawn(async move {
            loop {
                match rx_event.recv().await {
                    Some(event) => {
                        let callback_event = match &*event {
                            TorrentEvent::StateChanged(e) => {
                                Some(torrents::TorrentEvent::StateChanged(TorrentState::from(e)))
                            }
                            TorrentEvent::PieceCompleted(e) => {
                                Some(torrents::TorrentEvent::PieceFinished(*e as u32))
                            }
                            TorrentEvent::Stats(stats) => {
                                Some(torrents::TorrentEvent::DownloadStatus(DownloadStatus {
                                    progress: stats.progress(),
                                    seeds: stats.total_peers,
                                    peers: stats.total_peers,
                                    download_speed: stats.download_useful_rate,
                                    upload_speed: stats.upload_useful_rate,
                                    downloaded: stats.total_completed_size as u64,
                                    total_size: stats.total_size,
                                }))
                            }
                            _ => None,
                        };

                        if let Some(event) = callback_event {
                            if let Err(_) = sender.send(Arc::new(event)) {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
        });
    }
}

impl Callback<torrents::TorrentEvent> for Torrent {
    fn subscribe(&self) -> Subscription<torrents::TorrentEvent> {
        let (subscriber, receiver) = unbounded_channel();
        self.subscribe_and_map_event(subscriber);

        receiver
    }

    fn subscribe_with(&self, subscriber: Subscriber<torrents::TorrentEvent>) {
        self.subscribe_and_map_event(subscriber);
    }
}

#[async_trait]
impl popcorn_fx_core::core::torrents::Torrent for Torrent {
    fn handle(&self) -> TorrentHandle {
        self.handle()
    }

    async fn file(&self) -> PathBuf {
        // try to find the first file with a priority
        self.files()
            .await
            .into_iter()
            .find(|e| e.priority != FilePriority::None)
            .and_then(|e| self.absolute_filepath(&e).ok())
            .unwrap_or(PathBuf::from("UNKNOWN"))
    }

    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool {
        self.has_bytes(bytes).await
    }

    async fn has_piece(&self, piece: usize) -> bool {
        self.has_piece(piece as PieceIndex).await
    }

    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>) {
        self.prioritize_bytes(bytes).await
    }

    async fn prioritize_pieces(&self, pieces: &[u32]) {
        let mut priorities = Vec::new();

        for piece in pieces {
            priorities.push((*piece as PieceIndex, PiecePriority::High));
        }

        self.prioritize_pieces(priorities).await;
    }

    async fn total_pieces(&self) -> usize {
        self.total_pieces().await
    }

    async fn sequential_mode(&self) {
        self.add_options(TorrentFlags::SequentialDownload).await
    }

    async fn state(&self) -> TorrentState {
        let state = self.state().await;
        TorrentState::from(&state)
    }
}

impl From<&crate::torrent::TorrentState> for TorrentState {
    fn from(value: &crate::torrent::TorrentState) -> Self {
        match value {
            crate::torrent::TorrentState::Initializing => torrents::TorrentState::Initializing,
            crate::torrent::TorrentState::CheckingFiles => torrents::TorrentState::CheckingFiles,
            crate::torrent::TorrentState::RetrievingMetadata => {
                torrents::TorrentState::RetrievingMetadata
            }
            crate::torrent::TorrentState::Downloading => torrents::TorrentState::Downloading,
            crate::torrent::TorrentState::Finished => torrents::TorrentState::Completed,
            crate::torrent::TorrentState::Seeding => torrents::TorrentState::Completed,
            crate::torrent::TorrentState::Paused => torrents::TorrentState::Paused,
            crate::torrent::TorrentState::Error => torrents::TorrentState::Error,
        }
    }
}

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
    ) -> torrents::Result<Self> {
        let session: Box<dyn Session> = runtime
            .clone()
            .block_on(
                DefaultSession::builder()
                    .base_path(settings.user_settings().torrent_settings.directory())
                    .client_name("PopcornFX")
                    .runtime(runtime.clone())
                    .build(),
            )
            .map(|e| Box::new(e))
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;

        let instance = Self {
            inner: Arc::new(InnerTorrentManager {
                settings,
                session,
                torrent_files: Default::default(),
                callbacks: Default::default(),
                runtime,
            }),
        };

        let cloned_instance = instance.inner.clone();
        event_publisher.register(
            Box::new(move |event| {
                if let Event::PlayerStopped(e) = &event {
                    block_in_place_runtime(
                        cloned_instance.on_player_stopped(e.clone()),
                        &cloned_instance.runtime,
                    );
                }

                Some(event)
            }),
            event::DEFAULT_ORDER - 10,
        );

        Ok(instance)
    }
}

#[async_trait]
impl TorrentManager for DefaultTorrentManager {
    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth> {
        self.inner.health_from_uri(url).await
    }

    async fn create(&self, uri: &str) -> torrents::Result<TorrentHandle> {
        self.inner.create(uri).await
    }

    async fn info(&self, handle: &TorrentHandle) -> torrents::Result<TorrentInfo> {
        self.inner.info(handle).await
    }

    async fn download(
        &self,
        handle: &TorrentHandle,
        file_info: &TorrentFileInfo,
    ) -> torrents::Result<()> {
        self.inner.download(handle, file_info).await
    }

    async fn find_by_handle(
        &self,
        handle: &TorrentHandle,
    ) -> Option<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        self.inner.by_handle(handle).await
    }

    async fn subscribe(
        &self,
        handle: &TorrentHandle,
    ) -> Option<Subscription<torrents::TorrentEvent>> {
        self.inner.subscribe(handle).await
    }

    async fn remove(&self, handle: &TorrentHandle) {
        self.inner.remove(handle).await
    }

    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth {
        self.inner.calculate_health(seeds, leechers)
    }

    fn cleanup(&self) {
        self.inner.cleanup()
    }
}

impl Callbacks<TorrentManagerEvent> for DefaultTorrentManager {
    fn add_callback(&self, callback: CoreCallback<TorrentManagerEvent>) -> CallbackHandle {
        self.inner.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.inner.callbacks.remove_callback(handle)
    }
}

#[derive(Debug)]
struct InnerTorrentManager {
    /// The settings of the application
    settings: Arc<ApplicationConfig>,
    /// The underlying torrent sessions of the application
    session: Box<dyn Session>,
    /// The torrent files being downloaded,
    torrent_files: RwLock<HashMap<TorrentHandle, TorrentFileInfo>>,
    /// The callbacks of the torrent manager
    callbacks: CoreCallbacks<TorrentManagerEvent>,
    /// The shared runtime
    runtime: Arc<Runtime>,
}

impl InnerTorrentManager {
    async fn create(&self, uri: &str) -> torrents::Result<TorrentHandle> {
        self.session
            .add_torrent_from_uri(uri, TorrentFlags::Metadata)
            .await
            .map(|e| e.handle())
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))
    }

    async fn info<'a>(
        &'a self,
        handle: &TorrentHandle,
    ) -> popcorn_fx_core::core::torrents::Result<TorrentInfo> {
        match self.session.find_torrent_by_handle(handle).await {
            Some(torrent) => {
                let mut receiver = torrent.subscribe();

                if !torrent.is_metadata_known().await {
                    loop {
                        if let Some(event) = receiver.recv().await {
                            if let TorrentEvent::MetadataChanged = *event {
                                break;
                            }
                        } else {
                            return Err(torrents::Error::TorrentResolvingFailed(
                                "handle has been dropped".to_string(),
                            ));
                        }
                    }
                }

                let metadata = torrent
                    .metadata()
                    .await
                    .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;
                if let Some(info) = metadata.info {
                    let directory_name = if let TorrentInfoFile::Single { .. } = &info.files {
                        None
                    } else {
                        Some(info.name.clone())
                    };

                    return Ok(TorrentInfo {
                        handle: handle.clone(),
                        uri: "".to_string(),
                        info_hash: metadata.info_hash.to_string(),
                        total_files: info.total_files() as u32,
                        files: info
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
                        name: info.name,
                        directory_name,
                    });
                }

                Err(torrents::Error::TorrentResolvingFailed(
                    "metadata info is missing".to_string(),
                ))
            }
            None => Err(torrents::Error::InvalidHandle(handle.to_string())),
        }
    }

    async fn download(
        &self,
        handle: &TorrentHandle,
        file_info: &TorrentFileInfo,
    ) -> torrents::Result<()> {
        let torrent = self
            .session
            .find_torrent_by_handle(handle)
            .await
            .ok_or(torrents::Error::InvalidHandle(handle.to_string()))?;
        let mut receiver = torrent.subscribe();

        if torrent.total_files().await.unwrap_or(0) == 0 {
            trace!("Waiting for torrent {} to create the files", torrent);
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = *event {
                        break;
                    }
                } else {
                    return Err(torrents::Error::TorrentResolvingFailed(
                        "handle has been dropped".to_string(),
                    ));
                }
            }
        }

        debug!("Prioritizing file {:?} for torrent {}", file_info, torrent);
        let file_priorities: Vec<(FileIndex, FilePriority)> = torrent
            .files()
            .await
            .into_iter()
            .map(|file| {
                let priority = if file.index == file_info.file_index {
                    FilePriority::Normal
                } else {
                    FilePriority::None
                };

                (file.index, priority)
            })
            .collect();

        torrent.prioritize_files(file_priorities).await;
        torrent.add_options(TorrentFlags::AutoManaged).await;
        torrent.resume().await;

        // store the info
        let mut mutex = self.torrent_files.write().await;
        mutex.insert(torrent.handle(), file_info.clone());

        Ok(())
    }

    async fn health_from_uri<'a>(
        &'a self,
        url: &'a str,
    ) -> popcorn_fx_core::core::torrents::Result<TorrentHealth> {
        trace!("Retrieving torrent health from magnet link {}", url);
        self.session
            .torrent_health_from_uri(url)
            .await
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))
    }

    async fn by_handle(
        &self,
        handle: &TorrentHandle,
    ) -> Option<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        self.session
            .find_torrent_by_handle(handle)
            .await
            .map(|e| Box::new(e) as Box<dyn popcorn_fx_core::core::torrents::Torrent>)
    }

    async fn subscribe(
        &self,
        handle: &TorrentHandle,
    ) -> Option<Subscription<torrents::TorrentEvent>> {
        if let Some(torrent) = self.session.find_torrent_by_handle(handle).await {
            return Some(torrent.subscribe());
        }

        None
    }

    async fn remove(&self, handle: &TorrentHandle) {
        debug!("Torrent manager is removing torrent {}", handle);
        self.session.remove_torrent(handle).await
    }

    async fn on_player_stopped(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        let settings = self.settings.user_settings();
        let torrent_settings = &settings.torrent_settings;

        if torrent_settings.cleaning_mode == CleaningMode::Watched {
            debug!("Handling player stopped event for {:?}", event);
            if let Some(filename) = event.filename() {
                if let (Some(time), Some(duration)) = (&event.time, &event.duration) {
                    let percentage = (*time as f64 / *duration as f64) * 100f64;

                    trace!("Media {} has been watched for {:.2}", filename, percentage);
                    if percentage >= CLEANUP_WATCH_THRESHOLD {
                        debug!("Cleaning media file \"{}\"", filename);
                        if let Some(torrent) = self.find_by_filename(filename.as_str()).await {
                            // TODO cleanup torrent files
                            self.session.remove_torrent(&torrent.handle()).await;
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

    async fn find_by_filename(&self, filename: &str) -> Option<Torrent> {
        let torrent_files = self.torrent_files.read().await;

        if let Some((handle, _)) = torrent_files
            .iter()
            .find(|(_, file)| file.filename == filename)
        {
            return self.session.find_torrent_by_handle(handle).await;
        }

        None
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
        debug!("Cleaning torrents older than {:?}", cleanup_after);
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
                            if Local::now() - last_modified
                                >= chrono::Duration::from_std(cleanup_after).unwrap()
                            {
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
    use super::*;
    use popcorn_fx_core::core::config::{PopcornSettings, TorrentSettings};
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::copy_test_file;
    use std::fs::{File, FileTimes};
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::SystemTime;

    #[test]
    fn test_on_player_stopped() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let magnet_uri = "magnet:?ExampleMagnetUri";
        let filename = "torrents/lorem ipsum=[dolor].mp4";
        let filepath = PathBuf::from(temp_path).join(filename);
        let torrent_info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: magnet_uri.to_string(),
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
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime.clone()).unwrap();
        let (tx, rx) = channel();

        let torrent_info_callback = torrent_info.clone();

        // register the torrent information by invoking the callbacks
        // TODO

        event_publisher.register(
            Box::new(move |e| {
                tx.send(true).unwrap();
                Some(e)
            }),
            event::LOWEST_ORDER,
        );
        runtime.block_on(async {
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
        init_logger!();
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
        init_logger!();
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
        init_logger!();
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
        init_logger!();
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
        let modified = Local::now() - chrono::Duration::days(10);

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
