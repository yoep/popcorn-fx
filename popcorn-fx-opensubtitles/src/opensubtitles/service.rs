use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode, Url};
use reqwest::header::HeaderMap;

use popcorn_fx_core::core::config::Application;
use popcorn_fx_core::core::media::model::*;
use popcorn_fx_core::core::subtitles::errors::SubtitleError;
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleFile, SubtitleInfo, SubtitleType};
use popcorn_fx_core::core::subtitles::parsers::{Parser, VttParser};
use popcorn_fx_core::core::subtitles::parsers::SrtParser;
use popcorn_fx_core::core::subtitles::service::{Result, SubtitleService};

use crate::opensubtitles::model::*;

const API_HEADER_KEY: &str = "Api-Key";
const USER_AGENT_HEADER_KEY: &str = "User-Agent";
const IMDB_ID_PARAM_KEY: &str = "imdb_id";
const SEASON_PARAM_KEY: &str = "season_number";
const EPISODE_PARAM_KEY: &str = "episode_number";
const FILENAME_PARAM_KEY: &str = "query";
const DEFAULT_FILENAME_EXTENSION: &str = ".srt";

pub struct OpensubtitlesService {
    settings: Arc<Application>,
    client: Client,
    active_subtitle: Option<Subtitle>,
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
}

