use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::{block_in_place, media};
use crate::core::events::PlayerStoppedEvent;
use crate::core::media::MediaError;
use crate::core::media::resume::AutoResume;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "auto-resume.json";
/// The minimum duration a video playback should have
const VIDEO_DURATION_THRESHOLD: u64 = 5 * 60 * 1000;
/// The percentage of the video that should heva been watched
/// to be assumed as "viewed"
const RESUME_PERCENTAGE_THRESHOLD: u32 = 85;

/// The auto-resume service which handles the resume timestamp for video playbacks.
/// It stores the last known timestamp when needed based on a player stopped event.
#[automock]
pub trait AutoResumeService: Debug + Send + Sync {
    /// Retrieve the resume timestamp for the given media id and/or filename.
    ///
    /// It retrieves the timestamp when found, else [None].
    fn resume_timestamp<'a>(&self, id: Option<&'a str>, filename: Option<&'a str>) -> Option<u64>;

    /// Handle a player stopped event.
    /// The event should contain the information of the player before it stopped.
    ///
    /// When a video playback wasn't finished, it will be stored for later use.
    fn player_stopped(&self, event: &PlayerStoppedEvent);
}

/// The default auto-resume service for Popcorn FX.
#[derive(Debug)]
pub struct DefaultAutoResumeService {
    storage: Arc<Storage>,
    cache: Arc<Mutex<Option<AutoResume>>>,
}

