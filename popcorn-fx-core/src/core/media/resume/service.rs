use crate::core::event::{
    Event, EventCallback, EventHandler, EventPublisher, PlayerStoppedEvent, HIGHEST_ORDER,
};
use crate::core::media;
use crate::core::media::resume::AutoResume;
use crate::core::media::MediaError;
use crate::core::storage::{Storage, StorageError};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const FILENAME: &str = "auto-resume.json";
/// The minimum duration a video playback should have
const VIDEO_DURATION_THRESHOLD: u64 = 5 * 60 * 1000;
/// The percentage of the video that should have been watched
/// to be assumed as "viewed"
const RESUME_PERCENTAGE_THRESHOLD: u32 = 85;

/// The auto-resume service which handles the resume timestamp for video playbacks.
/// It stores the last known timestamp when needed based on a player stopped event.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait AutoResumeService: Debug + Send + Sync {
    /// Retrieve the resume timestamp for the given media id and/or filename.
    ///
    /// It retrieves the timestamp when found, else [None].
    async fn resume_timestamp(&self, id: Option<String>, filename: Option<String>) -> Option<u64>;

    /// Handle a player stopped event.
    /// The event should contain the information of the player before it stopped.
    ///
    /// When a video playback wasn't finished, it will be stored for later use.
    async fn player_stopped(&self, event: &PlayerStoppedEvent);
}

/// The default auto-resume service for Popcorn FX.
#[derive(Debug)]
pub struct DefaultAutoResumeService {
    inner: Arc<InnerAutoResumeService>,
}

impl DefaultAutoResumeService {
    pub fn builder() -> DefaultAutoResumeServiceBuilder {
        DefaultAutoResumeServiceBuilder::default()
    }
}

#[async_trait]
impl AutoResumeService for DefaultAutoResumeService {
    async fn resume_timestamp(&self, id: Option<String>, filename: Option<String>) -> Option<u64> {
        self.inner.resume_timestamp(id, filename).await
    }

    async fn player_stopped(&self, event: &PlayerStoppedEvent) {
        self.inner.player_stopped(event).await
    }
}

/// A builder for `DefaultAutoResumeService` which allows saving auto-resume timestamps of video playbacks.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use popcorn_fx_core::core::event::EventPublisher;
/// use popcorn_fx_core::core::media::resume::DefaultAutoResumeService;
///
/// let auto_resume_service = DefaultAutoResumeService::builder()
///     .storage_directory("my-storage-directory")
///     .event_publisher(EventPublisher::default())
///     .build();
/// ```
#[derive(Default)]
pub struct DefaultAutoResumeServiceBuilder {
    storage_directory: Option<String>,
    event_publisher: Option<EventPublisher>,
}

impl DefaultAutoResumeServiceBuilder {
    /// Sets the `storage_directory` field for the `DefaultAutoResumeService`.
    ///
    /// # Panics
    ///
    /// Panics if the `storage_directory` is not set when `build()` is called.
    pub fn storage_directory(mut self, storage_directory: &str) -> Self {
        self.storage_directory = Some(storage_directory.to_string());
        self
    }

    /// Sets the `event_publisher` field for the `DefaultAutoResumeService`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use popcorn_fx_core::core::event::EventPublisher;
    /// use popcorn_fx_core::core::media::resume::DefaultAutoResumeService;
    ///
    /// let auto_resume_service = DefaultAutoResumeService::builder()
    ///     .storage_directory("my-storage-directory")
    ///     .event_publisher(EventPublisher::default())
    ///     .build();
    /// ```
    pub fn event_publisher(mut self, event_publisher: EventPublisher) -> Self {
        self.event_publisher = Some(event_publisher);
        self
    }

    /// Builds a new `DefaultAutoResumeService`.
    ///
    /// # Panics
    ///
    /// Panics if the `storage_directory` is not set.
    pub fn build(self) -> DefaultAutoResumeService {
        let instance = DefaultAutoResumeService {
            inner: Arc::new(InnerAutoResumeService {
                storage: Mutex::new(Storage::from(
                    self.storage_directory
                        .expect("Storage directory not set")
                        .as_str(),
                )),
                cache: Mutex::new(None),
                cancellation_token: Default::default(),
            }),
        };

        if let Some(event_publisher) = self.event_publisher {
            let inner = instance.inner.clone();
            let callback = event_publisher
                .subscribe(HIGHEST_ORDER + 10)
                .expect("expected to receive a callback");
            tokio::spawn(async move {
                inner.start(callback).await;
            });
        } else {
            warn!("No EventPublisher configured for DefaultAutoResumeService, unable to automatically detect PlayerStopped events");
        }

        instance
    }
}

