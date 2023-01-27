use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode, Url};
use reqwest::header::HeaderMap;

use popcorn_fx_core::core::config::Application;
use popcorn_fx_core::core::media::*;
use popcorn_fx_core::core::subtitles::{Result, SubtitleError, SubtitleFile, SubtitleProvider};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
use popcorn_fx_core::core::subtitles::parsers::{Parser, VttParser};
use popcorn_fx_core::core::subtitles::parsers::SrtParser;

use crate::opensubtitles::model::*;

const API_HEADER_KEY: &str = "Api-Key";
const USER_AGENT_HEADER_KEY: &str = "User-Agent";
const IMDB_ID_PARAM_KEY: &str = "imdb_id";
const SEASON_PARAM_KEY: &str = "season_number";
const EPISODE_PARAM_KEY: &str = "episode_number";
const FILENAME_PARAM_KEY: &str = "query";
const PAGE_PARAM_KEY: &str = "page";
const DEFAULT_FILENAME_EXTENSION: &str = ".srt";

pub struct OpensubtitlesProvider {
    settings: Arc<Application>,
    client: Client,
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
}

impl OpensubtitlesProvider {
    /// Create a new OpenSubtitles service instance.
    pub fn new(settings: &Arc<Application>) -> Self {
        let mut default_headers = HeaderMap::new();
        let srt_parser: Box<dyn Parser> = Box::new(SrtParser::new());
        let vtt_parser: Box<dyn Parser> = Box::new(VttParser::default());
        let api_token = settings.properties().subtitle().api_token();
        let user_agent = settings.properties().subtitle().user_agent();

        default_headers.insert(USER_AGENT_HEADER_KEY, user_agent.parse().unwrap());
        default_headers.insert(API_HEADER_KEY, api_token.parse().unwrap());

        Self {
            settings: settings.clone(),
            client: ClientBuilder::new()
                .default_headers(default_headers)
                .build()
                .unwrap(),
            parsers: HashMap::from([
                (SubtitleType::Srt, srt_parser),
                (SubtitleType::Vtt, vtt_parser)
            ]),
        }
    }

    fn create_search_url(&self, media_id: Option<&String>, episode: Option<&Episode>, filename: Option<&String>, page: i32) -> Result<Url> {
        let mut query_params: Vec<(&str, &str)> = vec![];
        let imdb_id: String;
        let season: String;
        let episode_number: String;
        let page_query_value = page.to_string();
        let url = format!("{}/subtitles", self.settings.properties().subtitle().url());

        // only set the page if it's not the first one
        // this is because the first one (with query param) isn't cached on cloudfront
        // so the chance of receiving a 503 is much higher
        if page > 1 {
            query_params.push((PAGE_PARAM_KEY, page_query_value.as_str()));
        }

        match media_id {
            Some(e) => {
                imdb_id = e.replace("tt", "");
                query_params.push((IMDB_ID_PARAM_KEY, &imdb_id));

                if episode.is_some() {
                    let episode_instance = episode.unwrap();
                    trace!("Extending search url for episode {:?}", episode_instance);
                    season = episode_instance.season().to_string();
                    episode_number = episode_instance.episode().to_string();

                    query_params.push((SEASON_PARAM_KEY, season.as_str()));
                    query_params.push((EPISODE_PARAM_KEY, episode_number.as_str()));
                }
            }
            None => {}
        }

        if filename.is_some() {
            query_params.push((FILENAME_PARAM_KEY, filename.unwrap()));
        }

        match Url::parse_with_params(url.as_str(), &query_params) {
            Ok(url) => Ok(url),
            Err(err) => Err(SubtitleError::InvalidUrl(format!("failed to parse url, {}", err.to_string())))
        }
    }

    fn create_download_url(&self) -> Result<Url> {
        let url = format!("{}/download", self.settings.properties().subtitle().url());

        match Url::parse(url.as_str()) {
            Ok(e) => Ok(e),
            Err(e) => Err(SubtitleError::InvalidUrl(format!("failed to parse url, {}", e.to_string())))
        }
    }

