use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenSubtitlesResponse<T> {
    total_pages: i32,
    total_count: i32,
    page: i32,
    data: Vec<T>,
}

impl<T> OpenSubtitlesResponse<T> {
    pub fn total_pages(&self) -> &i32 {
        &self.total_pages
    }

    pub fn total_count(&self) -> &i32 {
        &self.total_count
    }

    pub fn page(&self) -> &i32 {
        &self.page
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
    subtitle_id: String,
    language: Option<String>,
    download_count: i32,
    new_download_count: i32,
    hearing_impaired: bool,
    hd: bool,
    fps: f32,
    votes: i32,
    points: Option<i32>,
    ratings: f32,
    from_trusted: Option<bool>,
    foreign_parts_only: bool,
    ai_translated: bool,
    machine_translated: bool,
    upload_date: String,
    release: String,
    url: String,
    files: Vec<OpenSubtitlesFile>,
    feature_details: OpenSubtitlesFeatureDetails,
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
            Some(e) => Some(e)
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
    file_id: i32,
    cd_number: Option<i32>,
    file_name: Option<String>,
}

impl OpenSubtitlesFile {
    pub fn new(file_id: i32) -> Self {
        Self {
            file_id,
            cd_number: None,
            file_name: None,
        }
    }

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
            None => None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenSubtitlesFeatureDetails {
    feature_id: i32,
    feature_type: String,
    year: i32,
    title: String,
    imdb_id: i32,
}

impl OpenSubtitlesFeatureDetails {
    pub fn new(feature_id: i32, feature_type: String, year: i32, title: String, imdb_id: i32) -> Self {
        Self {
            feature_id,
            feature_type,
            year,
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