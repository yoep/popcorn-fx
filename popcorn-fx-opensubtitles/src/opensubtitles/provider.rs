use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Display;
use futures::StreamExt;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode, Url};
use reqwest::header::HeaderMap;
use tokio::fs::OpenOptions;

use popcorn_fx_core::core::config::ApplicationConfig;
use popcorn_fx_core::core::media::*;
use popcorn_fx_core::core::subtitles::{Result, SubtitleError, SubtitleFile, SubtitleProvider};
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
use popcorn_fx_core::core::subtitles::parsers::Parser;

use crate::opensubtitles::model::*;

const API_HEADER_KEY: &str = "Api-Key";
const USER_AGENT_HEADER_KEY: &str = "User-Agent";
const IMDB_ID_PARAM_KEY: &str = "imdb_id";
const SEASON_PARAM_KEY: &str = "season_number";
const EPISODE_PARAM_KEY: &str = "episode_number";
const FILENAME_PARAM_KEY: &str = "query";
const PAGE_PARAM_KEY: &str = "page";
const DEFAULT_FILENAME_EXTENSION: &str = ".srt";

#[derive(Debug, Display)]
#[display(fmt = "Opensubtitles subtitle provider")]
pub struct OpensubtitlesProvider {
    settings: Arc<ApplicationConfig>,
    client: Client,
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
}

impl OpensubtitlesProvider {
    /// Returns a new `OpensubtitlesProviderBuilder` instance to configure an `OpensubtitlesProvider`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use popcorn_fx_core::core::config::ApplicationConfig;
    /// use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
    ///
    /// let settings = Arc::new(ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .build());
    /// let provider = OpensubtitlesProvider::builder()
    ///     .settings(settings)
    ///     .build();
    /// ```
    pub fn builder() -> OpensubtitlesProviderBuilder {
        OpensubtitlesProviderBuilder::default()
    }

