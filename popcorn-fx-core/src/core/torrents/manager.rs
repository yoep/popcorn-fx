use crate::core::config::{ApplicationConfig, CleaningMode, TorrentSettings};
use crate::core::event::{Event, EventCallback, EventHandler, EventPublisher, PlayerStoppedEvent};
use crate::core::storage::Storage;
use crate::core::torrents::{Error, Result, Torrent, TorrentHandle, TorrentInfo};
use crate::core::{event, torrents};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, trace, warn};
#[cfg(any(test, feature = "testing"))]
pub use mock::*;
use popcorn_fx_torrent::torrent::{
    FileIndex, FilePriority, FxTorrentSession, Magnet, Session, SessionEvent, SessionState,
    TorrentEvent, TorrentFiles, TorrentFlags, TorrentHealth, TorrentState,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

const CLEANUP_WATCH_THRESHOLD: f64 = 85f64;
const CLEANUP_AFTER: fn() -> Duration = || Duration::from_secs(10 * 24 * 60 * 60);

/// The events of the torrent manager.
#[derive(Debug, Display, Clone)]
pub enum TorrentManagerEvent {
    #[display(fmt = "torrent {} has been added", _0)]
    TorrentAdded(TorrentHandle),
    #[display(fmt = "torrent {} has been removed", _0)]
    TorrentRemoved(TorrentHandle),
}

/// The torrent manager stores the active sessions and torrents that are being processed.
#[async_trait]
pub trait TorrentManager: Debug + DowncastSync + Callback<TorrentManagerEvent> {
    /// Retrieve the health of the torrent based on the given magnet link.
    ///
    /// # Arguments
    ///
    /// * `url` - The magnet link of the torrent
    ///
    /// # Returns
    ///
    /// The torrent health on success, or a [torrent::TorrentError] if there was an error.
    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth>;

    /// Create a new idle torrent within the torrent manager.
    async fn create(&self, uri: &str) -> torrents::Result<Box<dyn Torrent>>;

    /// Retrieve the metadata information of the torrent.
    async fn info(&self, handle: &TorrentHandle) -> torrents::Result<TorrentInfo>;

    /// Start the download of the given file within the torrent.
    async fn download(&self, handle: &TorrentHandle, filename: &str) -> torrents::Result<()>;

    /// Get a torrent by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a weak reference to the torrent session if found, or `None` if not found.
    async fn find_by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>>;

    /// Remove a torrent session by its unique handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The unique handle of the torrent session to remove.
    async fn remove(&self, handle: &TorrentHandle);

    /// Calculate the health of the torrent based on the given seed count and peer count.
    ///
    /// # Arguments
    ///
    /// * `seeds` - The number of seeds the torrent has (completed peers).
    /// * `leechers` - The number of leechers the torrent has (incomplete peers).
    ///
    /// # Returns
    ///
    /// Returns the calculated torrent health.
    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth;

    /// Cleanup the torrents directory.
    ///
    /// This operation removes all torrents from the filesystem.
    async fn cleanup(&self);
}
impl_downcast!(sync TorrentManager);

/// The default torrent manager of the application.
/// It currently only cleans the torrent directory if needed.
/// No actual torrent implementation is available.
#[derive(Debug)]
pub struct FxTorrentManager {
    inner: Arc<InnerTorrentManager>,
}

impl FxTorrentManager {
    pub async fn new(settings: ApplicationConfig, event_publisher: EventPublisher) -> Result<Self> {
        let mut session = FxTorrentSession::builder();
        session
            .base_path(settings.user_settings().await.torrent_settings.directory())
            .client_name("PopcornFX");
        let session: Box<dyn Session> = session
            .build()
            .map(|e| Box::new(e))
            .map_err(|e| Error::TorrentError(e.to_string()))?;
        let inner = Arc::new(InnerTorrentManager {
            settings,
            session,
            torrent_files: Default::default(),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let event_receiver = event_publisher
            .subscribe(event::DEFAULT_ORDER - 10)
            .map_err(|e| Error::TorrentError(e.to_string()))?;
        let main_loop = inner.clone();
        tokio::spawn(async move {
            main_loop.start(event_receiver).await;
        });

        Ok(Self { inner })
    }
}

#[async_trait]
impl TorrentManager for FxTorrentManager {
    async fn health_from_uri<'a>(&'a self, url: &'a str) -> torrents::Result<TorrentHealth> {
        self.inner.health_from_uri(url).await
    }

    async fn create(&self, uri: &str) -> torrents::Result<Box<dyn torrents::Torrent>> {
        self.inner.create(uri).await
    }

    async fn info(&self, handle: &TorrentHandle) -> torrents::Result<TorrentInfo> {
        self.inner.info(handle).await
    }

    async fn download(&self, handle: &TorrentHandle, filename: &str) -> torrents::Result<()> {
        self.inner.download(handle, filename).await
    }

    async fn find_by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>> {
        self.inner.by_handle(handle).await
    }

    async fn remove(&self, handle: &TorrentHandle) {
        self.inner.remove(handle).await
    }

    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth {
        self.inner.calculate_health(seeds, leechers)
    }

    async fn cleanup(&self) {
        self.inner.cleanup().await
    }
}

impl Callback<TorrentManagerEvent> for FxTorrentManager {
    fn subscribe(&self) -> Subscription<TorrentManagerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentManagerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for FxTorrentManager {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerTorrentManager {
    /// The settings of the application
    settings: ApplicationConfig,
    /// The underlying torrent sessions of the application
    session: Box<dyn Session>,
    /// The torrent files being downloaded,
    torrent_files: RwLock<HashMap<TorrentHandle, String>>,
    /// The callbacks of the torrent manager
    callbacks: MultiThreadedCallback<TorrentManagerEvent>,
    cancellation_token: CancellationToken,
}

impl InnerTorrentManager {
    async fn start(&self, mut event_receiver: EventCallback) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(event) = event_receiver.recv() => self.handle_event(event).await,
            }
        }

        self.on_shutdown().await;
        debug!("Torrent manager main loop ended");
    }

    async fn handle_event(&self, mut handler: EventHandler) {
        if let Some(Event::PlayerStopped(event)) = handler.event_ref() {
            self.on_player_stopped(event.clone()).await;
        }

        handler.next();
    }

    async fn create(&self, uri: &str) -> Result<Box<dyn Torrent>> {
        self.await_session_ready_state().await?;

        trace!(
            "Torrent manager is creating torrent from magnet link {}",
            uri
        );
        self.session
            .add_torrent_from_uri(uri, TorrentFlags::Metadata)
            .await
            .map(|torrent| {
                let handle = torrent.handle();
                self.callbacks
                    .invoke(TorrentManagerEvent::TorrentAdded(handle));
                Box::new(torrent) as Box<dyn Torrent>
            })
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))
    }

    async fn info<'a>(&'a self, handle: &TorrentHandle) -> Result<TorrentInfo> {
        self.await_session_ready_state().await?;

        match self.session.find_torrent_by_handle(handle).await {
            Some(torrent) => {
                let mut receiver = torrent.subscribe();

                if torrent.total_files().await.unwrap_or_default() == 0 {
                    if let Some(value) =
                        Self::await_torrent_files(torrent.handle(), &mut receiver).await
                    {
                        return value;
                    }
                }

                let metadata = torrent
                    .metadata()
                    .await
                    .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;
                let magnet_uri = Magnet::try_from(&metadata)
                    .map(|e| e.to_string())
                    .map_err(|e| torrents::Error::TorrentError(e.to_string()))?;
                if let Some(info) = metadata.info {
                    let directory_name = if let TorrentFiles::Single { .. } = &info.files {
                        None
                    } else {
                        Some(info.name.clone())
                    };
                    let files = torrent.files().await;

                    debug!(
                        "Torrent manager has loaded the metadata of torrent {}, {:?}",
                        handle, info
                    );
                    return Ok(TorrentInfo {
                        handle: handle.clone(),
                        uri: magnet_uri,
                        info_hash: metadata.info_hash.to_string(),
                        total_files: info.total_files() as u32,
                        name: info.name,
                        files,
                        directory_name,
                    });
                }

                trace!(
                    "Torrent manager has failed to load the metadata of torrent {}",
                    handle
                );
                Err(Error::TorrentResolvingFailed(
                    "metadata info is missing".to_string(),
                ))
            }
            None => Err(Error::InvalidHandle(handle.to_string())),
        }
    }

    async fn download(&self, handle: &TorrentHandle, filename: &str) -> Result<()> {
        self.await_session_ready_state().await?;

        let torrent = self
            .session
            .find_torrent_by_handle(handle)
            .await
            .ok_or(Error::InvalidHandle(handle.to_string()))?;
        let mut receiver = torrent.subscribe();

        if torrent.total_files().await.unwrap_or(0) == 0 {
            if let Some(value) = Self::await_torrent_files(torrent.handle(), &mut receiver).await {
                return value;
            }
        }

        debug!("Prioritizing file {:?} for torrent {}", filename, torrent);
        let file_priorities: Vec<(FileIndex, FilePriority)> = torrent
            .files()
            .await
            .into_iter()
            .map(|file| {
                let priority =
                    if Self::normalize(file.filename().as_str()) == Self::normalize(filename) {
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
        mutex.insert(torrent.handle(), filename.to_string());

        Ok(())
    }

    async fn health_from_uri<'a>(&'a self, url: &'a str) -> Result<TorrentHealth> {
        self.await_session_ready_state().await?;

        trace!("Retrieving torrent health from magnet link {}", url);
        self.session
            .torrent_health_from_uri(url)
            .await
            .map_err(|e| torrents::Error::TorrentError(e.to_string()))
    }

    async fn by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>> {
        self.session
            .find_torrent_by_handle(handle)
            .await
            .map(|e| Box::new(e) as Box<dyn Torrent>)
    }

    async fn remove(&self, handle: &TorrentHandle) {
        debug!("Torrent manager is removing torrent {}", handle);
        self.session.remove_torrent(handle).await;
        self.callbacks
            .invoke(TorrentManagerEvent::TorrentRemoved(handle.clone()));
    }

    async fn on_player_stopped(&self, event: PlayerStoppedEvent) {
        trace!("Received player stopped event for {:?}", event);
        let settings = self.settings.user_settings().await;
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

    async fn find_by_filename(
        &self,
        filename: &str,
    ) -> Option<popcorn_fx_torrent::torrent::Torrent> {
        let torrent_files = self.torrent_files.read().await;

        if let Some((handle, _)) = torrent_files
            .iter()
            .find(|(_, file_filename)| *file_filename == filename)
        {
            return self.session.find_torrent_by_handle(handle).await;
        }

        None
    }

    fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth {
        TorrentHealth::from(seeds, leechers)
    }

    async fn on_shutdown(&self) {
        let settings = self
            .settings
            .user_settings_ref(|e| e.torrent().clone())
            .await;
        match settings.cleaning_mode {
            CleaningMode::OnShutdown => Self::clean_directory(&settings),
            CleaningMode::Watched => Self::clean_directory_after(&settings),
            _ => {}
        }
    }

    async fn cleanup(&self) {
        let settings = self
            .settings
            .user_settings_ref(|e| e.torrent().clone())
            .await;
        Self::clean_directory(&settings);
    }

    /// Wait for the session have be initialized and ready for accepting operations.
    async fn await_session_ready_state(&self) -> Result<()> {
        let mut receiver = self.session.subscribe();
        match self.session.state().await {
            SessionState::Initializing => {
                while let Some(event) = receiver.recv().await {
                    if let SessionEvent::StateChanged(state) = &*event {
                        if *state == SessionState::Running {
                            return Ok(());
                        } else {
                            return Err(Error::TorrentError(format!(
                                "session state is invalid, state {}",
                                state
                            )));
                        }
                    }
                }

                Err(Error::TorrentError(
                    "torrent session has closed".to_string(),
                ))
            }
            SessionState::Running => Ok(()),
            SessionState::Error => Err(Error::TorrentError(
                "session state is invalid, state SessionState::Error".to_string(),
            )),
        }
    }

    /// Wait for the torrent files to be created.
    ///
    /// # Arguments
    ///
    /// * `torrent_handle` - The handle of the torrent.
    /// * `receiver` - The receiver of the torrent events.
    ///
    /// # Returns
    ///
    /// It returns an [Err] when the torrent was not found.
    async fn await_torrent_files<T>(
        torrent_handle: TorrentHandle,
        receiver: &mut Subscription<TorrentEvent>,
    ) -> Option<Result<T>> {
        trace!(
            "Torrent manager is waiting for torrent {} files to be created",
            torrent_handle
        );
        loop {
            if let Some(event) = receiver.recv().await {
                match &*event {
                    TorrentEvent::FilesChanged => break,
                    TorrentEvent::StateChanged(state) => {
                        if state == &TorrentState::Error {
                            return Some(Err(torrents::Error::TorrentError(
                                "torrent encountered an error while loading".to_string(),
                            )));
                        }
                    }
                    _ => {}
                }
            } else {
                return Some(Err(torrents::Error::TorrentResolvingFailed(
                    "handle has been dropped".to_string(),
                )));
            }
        }
        None
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

    /// Normalize the given value.
    fn normalize(value: &str) -> String {
        value.trim().to_lowercase()
    }
}

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use fx_callback::Subscriber;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub TorrentManager {}

        #[async_trait]
        impl TorrentManager for TorrentManager {
            async fn health_from_uri<'a>(&'a self, url: &'a str) -> Result<TorrentHealth>;
            async fn create(&self, uri: &str) -> Result<Box<dyn Torrent>>;
            async fn info(&self, handle: &TorrentHandle) -> Result<TorrentInfo>;
            async fn download(&self, handle: &TorrentHandle, filename: &str) -> Result<()>;
            async fn find_by_handle(&self, handle: &TorrentHandle) -> Option<Box<dyn Torrent>>;
            async fn remove(&self, handle: &TorrentHandle);
            fn calculate_health(&self, seeds: u32, leechers: u32) -> TorrentHealth;
            async fn cleanup(&self);
        }

        impl Callback<TorrentManagerEvent> for TorrentManager {
            fn subscribe(&self) -> Subscription<TorrentManagerEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<TorrentManagerEvent>);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::config::PopcornSettings;
    use crate::testing::copy_test_file;
    use crate::{assert_timeout, init_logger};

    use std::fs::{File, FileTimes};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_torrent_manager_cleanup() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Off);
        let manager = FxTorrentManager::new(settings, EventPublisher::default())
            .await
            .unwrap();

        // copy some contents into the torrent working dir
        let filepath = copy_test_file(temp_path, "simple.txt", Some("torrents/debian.torrent"));
        // start the cleanup process
        manager.cleanup().await;

        let path_buf = PathBuf::from(filepath);
        assert_eq!(
            false,
            path_buf.exists(),
            "expected the file to have been removed"
        );
    }

    #[tokio::test]
    async fn test_torrent_manager_drop_cleaning_disabled() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Off);
        let manager = FxTorrentManager::new(settings, EventPublisher::default())
            .await
            .unwrap();

        // copy some contents into the torrent working dir
        let filepath = copy_test_file(temp_path, "simple.txt", Some("torrents/debian.torrent"));
        // trigger the automatic cleaning process
        drop(manager);

        assert_eq!(true, PathBuf::from(filepath).exists())
    }

    #[tokio::test]
    async fn test_torrent_manager_drop_cleaning_mode_set_to_on_shutdown() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::OnShutdown);
        let manager = FxTorrentManager::new(settings.clone(), EventPublisher::default())
            .await
            .unwrap();

        // copy some contents into the torrent working dir
        let _filepath = copy_test_file(temp_path, "simple.txt", Some("torrents/debian.torrent"));
        // trigger the automatic cleaning process
        drop(manager);

        let result = settings
            .user_settings_ref(|e| e.torrent_settings.directory.clone())
            .await;
        assert_timeout!(
            Duration::from_millis(200),
            result.read_dir().unwrap().next().is_none(),
            "Expected the directory to be empty"
        );
    }

    #[tokio::test]
    async fn test_torrent_manager_drop_cleaning_mode_set_to_watched() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = default_config(temp_path, CleaningMode::Watched);
        let _ = copy_test_file(
            temp_path,
            "simple.txt",
            Some("torrents/my-torrent/debian.torrent"),
        );
        let manager = FxTorrentManager::new(settings.clone(), EventPublisher::default())
            .await
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

        let result = settings
            .user_settings_ref(|e| e.torrent_settings.directory.clone())
            .await;
        assert_timeout!(
            Duration::from_millis(200),
            result.read_dir().unwrap().next().is_none(),
            "Expected the directory to be empty"
        );
    }

    fn default_config(temp_path: &str, cleaning_mode: CleaningMode) -> ApplicationConfig {
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
            .build()
    }
}