    fn search_result_to_subtitles(data: &Vec<SearchResult>) -> Vec<SubtitleInfo> {
        let mut id: String = String::new();
        let mut imdb_id: String = String::new();
        let mut languages: HashMap<SubtitleLanguage, Vec<SubtitleFile>> = HashMap::new();

        trace!("Mapping a total of {} subtitle search results", data.len());
        for search_result in data {
            let attributes = search_result.attributes();
            let optional_language: Option<SubtitleLanguage>;

            match attributes.language() {
                // skip this attribute as it's unusable
                None => continue,
                Some(e) => optional_language = SubtitleLanguage::from_code(e.clone())
            }

            if optional_language.is_some() {
                let language = optional_language.unwrap();

                if !languages.contains_key(&language) {
                    languages.insert(language.clone(), vec![]);
                }

                let language_files = languages
                    .get_mut(&language)
                    .unwrap();

                for file in attributes.files() {
                    language_files.push(SubtitleFile::new(
                        file.file_id().clone(),
                        Self::subtitle_file_name(file, attributes),
                        attributes.url().clone(),
                        attributes.ratings().clone(),
                        attributes.download_count().clone()));
                }

                if id.is_empty() {
                    id = search_result.id().clone();
                    imdb_id = format!("tt{}", attributes.feature_details().imdb_id());
                }
            } else {
                warn!("Unknown subtitle language detected: {}", attributes.language().unwrap())
            }
        }

        languages.iter()
            .map(|key| {
                let language = key.0;
                let files = key.1;

                SubtitleInfo::new_with_files(imdb_id.clone(), language.clone(), files.clone())
            })
            .collect()
    }

    async fn handle_search_result(id: &String, response: Response) -> Result<OpenSubtitlesResponse<SearchResult>> {
        match response.status() {
            StatusCode::OK => {
                trace!("Received response from OpenSubtitles for {}, decoding JSON...", id);
                match response.json::<OpenSubtitlesResponse<SearchResult>>().await {
                    Ok(e) => Ok(e),
                    Err(e) => Err(SubtitleError::SearchFailed(e.to_string()))
                }
            }
            _ => {
                let status = response.status();
                let body = &response.text().await.unwrap();
                error!("Received status {} for OpenSubtitles search with body {}", &status, body);

                Err(SubtitleError::SearchFailed(format!("received status code {}", &status)))
            }
        }
    }

    async fn start_search_request(&self, id: &String, media_id: Option<&String>, episode: Option<&Episode>, filename: Option<&String>) -> Result<Vec<SubtitleInfo>> {
        let mut search_data: Vec<SearchResult> = vec![];

        trace!("Fetching search result page 1");
        match self.fetch_search_page(id, media_id, episode, filename, 1).await {
            Err(e) => Err(e),
            Ok(response) => {
                let total_pages = response.total_pages();
                response.data().iter()
                    .for_each(|e| search_data.push(e.clone()));

                debug!("Fetching a total of {} search pages", total_pages);
                for fetch_page in 2..*total_pages {
                    trace!("Fetching search result page {}", fetch_page);
                    match self.fetch_search_page(id, media_id, episode, filename, fetch_page).await {
                        Err(e) => warn!("Failed to fetch search page {}, {}", fetch_page, e.to_string()),
                        Ok(page_response) => {
                            page_response.data().iter()
                                .for_each(|e| search_data.push(e.clone()));
                        }
                    }
                }

                let result = Self::search_result_to_subtitles(&search_data);
                debug!("Found a total of {} for IMDB ID {}, {:?}", result.len(), id, &result);
                Ok(result)
            }
        }
    }

    async fn fetch_search_page(&self, id: &String, media_id: Option<&String>, episode: Option<&Episode>, filename: Option<&String>, page: i32) -> Result<OpenSubtitlesResponse<SearchResult>> {
        let url = self.create_search_url(media_id, episode, filename, page)?;

        debug!("Retrieving available subtitles from {}", &url);
        match self.client.clone().get(url)
            .send()
            .await {
            Err(err) => Err(SubtitleError::SearchFailed(format!("OpenSubtitles request failed, {}", err))),
            Ok(response) => Self::handle_search_result(id, response).await
        }
    }

