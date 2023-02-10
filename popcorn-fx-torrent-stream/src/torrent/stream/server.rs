use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use futures::TryFutureExt;
use hyper::Body;
use local_ip_address::local_ip;
use log::{debug, error, info, trace, warn};
use tokio::sync::{Mutex, MutexGuard};
use url::Url;
use warp::{Filter, hyper, Rejection, Reply, Stream};
use warp::http::{header, HeaderValue, Response, StatusCode};
use warp::http::header::{ACCEPT_RANGES, CONNECTION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT};
use warp::hyper::HeaderMap;

use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{Torrent, TorrentError, TorrentStream, TorrentStreamingResource, TorrentStreamServer};

use crate::torrent::stream::{DefaultTorrentStream, MediaType, MediaTypeFactory, Range};

const SERVER_PROTOCOL: &str = "http";
const SERVER_VIDEO_PATH: &str = "video";
const USER_AGENT_JAVA: &str = "Java";
const ACCEPT_RANGES_TYPE: &str = "bytes";
const CONNECTION_TYPE: &str = "Keep-Alive";
const HEADER_DLNA_TRANSFER_MODE: &str = "TransferMode.dlna.org";
const DLNA_TRANSFER_MODE_TYPE: &str = "Streaming";
const PLAIN_TEXT_TYPE: &str = "text/plain";

/// The stream mutex type used within the server.
type StreamMutex = HashMap<String, Arc<DefaultTorrentStream>>;

/// The state of the torrent stream server.
#[derive(Debug, Clone, PartialEq)]
pub enum TorrentStreamServerState {
    Stopped,
    Running,
    Error,
}

/// The default server implementation for streaming torrents over HTTP.
#[derive(Debug)]
pub struct DefaultTorrentStreamServer {
    runtime: tokio::runtime::Runtime,
    socket: Arc<SocketAddr>,
    streams: Arc<Mutex<StreamMutex>>,
    state: Arc<Mutex<TorrentStreamServerState>>,
    media_type_factory: Arc<MediaTypeFactory>,
}

impl DefaultTorrentStreamServer {
    pub fn state(&self) -> TorrentStreamServerState {
        let mutex = futures::executor::block_on(self.state.lock());
        mutex.clone()
    }

    fn start_server(&self) {
        let streams_get = self.streams.clone();
        let factory_get = self.media_type_factory.clone();
        let streams_head = self.streams.clone();
        let factory_head = self.media_type_factory.clone();
        let socket = self.socket.clone();
        let state = self.state.clone();

        self.runtime.spawn(async move {
            let get = warp::get()
                .and(warp::path!("video" / String))
                .and(warp::filters::header::headers_cloned())
                .and_then(move |filename: String, headers: HeaderMap| {
                    let filename = percent_encoding::percent_decode(filename.as_bytes())
                        .decode_utf8()
                        .expect("expected a valid utf8 value")
                        .to_string();
                    let streams = streams_get.clone();
                    let factory = factory_get.clone();

                    async move {
                        let mutex = streams.lock().await;
                        Self::handle_video_request(mutex, factory, filename.as_str(), headers)
                    }
                });
            let head = warp::head()
                .and(warp::path!("video" / String))
                .and_then(move |filename: String| {
                    let filename = percent_encoding::percent_decode(filename.as_bytes())
                        .decode_utf8()
                        .expect("expected a valid utf8 value")
                        .to_string();
                    let streams = streams_head.clone();
                    let factory = factory_head.clone();

                    async move {
                        let mutex = streams.lock().await;
                        Self::handle_video_metadata_request(mutex, factory, filename.as_str())
                    }
                });
            let routes = get
                .or(head)
                .with(warp::cors().allow_any_origin());

            let server = warp::serve(routes);
            let mut state_lock = state.lock().await;

            match server.try_bind_ephemeral((socket.ip(), socket.port())) {
                Ok((_, e)) => {
                    debug!("Torrent stream server is running on {}:{}", socket.ip(), socket.port());
                    *state_lock = TorrentStreamServerState::Running;
                    drop(state_lock);
                    e.await
                }
                Err(e) => {
                    error!("Failed to start torrent stream server, {}", e);
                    *state_lock = TorrentStreamServerState::Error;
                }
            }
        });
    }

