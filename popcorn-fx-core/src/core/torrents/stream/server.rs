use crate::core::torrents;
use crate::core::torrents::stream::torrent_stream::DefaultTorrentStream;
use crate::core::torrents::stream::{MediaType, MediaTypeFactory, Range};
use crate::core::torrents::{
    Error, Torrent, TorrentStream, TorrentStreamEvent, TorrentStreamServer,
    TorrentStreamServerState,
};
use crate::core::utils::network::available_socket;
use async_trait::async_trait;
use fx_callback::{Callback, Subscription};
use fx_handle::Handle;
use hyper::Body;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;
use url::Url;
use warp::http::header::{
    ACCEPT_RANGES, CONNECTION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT,
};
use warp::http::{HeaderValue, Response, StatusCode};
use warp::hyper::HeaderMap;
use warp::{hyper, Filter, Rejection};

const SERVER_PROTOCOL: &str = "http";
const SERVER_VIDEO_PATH: &str = "video";
const USER_AGENT_JAVA: &str = "Java";
const ACCEPT_RANGES_TYPE: &str = "bytes";
const CONNECTION_TYPE: &str = "Keep-Alive";
const HEADER_DLNA_TRANSFER_MODE: &str = "transferMode.dlna.org";
const HEADER_DLNA_REAL_TIME_INFO: &str = "realTimeInfo.dlna.org";
const HEADER_DLNA_CONTENT_FEATURES: &str = "contentFeatures.dlna.org";
const DLNA_TRANSFER_MODE_TYPE: &str = "Streaming";
const DLNA_REAL_TIME_TYPE: &str = "DLNA.ORG_TLAG=*";
const DLNA_CONTENT_FEATURES: &str =
    "DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01100000000000000000000000000000";
const PLAIN_TEXT_TYPE: &str = "text/plain";

/// The active streams type of the stream server.
/// This is a map between the hosted url path and the underlying torrent stream.
type Streams = HashMap<String, Box<dyn TorrentStream>>;

/// The default server implementation for streaming torrents over HTTP.
#[derive(Debug, Clone)]
pub struct FXTorrentStreamServer {
    inner: Arc<TorrentStreamServerInner>,
}

impl FXTorrentStreamServer {
    pub fn new() -> Self {
        let inner = Arc::new(TorrentStreamServerInner::new());

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(&inner_main).await;
        });

        Self { inner }
    }
}

impl Drop for FXTorrentStreamServer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[async_trait]
impl TorrentStreamServer for FXTorrentStreamServer {
    async fn state(&self) -> TorrentStreamServerState {
        self.inner.state().await
    }

    async fn start_stream(
        &self,
        torrent: Box<dyn Torrent>,
        filename: &str,
    ) -> torrents::Result<Box<dyn TorrentStream>> {
        self.inner.start_stream(torrent, filename).await
    }

    async fn stop_stream(&self, handle: Handle) {
        self.inner.stop_stream(handle).await
    }

    async fn subscribe(&self, handle: Handle) -> Option<Subscription<TorrentStreamEvent>> {
        self.inner.subscribe(handle).await
    }
}

#[derive(Debug)]
struct TorrentStreamServerInner {
    socket: Arc<SocketAddr>,
    streams: Arc<Mutex<Streams>>,
    state: Arc<Mutex<TorrentStreamServerState>>,
    media_type_factory: Arc<MediaTypeFactory>,
    cancellation_token: CancellationToken,
}

