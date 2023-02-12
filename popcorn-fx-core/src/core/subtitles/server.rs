use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use local_ip_address::local_ip;
use log::{debug, error, info, trace, warn};
use reqwest::Url;
use tokio::sync::{Mutex, MutexGuard};
use warp::{Filter, Rejection};
use warp::http::{HeaderValue, Response};
use warp::http::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_DISPOSITION, CONTENT_TYPE};

use crate::core::subtitles;
use crate::core::subtitles::{SubtitleError, SubtitleProvider};
use crate::core::subtitles::model::{Subtitle, SubtitleType};

const SERVER_PROTOCOL: &str = "http";
const SERVER_SUBTITLE_PATH: &str = "subtitle";

/// The subtitle server state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ServerState {
    Stopped,
    Running,
    Error,
}

/// The subtitle server is responsible for serving [Subtitle]'s over http.
pub struct SubtitleServer {
    runtime: tokio::runtime::Runtime,
    socket: Arc<SocketAddr>,
    subtitles: Arc<Mutex<HashMap<String, DataHolder>>>,
    provider: Arc<Box<dyn SubtitleProvider>>,
    state: Arc<Mutex<Option<ServerState>>>,
}

impl SubtitleServer {
    pub fn new(provider: &Arc<Box<dyn SubtitleProvider>>) -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("expected a new runtime");
        let listener = TcpListener::bind("0.0.0.0:0").expect("expected a TCP address to be bound");
        let socket = listener.local_addr().expect("expected a valid socket");
        let ip = local_ip().expect("expected an ip address from a network interface");
        let port = socket.port();

        let instance = Self {
            runtime,
            socket: Arc::new(SocketAddr::new(ip, port)),
            subtitles: Arc::new(Mutex::new(HashMap::new())),
            provider: provider.clone(),
            state: Arc::new(Mutex::new(Some(ServerState::Stopped))),
        };