#[derive(Debug)]
struct InnerAutoResumeService {
    storage: Mutex<Storage>,
    cache: Mutex<Option<AutoResume>>,
    cancellation_token: CancellationToken,
}

impl InnerAutoResumeService {
    async fn start(&self, mut event_receiver: EventCallback) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(handler) = event_receiver.recv() => self.handle_event(handler).await,
            }
        }

        match self.cache.lock().await.as_ref() {
            None => {}
            Some(e) => self.save_async(e).await,
        }
        debug!("Auto-resume service main loop ended");
    }

    async fn handle_event(&self, mut handler: EventHandler) {
        if let Some(Event::PlayerStopped(player_stopped)) = handler.event_ref() {
            self.player_stopped(player_stopped).await;
        }
        handler.next();
    }

    async fn load_resume_cache(&self) -> media::Result<()> {
        let mut cache = self.cache.lock().await;

        if cache.is_none() {
            trace!("Loading auto-resume cache");
            return match self.load_resume_from_storage().await {
                Ok(e) => {
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e),
            };
        }

        trace!("Auto-resume cache already loaded, nothing to do");
        Ok(())
    }

    async fn load_resume_from_storage(&self) -> media::Result<AutoResume> {
        let mutex = self.storage.lock().await;
        match mutex.options().serializer(FILENAME).read() {
            Ok(e) => Ok(e),
            Err(e) => match e {
                StorageError::NotFound(file) => {
                    debug!("Creating new auto-resume file {}", file);
                    Ok(AutoResume::default())
                }
                StorageError::ReadingFailed(_, error) => {
                    error!("Failed to load auto-resume, {}", error);
                    Err(MediaError::AutoResumeLoadingFailed(error))
                }
                _ => {
                    warn!("Unexpected error returned from storage, {}", e);
                    Ok(AutoResume::default())
                }
            },
        }
    }

    async fn save_async(&self, resume: &AutoResume) {
        let mutex = self.storage.lock().await;
        match mutex
            .options()
            .serializer(FILENAME)
            .write_async(resume)
            .await
        {
            Ok(_) => info!("Auto-resume data has been saved"),
            Err(e) => error!("Failed to save auto-resume, {}", e),
        }
    }

    async fn resume_timestamp(&self, id: Option<String>, filename: Option<String>) -> Option<u64> {
        match self.load_resume_cache().await {
            Ok(_) => {
                debug!(
                    "Retrieving auto-resume info for id: {:?}, filename: {:?}",
                    &id, &filename
                );
                let mutex = self.cache.lock().await;
                let cache = mutex.as_ref().expect("expected the auto-resume cache");

                // always search first on the filename as it might be more correct
                // than the id which might have been watched on a different quality
                if let Some(filename) = filename {
                    trace!(
                        "Searching for auto resume timestamp with filename {}",
                        filename
                    );
                    match cache.find_filename(filename.as_str()) {
                        None => {}
                        Some(e) => {
                            info!(
                                "Found resume timestamp {} for {}",
                                e.last_known_timestamp(),
                                filename
                            );
                            return Some(*e.last_known_timestamp());
                        }
                    }
                }

                if let Some(id) = id {
                    trace!("Searching for auto resume timestamp with id {}", id);
                    match cache.find_id(id.as_str()) {
                        None => {}
                        Some(e) => {
                            info!(
                                "Found resume timestamp {} for {}",
                                e.last_known_timestamp(),
                                id
                            );
                            return Some(*e.last_known_timestamp());
                        }
                    }
                }

                None
            }
            Err(e) => {
                error!("Failed to retrieve auto-resume info, {}", e);
                None
            }
        }
    }

    async fn player_stopped(&self, event: &PlayerStoppedEvent) {
        trace!("Received player stop event {:?}", event);
        if let (Some(time), Some(duration)) = (event.time(), event.duration()) {
            if duration < &VIDEO_DURATION_THRESHOLD {
                debug!(
                    "Video playback {} is shorter than {} millis",
                    event.url(),
                    VIDEO_DURATION_THRESHOLD
                );
                return;
            }

            match self.load_resume_cache().await {
                Ok(_) => {
                    let mut mutex = self.cache.lock().await;
                    let cache = mutex.as_mut().expect("expected the cache to be available");
                    let percentage_watched = ((*time as f64 / *duration as f64) * 100f64) as u32;
                    let path = PathBuf::from(event.url());
                    let filename = path
                        .file_name()
                        .expect("expected a valid filename")
                        .to_str()
                        .unwrap();

                    trace!(
                        "Video playback {} has been played for {}%",
                        event.url(),
                        percentage_watched
                    );
                    if percentage_watched < RESUME_PERCENTAGE_THRESHOLD {
                        let id = event.media().map(|e| e.imdb_id());
                        debug!(
                            "Adding auto resume timestamp {} for id: {:?}, filename: {}",
                            time, id, filename
                        );
                        cache.insert(id, filename, time.clone());
                    } else {
                        let id = event.media().map(|e| e.imdb_id());

                        debug!(
                            "Removing auto resume timestamp for id: {:?}, filename: {}",
                            id, filename
                        );
                        cache.remove(id, filename);
                    }

                    self.save_async(cache).await;
                }
                Err(e) => {
                    error!("Failed to store the resume timestamp, {}", e)
                }
            }
        } else {
            debug!("Unable to determine auto-resume state, missing time and/or duration data")
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::media::{MediaIdentifier, MovieOverview};
    use crate::init_logger;
    use crate::testing::{copy_test_file, read_temp_dir_file_as_string};

    use super::*;

    #[tokio::test]
    async fn test_resume_timestamp_filename() {
        init_logger!();
        let filename = "Lorem.mp4";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        copy_test_file(temp_path, "auto-resume.json", None);

        let result = service
            .resume_timestamp(None, Some(filename.to_string()))
            .await;

        match result {
            Some(e) => assert_eq!(19826, e),
            None => assert!(false, "expected the timestamp to have been found"),
        }
    }

    #[tokio::test]
    async fn test_resume_timestamp_filename_not_found() {
        init_logger!();
        let filename = "random-video-not-known.mkv";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();

        let result = service
            .resume_timestamp(None, Some(filename.to_string()))
            .await;

        assert_eq!(None, result)
    }

    #[tokio::test]
    async fn test_resume_timestamp_id() {
        init_logger!();
        let id = "110999";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        copy_test_file(temp_path, "auto-resume.json", None);

        let result = service.resume_timestamp(Some(id.to_string()), None).await;

        match result {
            Some(e) => assert_eq!(19826, e),
            None => assert!(false, "expected the timestamp to have been found"),
        }
    }

    #[tokio::test]
    async fn test_resume_timestamp_no_data_passed() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();

        let result = service.resume_timestamp(None, None).await;

        assert_eq!(None, result)
    }

    #[tokio::test]
    async fn test_player_stopped_ignore_playback_shorter_than_5_mins() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        let event = PlayerStoppedEvent {
            url: "http://localhost/ipsum.mp4".to_string(),
            media: None,
            time: Some(30000),
            duration: Some(120000),
        };

        service.player_stopped(&event).await;
        let result = service
            .resume_timestamp(None, Some("ipsum.mp4".to_string()))
            .await;

        assert_eq!(None, result)
    }

    #[tokio::test]
    async fn test_player_stopped_add_resume_data() {
        init_logger!();
        let id = "tt0000111";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        let expected_timestamp = 40000;
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            id.to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent {
            url: "http://localhost/lorem.mp4".to_string(),
            media: Some(movie),
            time: Some(expected_timestamp.clone()),
            duration: Some(350000),
        };

        service.player_stopped(&event).await;
        let result = service
            .resume_timestamp(Some(id.to_string()), None)
            .await
            .expect("expected a timestamp to be returned");

        assert_eq!(expected_timestamp, result)
    }

    #[tokio::test]
    async fn test_player_stopped_remove_resume_data() {
        init_logger!();
        let id = "tt0000111";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        copy_test_file(temp_path, "auto-resume.json", None);
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            "tt11223344".to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent {
            url: "http://localhost/already-started-watching.mkv".to_string(),
            media: Some(movie),
            time: Some(550000),
            duration: Some(600000),
        };

        service.player_stopped(&event).await;
        let result = service.resume_timestamp(Some(id.to_string()), None).await;

        assert_eq!(None, result)
    }

    #[tokio::test]
    async fn test_player_stopped_save_data() {
        init_logger!();
        let id = "tt00001212";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let service = DefaultAutoResumeService::builder()
            .storage_directory(temp_path)
            .build();
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            id.to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent {
            url: "http://localhost/already-started-watching.mkv".to_string(),
            media: Some(movie),
            time: Some(20000),
            duration: Some(600000),
        };
        let expected_result = "{\"video_timestamps\":[{\"id\":\"tt00001212\",\"filename\":\"already-started-watching.mkv\",\"last_known_time\":20000}]}";

        service.player_stopped(&event).await;
        let result = read_temp_dir_file_as_string(&temp_dir, FILENAME).replace("\r\n", "\n");

        assert_eq!(expected_result, result.as_str())
    }
}