    async fn execute_download_request(&self, file_id: &i32, path: &Path, subtitle_info: &SubtitleInfo, download_response: DownloadResponse) -> Result<Subtitle> {
        let download_link = download_response.link();

        debug!("Downloading subtitle file from {}", download_link);
        match self.client.get(download_link)
            .send()
            .await {
            Ok(e) => self.handle_download_binary_response(file_id, path, subtitle_info, e).await,
            Err(err) => Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
        }
    }

    async fn handle_download_binary_response(&self, file_id: &i32, path: &Path, subtitle_info: &SubtitleInfo, response: Response) -> Result<Subtitle> {
        match response.status() {
            StatusCode::OK => {
                trace!("Storing subtitle response of {} into {:?}", file_id, path);
                match File::create(path) {
                    Ok(mut file) => {
                        let mut content = Cursor::new(response.bytes().await.unwrap());
                        match std::io::copy(&mut content, &mut file) {
                            Ok(_) => {
                                info!("Downloaded subtitle file {}", path.to_str().unwrap());
                                self.internal_parse(path, Some(subtitle_info))
                            }
                            Err(err) => return Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
                        }
                    }
                    Err(err) => Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
                }
            }
            _ => Err(SubtitleError::DownloadFailed(file_id.to_string(), format!("download failed with status code {}", response.status())))
        }
    }

    async fn handle_download_response(&self, file_id: &i32, path: &Path, subtitle_info: &SubtitleInfo, response: Response) -> Result<Subtitle> {
        match response.status() {
            StatusCode::OK => {
                match response.json::<DownloadResponse>()
                    .await
                    .map_err(|err| SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
                    .map(|download_response| async {
                        trace!("Received download link response {:?}", &download_response);
                        self.execute_download_request(file_id, path, subtitle_info, download_response).await
                    })
                {
                    Ok(e) => e.await,
                    Err(e) => Err(e)
                }
            }
            _ => Err(SubtitleError::DownloadFailed(file_id.to_string(), format!("download link request failed with status code {}", response.status())))
        }
    }

    /// Retrieve the storage [Path] for the given subtitle file.
    fn storage_file(&self, file: &SubtitleFile) -> PathBuf {
        let file_name = file.name();
        let settings = self.settings.settings().subtitle();

        settings.directory().join(file_name)
    }

    fn internal_parse(&self, file_path: &Path, info: Option<&SubtitleInfo>) -> Result<Subtitle> {
        trace!("Parsing subtitle file {}", file_path.to_str().unwrap());
        let path = String::from(file_path.to_str().unwrap());
        let extension = file_path
            .extension()
            .map(|e| String::from(e.to_str().unwrap()))
            .ok_or_else(|| SubtitleError::ParseFileError(path.clone(), "file has no extension".to_string()))?;
        let subtitle_type = SubtitleType::from_extension(&extension)
            .map_err(|err| SubtitleError::ParseFileError(path.clone(), err.to_string()))?;
        let parser = self.parsers.get(&subtitle_type)
            .ok_or_else(|| SubtitleError::TypeNotSupported(subtitle_type))?;

        File::open(&file_path)
            .map(|file| parser.parse_file(file))
            .map(|e| {
                info!("Parsed subtitle file {:?}", &file_path);
                Subtitle::new(e, info.map(|e| e.clone()), path.clone())
            })
            .map_err(|err| SubtitleError::ParseFileError(path.clone(), err.to_string()))
    }

    /// Find the subtitle for the default configured subtitle language.
    /// This uses the [SubtitleSettings::default_subtitle] setting.
    fn find_for_default_subtitle_language(&self, subtitles: &Vec<SubtitleInfo>) -> Option<SubtitleInfo> {
        let subtitle_language = self.settings.settings().subtitle().default_subtitle();

        subtitles.iter()
            .find(|e| e.language() == subtitle_language)
            .map(|e| e.clone())
    }

    /// Find the subtitle for the interface language.
    /// This uses the [UiSettings::default_language] setting.
    fn find_for_interface_language(&self, subtitles: &Vec<SubtitleInfo>) -> Option<SubtitleInfo> {
        let language = self.settings.settings().ui().default_language();

        subtitles.iter()
            .find(|e| &e.language().code() == language)
            .map(|e| e.clone())
    }

    /// Retrieve the subtitle filename from the given file or attributes.
    fn subtitle_file_name(file: &OpenSubtitlesFile, attributes: &OpenSubtitlesAttributes) -> String {
        let mut filename = file.file_name()
            .or_else(|| Some(attributes.release()))
            .unwrap()
            .clone();
        let extension = Path::new(&filename).extension();
        let mut append_extension = false;

        // verify if the filename has an extension
        // if not, add default [DEFAULT_FILENAME_EXTENSION]
        match extension {
            None => append_extension = true,
            Some(e) => {
                trace!("Checking validity of extension {:?}", e);
                if e.len() != 3 || Self::is_invalid_extension(e) {
                    append_extension = true;
                }
            }
        }

        if append_extension {
            debug!("Subtitle \"{}\" doesn't contain any valid extension, appending {}", &filename, DEFAULT_FILENAME_EXTENSION);
            filename += DEFAULT_FILENAME_EXTENSION;
        }

        filename
    }

    /// Filters any extension that should not be accepted as valid.
    fn is_invalid_extension(extension: &OsStr) -> bool {
        let normalized_extension = extension.to_ascii_lowercase();
        let extension = normalized_extension.to_str()
            .expect("expected the extension to be a valid unicode");
        let invalid_extensions: Vec<&str> = vec![
            "en",
            "lol",
        ];

        invalid_extensions.contains(&extension)
    }
}

#[async_trait]
impl SubtitleProvider for OpensubtitlesProvider {
    async fn movie_subtitles(&self, media: MovieDetails) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.imdb_id();