    fn handle_video_request(mutex: MutexGuard<StreamMutex>, media_type_factory: Arc<MediaTypeFactory>, filename: &str, headers: HeaderMap)
                            -> Result<warp::reply::Response, Rejection> {
        match mutex.get(filename) {
            None => {
                warn!("Torrent stream not found for {}", filename);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }
            Some(torrent_stream) => {
                let range = Self::extract_range(&headers);
                trace!("Handling video stream request for {} with range {}", filename, range.as_ref()
                    .map(|e|e.to_string())
                    .or_else(||Some("unknown".to_string()))
                    .unwrap());
                let stream = match range {
                    None => torrent_stream.stream(),
                    Some(e) => torrent_stream.stream_offset(e.start, e.end)
                };

                match stream {
                    Ok(stream) => {
                        let agent = headers.get(USER_AGENT);
                        let resource = stream.resource();
                        let video_length = resource.total_length();
                        let content_range = resource.content_range().to_string();
                        let mut status = StatusCode::PARTIAL_CONTENT;
                        let media_type = match media_type_factory.media_type(filename) {
                            Ok(e) => e,
                            Err(e) => {
                                warn!("Unable to parse media type, {}", e);
                                MediaType::octet_stream()
                            }
                        };

                        if resource.offset() > video_length {
                            return Ok(Self::request_not_satisfiable_response());
                        }

                        match agent {
                            None => {}
                            Some(agent) => {
                                Self::handle_user_agent(agent, &mut status, filename);
                            }
                        }

                        Ok(Response::builder()
                            .status(status)
                            .header(ACCEPT_RANGES, ACCEPT_RANGES_TYPE)
                            .header(HEADER_DLNA_TRANSFER_MODE, DLNA_TRANSFER_MODE_TYPE)
                            .header(CONTENT_RANGE, &content_range)
                            .header(RANGE, &content_range)
                            .header(CONTENT_LENGTH, resource.content_length())
                            .header(CONNECTION, CONNECTION_TYPE)
                            .header(CONTENT_TYPE, media_type)
                            .body(Body::wrap_stream(stream))
                            .unwrap())
                    }
                    Err(e) => {
                        error!("Failed to start stream for {}, {}", filename, e);
                        Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .unwrap())
                    }
                }
            }
        }
    }

    fn handle_video_metadata_request(mutex: MutexGuard<StreamMutex>, media_type_factory: Arc<MediaTypeFactory>, filename: &str)
                                     -> Result<warp::reply::Response, Rejection> {
        trace!("Handling video request for {}", filename);
        match mutex.get(filename) {
            None => {
                warn!("Failed to find metadata of stream {}", filename);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }
            Some(torrent_stream) => {
                return match torrent_stream.stream() {
                    Ok(stream) => {
                        let resource = stream.resource();
                        let content_range = resource.content_range();
                        let media_type = match media_type_factory.media_type(filename) {
                            Ok(e) => e,
                            Err(e) => {
                                warn!("Unable to parse media type, {}", e);
                                MediaType::octet_stream()
                            }
                        };

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(ACCEPT_RANGES, ACCEPT_RANGES_TYPE)
                            .header(HEADER_DLNA_TRANSFER_MODE, DLNA_TRANSFER_MODE_TYPE)
                            .header(CONTENT_RANGE, &content_range)
                            .header(CONTENT_LENGTH, resource.content_length())
                            .header(RANGE, &content_range)
                            .header(CONTENT_TYPE, media_type.to_string())
                            .body(Body::empty())
                            .expect("expected a valid response"))
                    }
                    Err(e) => {
                        error!("Failed to start metadata of stream {}, {}", filename, e);
                        Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .unwrap())
                    }
                };
            }
        }
    }

    fn handle_user_agent(agent: &HeaderValue, status: &mut StatusCode, filename: &str) {
        match agent.to_str() {
            Ok(e) => {
                if e == USER_AGENT_JAVA {
                    *status = StatusCode::OK;
                    debug!("Detected {} user agent, using status {} instead", USER_AGENT_JAVA, &status);
                }
            }
            Err(_) => warn!("User agent value is invalid for {}", filename)
        }
    }

    fn extract_range(headers: &HeaderMap) -> Option<Range> {
        match headers.get(header::RANGE) {
            None => None,
            Some(value) => {
                return match Range::parse(value.to_str().expect("Expected a value string")) {
                    Ok(e) => Some(e.first().unwrap().clone()),
                    Err(e) => {
                        error!("Range header is invalid, {}", e);
                        None
                    }
                };
            }
        }
    }

    /// The response for when the requested [Range] couldn't be satisfied.
    /// This is used mostly when the requested range is out of bounds for the streaming resource.
    fn request_not_satisfiable_response() -> Response<hyper::Body> {
        Response::builder()
            .status(StatusCode::from_u16(416).unwrap())
            .header(CONTENT_TYPE, PLAIN_TEXT_TYPE)
            .body(Body::empty())
            .unwrap()
    }

    /// Build a torrent stream url on which a new stream can be reached for the given filename.
    /// The filename should consist out of a valid name with video extension.
    /// This is done as some media players might use the url to determine the video format.
    fn build_url(&self, filename: &str) -> Result<Url, url::ParseError> {
        let host = format!("{}://{}", SERVER_PROTOCOL, self.socket);
        let path = format!("{}/{}", SERVER_VIDEO_PATH, filename);
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }
}