    async fn create_search_url(&self, media_id: Option<&str>, episode: Option<&Episode>, filename: Option<&str>, page: i32) -> Result<Url> {
        let mut query_params: Vec<(&str, &str)> = vec![];
        let imdb_id: String;
        let season: String;
        let episode_number: String;
        let page_query_value = page.to_string();
        let properties = self.settings.properties();
        let url = format!("{}/subtitles", properties.subtitle().url());

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

    async fn create_download_url(&self) -> Result<Url> {
        let properties = self.settings.properties();
        let url = format!("{}/download", properties.subtitle().url());

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
                    language_files.push(SubtitleFile::builder()
                        .file_id(file.file_id().clone())
                        .name(Self::subtitle_file_name(file, attributes))
                        .url(attributes.url().clone())
                        .score(attributes.ratings().clone())
                        .downloads(attributes.download_count().clone())
                        .build());
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

                SubtitleInfo::builder()
                    .imdb_id(imdb_id.clone())
                    .language(language.clone())
                    .files(files.clone())
                    .build()
            })
            .sorted()
            .collect()
    }

    async fn handle_search_result(id: &str, response: Response) -> Result<OpenSubtitlesResponse<SearchResult>> {
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

    async fn start_search_request(&self, id: &str, media_id: Option<&str>, episode: Option<&Episode>, filename: Option<&str>) -> Result<Vec<SubtitleInfo>> {
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

    async fn fetch_search_page(&self, id: &str, media_id: Option<&str>, episode: Option<&Episode>, filename: Option<&str>, page: i32) -> Result<OpenSubtitlesResponse<SearchResult>> {
        let url = self.create_search_url(media_id, episode, filename, page).await?;

        debug!("Retrieving available subtitles from {}", &url);
        match self.client.clone().get(url)
            .send()
            .await {
            Err(err) => Err(SubtitleError::SearchFailed(format!("OpenSubtitles request failed, {}", err))),
            Ok(response) => Self::handle_search_result(id, response).await
        }
    }

    async fn execute_download_request(&self, file_id: &i32, path: &Path, download_response: DownloadResponse) -> Result<String> {
        let download_link = download_response.link();

        debug!("Downloading subtitle file from {}", download_link);
        match self.client.get(download_link)
            .send()
            .await {
            Ok(e) => self.handle_download_binary_response(file_id, path, e).await,
            Err(err) => Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
        }
    }

    async fn handle_download_binary_response(&self, file_id: &i32, path: &Path, response: Response) -> Result<String> {
        match response.status() {
            StatusCode::OK => {
                // create the parent directory if needed
                let directory_path = path.to_path_buf();
                let directory = directory_path.parent().unwrap();
                trace!("Creating subtitle directory {}", directory.to_str().unwrap());
                fs::create_dir_all(directory)
                    .map_err(|e| SubtitleError::IO(directory.to_str().unwrap().to_string(), e.to_string()))?;

                // open the subtitle file that will be written
                let filepath = path.to_str().unwrap();
                trace!("Opening subtitle file {}", filepath);
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .await
                    .map_err(|e| SubtitleError::IO(filepath.to_string(), e.to_string()))?;

                // stream the bytes to the opened file
                debug!("Writing subtitle file {} to {}", file_id, filepath);
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|e| {
                        error!("Failed to read subtitle response chunk, {}", e);
                        SubtitleError::DownloadFailed(filepath.to_string(), e.to_string())
                    })?;

                    tokio::io::copy(&mut chunk.as_ref(), &mut file).await.map_err(|e| {
                        error!("Failed to write subtitle file, {}", e);
                        SubtitleError::IO(filepath.to_string(), e.to_string())
                    })?;
                }

                info!("Downloaded subtitle file {}", filepath);
                Ok(filepath.to_string())
            }
            _ => Err(SubtitleError::DownloadFailed(file_id.to_string(), format!("download failed with status code {}", response.status())))
        }
    }

    async fn handle_download_response(&self, file_id: &i32, path: &Path, response: Response) -> Result<String> {
        match response.status() {
            StatusCode::OK => {
                match response.json::<DownloadResponse>()
                    .await
                    .map_err(|err| SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
                    .map(|download_response| async {
                        trace!("Received download link response {:?}", &download_response);
                        self.execute_download_request(file_id, path, download_response).await
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
    async fn storage_file(&self, file: &SubtitleFile) -> PathBuf {
        let file_name = file.name();
        let settings = self.settings.user_settings();
        let settings = settings.subtitle().clone();

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
    fn find_for_default_subtitle_language(&self, subtitles: &[SubtitleInfo]) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings();
        let subtitle_language = settings.subtitle().default_subtitle();

        subtitles.iter()
            .find(|e| e.language() == subtitle_language)
            .map(|e| e.clone())
    }

    /// Find the subtitle for the interface language.
    /// This uses the [UiSettings::default_language] setting.
    fn find_for_interface_language(&self, subtitles: &[SubtitleInfo]) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings();
        let language = settings.ui().default_language();

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
        let normalized_extension = extension.to_str()
            .expect("expected a valid utf-8 extension")
            .to_lowercase();
        let invalid_extensions: Vec<&str> = vec![
            "com",
            "de",
            "en",
            "eng",
            "fr",
            "lol",
            "ned",
            "nl",
        ];

        invalid_extensions.contains(&normalized_extension.as_str())
    }
}

#[async_trait]
impl SubtitleProvider for OpensubtitlesProvider {
    async fn movie_subtitles(&self, media: &MovieDetails) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.imdb_id();

        debug!("Searching movie subtitles for IMDB ID {}", &imdb_id);
        self.start_search_request(&imdb_id, Some(&imdb_id), None, None)
            .await
    }

    async fn episode_subtitles(&self, media: &ShowDetails, episode: &Episode) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.imdb_id();

        debug!("Searching episode subtitles for IMDB ID {}", &imdb_id);
        self.start_search_request(&imdb_id, Some(&imdb_id), Some(&episode), None)
            .await
    }

    async fn file_subtitles(&self, filename: &str) -> Result<Vec<SubtitleInfo>> {
        debug!("Searching filename subtitles for {}", filename);
        self.start_search_request(filename, None, None, Some(filename))
            .await
    }

    async fn download(&self, subtitle_info: &SubtitleInfo, matcher: &SubtitleMatcher) -> Result<String> {
        trace!("Starting subtitle download for {}", subtitle_info);
        let subtitle_file = subtitle_info.best_matching_file(matcher)?;
        let file_location = self.storage_file(&subtitle_file).await;
        let file_id = subtitle_file.file_id();
        let path = file_location.as_path();

        // verify if the file has been downloaded in the past
        trace!("Verifying subtitle path {:?}", path);
        if path.exists() {
            info!("Subtitle file {:?} already exists, skipping download", path.as_os_str());
            return Ok(path.to_str().expect("expected the subtitle path to be valid").to_string());
        }

        let url = self.create_download_url().await?;
        debug!("Starting subtitle download of {} ({}) for IMDB ID {:?}", subtitle_file.name(), file_id, subtitle_info.imdb_id());
        trace!("Requesting subtitle file {}", &url);
        match self.client.post(url)
            .json(&DownloadRequest::new(subtitle_file.file_id().clone()))
            .send()
            .await {
            Ok(response) => self.handle_download_response(file_id, path, response).await,
            Err(err) => Err(SubtitleError::DownloadFailed(file_id.to_string(), err.to_string()))
        }
    }

    async fn download_and_parse(&self, subtitle_info: &SubtitleInfo, matcher: &SubtitleMatcher) -> Result<Subtitle> {
        match self.download(subtitle_info, matcher).await {
            Err(e) => Err(e),
            Ok(path) => {
                let path = Path::new(&path);
                self.internal_parse(path, Some(subtitle_info))
            }
        }
    }

    fn parse(&self, file_path: &Path) -> Result<Subtitle> {
        self.internal_parse(file_path, None)
    }

    fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
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

#[derive(Default)]
pub struct OpensubtitlesProviderBuilder {
    settings: Option<Arc<ApplicationConfig>>,
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
    insecure: bool,
}

impl OpensubtitlesProviderBuilder {
    /// Sets the `ApplicationConfig` instance to use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use popcorn_fx_core::core::config::ApplicationConfig;
    /// use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
    ///
    /// let settings = Arc::new(ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .build());
    /// let provider = OpensubtitlesProvider::builder()
    ///     .settings(settings)
    ///     .build();
    /// ```
    pub fn settings(mut self, settings: Arc<ApplicationConfig>) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Adds a parser instance to the `HashMap` of parsers used by the provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_core::core::subtitles::model::SubtitleType;
    /// use popcorn_fx_core::core::subtitles::parsers::{Parser, SrtParser};
    /// use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
    ///
    /// let srt_parser: Box<dyn Parser> = Box::new(SrtParser::new());
    /// let provider = OpensubtitlesProvider::builder()
    ///     .with_parser(SubtitleType::Srt, srt_parser)
    ///     .build();
    /// ```
    pub fn with_parser(mut self, parser_type: SubtitleType, parser: Box<dyn Parser>) -> Self {
        self.parsers.insert(parser_type, parser);
        self
    }

    /// Sets whether insecure connections are allowed the API requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
    ///
    /// let provider = OpensubtitlesProvider::builder()
    ///     .insecure(true)
    ///     .build();
    /// ```
    pub fn insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    /// Builds an `OpensubtitlesProvider` object with the specified parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if the `settings` have not been set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    /// use popcorn_fx_core::core::config::ApplicationConfig;
    /// use popcorn_fx_core::core::subtitles::model::SubtitleType;
    /// use popcorn_fx_core::core::subtitles::parsers::{Parser, SrtParser};
    /// use popcorn_fx_opensubtitles::opensubtitles::OpensubtitlesProvider;
    ///
    /// let srt_parser: Box<dyn Parser> = Box::new(SrtParser::new());
    /// let settings = Arc::new(ApplicationConfig::builder()
    ///     .storage("storage/path")
    ///     .build());
    /// let provider = OpensubtitlesProvider::builder()
    ///     .settings(settings)
    ///     .with_parser(SubtitleType::Srt, srt_parser)
    ///     .build();
    /// ```
    pub fn build(self) -> OpensubtitlesProvider {
        let settings = self.settings.expect("Settings have not been set for OpensubtitlesProvider");
        let mut default_headers = HeaderMap::new();
        let properties = settings.properties();
        let api_token = properties.subtitle().api_token().to_string();
        let user_agent = properties.subtitle().user_agent().to_string();

        default_headers.insert(USER_AGENT_HEADER_KEY, user_agent.parse().unwrap());
        default_headers.insert(API_HEADER_KEY, api_token.parse().unwrap());

        OpensubtitlesProvider {
            settings,
            client: ClientBuilder::new()
                .default_headers(default_headers)
                .danger_accept_invalid_certs(self.insecure)
                .build()
                .unwrap(),
            parsers: self.parsers,
        }
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use tokio::runtime;

    use popcorn_fx_core::core::config::*;
    use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage::English;
    use popcorn_fx_core::core::subtitles::parsers::{SrtParser, VttParser};
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file_to_string};

    use super::*;

    fn start_mock_server() -> (MockServer, Arc<ApplicationConfig>) {
        start_mock_server_with_subtitle_dir(None)
    }

    fn start_mock_server_with_subtitle_dir(subdirectory: Option<&str>) -> (MockServer, Arc<ApplicationConfig>) {
        let server = MockServer::start();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: SubtitleProperties {
                    url: server.url(""),
                    user_agent: String::new(),
                    api_token: String::new(),
                },
                tracking: Default::default(),
            })
            .settings(PopcornSettings {
                subtitle_settings: SubtitleSettings {
                    directory: PathBuf::from(temp_path).join(subdirectory.or_else(|| Some("")).unwrap()).to_str().unwrap().to_string(),
                    auto_cleaning_enabled: false,
                    default_subtitle: English,
                    font_family: SubtitleFamily::Arial,
                    font_size: 28,
                    decoration: DecorationType::None,
                    bold: false,
                },
                ui_settings: Default::default(),
                server_settings: Default::default(),
                torrent_settings: Default::default(),
                playback_settings: Default::default(),
                tracking_settings: Default::default(),
            })
            .build());

        (server, settings)
    }

    #[test]
    fn test_movie_subtitles() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let imdb_id = "tt1156398".to_string();
        let movie = MovieDetails::new(
            "lorem".to_string(),
            imdb_id.clone(),
            "2021".to_string(),
        );
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(service.movie_subtitles(&movie));

        match result {
            Ok(subtitles) => {
                assert!(subtitles.len() > 0, "Expected at least one subtitle to have been found");
            }
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[test]
    fn test_movie_subtitles_search_2_subtitles() {
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
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "1156398".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("search_result_tt1156398.json"));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "12003946".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("search_result_tt12003946.json"));
        });
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(async {
            service.movie_subtitles(&movie1)
                .await
                .expect("Expected the first search to succeed");
            service.movie_subtitles(&movie2)
                .await
        });

        match result {
            Ok(subtitles) => {
                assert!(subtitles.len() > 0, "Expected the second search to succeed");
            }
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[test]
    fn test_episode_subtitles() {
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
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "4236770".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("search_result_episode.json"));
        });
        let expected_result = SubtitleInfo::builder()
            .imdb_id("tt2861424")
            .language(English)
            .build();
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(service.episode_subtitles(&show, &episode));

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

    #[test]
    fn test_filename_subtitles() {
        init_logger();
        let (server, settings) = start_mock_server();
        let filename = "House.of.the.Dragon.S01E01.HMAX.WEBRip.x264-XEN0N.mkv".to_string();
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(FILENAME_PARAM_KEY, filename.clone());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("search_result_episode.json"));
        });
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(service.file_subtitles(&filename));

        match result {
            Ok(subtitles) => assert!(subtitles.len() > 0, "Expected at least one subtitle to have been found"),
            Err(err) => {
                assert!(false, "{:?}", &err)
            }
        }
    }

    #[test]
    fn test_download_should_return_the_expected_subtitle() {
        init_logger();
        let (server, settings) = start_mock_server();
        let temp_dir = settings.user_settings().subtitle().directory().to_str().unwrap().to_string();
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .with_parser(SubtitleType::Srt, Box::new(SrtParser::new()))
            .build();
        let filename = "test-subtitle-file.srt".to_string();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("tt7405458")
            .language(SubtitleLanguage::German)
            .files(vec![
                SubtitleFile::builder()
                    .file_id(91135)
                    .name(filename.clone())
                    .url("")
                    .score(0.0)
                    .downloads(0)
                    .build(),
            ])
            .build();
        let matcher = SubtitleMatcher::from_string(Some(String::new()), Some(String::from("720")));
        let response_body = read_test_file_to_string("download_response.json");
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
                .body(read_test_file_to_string("subtitle_example.srt"));
        });
        let expected_file: PathBuf = [temp_dir, filename].iter().collect();
        let expected_result = Subtitle::new(vec![SubtitleCue::new("1".to_string(), 30296, 34790, vec![
            SubtitleLine::new(vec![
                StyledText::new("Drink up, me hearties, yo ho".to_string(), true, false, false)
            ])])], Some(subtitle_info.clone()), expected_file.to_str().unwrap().to_string());
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(service.download_and_parse(&subtitle_info, &matcher)).unwrap();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_download_should_create_subtitle_directory() {
        init_logger();
        let subdirectory = "subtitles";
        let (server, settings) = start_mock_server_with_subtitle_dir(Some(subdirectory));
        let temp_dir = settings.user_settings().subtitle().directory().to_str().unwrap().to_string();
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .with_parser(SubtitleType::Srt, Box::new(SrtParser::new()))
            .build();
        let filename = "test-subtitle-file.srt".to_string();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("tt7405458")
            .language(SubtitleLanguage::German)
            .files(vec![
                SubtitleFile::builder()
                    .file_id(91135)
                    .name(filename.clone())
                    .url("")
                    .score(0.0)
                    .downloads(0)
                    .build(),
            ])
            .build();
        let matcher = SubtitleMatcher::from_string(Some(String::new()), Some(String::from("720")));
        let response_body = read_test_file_to_string("download_response.json");
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
                .body(read_test_file_to_string("subtitle_example.srt"));
        });
        let runtime = runtime::Runtime::new().unwrap();

        let _ = runtime.block_on(service.download_and_parse(&subtitle_info, &matcher))
            .expect("expected the download to succeed");

        // the temp_dir already contains the subdirectory
        assert!(PathBuf::from(temp_dir.as_str()).exists(), "expected the subtitle directory to have been created");
        assert!(PathBuf::from(temp_dir.as_str()).join(filename).exists(), "expected the subtitle to have been created");
    }

    #[test]
    fn test_download_when_subtitle_file_exists_should_return_existing_file() {
        init_logger();
        let test_file = "subtitle_existing.srt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let popcorn_settings = PopcornSettings {
            subtitle_settings: SubtitleSettings
            {
                directory: temp_path.to_string(),
                auto_cleaning_enabled: false,
                default_subtitle: English,
                font_family: SubtitleFamily::Arial,
                font_size: 28,
                decoration: DecorationType::None,
                bold: false,
            },
            ui_settings: UiSettings {
                default_language: "en".to_string(),
                ui_scale: UiScale::new(1f32).expect("Expected ui scale to be valid"),
                start_screen: Category::Movies,
                maximized: false,
                native_window_enabled: false,
            },
            server_settings: ServerSettings::default(),
            torrent_settings: TorrentSettings::default(),
            playback_settings: Default::default(),
            tracking_settings: Default::default(),
        };
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .settings(popcorn_settings)
            .build());
        let destination = copy_test_file(temp_path, test_file, None);
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .with_parser(SubtitleType::Srt, Box::new(SrtParser::new()))
            .build();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("tt00001")
            .language(SubtitleLanguage::German)
            .files(vec![
                SubtitleFile::builder()
                    .file_id(10001111)
                    .name("subtitle_existing.srt")
                    .url("")
                    .score(0.0)
                    .downloads(0)
                    .build(),
            ])
            .build();
        let matcher = SubtitleMatcher::from_string(Some(String::new()), Some(String::from("720")));
        let expected_cues: Vec<SubtitleCue> = vec![
            SubtitleCue::new("1".to_string(), 8224, 10124, vec![
                SubtitleLine::new(vec![StyledText::new("Okay, if no one else will say it, I will.".to_string(), false, false, false)])
            ])
        ];
        let expected_result = Subtitle::new(expected_cues.clone(), Some(subtitle_info.clone()), destination.clone());
        let runtime = runtime::Runtime::new().unwrap();

        let result = runtime.block_on(service.download_and_parse(&subtitle_info, &matcher)).unwrap();

        assert_eq!(expected_result, result);
        assert_eq!(&expected_cues, result.cues())
    }

    #[test]
    fn test_parse_valid_file() {
        init_logger();
        let test_file = "subtitle_example.srt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .with_parser(SubtitleType::Srt, Box::new(SrtParser::new()))
            .build();
        let destination = copy_test_file(temp_dir.into_path().to_str().unwrap(), test_file, None);
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
        let temp_path = temp_dir.path().to_str().unwrap();
        let popcorn_settings = PopcornSettings {
            subtitle_settings: SubtitleSettings
            {
                directory: temp_path.to_string(),
                auto_cleaning_enabled: false,
                default_subtitle: English,
                font_family: SubtitleFamily::Arial,
                font_size: 28,
                decoration: DecorationType::None,
                bold: false,
            },
            ui_settings: UiSettings::default(),
            server_settings: ServerSettings::default(),
            torrent_settings: TorrentSettings::default(),
            playback_settings: Default::default(),
            tracking_settings: Default::default(),
        };
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .settings(popcorn_settings)
            .build());
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("lorem")
            .language(SubtitleLanguage::English)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = service.select_or_default(&subtitles);

        assert_eq!(subtitle_info, result)
    }

    #[test]
    fn test_select_or_default_select_for_interface_language() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let popcorn_settings = PopcornSettings {
            subtitle_settings: SubtitleSettings {
                directory: temp_path.to_string(),
                auto_cleaning_enabled: false,
                default_subtitle: SubtitleLanguage::Croatian,
                font_family:
                SubtitleFamily::Arial,
                font_size: 28,
                decoration: DecorationType::None,
                bold: false,
            },
            ui_settings: UiSettings {
                default_language: "fr".to_string(),
                ui_scale: UiScale::new(1f32).expect("Expected ui scale to be valid"),
                start_screen: Category::Movies,
                maximized: false,
                native_window_enabled: false,
            },
            server_settings: Default::default(),
            torrent_settings: Default::default(),
            playback_settings: Default::default(),
            tracking_settings: Default::default(),
        };
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .settings(popcorn_settings)
            .build());
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .build();
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("ipsum")
            .language(SubtitleLanguage::French)
            .build();
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
        let file = OpenSubtitlesFile {
            file_id: 687,
            cd_number: None,
            file_name: None,
        };
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
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let service = OpensubtitlesProvider::builder()
            .settings(settings)
            .with_parser(SubtitleType::Vtt, Box::new(VttParser::default()))
            .build();
        let expected_result = read_test_file_to_string("example-conversion.vtt")
            .replace("\r\n", "\n");

        let result = service.convert(subtitle, SubtitleType::Vtt);

        assert_eq!(expected_result, result.expect("Expected the conversion to have succeeded"))
    }

    #[test]
    fn test_invalid_extensions() {
        let filename1 = OpensubtitlesProvider::subtitle_file_name(
            &OpenSubtitlesFile::new_with_filename(0, "tpz-house302.Ned".to_string()),
            &OpenSubtitlesAttributes::new("tt11110".to_string(), String::new()));
        let filename2 = OpensubtitlesProvider::subtitle_file_name(
            &OpenSubtitlesFile::new_with_filename(0, "lorem.2009.Bluray.1080p.DTSMA5.1.x264.dxva-FraMeSToR.ENG".to_string()),
            &OpenSubtitlesAttributes::new("tt11110".to_string(), String::new()));

        assert_eq!("tpz-house302.Ned.srt".to_string(), filename1);
        assert_eq!("lorem.2009.Bluray.1080p.DTSMA5.1.x264.dxva-FraMeSToR.ENG.srt".to_string(), filename2);
    }
}