        debug!("Searching movie subtitles for IMDB ID {}", &imdb_id);
        self.start_search_request(&imdb_id, Some(&imdb_id), None, None)
            .await
    }

    async fn episode_subtitles(&self, media: ShowDetails, episode: Episode) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.imdb_id();

        debug!("Searching episode subtitles for IMDB ID {}", &imdb_id);
        self.start_search_request(&imdb_id, Some(&imdb_id), Some(&episode), None)
            .await
    }

    async fn file_subtitles(&self, filename: &String) -> Result<Vec<SubtitleInfo>> {
        debug!("Searching filename subtitles for {}", filename);
        self.start_search_request(filename, None, None, Some(filename))
            .await
    }

    async fn download(&self, subtitle_info: &SubtitleInfo, matcher: &SubtitleMatcher) -> Result<Subtitle> {
        trace!("Starting subtitle download for {}", subtitle_info);
        let subtitle_file = subtitle_info.best_matching_file(matcher)?;
        let file_location = self.storage_file(&subtitle_file);
        let file_id = subtitle_file.file_id();
        let path = file_location.as_path();

        // verify if the file has been downloaded in the past
        trace!("Verifying subtitle path {:?}", path);
        if path.exists() {
            info!("Subtitle file {:?} already exists, skipping download", path.as_os_str());
            return self.internal_parse(path, Some(subtitle_info));
        }

        let url = self.create_download_url()?;
        debug!("Starting subtitle download of {} ({}) for IMDB ID {:?}", subtitle_file.name(), file_id, subtitle_info.imdb_id());
        trace!("Requesting subtitle file {}", &url);
        match self.client.post(url)
            .json(&DownloadRequest::new(subtitle_file.file_id().clone()))
            .send()
            .await {
            Ok(response) => self.handle_download_response(file_id, path, subtitle_info, response).await,
            Err(err) => Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
        }
    }

    fn parse(&self, file_path: &Path) -> Result<Subtitle> {
        self.internal_parse(file_path, None)
    }

    fn select_or_default(&self, subtitles: &Vec<SubtitleInfo>) -> SubtitleInfo {
        trace!("Selecting subtitle out of {:?}", subtitles);
        let subtitle = self.find_for_default_subtitle_language(subtitles)
            .or_else(|| self.find_for_interface_language(subtitles))
            .unwrap_or(SubtitleInfo::none());
        debug!("Selected subtitle {:?}", &subtitle);
        subtitle
    }

    fn convert(&self, subtitle: Subtitle, output_type: SubtitleType) -> Result<String> {
        trace!("Retrieving compatible parser for output type {}", &output_type);
        match self.parsers.get(&output_type) {
            None => Err(SubtitleError::TypeNotSupported(output_type)),
            Some(parser) => {
                debug!("Converting subtitle to raw format of {} for {}", &output_type, subtitle);
                match parser.convert(subtitle.cues()) {
                    Err(err) => {
                        error!("Subtitle parsing to raw {} failed, {}", &output_type, err);
                        Err(SubtitleError::ConversionFailed(output_type.clone(), err.to_string()))
                    }
                    Ok(e) => {
                        debug!("Converted subtitle {:?} to raw {}", &subtitle.file(), &output_type);
                        Ok(e)
                    }
                }
            }
        }
    }
}

