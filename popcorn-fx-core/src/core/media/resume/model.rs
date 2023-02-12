use serde::{Deserialize, Serialize};

/// The auto-resume data structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoResume {
    video_timestamps: Vec<VideoTimestamp>,
}

impl AutoResume {
    /// Find the video timestamp by the given filename.
    ///
    /// It returns the video timestamp reference when found, else [None].
    pub fn find_filename(&self, filename: &str) -> Option<&VideoTimestamp> {
        self.video_timestamps.iter()
            .find(|e| e.filename.eq_ignore_ascii_case(filename))
    }

    /// Find the video timestamp by the given media id.
    ///
    /// It returns the video timestamp reference when found, else [None].
    pub fn find_id(&self, id: &str) -> Option<&VideoTimestamp> {
        self.video_timestamps.iter()
            .find(|e| {
                if let Some(video_id) = e.id() {
                    return video_id.as_str() == id;
                }

                false
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// The last known timestamp for this video.
    pub fn last_known_timestamp(&self) -> &u64 {
        &self.last_known_time
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_filename() {
        let filename = "lorem-ipsum.mkv";
        let last_known_timestamp = 548885;
        let resume = AutoResume {
            video_timestamps: vec![
                VideoTimestamp::new(None, filename, last_known_timestamp.clone())
            ]
        };

        let result = resume.find_filename(filename);

        match result {
            None => assert!(false, "expected the filename to have been found"),
            Some(e) => assert_eq!(last_known_timestamp, e.last_known_time)
        }
    }

    #[test]
    fn test_find_id() {
        let id = "tt875554";
        let last_known_timestamp = 877777;
        let resume = AutoResume {
            video_timestamps: vec![
                VideoTimestamp::new(Some(id.to_string()), "something.mp4", last_known_timestamp.clone())
            ]
        };

        let result = resume.find_id(id);

        match result {
            None => assert!(false, "expected the id to have been found"),
            Some(e) => assert_eq!(last_known_timestamp, e.last_known_time)
        }
    }
}