impl TorrentStreamServerInner {
    fn new() -> Self {
        let socket = available_socket();

        Self {
            socket: Arc::new(socket),
            streams: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(TorrentStreamServerState::Stopped)),
            media_type_factory: Arc::new(MediaTypeFactory::default()),
            cancellation_token: Default::default(),
        }
    }

    async fn start(&self, instance: &Arc<TorrentStreamServerInner>) {
        trace!("Starting torrent stream server");
        let instance_get = instance.clone();
        let instance_head = instance.clone();
        let get = warp::get()
            .and(warp::path!("video" / String))
            .and(warp::filters::header::headers_cloned())
            .and_then(move |filename: String, headers: HeaderMap| {
                let filename = Self::url_decode(filename.as_str());
                let streams = instance_get.streams.clone();
                let factory = instance_get.media_type_factory.clone();

                async move {
                    let mutex = streams.lock().await;
                    Self::handle_video_request(mutex, factory, filename.as_str(), headers).await
                }
            });
        let head =
            warp::head()
                .and(warp::path!("video" / String))
                .and_then(move |filename: String| {
                    let filename = Self::url_decode(filename.as_str());
                    let streams = instance_head.streams.clone();
                    let factory = instance_head.media_type_factory.clone();

                    async move {
                        let mutex = streams.lock().await;
                        Self::handle_video_metadata_request(mutex, factory, filename.as_str()).await
                    }
                });
        let routes = get.or(head).with(warp::cors().allow_any_origin());

        let server = warp::serve(routes);
        let mut state_lock = instance.state.lock().await;
        let socket = instance.socket.clone();

        trace!("Binding torrent stream to socket {:?}", socket);
        match server.try_bind_ephemeral((socket.ip(), socket.port())) {
            Ok((_, e)) => {
                info!(
                    "Torrent stream server is running on {}:{}",
                    socket.ip(),
                    socket.port()
                );
                *state_lock = TorrentStreamServerState::Running;
                drop(state_lock);
                e.await
            }
            Err(e) => {
                error!("Failed to start torrent stream server, {}", e);
                *state_lock = TorrentStreamServerState::Error;
            }
        }
    }

    async fn handle_video_request(
        mutex: MutexGuard<'_, Streams>,
        media_type_factory: Arc<MediaTypeFactory>,
        filename: &str,
        headers: HeaderMap,
    ) -> Result<warp::reply::Response, Rejection> {
        match mutex.get(filename) {
            None => {
                warn!("Torrent stream not found for {}", filename);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("expected a valid response body"))
            }
            Some(torrent_stream) => {
                let range = Self::extract_range(&headers);
                trace!(
                    "Handling video stream request for {} with range {}",
                    filename,
                    range
                        .as_ref()
                        .map(|e| e.to_string())
                        .or_else(|| Some("unknown".to_string()))
                        .unwrap()
                );
                let stream = match range {
                    None => torrent_stream.stream().await,
                    Some(e) => torrent_stream.stream_offset(e.start, e.end).await,
                };

                match stream {
                    Ok(stream) => {
                        let agent = headers.get(USER_AGENT);
                        let resource = stream.resource();
                        let video_length = resource.total_length();
                        let content_range = resource.content_range().to_string();
                        let mut status = StatusCode::PARTIAL_CONTENT;
                        let media_type =
                            media_type_factory.media_type(filename).unwrap_or_else(|e| {
                                warn!("Unable to parse media type, {}", e);
                                MediaType::octet_stream()
                            });

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
                            .header(HEADER_DLNA_REAL_TIME_INFO, DLNA_REAL_TIME_TYPE)
                            .header(HEADER_DLNA_CONTENT_FEATURES, DLNA_CONTENT_FEATURES)
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
                            .expect("expected a valid response body"))
                    }
                }
            }
        }
    }

    async fn handle_video_metadata_request(
        mutex: MutexGuard<'_, Streams>,
        media_type_factory: Arc<MediaTypeFactory>,
        filename: &str,
    ) -> Result<warp::reply::Response, Rejection> {
        trace!("Handling video request for {}", filename);
        match mutex.get(filename) {
            None => {
                warn!("Failed to find metadata of stream {}", filename);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("expected a valid response body"))
            }
            Some(torrent_stream) => match torrent_stream.stream().await {
                Ok(stream) => {
                    let resource = stream.resource();
                    let content_range = resource.content_range();
                    let media_type = media_type_factory.media_type(filename).unwrap_or_else(|e| {
                        warn!("Unable to parse media type, {}", e);
                        MediaType::octet_stream()
                    });

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
                        .expect("expected a valid response body"))
                }
            },
        }
    }

    fn handle_user_agent(agent: &HeaderValue, status: &mut StatusCode, filename: &str) {
        match agent.to_str() {
            Ok(e) => {
                if e == USER_AGENT_JAVA {
                    *status = StatusCode::OK;
                    debug!(
                        "Detected {} user agent, using status {} instead",
                        USER_AGENT_JAVA, &status
                    );
                }
            }
            Err(_) => warn!("User agent value is invalid for {}", filename),
        }
    }

    fn extract_range(headers: &HeaderMap) -> Option<Range> {
        match headers.get(RANGE) {
            None => None,
            Some(value) => match Range::parse(value.to_str().expect("Expected a value string")) {
                Ok(e) => Some(e.first().unwrap().clone()),
                Err(e) => {
                    error!("Range header is invalid, {}", e);
                    None
                }
            },
        }
    }

    /// The response for when the requested [Range] couldn't be satisfied.
    /// This is used mostly when the requested range is out of bounds for the streaming resource.
    fn request_not_satisfiable_response() -> Response<Body> {
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
        let path = format!("{}/{}", SERVER_VIDEO_PATH, Self::url_encode(filename));
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }

    /// Encode the given filename to be compatible with the url specification.
    fn url_encode(filename: &str) -> String {
        url::form_urlencoded::byte_serialize(filename.as_bytes()).collect::<String>()
    }

    /// Decode the given url filename back to it's original name.
    fn url_decode(filename: &str) -> String {
        url::form_urlencoded::parse(filename.as_bytes())
            .map(|(key, value)| key.to_string() + value.as_ref())
            .join("")
    }
}

