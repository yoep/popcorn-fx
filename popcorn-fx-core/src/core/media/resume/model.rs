use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

/// The auto-resume data structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoResume {
    /// The available/known resume timestamps
    pub video_timestamps: Vec<VideoTimestamp>,
}

impl AutoResume {
    /// Find the video timestamp by the given filename.
    ///
    /// It returns the video timestamp reference when found, else [None].
    pub fn find_filename(&self, filename: &str) -> Option<&VideoTimestamp> {
        self.video_timestamps
            .iter()
            .find(|e| Self::internal_find_by_filename(filename, e))
    }

    /// Find the video timestamp by the given media id.
    ///
    /// It returns the video timestamp reference when found, else [None].
    pub fn find_id(&self, id: &str) -> Option<&VideoTimestamp> {
        self.video_timestamps.iter().find(|e| {
            if let Some(video_id) = e.id() {
                return video_id.as_str() == id;
            }

            false
        })
    }

    /// Add or update a video `timestamp` within the resume data.
    /// The `timestamp` will be update if a record already exists,
    /// else a new one will be created.
    pub fn insert<'a>(&mut self, id: Option<&'a str>, filename: &'a str, timestamp: u64) {
        // check if the timestamp already exists
        // if so, we update the information of the existing one
        match self
            .video_timestamps
            .iter_mut()
            .find(|e| Self::internal_find_by_filename(filename, &&**e))
        {
            None => {
                trace!(
                    "Adding new video timestamp for id: {:?}, filename: {}",
                    id,
                    filename
                );
                self.video_timestamps.push(VideoTimestamp::new(
                    id.map(|e| e.to_string()),
                    filename,
                    timestamp,
                ));
            }
            Some(e) => {
                trace!(
                    "Updating existing video timestamp for id: {:?}, filename: {}",
                    id,
                    filename
                );
                e.last_known_time = timestamp;
            }
        }
    }

    /// Remove a possible known timestamp from the resume data.
    pub fn remove<'a>(&mut self, id: Option<&'a str>, filename: &'a str) {
        trace!(
            "Removing all timestamps matching id: {:?}, filename: {}",
            id,
            filename
        );
        let timestamps = &mut self.video_timestamps;
        let positions: Vec<usize> = timestamps
            .iter()
            .positions(|e| {
                if let (Some(remove_id), Some(this_id)) = (id, e.id()) {
                    return this_id.as_str() == remove_id;
                }

                e.filename() == filename
            })
            .rev()
            .collect();

        for position in positions {
            let timestamp = timestamps.remove(position);
            debug!("Removed video timestamp {}", timestamp)
        }
    }

    fn internal_find_by_filename(filename: &str, video_timestamp: &&VideoTimestamp) -> bool {
        video_timestamp.filename.eq_ignore_ascii_case(filename)
    }
}

#[derive(Debug, Display, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[display(
    fmt = "id: {:?}, filename: {}, last_known_time: {}",
    id,
    filename,
    last_known_time
)]
pub struct VideoTimestamp {
    id: Option<String>,
    filename: String,
    last_known_time: u64,
}

impl VideoTimestamp {
    pub fn new(id: Option<String>, filename: &str, last_known_time: u64) -> Self {
        Self {
            id,
            filename: filename.to_string(),
            last_known_time,
        }
    }

    /// The media id of the video
    pub fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }

    /// The filename of the video timestamp
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    /// The last known timestamp for this video.
    pub fn last_known_timestamp(&self) -> &u64 {
        &self.last_known_time
    }
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_find_filename() {
        let filename = "lorem-ipsum.mkv";
        let last_known_timestamp = 548885;
        let resume = AutoResume {
            video_timestamps: vec![VideoTimestamp::new(
                None,
                filename,
                last_known_timestamp.clone(),
            )],
        };

        let result = resume.find_filename(filename);

        match result {
            None => assert!(false, "expected the filename to have been found"),
            Some(e) => assert_eq!(last_known_timestamp, e.last_known_time),
        }
    }

    #[test]
    fn test_find_id() {
        let id = "tt875554";
        let last_known_timestamp = 877777;
        let resume = AutoResume {
            video_timestamps: vec![VideoTimestamp::new(
                Some(id.to_string()),
                "something.mp4",
                last_known_timestamp.clone(),
            )],
        };

        let result = resume.find_id(id);

        match result {
            None => assert!(false, "expected the id to have been found"),
            Some(e) => assert_eq!(last_known_timestamp, e.last_known_time),
        }
    }

    #[test]
    fn test_insert_non_existing() {
        init_logger();
        let filename = "my-file.mp4";
        let timestamp = 30000;
        let mut resume = AutoResume {
            video_timestamps: vec![],
        };

        resume.insert(Some("tt11111"), filename, timestamp.clone());
        let result = resume
            .find_filename(filename)
            .expect("expected video timestamp to be found");

        assert_eq!(timestamp, result.last_known_time)
    }

    #[test]
    fn test_insert_existing() {
        init_logger();
        let id = Some("tt11212");
        let filename = "lipsum-the-movie.mp4";
        let timestamp = 120000;
        let mut resume = AutoResume {
            video_timestamps: vec![VideoTimestamp::new(
                id.clone().map(|e| e.to_string()),
                filename,
                60000,
            )],
        };

        resume.insert(id, filename, timestamp.clone());
        let result = resume
            .find_filename(filename)
            .expect("expected video timestamp to be found");

        assert_eq!(timestamp, result.last_known_time)
    }

    #[test]
    fn test_remove_id() {
        let id = "tt000222";
        let remaining_video = VideoTimestamp::new(Some("tt000111".to_string()), "lorem.mp4", 60000);
        let mut resume = AutoResume {
            video_timestamps: vec![
                remaining_video.clone(),
                VideoTimestamp::new(Some(id.to_string()), "ipsum_720p.mp4", 60000),
                VideoTimestamp::new(Some(id.to_string()), "ipsum_1080p.mp4", 65000),
            ],
        };

        resume.remove(Some(id), "");
        let result = resume.video_timestamps;

        assert_eq!(vec![remaining_video], result)
    }

    #[test]
    fn test_remove_filename() {
        let id = "tt000222";
        let filename = "ipsum_720p.mp4";
        let remaining_timestamp =
            VideoTimestamp::new(Some(id.to_string()), "ipsum_1080p.mp4", 65000);
        let mut resume = AutoResume {
            video_timestamps: vec![
                VideoTimestamp::new(Some(id.to_string()), filename, 60000),
                remaining_timestamp.clone(),
            ],
        };

        resume.remove(None, filename);
        let result = resume.video_timestamps;

        assert_eq!(vec![remaining_timestamp], result)
    }
}