impl TorrentStreamServer for DefaultTorrentStreamServer {
    fn start_stream(&self, torrent: Box<dyn Torrent>) -> torrent::Result<Arc<dyn TorrentStream>> {
        let streams = self.streams.clone();
        let mut mutex = streams.blocking_lock();
        let filepath = torrent.file();
        let filename = filepath.file_name()
            .expect("expected a valid filename")
            .to_str()
            .unwrap();

        if mutex.contains_key(filename) {
            debug!("Torrent stream already exists for {}, ignoring stream creation", filename);
            return Ok(mutex.get(filename)
                .map(|e| e.clone())
                .unwrap());
        }

        trace!("Creating new torrent stream for {:?}", torrent);
        match self.build_url(filename) {
            Ok(url) => {
                debug!("Starting url stream for {}", &url);
                let stream = Arc::new(DefaultTorrentStream::new(url, torrent));
                let stream_ref = stream.clone();
                let url = stream.url();

                mutex.insert(filename.to_string(), stream);

                Ok(stream_ref)
            }
            Err(e) => {
                warn!("Torrent stream url creation failed, {}", e);
                Err(TorrentError::InvalidUrl(filepath.to_str().unwrap().to_string()))
            }
        }
    }

    fn stop_stream(&self, stream: &Arc<dyn TorrentStream>) {
        todo!()
    }
}

impl Default for DefaultTorrentStreamServer {
    fn default() -> Self {
        let listener = TcpListener::bind("0.0.0.0:0").expect("expected a TCP address to be bound");
        let socket = listener.local_addr().expect("expected a valid socket");
        let ip = local_ip().expect("expected an ip address from a network interface");
        let port = socket.port();

        let instance = Self {
            runtime: tokio::runtime::Runtime::new().expect("expected a new runtime"),
            socket: Arc::new(SocketAddr::new(ip, port)),
            streams: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(TorrentStreamServerState::Stopped)),
            media_type_factory: Arc::new(MediaTypeFactory::default()),
        };

        instance.start_server();
        instance
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;

    use log::info;
    use reqwest::Client;

    use popcorn_fx_core::core::torrent::MockTorrent;
    use popcorn_fx_core::testing::{copy_test_file, init_logger, read_test_file};

    use crate::torrent::stream::TorrentStreamServerState::Stopped;

    use super::*;

    #[test]
    fn test_stream_metadata_info() {
        init_logger();
        let filename = "large.txt";
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join(filename);
        let client = Client::builder().build().expect("Client should have been created");
        let server = DefaultTorrentStreamServer::default();
        let mut torrent = MockTorrent::new();
        torrent.expect_file()
            .returning(move || file.clone());
        torrent.expect_has_bytes()
            .return_const(true);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);

        wait_for_server(&server);
        let stream = server.start_stream(Box::new(torrent) as Box<dyn Torrent>)
            .expect("expected the torrent stream to have started");

        let result = runtime.block_on(async {
            let response = client.head(stream.url())
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                response.headers().clone()
            } else {
                panic!("invalid response received with status {}", response.status().as_u16())
            }
        });

        assert_eq!(ACCEPT_RANGES_TYPE, result.get(ACCEPT_RANGES).unwrap().to_str().unwrap());
        assert_eq!("text/plain", result.get(CONTENT_TYPE).unwrap().to_str().unwrap());
    }

    #[test]
    fn test_stream_metadata_info_not_found() {
        init_logger();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = Client::builder().build().expect("Client should have been created");
        let server = DefaultTorrentStreamServer::default();

        wait_for_server(&server);
        let result = runtime.block_on(async {
            let response = client.head(server.build_url("lorem").unwrap())
                .send()
                .await
                .expect("expected a valid response");

            response.status()
        });

        assert_eq!(StatusCode::NOT_FOUND, result)
    }

    #[test]
    fn test_start_stream() {
        init_logger();
        let filename = "large.txt";
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join(filename);
        let client = Client::builder().build().expect("Client should have been created");
        let server = DefaultTorrentStreamServer::default();
        let mut torrent = MockTorrent::new();
        torrent.expect_file()
            .returning(move || file.clone());
        torrent.expect_has_bytes()
            .return_const(true);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let expected_result = read_test_file(filename);

        wait_for_server(&server);
        let stream = server.start_stream(Box::new(torrent) as Box<dyn Torrent>)
            .expect("expected the torrent stream to have started");
        let result = runtime.block_on(async {
            let response = client.get(stream.url())
                .header(RANGE, "bytes=0-50000")
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                response.text()
                    .await
                    .unwrap()
            } else {
                panic!("invalid response received with status {}", response.status().as_u16())
            }
        });

        assert_eq!(expected_result, result.replace("\r\n", "\n"))
    }

    #[test]
    fn test_stream_not_found() {
        init_logger();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = Client::builder().build().expect("Client should have been created");
        let server = DefaultTorrentStreamServer::default();

        wait_for_server(&server);
        let result = runtime.block_on(async {
            let response = client.get(server.build_url("lorem").unwrap())
                .send()
                .await
                .expect("expected a valid response");

            response.status()
        });

        assert_eq!(StatusCode::NOT_FOUND, result)
    }

    fn wait_for_server(server: &DefaultTorrentStreamServer) {
        while server.state() == Stopped {
            info!("Waiting for torrent stream server to be started");
            thread::sleep(Duration::from_millis(50))
        }
    }
}