#[async_trait]
impl TorrentStreamServer for TorrentStreamServerInner {
    async fn state(&self) -> TorrentStreamServerState {
        let mutex = self.state.lock().await;
        mutex.clone()
    }

    async fn start_stream(
        &self,
        torrent: Box<dyn Torrent>,
        filename: &str,
    ) -> torrents::Result<Box<dyn TorrentStream>> {
        let mut streams = self.streams.lock().await;
        let files = torrent.files().await;
        let handle = torrent.handle();

        // check if the handle is still valid and the files could be successfully retrieved
        if files.is_empty() {
            return Err(Error::InvalidHandle(handle.to_string()));
        }

        // try to find the specified filename within the torrent
        if let Some(file) = files
            .into_iter()
            .find(|e| e.filename().to_lowercase() == filename.to_lowercase())
        {
            let filename = file.filename();
            let filepath = torrent.absolute_file_path(&file).await;

            if streams.contains_key(&filename) {
                debug!(
                    "Torrent stream already exists for {}, ignoring stream creation",
                    filename
                );

                return streams
                    .get(&filename)
                    .and_then(|e| {
                        e.downcast_ref::<DefaultTorrentStream>()
                            .map(|e| Box::new(e.clone()) as Box<dyn TorrentStream>)
                    })
                    .ok_or(Error::InvalidHandle(handle.to_string()));
            }

            trace!("Creating new torrent stream for {:?}", torrent);
            match self.build_url(&filename) {
                Ok(url) => {
                    debug!("Starting url stream for {}", &url);
                    let stream = DefaultTorrentStream::new(url, torrent, &filename).await;
                    let stream_borrowed = stream.clone();
                    streams.insert(filename.to_string(), Box::new(stream));
                    Ok(Box::new(stream_borrowed))
                }
                Err(e) => {
                    warn!("Torrent stream url creation failed, {}", e);
                    Err(Error::InvalidUrl(
                        filepath
                            .to_str()
                            .map(|e| e.to_string())
                            .unwrap_or("INVALID".to_string()),
                    ))
                }
            }
        } else {
            Err(Error::FileNotFound(filename.to_string()))
        }
    }

    async fn stop_stream(&self, handle: Handle) {
        trace!("Stopping torrent stream handle {}", handle);
        let mut mutex = self.streams.lock().await;

        if let Some(filename) = mutex
            .iter()
            .find(|(_, e)| e.handle() == handle)
            .map(|(filename, _)| filename.clone())
        {
            debug!("Trying to stop stream of {}", filename);
            match mutex.remove(filename.as_str()) {
                None => warn!("Unable to stop stream of {}, stream not found", filename),
                Some(stream) => {
                    stream.stop_stream();
                    info!("Stream {} has been stopped", stream.url())
                }
            }
        }
    }

