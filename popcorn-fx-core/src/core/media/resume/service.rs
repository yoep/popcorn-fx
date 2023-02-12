use std::fmt::Debug;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use mockall::automock;
use tokio::sync::Mutex;

use crate::core::media;
use crate::core::media::MediaError;
use crate::core::media::resume::AutoResume;
use crate::core::storage::{Storage, StorageError};

const FILENAME: &str = "auto-resume.json";

#[automock]
pub trait AutoResumeService: Debug + Send + Sync {
    /// Retrieve the resume timestamp for the given media id and/or filename.
    ///
    /// It retrieves the timestamp when found, else [None].
    fn resume_timestamp<'a>(&self, id: Option<&'a str>, filename: Option<&'a str>) -> Option<u64>;
}

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
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::testing::{copy_test_file, init_logger};

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
}