impl DefaultAutoResumeService {
    pub fn new(storage: &Arc<Storage>) -> Self {
        Self {
            storage: storage.clone(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    async fn load_resume_cache(&self) -> media::Result<()> {
        let mutex = self.cache.clone();
        let mut cache = mutex.lock().await;

        if cache.is_none() {
            trace!("Loading auto-resume cache");
            return match self.load_resume_from_storage() {
                Ok(e) => {
                    let _ = cache.insert(e);
                    Ok(())
                }
                Err(e) => Err(e)
            };
        }

        trace!("Auto-resume cache already loaded, nothing to do");
        Ok(())
    }

    fn load_resume_from_storage(&self) -> media::Result<AutoResume> {
        match self.storage.read::<AutoResume>(FILENAME) {
            Ok(e) => Ok(e),
            Err(e) => {
                match e {
                    StorageError::FileNotFound(file) => {
                        debug!("Creating new auto-resume file {}", file);
                        Ok(AutoResume::default())
                    }
                    StorageError::CorruptRead(_, error) => {
                        error!("Failed to load auto-resume, {}", error);
                        Err(MediaError::AutoResumeLoadingFailed(error))
                    }
                    _ => {
                        warn!("Unexpected error returned from storage, {}", e);
                        Ok(AutoResume::default())
                    }
                }
            }
        }
    }

    fn save(&self, resume: &AutoResume) {
        block_in_place(self.save_async(resume))
    }

    async fn save_async(&self, resume: &AutoResume) {
        match self.storage.write(FILENAME, &resume).await {
            Ok(_) => info!("Auto-resume data has been saved"),
            Err(e) => error!("Failed to save auto-resume, {}", e)
        }
    }
}

impl AutoResumeService for DefaultAutoResumeService {
    fn resume_timestamp<'a>(&self, id: Option<&'a str>, filename: Option<&'a str>) -> Option<u64> {
        match futures::executor::block_on(self.load_resume_cache()) {
            Ok(_) => {
                debug!("Retrieving auto-resume info for id: {:?}, filename: {:?}", &id, &filename);
                tokio::task::block_in_place(|| {
                    let mutex = self.cache.blocking_lock();
                    let cache = mutex.as_ref().expect("expected the auto-resume cache");

                    // always search first on the filename as it might be more correct
                    // than the id which might have been watched on a different quality
                    if let Some(filename) = filename {
                        match cache.find_filename(filename) {
                            None => {}
                            Some(e) => {
                                info!("Found resume timestamp {} for {}", e.last_known_timestamp(), filename);
                                return Some(*e.last_known_timestamp());
                            }
                        }
                    }

                    if let Some(id) = id {
                        match cache.find_id(id) {
                            None => {}
                            Some(e) => {
                                info!("Found resume timestamp {} for {}", e.last_known_timestamp(), id);
                                return Some(*e.last_known_timestamp());
                            }
                        }
                    }

                    None
                })
            }
            Err(e) => {
                error!("Failed to retrieve auto-resume info, {}", e);
                None
            }
        }
    }

    fn player_stopped(&self, event: &PlayerStoppedEvent) {
        trace!("Received player stop event {:?}", event);
        if let (Some(time), Some(duration)) = (event.time(), event.duration()) {
            if duration < &VIDEO_DURATION_THRESHOLD {
                debug!("Video playback {} is shorter than {} millis", event.url(), VIDEO_DURATION_THRESHOLD);
                return;
            }

            match futures::executor::block_on(self.load_resume_cache()) {
                Ok(_) => {
                    let mut mutex = self.cache.blocking_lock();
                    let cache = mutex.as_mut().expect("expected the cache to be available");
                    let percentage_watched = ((*time as f64 / *duration as f64) * 100f64) as u32;
                    let path = PathBuf::from(event.url());
                    let filename = path
                        .file_name()
                        .expect("expected a valid filename")
                        .to_str()
                        .unwrap();

                    trace!("Video playback {} has been played for {}%", event.url(), percentage_watched);
                    if percentage_watched < RESUME_PERCENTAGE_THRESHOLD {
                        cache.insert(event.media().map(|e| e.imdb_id()), filename, time.clone());
                    } else {
                        let id = event.media()
                            .map(|e| e.imdb_id());

                        cache.remove(id, filename);
                    }

                    self.save(cache)
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

impl Drop for DefaultAutoResumeService {
    fn drop(&mut self) {
        let mutex = self.cache.blocking_lock();
        let cache = mutex.as_ref();

        match cache {
            None => {}
            Some(e) => self.save(e)
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::media::{MediaIdentifier, MovieOverview};
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file};

    use super::*;

    #[test]
    fn test_resume_timestamp_filename() {
        init_logger();
        let filename = "Lorem.mp4";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        copy_test_file(temp_path, "auto-resume.json");

        let result = service.resume_timestamp(None, Some(filename));

        match result {
            Some(e) => assert_eq!(19826, e),
            None => assert!(false, "expected the timestamp to have been found")
        }
    }

    #[test]
    fn test_resume_timestamp_filename_not_found() {
        init_logger();
        let filename = "random-video-not-known.mkv";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);

        let result = service.resume_timestamp(None, Some(filename));

        assert_eq!(None, result)
    }

    #[test]
    fn test_resume_timestamp_id() {
        init_logger();
        let id = "110999";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        copy_test_file(temp_path, "auto-resume.json");

        let result = service.resume_timestamp(Some(id), None);

        match result {
            Some(e) => assert_eq!(19826, e),
            None => assert!(false, "expected the timestamp to have been found")
        }
    }

    #[test]
    fn test_resume_timestamp_no_data_passed() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);

        let result = service.resume_timestamp(None, None);

        assert_eq!(None, result)
    }

    #[test]
    fn test_player_stopped_ignore_playback_shorter_than_5_mins() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        let event = PlayerStoppedEvent::new(
            "http://localhost/ipsum.mp4".to_string(),
            None,
            Some(30000),
            Some(120000),
        );

        service.player_stopped(&event);
        let result = service.resume_timestamp(None, Some("ipsum.mp4"));

        assert_eq!(None, result)
    }

    #[test]
    fn test_player_stopped_add_resume_data() {
        init_logger();
        let id = "tt0000111";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        let expected_timestamp = 40000;
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            id.to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent::new(
            "http://localhost/lorem.mp4".to_string(),
            Some(movie),
            Some(expected_timestamp.clone()),
            Some(350000),
        );

        service.player_stopped(&event);
        let result = service.resume_timestamp(Some(id), None)
            .expect("expected a timestamp to be returned");

        assert_eq!(expected_timestamp, result)
    }

    #[test]
    fn test_player_stopped_remove_resume_data() {
        init_logger();
        let id = "tt0000111";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        copy_test_file(temp_path, "auto-resume.json");
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            "tt11223344".to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent::new(
            "http://localhost/already-started-watching.mkv".to_string(),
            Some(movie),
            Some(550000),
            Some(600000),
        );

        service.player_stopped(&event);
        let result = service.resume_timestamp(Some(id), None);

        assert_eq!(None, result)
    }

    #[test]
    fn test_player_stopped_save_data() {
        init_logger();
        let id = "tt00001212";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let storage = Arc::new(Storage::from_directory(temp_path));
        let service = DefaultAutoResumeService::new(&storage);
        let movie = Box::new(MovieOverview::new(
            "My video".to_string(),
            id.to_string(),
            "2022".to_string(),
        )) as Box<dyn MediaIdentifier>;
        let event = PlayerStoppedEvent::new(
            "http://localhost/already-started-watching.mkv".to_string(),
            Some(movie),
            Some(20000),
            Some(600000),
        );
        let expected_result = "{\"video_timestamps\":[{\"id\":\"tt00001212\",\"filename\":\"already-started-watching.mkv\",\"last_known_time\":20000}]}";

        service.player_stopped(&event);
        let result = read_temp_dir_file(temp_dir.into_path(), FILENAME)
            .replace("\r\n", "\n");

        assert_eq!(expected_result, result.as_str())
    }
}