    async fn subscribe(&self, handle: Handle) -> Option<Subscription<TorrentStreamEvent>> {
        let mutex = self.streams.lock().await;
        let position = mutex.iter().position(|(_, e)| e.handle() == handle);

        if let Some((_, stream)) = position.and_then(|e| mutex.iter().nth(e)) {
            debug!("Subscribing callback to stream handle {}", handle);
            return Some(Callback::<TorrentStreamEvent>::subscribe(&**stream));
        }

        warn!("Unable to subscribe to {}, stream handle not found", handle);
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::core::torrents::{MockTorrent, TorrentHandle, TorrentState, TorrentStreamState};
    use crate::testing::{copy_test_file, read_test_file_to_bytes, read_test_file_to_string};
    use crate::{assert_timeout, assert_timeout_eq, init_logger};

    use fx_callback::MultiThreadedCallback;
    use fx_torrent;
    use fx_torrent::{Metrics, PieceIndex, PiecePriority, TorrentEvent, TorrentFileInfo};
    use reqwest::Client;
    use std::path::PathBuf;
    use std::time::Duration;

    #[tokio::test]
    async fn test_stream_metadata_info() {
        init_logger!();
        let filename = "large-[123].txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let file = temp_dir.path().join(filename);
        let file_len = read_test_file_to_bytes(filename).len() as u64;
        let torrent_files = create_torrent_files(&file, file_len);
        let total_pieces = torrent_files[0].pieces.len();
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let server = FXTorrentStreamServer::new();
        let callback = MultiThreadedCallback::<TorrentEvent>::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent
            .expect_files()
            .returning(move || torrent_files.clone());
        torrent.expect_has_bytes().return_const(true);
        torrent.expect_has_piece().returning(|_: usize| true);
        torrent.expect_total_pieces().return_const(total_pieces);
        torrent
            .expect_absolute_file_path()
            .returning(move |file| temp_dir_path.join(&file.torrent_path));
        torrent
            .expect_prioritize_pieces()
            .returning(|_: &[PieceIndex]| {});
        torrent.expect_sequential_mode().returning(|| {});
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent
            .expect_stats()
            .return_const(create_incomplete_stats());
        torrent.expect_piece_priorities().returning(move || {
            (0..total_pieces)
                .into_iter()
                .map(|idx| (idx, PiecePriority::Normal))
                .collect()
        });
        let subscribe_callback = callback.clone();
        torrent
            .expect_subscribe()
            .returning(move || subscribe_callback.subscribe());
        let torrent = Box::new(torrent) as Box<dyn Torrent>;
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);

        assert_timeout_eq!(
            Duration::from_millis(500),
            TorrentStreamServerState::Running,
            server.state().await
        );
        let stream = server
            .start_stream(torrent, filename)
            .await
            .expect("expected the torrent stream to have started");

        // inform about the pieces that have been completed
        for i in 0..total_pieces {
            callback.invoke(TorrentEvent::PieceCompleted(i));
        }
        assert_timeout!(
            Duration::from_millis(250),
            stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to have been streaming"
        );

        assert_eq!("/video/large-%5B123%5D.txt", stream.url().path());
        let result = async {
            let response = client
                .head(stream.url())
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                response.headers().clone()
            } else {
                panic!(
                    "invalid response received with status {}",
                    response.status().as_u16()
                )
            }
        }
        .await;

        assert_eq!(
            ACCEPT_RANGES_TYPE,
            result
                .get(ACCEPT_RANGES.as_str())
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(
            "text/plain",
            result.get(CONTENT_TYPE.as_str()).unwrap().to_str().unwrap()
        );
    }

    #[tokio::test]
    async fn test_stream_metadata_info_not_found() {
        init_logger!();
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let server = FXTorrentStreamServer::new();

        assert_timeout_eq!(
            Duration::from_millis(500),
            TorrentStreamServerState::Running,
            server.state().await
        );
        let result = async {
            let response = client
                .head(server.inner.build_url("lorem").unwrap())
                .send()
                .await
                .expect("expected a valid response");

            response.status()
        }
        .await;

        assert_eq!(reqwest::StatusCode::NOT_FOUND, result)
    }