        instance.start_subtitle_server();
        instance
    }

    /// Serve the given [Subtitle] as a raw format over HTTP.
    ///
    /// It returns the served url on success, else the error.
    pub fn serve(&self, subtitle: Subtitle, serving_type: SubtitleType) -> subtitles::Result<String> {
        trace!("Trying to service subtitle type {} for {}", &serving_type, &subtitle);
        let filename = Path::new(subtitle.file()).file_stem()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string());

        match filename {
            None => Err(SubtitleError::InvalidFile(subtitle.file().clone(), "no extension".to_string())),
            Some(base_name) => self.subtitle_to_serving_url(base_name, subtitle, serving_type)
        }
    }

    /// Retrieve the current state of the subtitle server.
    ///
    /// It returns the state of the server.
    pub fn state(&self) -> ServerState {
        let state = self.state.clone();
        let state_lock = futures::executor::block_on(state.lock());

        match state_lock.as_ref() {
            None => {
                warn!("Server state couldn't be retrieved, subtitle server state should always be present");
                ServerState::Stopped
            }
            Some(e) => e.clone()
        }
    }

    fn start_subtitle_server(&self) {
        let subtitles = self.subtitles.clone();
        let socket = self.socket.clone();
        let state = self.state.clone();

        self.runtime.spawn(async move {
            let routes = warp::get()
                .and(warp::path!("subtitle" / String))
                .and_then(move |subtitle: String| {
                    let subtitle = percent_encoding::percent_decode(subtitle.as_bytes())
                        .decode_utf8()
                        .expect("expected a valid utf8 value")
                        .to_string();
                    let subtitles = subtitles.clone();
                    trace!("Handling request for subtitle filename {}", &subtitle);

                    async move {
                        let subtitles = subtitles.lock().await;
                        Self::handle_subtitle_request(subtitles, subtitle)
                    }
                })
                .with(warp::cors().allow_any_origin());
            let socket = socket.clone();

            trace!("Starting subtitle server on {}:{}", socket.ip(), socket.port());
            let server = warp::serve(routes);
            let mut state_lock = state.lock().await;

            match server.try_bind_ephemeral((socket.ip(), socket.port())) {
                Ok((_, e)) => {
                    debug!("Subtitle server is running on {}:{}", socket.ip(), socket.port());
                    let _ = state_lock.borrow_mut().insert(ServerState::Running);
                    drop(state_lock);
                    e.await
                }
                Err(e) => {
                    error!("Failed to start subtitle server, {}", e);
                    let _ = state_lock.borrow_mut().insert(ServerState::Error);
                }
            }
        });
    }

    fn subtitle_to_serving_url(&self, filename_base: String, subtitle: Subtitle, serving_type: SubtitleType) -> subtitles::Result<String> {
        match self.provider.convert(subtitle, serving_type.clone()) {
            Ok(data) => {
                debug!("Converted subtitle for serving");
                let mutex = self.subtitles.clone();
                let filename_full = format!("{}.{}", filename_base, &serving_type.extension());
                let url = self.build_url(&filename_full);

                match url {
                    Ok(result) => {
                        let execute = async move {
                            let mut subtitles = mutex.lock().await;
                            subtitles.insert(filename_full.clone(), DataHolder::new(data, serving_type.clone()));
                            debug!("Registered new subtitle entry {}", filename_full);
                        };

                        self.runtime.block_on(execute);

                        info!("Serving new subtitle url {}", &result);
                        Ok(result.to_string())
                    }
                    Err(e) => Err(SubtitleError::ParseUrlError(e.to_string()))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn build_url(&self, filename_full: &str) -> Result<Url, url::ParseError> {
        let host = format!("{}://{}", SERVER_PROTOCOL, self.socket);
        let path = format!("{}/{}", SERVER_SUBTITLE_PATH, filename_full);
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }

    /// Handle a request send to the subtitle server for the given filename.
    /// It takes a lock on the subtitles and the filename to verify the validity of the request.
    ///
    /// * `subtitles`   - the locked subtitles
    /// * `filename`    - the filename which is requested to being served.
    ///
    /// If the filename isn't being served, it will return a `404`.
    fn handle_subtitle_request(subtitles: MutexGuard<HashMap<String, DataHolder>>, filename: String) -> Result<Response<String>, Rejection> {
        match subtitles.get(filename.as_str()) {
            None => Err(warp::reject()),
            Some(e) => {
                let content_type = format!("{}; charset=utf-8", e.data_type.content_type());
                let header_value = HeaderValue::from_bytes(content_type.as_bytes()).expect("expected a valid header value");
                let mut response = Response::new(e.data());
                let headers = response.headers_mut();

                headers.insert(CONTENT_TYPE, header_value);
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
                headers.insert(ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET,HEAD"));
                headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(CONTENT_TYPE.as_str()));
                headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static(""));

                debug!("Handled subtitle request for {}", filename);
                Ok(response)
            }
        }
    }
}

unsafe impl Send for SubtitleServer {}

unsafe impl Sync for SubtitleServer {}

/// Holds the raw format data of a [Subtitle] with additional information.
pub struct DataHolder {
    data: String,
    data_type: SubtitleType,
}

impl DataHolder {
    fn new(data: String, data_type: SubtitleType) -> Self {
        Self {
            data,
            data_type,
        }
    }

    /// Retrieve a copy of the raw data.
    pub fn data(&self) -> String {
        self.data.clone()
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;

    use reqwest::{Client, Url};
    use reqwest::header::CONTENT_TYPE;

    use crate::core::subtitles::MockSubtitleProvider;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_state() {
        init_logger();
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let arc = Arc::new(provider as Box<dyn SubtitleProvider>);
        let server = SubtitleServer::new(&arc);

        let result = server.state();

        assert_eq!(ServerState::Stopped, result)
    }

    #[test]
    fn test_subtitle_is_served() {
        init_logger();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let subtitle = Subtitle::new(
            vec![],
            None,
            "my-subtitle - heavy.srt".to_string(),
        );
        let client = Client::builder().build().expect("Client should have been created");
        provider.expect_convert()
            .returning(|_: Subtitle, _: SubtitleType| -> subtitles::Result<String> {
                Ok("lorem ipsum".to_string())
            });
        let arc = Arc::new(provider as Box<dyn SubtitleProvider>);
        let server = SubtitleServer::new(&arc);

        wait_for_server(&server);
        let serving_url = server.serve(subtitle, SubtitleType::Vtt)
            .expect("expected the subtitle to be served");

        let (content_type, body) = runtime.block_on(async {
            let response = client.get(Url::parse(serving_url.as_str()).unwrap())
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                let headers = response.headers().clone();
                let content_type = headers.get(CONTENT_TYPE).expect("expected the content type within the response");
                let body = response.text()
                    .await
                    .expect("expected a string body");

                (content_type.clone(), body)
            } else {
                panic!("invalid response received with status {}", response.status().as_u16())
            }
        });

        assert_eq!(String::from("lorem ipsum"), body);
        assert_eq!("text/vtt; charset=utf-8", content_type.to_str().unwrap())
    }

    #[test]
    fn test_subtitle_not_being_served() {
        init_logger();
        let filename = "lorem.srt";
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let client = Client::builder().build().expect("Client should have been created");
        let arc = Arc::new(provider as Box<dyn SubtitleProvider>);
        let server = SubtitleServer::new(&arc);

        wait_for_server(&server);
        let serving_url = server.build_url(filename).unwrap();

        let status_code = runtime.block_on(async move {
            client.get(serving_url)
                .send()
                .await
                .expect("expected a response")
                .status()
        });

        assert_eq!(404, status_code.as_u16(), "expected the subtitle to not have been found")
    }

    #[test]
    fn test_build_url_escape_characters() {
        init_logger();
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let arc = Arc::new(provider as Box<dyn SubtitleProvider>);
        let server = SubtitleServer::new(&arc);
        let expected_result = format!("{}://{}/{}/Lorem.S01E16%20720p%20-%20Heavy.vtt",
                                      SERVER_PROTOCOL,
                                      server.socket.to_string(),
                                      SERVER_SUBTITLE_PATH);

        let result = server.build_url("Lorem.S01E16 720p - Heavy.vtt").unwrap();

        assert_eq!(expected_result, result.to_string())
    }

    fn wait_for_server(server: &SubtitleServer) {
        while server.state() == ServerState::Stopped {
            info!("Waiting for subtitle server to be started");
            thread::sleep(Duration::from_millis(50))
        }
    }
}