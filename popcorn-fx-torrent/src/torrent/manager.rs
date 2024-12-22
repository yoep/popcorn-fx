use crate::torrent::{
    errors, torrent, DefaultSession, FilePriority, PieceIndex, PiecePriority, Session, Torrent,
    TorrentEvent, TorrentFlags, TorrentInfoFile,
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
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;

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
            .map(|e| e.path)
            .unwrap_or(PathBuf::from("unknown"))
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
                    .runtime(runtime.clone())
                    .build(),
            )
            .map(|e| Box::new(e))
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;

        let instance = Self {
            inner: Arc::new(InnerTorrentManager {
                settings,
                session,
                callbacks: Default::default(),
                runtime,
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
            event::DEFAULT_ORDER - 10,
        );

        Ok(instance)
    }
}

#[async_trait]
impl TorrentManager for DefaultTorrentManager {
    async fn info<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentInfo> {
        self.inner.info(url).await
    }

    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth> {
        self.inner.health_from_uri(url).await
    }

    async fn create(
        &self,
        uri: &str,
        file_info: &TorrentFileInfo,
        auto_download: bool,
    ) -> torrents::Result<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        self.inner.create(uri, file_info, auto_download).await
    }

    async fn by_handle(
        &self,
        handle: TorrentHandle,
    ) -> Option<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        self.inner.by_handle(handle).await
    }

    fn remove(&self, handle: TorrentHandle) {
        self.inner.remove(handle)
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
    /// The callbacks of the torrent manager
    callbacks: CoreCallbacks<TorrentManagerEvent>,
    /// The shared runtime
    runtime: Arc<Runtime>,
}

impl InnerTorrentManager {
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
                        info_hash: torrent_info.info_hash.to_string(),
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
                    return torrents::Error::TorrentResolvingFailed(e.to_string());
                }

                torrents::Error::TorrentError(e.to_string())
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
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))
    }

    async fn create(
        &self,
        uri: &str,
        file_info: &TorrentFileInfo,
        auto_download: bool,
    ) -> torrents::Result<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        let torrent = self
            .session
            .add_torrent_from_uri(uri, TorrentFlags::Metadata)
            .await
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;

        // make sure that the metadata if the torrent is fetched
        torrent
            .add_options(TorrentFlags::Metadata | TorrentFlags::UploadMode)
            .await;
        let mut files = torrent.files().await;

        if files.is_empty() {
            let mut receiver = torrent.subscribe();

            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = *event {
                        break;
                    }
                } else {
                    return Err(torrents::Error::TorrentError(
                        "torrent got invalidated".to_string(),
                    ));
                }
            }

            files = torrent.files().await;
        }

        let file_priorities = files
            .iter_mut()
            .map(|file| {
                let priority = if file.index == file_info.file_index {
                    FilePriority::Normal
                } else {
                    FilePriority::None
                };

                (file.index, priority)
            })
            .collect();
        torrent.priorities_files(file_priorities).await;

        if auto_download {
            torrent.resume().await;
        }

        Ok(Box::new(torrent))
    }

    async fn by_handle(
        &self,
        handle: TorrentHandle,
    ) -> Option<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        self.session
            .find_torrent_by_handle(handle)
            .await
            .map(|e| Box::new(e) as Box<dyn popcorn_fx_core::core::torrents::Torrent>)
    }

    fn remove(&self, handle: TorrentHandle) {
        self.session.remove_torrent(handle)
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
                            let filepath = block_in_place_runtime(torrent.file(), &self.runtime);
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

    fn find_by_filename(
        &self,
        _filename: &str,
    ) -> Option<Box<dyn popcorn_fx_core::core::torrents::Torrent>> {
        todo!()
    }

    fn remove_by_filename(&self, _filename: &str) {
        todo!()
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
    use popcorn_fx_core::testing::{copy_test_file, init_logger};
    use std::fs::{File, FileTimes};
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::time::SystemTime;

    #[test]
    fn test_info() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let settings = default_config(temp_path, CleaningMode::Off);
        let event_publisher = Arc::new(EventPublisher::default());
        let manager =
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime.clone()).unwrap();
        let expected_result = TorrentInfo {
            info_hash: "EADAF0EFEA39406914414D359E0EA16416409BD7".to_string(),
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
    fn test_create() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        // let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let uri = "magnet:?xt=urn:btih:95EF921A3E1128F65CDD7B85E87D738249FA94A1&tr=udp://tracker.opentrackr.org:1337&tr=udp://tracker.tiny-vps.com:6969&tr=udp://tracker.openbittorrent.com:1337&tr=udp://tracker.coppersurfer.tk:6969&tr=udp://tracker.leechers-paradise.org:6969&tr=udp://p4p.arenabg.ch:1337&tr=udp://p4p.arenabg.com:1337&tr=udp://tracker.internetwarriors.net:1337&tr=udp://9.rarbg.to:2710&tr=udp://9.rarbg.me:2710&tr=udp://exodus.desync.com:6969&tr=udp://tracker.cyberia.is:6969&tr=udp://tracker.torrent.eu.org:451&tr=udp://open.stealth.si:80&tr=udp://tracker.moeking.me:6969&tr=udp://tracker.zerobytes.xyz:1337";
        let settings = default_config(temp_path, CleaningMode::Off);
        let event_publisher = Arc::new(EventPublisher::default());
        let (tx, rx) = channel();
        let manager =
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime.clone()).unwrap();

        let torrent_info = runtime
            .block_on(manager.info(uri))
            .expect("expected the torrent info to have been returned");
        let torrent_file = torrent_info
            .largest_file()
            .expect("expected a file to be returned");

        let torrent = runtime
            .block_on(manager.create(uri, &torrent_file, true))
            .expect("expected the torrent to have been created");

        torrent.add_callback(Box::new(move |event| {
            if let torrents::TorrentEvent::PieceFinished(piece) = event {
                info!("Received piece finished event for piece {}", piece);
                tx.send(()).unwrap()
            }
        }));

        rx.recv_timeout(Duration::from_secs(120))
            .expect("expected the download to have been started");
    }

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
            info_hash: String::new(),
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
            DefaultTorrentManager::new(settings, event_publisher.clone(), runtime.clone()).unwrap();
        let (tx, rx) = channel();

        let torrent_info_callback = torrent_info.clone();

        // register the torrent information by invoking the callbacks
        match runtime.block_on(manager.info(magnet_uri)) {
            Ok(result) => {
                assert_eq!(torrent_info, result);

                let torrent_file_info = result
                    .largest_file()
                    .expect("expected a torrent file to have been present in the torrent info");
                let result = runtime.block_on(manager.create(magnet_uri, &torrent_file_info, true));
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
