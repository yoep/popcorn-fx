use derive_new::new;
use serde::{Deserialize, Serialize};

/// The response model of opensubtitles.com
/// This is a pagination response of `json` data with a generic `T` type as data.
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenSubtitlesResponse<T> {
    /// The total pages available for the query
    pub total_pages: i32,
    /// The total items available for the query
    pub total_count: i32,
    /// The current page index of the query
    pub page: i32,
    pub data: Vec<T>,
}

impl<T> OpenSubtitlesResponse<T> {
    pub fn total_pages(&self) -> &i32 {
        &self.total_pages
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    id: String,
    #[serde(alias = "type")]
    data_type: String,
    attributes: OpenSubtitlesAttributes,
}

impl SearchResult {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn attributes(&self) -> &OpenSubtitlesAttributes {
        &self.attributes
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenSubtitlesAttributes {
    pub subtitle_id: String,
    pub language: Option<String>,
    pub download_count: i32,
    pub new_download_count: i32,
    pub hearing_impaired: bool,
    pub hd: bool,
    pub fps: f32,
    pub votes: i32,
    pub points: Option<i32>,
    pub ratings: f32,
    pub from_trusted: Option<bool>,
    pub foreign_parts_only: bool,
    pub ai_translated: bool,
    pub machine_translated: bool,
    pub upload_date: String,
    pub release: String,
    pub url: String,
    pub files: Vec<OpenSubtitlesFile>,
    pub feature_details: OpenSubtitlesFeatureDetails,
}

impl OpenSubtitlesAttributes {
    pub fn new(subtitle_id: String, release: String) -> Self {
        Self {
            subtitle_id,
            language: None,
            download_count: 0,
            new_download_count: 0,
            hearing_impaired: false,
            hd: false,
            fps: 32.0,
            votes: 0,
            points: None,
            ratings: 0.0,
            from_trusted: None,
            foreign_parts_only: false,
            ai_translated: false,
            machine_translated: false,
            upload_date: "".to_string(),
            release,
            url: "".to_string(),
            files: vec![],
            feature_details: OpenSubtitlesFeatureDetails::new(
                -1,
                "Movie".to_string(),
                1970,
                "".to_string(),
                -1,
            ),
        }
    }

    pub fn language(&self) -> Option<&String> {
        match &self.language {
            None => None,
            Some(e) => Some(e),
        }
    }

    pub fn download_count(&self) -> &i32 {
        &self.download_count
    }

    pub fn ratings(&self) -> &f32 {
        &self.ratings
    }

    pub fn release(&self) -> &String {
        &self.release
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn files(&self) -> &Vec<OpenSubtitlesFile> {
        &self.files
    }

    pub fn feature_details(&self) -> &OpenSubtitlesFeatureDetails {
        &self.feature_details
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenSubtitlesFile {
    pub file_id: i32,
    pub cd_number: Option<i32>,
    pub file_name: Option<String>,
}

impl OpenSubtitlesFile {
    pub fn new_with_filename(file_id: i32, file_name: String) -> Self {
        Self {
            file_id,
            cd_number: None,
            file_name: Some(file_name),
        }
    }

    pub fn file_id(&self) -> &i32 {
        &self.file_id
    }

    pub fn file_name(&self) -> Option<&String> {
        match &self.file_name {
            Some(e) => Some(&e),
            None => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenSubtitlesFeatureDetails {
    feature_id: i32,
    feature_type: String,
    year: Option<i32>,
    title: String,
    imdb_id: i32,
}

impl OpenSubtitlesFeatureDetails {
    pub fn new(
        feature_id: i32,
        feature_type: String,
        year: i32,
        title: String,
        imdb_id: i32,
    ) -> Self {
        Self {
            feature_id,
            feature_type,
            year: Some(year),
            title,
            imdb_id,
        }
    }

    pub fn imdb_id(&self) -> &i32 {
        &self.imdb_id
    }
}

#[derive(Serialize, Deserialize, Debug, new)]
pub struct DownloadRequest {
    file_id: i32,
}

#[derive(Serialize, Deserialize, Debug, new)]
pub struct DownloadResponse {
    link: String,
    file_name: String,
    requests: i32,
    remaining: i32,
    message: String,
}

impl DownloadResponse {
    pub fn link(&self) -> &String {
        &self.link
    }
}
