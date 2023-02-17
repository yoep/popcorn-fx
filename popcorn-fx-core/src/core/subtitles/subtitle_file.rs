use std::cmp::Ordering;

use derive_more::Display;
use log::trace;
use regex::Regex;

const QUALITY_PATTERN: &str = "([0-9]{3,4})p";

/// An available subtitle file which can be fetched from the [crate::core::subtitles::SubtitleProvider].
/// It describes all available metadata of the subtitle which can be used to make
/// a decision of which subtitle file should be used for a media item playback.
#[derive(Debug, Clone, PartialEq, Display)]
#[display(fmt = "name: {}, url: {}, quality: {:?}, downloads: {}", name, url, quality, downloads)]
pub struct SubtitleFile {
    file_id: i32,
    name: String,
    url: String,
    score: f32,
    downloads: i32,
    quality: Option<i32>,
}

impl SubtitleFile {
    /// Create a new subtitle file instance.
    /// The quality is automatically parsed from the `name`.
    pub fn new(file_id: i32, name: String, url: String, score: f32, downloads: i32) -> Self {
        let quality = Self::try_parse_subtitle_quality(&name);
        trace!("Parsed subtitle quality {:?} from \"{}\"", &quality, &name);

        Self {
            file_id,
            name,
            url,
            score,
            downloads,
            quality,
        }
    }

    /// Create a new subtitle file instance with the given quality.
    pub fn new_with_quality(file_id: i32, name: String, url: String, score: f32, downloads: i32, quality: Option<i32>) -> Self {
        Self {
            file_id,
            name,
            url,
            score,
            downloads,
            quality,
        }
    }

    pub fn file_id(&self) -> &i32 {
        &self.file_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn score(&self) -> &f32 {
        &self.score
    }

    pub fn downloads(&self) -> &i32 {
        &self.downloads
    }

    pub fn quality(&self) -> Option<&i32> {
        match &self.quality {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Try to parse the quality for the subtitle file based on the filename.
    fn try_parse_subtitle_quality(name: &str) -> Option<i32> {
        let regex = Regex::new(QUALITY_PATTERN).unwrap();
        regex.captures(name)
            .map(|e| e.get(1).unwrap())
            .map(|e| String::from(e.as_str()))
            .map(|e| e.parse::<i32>().unwrap())
    }

    /// Order based on the quality of the subtitle file.
    /// Files with qualities will be [Ordering::Less] in regards to files without known quality.
    fn quality_cmp(&self, other: &Self) -> Ordering {
        if self.quality.is_some() && other.quality.is_none() {
            return Ordering::Less;
        } else if self.quality.is_none() && other.quality.is_some() {
            return Ordering::Greater;
        }

        Ordering::Equal
    }

    /// Order based on the total download of the subtitle file.
    /// If the total download is higher, it will be returned as first in the array.
    fn download_cmp(&self, other: &Self) -> Ordering {
        self.downloads.cmp(other.downloads()).reverse()
    }
}

impl Eq for SubtitleFile {}

impl PartialOrd<Self> for SubtitleFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let download_cmp = self.download_cmp(other);
        if download_cmp != Ordering::Equal {
            return Some(download_cmp);
        }

        let quality_cmp = self.quality_cmp(other);
        if quality_cmp != Ordering::Equal {
            return Some(quality_cmp);
        }

        self.score.partial_cmp(other.score())
            .map(|e| e.reverse())
    }
}

impl Ord for SubtitleFile {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            None => Ordering::Equal,
            Some(e) => e
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::core::subtitles::SubtitleFile;

    #[test]
    fn test_ordering_quality() {
        let file1 = SubtitleFile {
            file_id: 1,
            name: "lorem".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 0,
            quality: None,
        };
        let file2 = SubtitleFile {
            file_id: 2,
            name: "ipsum".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 0,
            quality: Some(1080),
        };
        let file3 = SubtitleFile {
            file_id: 3,
            name: "dolor".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 0,
            quality: Some(1080),
        };

        assert_eq!(Ordering::Greater, file1.cmp(&file2));
        assert_eq!(Ordering::Less, file2.cmp(&file1));
        assert_eq!(Ordering::Equal, file2.cmp(&file3))
    }

    #[test]
    fn test_ordering_downloads() {
        let file1 = SubtitleFile {
            file_id: 1,
            name: "lorem".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 10,
            quality: None,
        };
        let file2 = SubtitleFile {
            file_id: 2,
            name: "ipsum".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 100,
            quality: None,
        };

        let file3 = SubtitleFile {
            file_id: 3,
            name: "dolor".to_string(),
            url: "".to_string(),
            score: 0.0,
            downloads: 100,
            quality: None,
        };

        assert_eq!(Ordering::Greater, file1.cmp(&file2));
        assert_eq!(Ordering::Less, file2.cmp(&file1));
        assert_eq!(Ordering::Equal, file2.cmp(&file3))
    }

    #[test]
    fn test_ordering_score() {
        let file1 = SubtitleFile {
            file_id: 1,
            name: "lorem".to_string(),
            url: "".to_string(),
            score: 8.0,
            downloads: 0,
            quality: None,
        };
        let file2 = SubtitleFile {
            file_id: 2,
            name: "ipsum".to_string(),
            url: "".to_string(),
            score: 5.0,
            downloads: 0,
            quality: None,
        };

        let file3 = SubtitleFile {
            file_id: 3,
            name: "dolor".to_string(),
            url: "".to_string(),
            score: 5.0,
            downloads: 0,
            quality: None,
        };

        assert_eq!(Ordering::Less, file1.cmp(&file2));
        assert_eq!(Ordering::Greater, file2.cmp(&file1));
        assert_eq!(Ordering::Equal, file2.cmp(&file3))
    }
}