impl Drop for OpensubtitlesProvider {
    fn drop(&mut self) {
        let settings = self.settings.settings().subtitle();

        if *settings.auto_cleaning_enabled() {
            let path = settings.directory();
            debug!("Cleaning subtitle directory {:?}", &path);

            match fs::read_dir(&path) {
                Ok(e) => {
                    for file in e {
                        let sub_path = file.expect("expected file entry to be valid").path();

                        match fs::remove_file(&sub_path) {
                            Ok(_) => {}
                            Err(err) => {
                                warn!("Failed to delete subtitle file {:?}, {}", &sub_path, err);
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("Failed to clean subtitle directory {:?}, {}", &path, err)
                }
            }
        } else {
            trace!("Skipping subtitle directory cleaning")
        }
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;

    use popcorn_fx_core::core::config::*;
    use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage::{English, French};
    use popcorn_fx_core::core::subtitles::model::SubtitleType::Vtt;
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file};

    use super::*;

    fn start_mock_server() -> (MockServer, Arc<Application>) {
        let server = MockServer::start();
        let temp_dir = tempfile::tempdir().unwrap();
        let settings = Arc::new(Application::new(
            PopcornProperties::new(SubtitleProperties::new(
                server.url(""),
                String::new(),
                String::new())),
            PopcornSettings::new(
                SubtitleSettings::new(
                    temp_dir.into_path().to_str().unwrap().to_string(),
                    false,
                    English,
                    SubtitleFamily::Arial,
                ),
                UiSettings::default(),
                ServerSettings::default(),
            ),
        ));

        (server, settings)
    }

    #[tokio::test]
    async fn test_movie_subtitles() {
        init_logger();
        let settings = Arc::new(Application::default());
        let imdb_id = "tt1156398".to_string();
        let movie = MovieDetails::new(
            "lorem".to_string(),
            imdb_id.clone(),
            "2021".to_string(),
        );
        let service = OpensubtitlesProvider::new(&settings);

        let result = service.movie_subtitles(movie)
            .await;

        match result {
            Ok(subtitles) => {
                assert!(subtitles.len() > 0, "Expected at least one subtitle to have been found");
            }
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[tokio::test]
    async fn test_movie_subtitles_search_2_subtitles() {
        init_logger();
        let (server, settings) = start_mock_server();
        let movie1 = MovieDetails::new(
            "lorem".to_string(),
            "tt1156398".to_string(),
            "2021".to_string());
        let movie2 = MovieDetails::new(
            "ipsum".to_string(),
            "tt12003946".to_string(),
            "2021".to_string());
        let service = OpensubtitlesProvider::new(&settings);
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "1156398".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file("search_result_tt1156398.json"));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "12003946".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file("search_result_tt12003946.json"));
        });

        service.movie_subtitles(movie1)
            .await
            .expect("Expected the first search to succeed");
        let result = service.movie_subtitles(movie2)
            .await;

        match result {
            Ok(subtitles) => {
                assert!(subtitles.len() > 0, "Expected the second search to succeed");
            }
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[tokio::test]
    async fn test_episode_subtitles() {
        init_logger();
        let (server, settings) = start_mock_server();
        let show = ShowDetails::new(
            "tt4236770".to_string(),
            "tt4236770".to_string(),
            "lorem ipsum".to_string(),
            "2022".to_string(),
            1,
            Images::none(),
            None);
        let episode = Episode::new(
            1,
            1,
            1673136000,
            "tt2169080".to_string(),
            "Pilot".to_string(),
            9238597);
        let service = OpensubtitlesProvider::new(&settings);
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "4236770".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file("search_result_episode.json"));
        });
        let expected_result = SubtitleInfo::new(
            "tt2861424".to_string(),
            English,
        );

        let result = service.episode_subtitles(show, episode)
            .await;

        match result {
            Ok(subtitles) => {
                assert_eq!(1, subtitles.len(), "Expected 1 subtitle to have been returned");
                assert_eq!(&expected_result, subtitles.get(0).unwrap(), "Expected 1 subtitle to have been returned");
            }
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[tokio::test]
    async fn test_filename_subtitles() {
        init_logger();
        let (server, settings) = start_mock_server();
        let filename = "House.of.the.Dragon.S01E01.HMAX.WEBRip.x264-XEN0N.mkv".to_string();
        let service = OpensubtitlesProvider::new(&settings);
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(FILENAME_PARAM_KEY, filename.clone());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file("search_result_episode.json"));
        });