impl OpensubtitlesService {
    /// Create a new OpenSubtitles service instance.
    pub fn new(settings: &Arc<Application>) -> Self {
        let mut default_headers = HeaderMap::new();
        let srt_parser: Box<dyn Parser> = Box::new(SrtParser::new());
        let vtt_parser: Box<dyn Parser> = Box::new(VttParser::new());
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
            active_subtitle: None,
            parsers: HashMap::from([
                (SubtitleType::Srt, srt_parser),
                (SubtitleType::Vtt, vtt_parser)
            ]),
        }
    }

    fn create_search_url(&self, media: Option<&dyn MediaIdentifier>, episode: Option<&Episode>, filename: Option<&String>) -> Result<Url> {
        let mut query_params: Vec<(&str, &str)> = vec![];
        let imdb_id: String;
        let season: String;
        let episode_number: String;
        let url = format!("{}/subtitles", self.settings.properties().subtitle().url());

        if media.is_some() {
            trace!("Creating search url for media {:?}", media.unwrap());
            imdb_id = media.unwrap().id().replace("tt", "");
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

    fn search_result_to_subtitles(result: OpenSubtitlesResponse<SearchResult>) -> Vec<SubtitleInfo> {
        let mut id: String = String::new();
        let mut imdb_id: String = String::new();
        let mut languages: HashMap<SubtitleLanguage, Vec<SubtitleFile>> = HashMap::new();

        trace!("Mapping a total of {} subtitle search results", result.data().len());
        for search_result in result.data() {
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

    async fn handle_search_result(id: &String, response: Response) -> Result<Vec<SubtitleInfo>> {
        match response.status() {
            StatusCode::OK => {
                trace!("Received response from OpenSubtitles for {}, decoding JSON...", id);
                let parser = response.json::<OpenSubtitlesResponse<SearchResult>>()
                    .await
                    .map(|e| Self::search_result_to_subtitles(e));

                match parser {
                    Ok(e) => {
                        debug!("Found subtitles for IMDB ID {}: {:?}", id, &e);
                        Ok(e)
                    }
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

    async fn execute_search_request(&self, id: &String, url: Url) -> Result<Vec<SubtitleInfo>> {
        debug!("Retrieving available subtitles from {}", &url);
        match self.client.clone().get(url)
            .send()
            .await {
            Err(err) => Err(SubtitleError::SearchFailed(format!("OpenSubtitles request failed, {}", err))),
            Ok(e) => Self::handle_search_result(id, e).await
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
            .ok_or_else(|| SubtitleError::ParsingFailed(path.clone(), "file has no extension".to_string()))?;
        let subtitle_type = SubtitleType::from_extension(&extension)
            .map_err(|err| SubtitleError::ParsingFailed(path.clone(), err.to_string()))?;
        let parser = self.parsers.get(&subtitle_type)
            .ok_or_else(|| SubtitleError::TypeNotSupported(subtitle_type))?;

        File::open(&file_path)
            .map(|file| parser.parse_file(file))
            .map(|e| {
                info!("Parsed subtitle file {:?}", &file_path);
                Subtitle::new(e, info.map(|e| e.clone()), Some(path.clone()))
            })
            .map_err(|err| SubtitleError::ParsingFailed(path.clone(), err.to_string()))
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
                if e.len() != 3 {
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
}

#[async_trait]
impl SubtitleService for OpensubtitlesService {
    fn active_subtitle(&self) -> Option<&Subtitle> {
        match &self.active_subtitle {
            None => None,
            Some(x) => {
                Some(x)
            }
        }
    }

    fn update_active_subtitle(&mut self, subtitle: Option<Subtitle>) {
        self.active_subtitle = subtitle;
    }

    async fn movie_subtitles(&self, media: Movie) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.id();
        let url = self.create_search_url(Some(&media), None, None)?;

        debug!("Searching movie subtitles for IMDB ID {}", &imdb_id);
        self.execute_search_request(imdb_id, url)
            .await
    }

    async fn episode_subtitles(&self, media: Show, episode: Episode) -> Result<Vec<SubtitleInfo>> {
        let imdb_id = media.id();
        let url = self.create_search_url(Some(&media), Some(&episode), None)?;

        debug!("Searching episode subtitles for IMDB ID {}", &imdb_id);
        self.execute_search_request(imdb_id, url)
            .await
    }

    async fn file_subtitles(&self, filename: &String) -> Result<Vec<SubtitleInfo>> {
        let url = self.create_search_url(None, None, Some(filename))?;

        debug!("Searching filename subtitles for {}", filename);
        self.execute_search_request(filename, url)
            .await
    }

    async fn download(&self, subtitle_info: &SubtitleInfo, matcher: &SubtitleMatcher) -> Result<Subtitle> {
        trace!("Starting subtitle download for {}", subtitle_info);
        let url = self.create_download_url()?;
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
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;

    use popcorn_fx_core::core::config::*;
    use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage::{English, French};
    use popcorn_fx_core::test::{copy_test_file, init_logger, read_test_file};

    use super::*;

    fn start_mock_server() -> (MockServer, Arc<Application>) {
        let server = MockServer::start();
        let settings = Arc::new(Application::new(
            PopcornProperties::new(SubtitleProperties::new(
                server.url(""),
                String::new(),
                String::new(),
            )),
            PopcornSettings::default()));

        (server, settings)
    }

    #[tokio::test]
    async fn test_update_active_subtitle_should_update_the_subtitle() {
        init_logger();
        let settings = Arc::new(Application::default());
        let subtitle = Subtitle::new(vec![], Some(SubtitleInfo::none()), None);
        let mut service = OpensubtitlesService::new(&settings);

        service.update_active_subtitle(Some(subtitle.clone()));
        let result = service.active_subtitle();

        assert_eq!(result.unwrap(), &subtitle);
    }

    #[tokio::test]
    async fn test_movie_subtitles() {
        init_logger();
        let settings = Arc::new(Application::default());
        let imdb_id = "tt1156398".to_string();
        let movie = Movie::new(imdb_id.clone(), "lorem".to_string());
        let service = OpensubtitlesService::new(&settings);

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
        let movie1 = Movie::new("tt1156398".to_string(), "lorem".to_string());
        let movie2 = Movie::new("tt12003946".to_string(), "ipsum".to_string());
        let service = OpensubtitlesService::new(&settings);
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
        let show = Show::new("tt2861424".to_string(), "275274".to_string(), "Rick and Morty".to_string());
        let episode = Episode::new("tt2169080".to_string(), "Pilot".to_string(), 1, 1);
        let service = OpensubtitlesService::new(&settings);
        server.mock(|when, then| {
            when.method(GET)
                .path("/subtitles")
                .query_param(IMDB_ID_PARAM_KEY, "2861424".to_string());
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
        let settings = Arc::new(Application::default());
        let filename = "House.of.the.Dragon.S01E01.HMAX.WEBRip.x264-XEN0N.mkv".to_string();
        let service = OpensubtitlesService::new(&settings);

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
        let temp_dir = tempfile::tempdir().unwrap();
        let popcorn_settings = PopcornSettings::new(
            SubtitleSettings::new(
                temp_dir.into_path().into_os_string().into_string().unwrap(),
                false,
                English,
                SubtitleFamily::Arial,
            ),
            UiSettings::new(
                "en".to_string(),
                UiScale::new(1f32).expect("Expected ui scale to be valid"),
                StartScreen::Movies,
                false,
                false,
            ),
        );
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let service = OpensubtitlesService::new(&settings);
        let subtitle_info = SubtitleInfo::new_with_files("tt00001".to_string(), SubtitleLanguage::German, vec![
            SubtitleFile::new(91135, "test-subtitle-file.srt".to_string(), String::new(), 0.0, 0)
        ]);
        let matcher = SubtitleMatcher::new(Some(String::new()), Some(String::from("720")));

        let result = service.download(&subtitle_info, &matcher)
            .await
            .unwrap();

        assert_eq!(&subtitle_info, result.info().unwrap());
        assert!(result.file().is_some(), "Expected a file to have been downloaded and parsed")
    }

    #[tokio::test]
    async fn test_download_when_subtitle_file_exists_should_return_existing_file() {
        init_logger();
        let test_file = "subtitle_existing.srt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path().into_os_string().into_string().unwrap();
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
        ));
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let destination = copy_test_file(temp_path.clone().as_str(), test_file);
        let service = OpensubtitlesService::new(&settings);
        let subtitle_info = SubtitleInfo::new_with_files("tt00001".to_string(), SubtitleLanguage::German, vec![
            SubtitleFile::new(10001111, "subtitle_existing.srt".to_string(), String::new(), 0.0, 0)
        ]);
        let matcher = SubtitleMatcher::new(Some(String::new()), Some(String::from("720")));
        let expected_cues: Vec<SubtitleCue> = vec![
            SubtitleCue::new("1".to_string(), 8224, 10124, vec![
                SubtitleLine::new(vec![StyledText::new("Okay, if no one else will say it, I will.".to_string(), false, false, false)])
            ])
        ];
        let expected_result = Subtitle::new(expected_cues.clone(), Some(subtitle_info.clone()), Some(destination.clone()));

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
        let service = OpensubtitlesService::new(&settings);
        let destination = copy_test_file(temp_dir.into_path().to_str().unwrap(), test_file);
        let expected_result = Subtitle::new(vec![
            SubtitleCue::new("1".to_string(), 0, 0, vec![SubtitleLine::new(vec![StyledText::new("Drink up, me hearties, yo ho".to_string(), true, false, false)])])
        ], None, Some(destination.clone()));

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
        ), UiSettings::default());
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let service = OpensubtitlesService::new(&settings);
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
        ));
        let settings = Arc::new(Application::new(PopcornProperties::default(), popcorn_settings));
        let service = OpensubtitlesService::new(&settings);
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

        let result = OpensubtitlesService::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_missing_extension_in_release() {
        init_logger();
        let file = OpenSubtitlesFile::new(687);
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "lorem".to_string());
        let expected_result = "lorem.srt".to_string();

        let result = OpensubtitlesService::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_too_long_extension() {
        init_logger();
        let file = OpenSubtitlesFile::new_with_filename(0, "lorem.XviD-DEViSE".to_string());
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "".to_string());
        let expected_result = "lorem.XviD-DEViSE.srt".to_string();

        let result = OpensubtitlesService::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_file_name_too_short_extension() {
        init_logger();
        let file = OpenSubtitlesFile::new_with_filename(0, "lorem.en".to_string());
        let attributes = OpenSubtitlesAttributes::new("123".to_string(), "".to_string());
        let expected_result = "lorem.en.srt".to_string();

        let result = OpensubtitlesService::subtitle_file_name(&file, &attributes);

        assert_eq!(expected_result, result)
    }
}