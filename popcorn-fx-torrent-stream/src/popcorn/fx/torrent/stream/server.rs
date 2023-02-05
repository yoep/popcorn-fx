use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use futures::TryFutureExt;
use local_ip_address::local_ip;
use log::{debug, error, trace, warn};
use tokio::sync::{Mutex, MutexGuard};
use url::Url;
use warp::{Filter, hyper, Rejection, Reply, Stream};
use warp::http::{header, HeaderValue, Response, StatusCode};
use warp::http::header::USER_AGENT;
use warp::hyper::HeaderMap;

use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{Torrent, TorrentError, TorrentStream, TorrentStreamingResource, TorrentStreamServer};

use crate::popcorn::fx::torrent::stream::DefaultTorrentStream;

const SERVER_PROTOCOL: &str = "http";
const SERVER_VIDEO_PATH: &str = "video";
const USER_AGENT_JAVA: &str = "Java";

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
    streams: Arc<Mutex<HashMap<String, DefaultTorrentStream>>>,
    state: Arc<Mutex<TorrentStreamServerState>>,
}

impl DefaultTorrentStreamServer {
    pub fn state(&self) -> TorrentStreamServerState {
        let mutex = futures::executor::block_on(self.state.lock());
        mutex.clone()
    }

    fn start_server(&self) {
        let streams = self.streams.clone();
        let socket = self.socket.clone();
        let state = self.state.clone();

        self.runtime.spawn(async move {
            let routes = warp::get()
                .and(warp::path!("video" / String))
                .and(warp::filters::header::headers_cloned())
                .and_then(move |filename: String, headers: HeaderMap| {
                    let filename = percent_encoding::percent_decode(filename.as_bytes())
                        .decode_utf8()
                        .expect("expected a valid utf8 value")
                        .to_string();
                    let streams = streams.clone();

                    async move {
                        let mutex = streams.lock().await;
                        Self::handle_video_request(mutex, filename.as_str(), headers)
                    }
                });

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

    fn handle_video_request(mutex: MutexGuard<HashMap<String, DefaultTorrentStream>>, filename: &str, headers: HeaderMap) -> Result<impl Reply, Rejection> {
        trace!("Handling video request for {}", filename);
        match mutex.get(filename) {
            None => {
                warn!("Torrent stream not found for {}", filename);
                Err(warp::reject())
            }
            Some(torrent_stream) => {
                match torrent_stream.stream() {
                    Ok(stream) => Self::handle_video_stream(stream, filename, headers),
                    Err(e) => {
                        error!("Failed to start stream for {}, {}", filename, e);
                        Err(warp::reject())
                    }
                }
            }
        }
    }

    fn handle_video_stream(stream: TorrentStreamingResource, filename: &str, headers: HeaderMap) -> Result<impl Reply + Sized, Rejection> {
        let agent = headers.get(USER_AGENT);
        let mut status = StatusCode::PARTIAL_CONTENT;
        let mut response = Response::new(hyper::Body::wrap_stream(stream));

        match agent {
            None => {}
            Some(agent) => {
                Self::handle_user_agent(agent, &mut status, filename);
            }
        }

        *response.status_mut() = status;

        Ok(response)
    }

    fn handle_user_agent(agent: &HeaderValue, status: &mut StatusCode, filename: &str) {
        match agent.to_str() {
            Ok(e) => {
                if e == USER_AGENT_JAVA {
                    *status = StatusCode::OK;
                    debug!("Detected {} user agent, using status {} instead", USER_AGENT_JAVA, &status);
                }
            }
            Err(e) => warn!("User agent value is invalid for {}", filename)
        }
    }

    fn extract_range(headers: &HeaderMap) -> Option<()> {
        match headers.get(header::RANGE) {
            None => None,
            Some(value) => {
                None
            }
        }
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
    fn start_stream(&self, torrent: Box<dyn Torrent>) -> torrent::Result<Url> {
        trace!("Creating new torrent stream for {:?}", torrent);
        let streams = self.streams.clone();
        let filepath = torrent.file();
        let filename = filepath.file_name()
            .expect("expected a valid filename")
            .to_str()
            .unwrap();

        match self.build_url(filename) {
            Ok(url) => {
                debug!("Starting url stream for {}", &url);
                futures::executor::block_on(async move {
                    let mut mutex = streams.lock().await;
                    let stream = DefaultTorrentStream::new(url, torrent);
                    let url = stream.url();

                    mutex.insert(filename.to_string(), stream);

                    Ok(url)
                })
            }
            Err(e) => {
                warn!("Torrent stream url creation failed, {}", e);
                Err(TorrentError::InvalidUrl(filepath.to_str().unwrap().to_string()))
            }
        }
    }

    fn stop_stream(&self, stream: Box<&dyn TorrentStream>) {
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
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use crate::popcorn::fx::torrent::stream::TorrentStreamServerState::Stopped;

    use super::*;

    #[test]
    fn test_start_stream() {
        init_logger();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("example.mp4");
        let client = Client::builder().build().expect("Client should have been created");
        let server = DefaultTorrentStreamServer::default();
        let mut torrent = MockTorrent::new();
        torrent.expect_file()
            .returning(move || file.clone());
        torrent.expect_has_byte()
            .returning(|e| true);
        copy_test_file(temp_dir.path().to_str().unwrap(), "example.mp4");

        wait_for_server(&server);
        let stream = server.start_stream(Box::new(torrent) as Box<dyn Torrent>)
            .expect("expected the torrent stream to have started");
        let body = runtime.block_on(async {
            let response = client.get(stream)
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
    }

    fn wait_for_server(server: &DefaultTorrentStreamServer) {
        while server.state() == Stopped {
            info!("Waiting for torrent stream server to be started");
            thread::sleep(Duration::from_millis(50))
        }
    }
}