        let result = service.file_subtitles(&filename)
            .await;

        match result {
            Ok(subtitles) => assert!(subtitles.len() > 0, "Expected at least one subtitle to have been found"),
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[tokio::test]
    async fn test_download_should_return_the_expected_subtitle() {
        init_logger();
        let (server, settings) = start_mock_server();
        let temp_dir = settings.settings().subtitle().directory().to_str().unwrap().to_string();
        let service = OpensubtitlesProvider::new(&settings);
        let filename = "test-subtitle-file.srt".to_string();
        let subtitle_info = SubtitleInfo::new_with_files("tt7405458".to_string(), SubtitleLanguage::German, vec![
            SubtitleFile::new(91135, filename.clone(), String::new(), 0.0, 0)
        ]);
        let matcher = SubtitleMatcher::from_string(Some(String::new()), Some(String::from("720")));
        let response_body = read_test_file("download_response.json");
        server.mock(|when, then| {
            when.method(POST)
                .path("/download");
            then.status(200)
                .header("content-type", "application/json")
                .body(response_body
                    .replace("[[host]]", server.host().as_str())
                    .replace("[[port]]", server.port().to_string().as_str()));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/download/example.srt");
            then.status(200)
                .header("content-type", "text")
                .body(read_test_file("subtitle_example.srt"));
        });
        let expected_file: PathBuf = [temp_dir, filename].iter().collect();
        let expected_result = Subtitle::new(vec![SubtitleCue::new("1".to_string(), 30296, 34790, vec![
            SubtitleLine::new(vec![
                StyledText::new("Drink up, me hearties, yo ho".to_string(), true, false, false)
            ])])], Some(subtitle_info.clone()), expected_file.to_str().unwrap().to_string());

        let result = service.download(&subtitle_info, &matcher)
            .await
            .unwrap();

        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_download_when_subtitle_file_exists_should_return_existing_file() {
        init_logger();
        let test_file = "subtitle_existing.srt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path().to_str().unwrap().to_string();
        let popcorn_settings = PopcornSettings::new(SubtitleSettings::new(
            temp_path.clone(),
            false,
            English,
            SubtitleFamily::Arial,
        ), UiSettings::new(
            "en".to_string(),
            UiScale::new(1f32).expect("Expected ui scale to be valid"),
            StartScreen::Movies,
            false,
            false,
        ), ServerSettings::default());
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let destination = copy_test_file(temp_path.clone().as_str(), test_file);
        let service = OpensubtitlesProvider::new(&settings);
        let subtitle_info = SubtitleInfo::new_with_files("tt00001".to_string(), SubtitleLanguage::German, vec![
            SubtitleFile::new(10001111, "subtitle_existing.srt".to_string(), String::new(), 0.0, 0)
        ]);
        let matcher = SubtitleMatcher::from_string(Some(String::new()), Some(String::from("720")));
        let expected_cues: Vec<SubtitleCue> = vec![
            SubtitleCue::new("1".to_string(), 8224, 10124, vec![
                SubtitleLine::new(vec![StyledText::new("Okay, if no one else will say it, I will.".to_string(), false, false, false)])
            ])
        ];
        let expected_result = Subtitle::new(expected_cues.clone(), Some(subtitle_info.clone()), destination.clone());

        let result = service.download(&subtitle_info, &matcher)
            .await
            .unwrap();

        assert_eq!(expected_result, result);
        assert_eq!(&expected_cues, result.cues())
    }

    #[test]
    fn test_parse_valid_file() {
        init_logger();
        let test_file = "subtitle_example.srt";
        let temp_dir = tempfile::tempdir().unwrap();
        let settings = Arc::new(Application::default());
        let service = OpensubtitlesProvider::new(&settings);
        let destination = copy_test_file(temp_dir.into_path().to_str().unwrap(), test_file);
        let expected_result = Subtitle::new(
            vec![
                SubtitleCue::new("1".to_string(), 0, 0, vec![SubtitleLine::new(vec![StyledText::new("Drink up, me hearties, yo ho".to_string(), true, false, false)])])
            ],
            None,
            destination.clone(),
        );

        let result = service.parse(Path::new(&destination)).unwrap();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_select_or_default_select_for_default_subtitle_language() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let popcorn_settings = PopcornSettings::new(SubtitleSettings::new(
            temp_dir.into_path().into_os_string().into_string().unwrap(),
            false,
            English,
            SubtitleFamily::Arial,
        ), UiSettings::default(), ServerSettings::default());
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let service = OpensubtitlesProvider::new(&settings);
        let subtitle_info = SubtitleInfo::new("lorem".to_string(), English);
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = service.select_or_default(&subtitles);

        assert_eq!(subtitle_info, result)
    }

    #[test]
    fn test_select_or_default_select_for_interface_language() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let popcorn_settings = PopcornSettings::new(SubtitleSettings::new(
            temp_dir.into_path().into_os_string().into_string().unwrap(),
            false,
            SubtitleLanguage::Croatian,
            SubtitleFamily::Arial,
        ), UiSettings::new(
            "fr".to_string(),
            UiScale::new(1f32).expect("Expected ui scale to be valid"),
            StartScreen::Movies,
            false,
            false,
        ), ServerSettings::default());
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let service = OpensubtitlesProvider::new(&settings);
        let subtitle_info = SubtitleInfo::new("ipsum".to_string(), French);
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = service.select_or_default(&subtitles);

        assert_eq!(subtitle_info, result)
    }

    #[test]
    fn test_subtitle_file_name_missing_extension_in_file() {
        init_logger();
        let file = OpenSubtitlesFile::new_with_filename(0, "my-filename".to_string());
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "".to_string());
        let expected_result = "my-filename.srt".to_string();

        let result = OpensubtitlesProvider::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_missing_extension_in_release() {
        init_logger();
        let file = OpenSubtitlesFile::new(687);
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "lorem".to_string());
        let expected_result = "lorem.srt".to_string();

        let result = OpensubtitlesProvider::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_too_long_extension() {
        init_logger();
        let file = OpenSubtitlesFile::new_with_filename(0, "lorem.XviD-DEViSE".to_string());
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "".to_string());
        let expected_result = "lorem.XviD-DEViSE.srt".to_string();

        let result = OpensubtitlesProvider::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_too_short_extension() {
        init_logger();
        let file = OpenSubtitlesFile::new_with_filename(0, "lorem.en".to_string());
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "".to_string());
        let expected_result = "lorem.en.srt".to_string();

        let result = OpensubtitlesProvider::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_convert_to_vtt() {
        let subtitle = Subtitle::new(
            vec![SubtitleCue::new(
                "1".to_string(),
                45000,
                46890,
                vec![SubtitleLine::new(vec![
                    StyledText::new("lorem".to_string(), false, false, true)
                ])],
            )],
            None,
            String::new(),
        );
        let settings = Arc::new(Application::default());
        let service = OpensubtitlesProvider::new(&settings);
        let expected_result = read_test_file("example-conversion.vtt");

        let result = service.convert(subtitle, Vtt);

        assert_eq!(expected_result, result.expect("Expected the conversion to have succeeded"))
    }
}