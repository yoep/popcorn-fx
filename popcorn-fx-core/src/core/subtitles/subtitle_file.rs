use std::cmp::Ordering;

use derive_more::Display;
use log::trace;
use regex::Regex;

const QUALITY_PATTERN: &str = "([0-9]{3,4})p";

/// An available subtitle file which can be fetched from the [crate::core::subtitles::SubtitleProvider].
/// It describes all available metadata of the subtitle which can be used to make
/// a decision of which subtitle file should be used for a media item playback.
#[derive(Debug, Clone, PartialEq, Display)]
#[display(
    fmt = "name: {}, url: {}, quality: {:?}, downloads: {}",
    name,
    url,
    quality,
    downloads
)]
pub struct SubtitleFile {
    /// The ID of the subtitle file.
    file_id: i32,
    /// The name of the subtitle file.
    name: String,
    /// The URL of the subtitle file.
    url: String,
    /// The score of the subtitle file.
    score: f32,
    /// The number of downloads for the subtitle file.
    downloads: i32,
    /// The quality of the subtitle file, if known.
    quality: Option<i32>,
}

impl SubtitleFile {
    /// Creates a new instance of `SubtitleFileBuilder` for building `SubtitleFile` instances.
    ///
    /// # Returns
    ///
    /// A new `SubtitleFileBuilder` instance.
    pub fn builder() -> SubtitleFileBuilder {
        SubtitleFileBuilder::builder()
    }

    /// Gets the ID of the subtitle file.
    ///
    /// # Returns
    ///
    /// A reference to the ID of the subtitle file.
    pub fn file_id(&self) -> &i32 {
        &self.file_id
    }

    /// Gets the name of the subtitle file.
    ///
    /// # Returns
    ///
    /// A reference to the name of the subtitle file.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the URL of the subtitle file.
    ///
    /// # Returns
    ///
    /// A reference to the URL of the subtitle file.
    pub fn url(&self) -> &String {
        &self.url
    }

    /// Gets the score of the subtitle file.
    ///
    /// # Returns
    ///
    /// A reference to the score of the subtitle file.
    pub fn score(&self) -> &f32 {
        &self.score
    }

    /// Gets the number of downloads for the subtitle file.
    ///
    /// # Returns
    ///
    /// A reference to the number of downloads for the subtitle file.
    pub fn downloads(&self) -> &i32 {
        &self.downloads
    }

    /// Gets the quality of the subtitle file, if known.
    ///
    /// # Returns
    ///
    /// An option containing a reference to the quality of the subtitle file, if known.
    pub fn quality(&self) -> Option<&i32> {
        self.quality.as_ref()
    }

    /// Tries to parse the quality for the subtitle file based on the filename.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subtitle file.
    ///
    /// # Returns
    ///
    /// An option containing the parsed quality of the subtitle file, if successful.
    fn try_parse_subtitle_quality(name: &str) -> Option<i32> {
        let regex = Regex::new(QUALITY_PATTERN).unwrap();
        regex
            .captures(name)
            .map(|e| e.get(1).unwrap())
            .map(|e| String::from(e.as_str()))
            .map(|e| e.parse::<i32>().unwrap())
    }

    /// Orders based on the quality of the subtitle file.
    ///
    /// Files with qualities will be `Ordering::Less` in regards to files without known quality.
    ///
    /// # Arguments
    ///
    /// * `other` - The other `SubtitleFile` instance to compare with.
    ///
    /// # Returns
    ///
    /// The ordering based on the quality of the subtitle file.
    fn quality_cmp(&self, other: &Self) -> Ordering {
        if self.quality.is_some() && other.quality.is_none() {
            return Ordering::Less;
        } else if self.quality.is_none() && other.quality.is_some() {
            return Ordering::Greater;
        }

        Ordering::Equal
    }

    /// Orders based on the total download of the subtitle file.
    ///
    /// If the total download is higher, it will be returned as first in the array.
    ///
    /// # Arguments
    ///
    /// * `other` - The other `SubtitleFile` instance to compare with.
    ///
    /// # Returns
    ///
    /// The ordering based on the total download of the subtitle file.
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

        self.score.partial_cmp(other.score()).map(|e| e.reverse())
    }
}

impl Ord for SubtitleFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or_else(|| Ordering::Equal)
    }
}

/// Builder for creating `SubtitleFile` instances.
///
/// This builder allows creating instances of `SubtitleFile` with optional fields.
/// Fields are set using the provided setter methods, and the `build` method is used to create the `SubtitleFile`.
#[derive(Debug, Default)]
pub struct SubtitleFileBuilder {
    file_id: Option<i32>,
    name: Option<String>,
    url: Option<String>,
    score: Option<f32>,
    downloads: Option<i32>,
    quality: Option<i32>,
}

impl SubtitleFileBuilder {
    /// Creates a new `SubtitleFileBuilder` instance.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the file ID for the subtitle file.
    pub fn file_id(mut self, file_id: i32) -> Self {
        self.file_id = Some(file_id);
        self
    }

    /// Sets the name of the subtitle file.
    pub fn name<T: ToString>(mut self, name: T) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the URL of the subtitle file.
    pub fn url<T: ToString>(mut self, url: T) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the score of the subtitle file.
    pub fn score(mut self, score: f32) -> Self {
        self.score = Some(score);
        self
    }

    /// Sets the number of downloads for the subtitle file.
    pub fn downloads(mut self, downloads: i32) -> Self {
        self.downloads = Some(downloads);
        self
    }

    /// Sets the quality of the subtitle file.
    pub fn quality(mut self, quality: i32) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Builds the `SubtitleFile` struct.
    ///
    /// # Panics
    ///
    /// This method will panic if any required field is not set.
    pub fn build(self) -> SubtitleFile {
        trace!("Building SubtitleFile from {:?}", self);
        let name = self.name.expect("name is not set");
        let quality = self
            .quality
            .or_else(|| SubtitleFile::try_parse_subtitle_quality(name.as_str()));

        SubtitleFile {
            file_id: self.file_id.expect("file_id is not set"),
            name,
            url: self.url.expect("url is not set"),
            score: self.score.expect("score is not set"),
            downloads: self.downloads.expect("downloads is not set"),
            quality,
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::core::subtitles::SubtitleFile;
    use crate::testing::init_logger;

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

    #[test]
    fn test_build_no_quality_set() {
        init_logger();
        let name = "MyFilename.720p.srt";

        let result = SubtitleFile::builder()
            .file_id(1001)
            .name(name)
            .url("")
            .score(0.0)
            .downloads(200)
            .build();

        assert_eq!(Some(720), result.quality);
    }
}