    #[tokio::test]
    async fn test_start_stream() {
        init_logger!();
        let filename = "large-[123].txt";
        let total_pieces: usize = 100;
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let file = temp_dir.path().join(filename);
        let file_len = read_test_file_to_bytes(filename).len() as u64;
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let server = FXTorrentStreamServer::new();
        let callback = MultiThreadedCallback::new();
        let torrent_handle = TorrentHandle::new();
        let callback_receiver = callback.subscribe();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(torrent_handle);
        torrent
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        torrent
            .expect_files()
            .returning(move || create_torrent_files(&file, file_len));
        torrent
            .expect_absolute_file_path()
            .returning(move |file| temp_dir_path.join(&file.torrent_path));
        torrent.expect_has_bytes().return_const(true);
        torrent.expect_has_piece().returning(|_: usize| true);
        torrent.expect_total_pieces().return_const(total_pieces);
        torrent
            .expect_prioritize_pieces()
            .returning(|_: &[PieceIndex]| {});
        torrent.expect_sequential_mode().returning(|| {});
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent
            .expect_stats()
            .return_const(create_incomplete_stats());
        torrent.expect_piece_priorities().returning(move || {
            (0..total_pieces)
                .into_iter()
                .map(|idx| (idx, PiecePriority::Normal))
                .collect()
        });
        torrent.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let torrent = Box::new(torrent) as Box<dyn Torrent>;
        copy_test_file(temp_dir.path().to_str().unwrap(), filename, None);
        let expected_result = read_test_file_to_string(filename).replace("\r\n", "\n");

        assert_timeout_eq!(
            Duration::from_millis(500),
            TorrentStreamServerState::Running,
            server.state().await
        );
        let stream = server
            .start_stream(torrent, filename)
            .await
            .expect("expected the torrent stream to have started");

        // inform about the pieces that have been completed
        for i in 0..total_pieces {
            callback.invoke(TorrentEvent::PieceCompleted(i));
        }
        assert_timeout!(
            Duration::from_millis(250),
            stream.stream_state().await == TorrentStreamState::Streaming,
            "expected the stream to have been streaming"
        );

        let result = async {
            let response = client
                .get(stream.url())
                .header(RANGE.as_str(), "bytes=0-50000")
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                response.text().await.unwrap()
            } else {
                panic!(
                    "invalid response received with status {}",
                    response.status().as_u16()
                )
            }
        }
        .await;

        assert_eq!(expected_result, result.replace("\r\n", "\n"))
    }

    #[tokio::test]
    async fn test_stop_stream() {
        init_logger!();
        let filename = "large-[123].txt";
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let file_len = read_test_file_to_bytes(filename).len() as u64;
        let absolute_filepath = PathBuf::from(copy_test_file(
            temp_dir.path().to_str().unwrap(),
            filename,
            None,
        ));
        let total_pieces = 15usize;
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let callback = MultiThreadedCallback::new();
        let callback_receiver = callback.subscribe();
        let server = FXTorrentStreamServer::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent
            .expect_files()
            .returning(move || create_torrent_files(&absolute_filepath, file_len));
        torrent
            .expect_absolute_file_path()
            .returning(move |file| temp_dir_path.join(&file.torrent_path));
        torrent.expect_total_pieces().return_const(total_pieces);
        torrent
            .expect_prioritize_pieces()
            .returning(|_: &[PieceIndex]| {});
        torrent
            .expect_state()
            .return_const(TorrentState::Downloading);
        torrent
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        torrent.expect_stats().return_const(Metrics::default());
        let torrent = Box::new(torrent) as Box<dyn Torrent>;

        assert_timeout_eq!(
            Duration::from_millis(500),
            TorrentStreamServerState::Running,
            server.state().await
        );
        let stream = server
            .start_stream(torrent, filename)
            .await
            .expect("expected the torrent stream to have started");
        let stream_url = stream.url();

        server.stop_stream(stream.handle()).await;
        let result = async {
            let response = client
                .get(stream_url)
                .header(RANGE.as_str(), "bytes=0-50000")
                .send()
                .await
                .expect("expected a valid response");

            response.status()
        }
        .await;

        assert_eq!(reqwest::StatusCode::NOT_FOUND, result)
    }

    #[tokio::test]
    async fn test_stream_not_found() {
        init_logger!();
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let server = FXTorrentStreamServer::new();

        assert_timeout_eq!(
            Duration::from_millis(500),
            TorrentStreamServerState::Running,
            server.state().await
        );
        let result = async {
            let response = client
                .get(server.inner.build_url("lorem").unwrap())
                .send()
                .await
                .expect("expected a valid response");

            response.status()
        }
        .await;

        assert_eq!(reqwest::StatusCode::NOT_FOUND, result)
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(
            "lorem ipsum=[dolor].txt",
            TorrentStreamServerInner::url_decode("lorem%20ipsum%3D%5Bdolor%5D.txt")
        )
    }

    fn create_torrent_files(file: &PathBuf, length: u64) -> Vec<fx_torrent::File> {
        vec![fx_torrent::File {
            index: 0,
            torrent_path: file.clone(),
            torrent_offset: 0,
            info: TorrentFileInfo {
                length,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
            pieces: 0..100,
        }]
    }

    fn create_incomplete_stats() -> Metrics {
        let metrics = Metrics::default();
        metrics.wanted_pieces.inc_by(10);
        metrics.wanted_completed_pieces.inc_by(2);
        metrics.wanted_size.inc_by(10000);
        metrics.wanted_completed_size.inc_by(2000);
        metrics.peers.inc_by(10);
        metrics
    }
}
