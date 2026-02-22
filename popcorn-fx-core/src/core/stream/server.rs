use crate::core::stream::media_type::{MediaType, MediaTypeFactory};
use crate::core::stream::range::Range;
use crate::core::stream::{Error, Result, StreamEvent, StreamState, StreamingResource};
use crate::core::utils::network::ip_addr;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{
    ACCEPT_RANGES, CONNECTION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT,
};
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, head};
use axum::{http, Router};
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, trace, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{io, result};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use url::Url;

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

/// The information about a video stream that has been started on the server.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerStream {
    /// The url to access the video stream.
    pub url: Url,
    /// The filename that is being streamed.
    pub filename: String,
}

impl ServerStream {
    /// Returns the url to access the video stream.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the filename that is being streamed.
    pub fn filename(&self) -> &str {
        &self.filename
    }
}

/// The events that can be emitted by the [FXStreamServer].
#[derive(Debug, Clone)]
pub enum StreamServerEvent {
    /// Invoked when a new stream has been started.
    StreamStarted(ServerStream),
    /// Invoked when a stream has been stopped.
    StreamStopped(String),
}

/// The server stream handling incoming video requests.
#[derive(Debug, Clone)]
pub struct StreamServer {
    inner: Arc<InnerStreamServer>,
}

impl StreamServer {
    /// Create a new streaming server instance on a random port.
    /// The port will be assigned by the OS.
    pub async fn new() -> Result<Self> {
        Self::with_port(0).await
    }

    /// Try to create a new streaming server instance on the specified port.
    /// If the port is already in use, an error will be returned.
    pub async fn with_port(port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
        let addr = (ip_addr(), listener.local_addr()?.port()).into();
        let inner = Arc::new(InnerStreamServer {
            addr,
            streams: Default::default(),
            media_type_factory: Default::default(),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let state = inner.clone();
        tokio::spawn(async move {
            let cancellation_token = state.cancellation_token.clone();
            let router = Router::new()
                .route("/video/{filename}", get(Self::do_get_video))
                .route("/video/{filename}", head(Self::do_head_video))
                .with_state(state);

            if let Err(e) = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            {
                error!("Failed to start stream server, {}", e);
            }
        });

        Ok(Self { inner })
    }

    /// Start a new stream for the given resource.
    ///
    /// If the [StreamingResource::filename] is already present within the stream server, the error [Error::AlreadyExists] will be returned.
    pub async fn start_stream<T>(&self, resource: T) -> Result<ServerStream>
    where
        T: StreamingResource + Send + 'static,
    {
        self.inner.start_stream(Box::new(resource)).await
    }

    /// Stop streaming a resource for the given filename.
    /// This is effectively a no-op when the filename is not found within the streaming resources of the server.
    pub async fn stop_stream(&self, filename: &str) {
        self.inner.stop_stream(filename).await;
    }

    /// Returns the state of the stream for the given filename.
    pub async fn state(&self, filename: &str) -> Result<StreamState> {
        match self.inner.streams.read().await.get(filename) {
            None => Err(Error::NotFound(filename.to_string())),
            Some(e) => Ok(e.state().await),
        }
    }

    /// Subscribe to the stream events for the given filename.
    /// It returns the subscription if the filename is found within the streaming resources of the server.
    pub async fn subscribe_stream(&self, filename: &str) -> Result<Subscription<StreamEvent>> {
        self.inner
            .streams
            .read()
            .await
            .get(filename)
            .map(|e| e.subscribe())
            .ok_or(Error::NotFound(filename.to_string()))
    }

    async fn do_get_video(
        State(state): State<Arc<InnerStreamServer>>,
        Path(filename): Path<String>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        match percent_encoding::percent_decode(filename.as_bytes()).decode_utf8() {
            Err(_) => (StatusCode::BAD_REQUEST, Body::empty()).into_response(),
            Ok(filename) => state.handle_video_request(&*filename, headers).await,
        }
    }

    async fn do_head_video(
        State(state): State<Arc<InnerStreamServer>>,
        Path(filename): Path<String>,
    ) -> impl IntoResponse {
        match percent_encoding::percent_decode(filename.as_bytes()).decode_utf8() {
            Err(_) => (StatusCode::BAD_REQUEST, Body::empty()).into_response(),
            Ok(filename) => state.handle_video_metadata_request(&*filename).await,
        }
    }
}

impl Callback<StreamServerEvent> for StreamServer {
    fn subscribe(&self) -> Subscription<StreamServerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<StreamServerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber);
    }
}

impl Drop for StreamServer {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            self.inner.cancellation_token.cancel();
        }
    }
}

#[derive(Debug)]
struct InnerStreamServer {
    addr: SocketAddr,
    streams: RwLock<HashMap<String, Box<dyn StreamingResource>>>,
    media_type_factory: MediaTypeFactory,
    callbacks: MultiThreadedCallback<StreamServerEvent>,
    cancellation_token: CancellationToken,
}

impl InnerStreamServer {
    async fn handle_video_request(&self, filename: &str, headers: HeaderMap) -> Response<Body> {
        match self.streams.read().await.get(filename) {
            None => {
                warn!("Torrent stream not found for {}", filename);
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap_or_else(Self::handle_internal_error)
            }
            Some(streaming_resource) => {
                let range = Self::extract_range(&headers);
                trace!(
                    "Handling video stream request for {} with range {}",
                    filename,
                    range
                        .as_ref()
                        .map(|e| e.to_string())
                        .or_else(|| Some("unknown".to_string()))
                        .unwrap_or_default()
                );
                let stream = match range {
                    None => streaming_resource.stream().await,
                    Some(e) => streaming_resource.stream_range(e.start, e.end).await,
                };

                match stream {
                    Ok(stream) => {
                        let agent = headers.get(USER_AGENT);
                        let video_length = stream.resource_len();
                        let content_range = stream.content_range().to_string();
                        let mut status = StatusCode::PARTIAL_CONTENT;
                        let media_type = self
                            .media_type_factory
                            .media_type(filename)
                            .unwrap_or_else(|e| {
                                warn!("Unable to parse media type, {}", e);
                                MediaType::octet_stream()
                            });

                        if stream.range().end as u64 > video_length {
                            return Self::request_not_satisfiable_response();
                        }

                        match agent {
                            None => {}
                            Some(agent) => {
                                Self::handle_user_agent(agent, &mut status, filename);
                            }
                        }

                        Response::builder()
                            .status(status)
                            .header(ACCEPT_RANGES, ACCEPT_RANGES_TYPE)
                            .header(HEADER_DLNA_TRANSFER_MODE, DLNA_TRANSFER_MODE_TYPE)
                            .header(HEADER_DLNA_REAL_TIME_INFO, DLNA_REAL_TIME_TYPE)
                            .header(HEADER_DLNA_CONTENT_FEATURES, DLNA_CONTENT_FEATURES)
                            .header(CONTENT_RANGE, &content_range)
                            .header(RANGE, &content_range)
                            .header(CONTENT_LENGTH, stream.range().len())
                            .header(CONNECTION, CONNECTION_TYPE)
                            .header(CONTENT_TYPE, media_type)
                            .body(Body::from_stream(Box::into_pin(stream)))
                            .unwrap_or_else(Self::handle_internal_error)
                    }
                    Err(e) => {
                        error!("Failed to start stream for {}, {}", filename, e);
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .unwrap_or_else(Self::handle_internal_error)
                    }
                }
            }
        }
    }

    async fn handle_video_metadata_request(&self, filename: &str) -> Response<Body> {
        trace!("Handling video request for {}", filename);
        match self.streams.read().await.get(filename) {
            None => {
                warn!("Failed to find metadata of stream {}", filename);
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap_or_else(Self::handle_internal_error)
            }
            Some(torrent_stream) => match torrent_stream.stream().await {
                Ok(stream) => {
                    let content_range = stream.content_range();
                    let media_type =
                        self.media_type_factory
                            .media_type(filename)
                            .unwrap_or_else(|e| {
                                warn!("Unable to parse media type, {}", e);
                                MediaType::octet_stream()
                            });

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(ACCEPT_RANGES, ACCEPT_RANGES_TYPE)
                        .header(HEADER_DLNA_TRANSFER_MODE, DLNA_TRANSFER_MODE_TYPE)
                        .header(CONTENT_RANGE, &content_range)
                        .header(CONTENT_LENGTH, stream.range().len())
                        .header(RANGE, &content_range)
                        .header(CONTENT_TYPE, media_type.to_string())
                        .body(Body::empty())
                        .unwrap_or_else(Self::handle_internal_error)
                }
                Err(e) => {
                    error!("Failed to start metadata of stream {}, {}", filename, e);
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap_or_else(Self::handle_internal_error)
                }
            },
        }
    }

    async fn start_stream(&self, resource: Box<dyn StreamingResource>) -> Result<ServerStream> {
        let filename = resource.filename();
        let mut streams = self.streams.write().await;
        if streams.contains_key(filename) {
            return Err(Error::AlreadyExists(filename.to_string()));
        }

        trace!("Creating new stream for {:?}", resource);
        let url = self
            .build_url(filename)
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        debug!("Starting {} stream on {}", filename, &url);
        let filename = filename.to_string();
        streams.insert(filename.clone(), resource);

        let stream = ServerStream { url, filename };
        self.invoke_event(StreamServerEvent::StreamStarted(stream.clone()));
        Ok(stream)
    }

    async fn stop_stream(&self, filename: &str) {
        let stream = match self.streams.write().await.remove(filename) {
            None => return,
            Some(e) => e,
        };

        stream.stop().await;
        self.invoke_event(StreamServerEvent::StreamStopped(filename.to_string()));
    }

    /// Build a stream url on which the stream can be reached for the given filename.
    /// The filename should consist out of a valid name with video extension.
    /// This is done as some media players might use the url to determine the video format.
    fn build_url(&self, filename: &str) -> result::Result<Url, url::ParseError> {
        let host = format!("{}://{}", SERVER_PROTOCOL, self.addr);
        let path = format!("{}/{}", SERVER_VIDEO_PATH, Self::url_encode(filename));
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }

    /// Invoke the given event on the callback thread.
    fn invoke_event(&self, event: StreamServerEvent) {
        self.callbacks.invoke(event);
    }

    /// Handle the status of the video being served based on the user agent.
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

    /// Try to extract the range header from the given headers.
    /// If the header is not present or invalid, `None` will be returned.
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

    /// Encode the given filename to be compatible with the url specification.
    fn url_encode(filename: &str) -> String {
        url::form_urlencoded::byte_serialize(filename.as_bytes()).collect::<String>()
    }

    /// The response for when the requested [Range] couldn't be satisfied.
    /// This is used mostly when the requested range is out of bounds for the streaming resource.
    fn request_not_satisfiable_response() -> Response<Body> {
        Response::builder()
            .status(StatusCode::from_u16(416).unwrap())
            .header(CONTENT_TYPE, PLAIN_TEXT_TYPE)
            .body(Body::empty())
            .unwrap_or_else(Self::handle_internal_error)
    }

    /// Handle an internal error that occurred while handling a request.
    /// This will return a 500 status code with an empty body.
    fn handle_internal_error(err: http::Error) -> Response<Body> {
        error!("Stream server request failed, {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use reqwest::Client;

    mod metadata {
        use super::*;
        use crate::core::stream::file_stream::FileStreamingResource;
        use crate::testing::copy_test_file;
        use std::path::PathBuf;

        #[tokio::test]
        async fn test_metadata() {
            init_logger!();
            let filename = "large-[123].txt";
            let temp_dir = tempfile::tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let filepath = copy_test_file(temp_path, filename, None);
            let client = Client::builder()
                .build()
                .expect("Client should have been created");
            let server = StreamServer::new().await.unwrap();
            let streaming_resource = FileStreamingResource::new(PathBuf::from(filepath)).unwrap();

            let stream = server
                .start_stream(streaming_resource)
                .await
                .expect("expected the torrent stream to have started");

            assert_eq!("/video/large-%5B123%5D.txt", stream.url().path());
            let result = async {
                let response = client
                    .head(stream.url().clone())
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
        async fn test_metadata_not_found() {
            init_logger!();
            let client = Client::builder()
                .build()
                .expect("Client should have been created");
            let server = StreamServer::new().await.unwrap();

            let result = async {
                let response = client
                    .head(server.inner.build_url("lorem").unwrap())
                    .send()
                    .await
                    .expect("expected a valid response");

                response.status()
            }
            .await;

            assert_eq!(StatusCode::NOT_FOUND, result)
        }
    }

    mod video {
        use super::*;
        use crate::core::stream::FileStreamingResource;
        use crate::testing::{copy_test_file, read_test_file_to_string};
        use std::path::PathBuf;

        #[tokio::test]
        async fn test_stream_not_found() {
            init_logger!();
            let client = Client::builder().build().unwrap();
            let server = StreamServer::new().await.unwrap();

            let result = async {
                let response = client
                    .get(server.inner.build_url("lorem").unwrap())
                    .send()
                    .await
                    .unwrap();
                response.status()
            }
            .await;

            assert_eq!(StatusCode::NOT_FOUND, result);
        }

        #[tokio::test]
        async fn test_stream_video() {
            init_logger!();
            let filename = "large-[123].txt";
            let temp_dir = tempfile::tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let filepath = copy_test_file(temp_path, filename, None);
            let client = Client::builder().build().unwrap();
            let server = StreamServer::new().await.unwrap();
            let resource = FileStreamingResource::new(PathBuf::from(filepath)).unwrap();
            let expected_result = read_test_file_to_string(filename).replace("\r\n", "\n");

            // start the stream for the video
            let stream = server.start_stream(resource).await.unwrap();

            // retrieve the stream data
            let result = async {
                let response = client
                    .get(stream.url().clone())
                    .header(RANGE.as_str(), "bytes=0-50000")
                    .send()
                    .await
                    .expect("expected a valid response");

                if response.status().is_success() {
                    response.text().await.unwrap()
                } else {
                    assert!(
                        false,
                        "invalid response received with status {}",
                        response.status().as_u16()
                    );
                    return String::new();
                }
            }
            .await;

            assert_eq!(expected_result, result.replace("\r\n", "\n"))
        }
    }

    mod stop_stream {
        use super::*;
        use crate::core::stream::tests::MockStreamingResource;
        use crate::recv_timeout;
        use std::time::Duration;
        use tokio::sync::mpsc::unbounded_channel;

        #[tokio::test]
        async fn test_stop_stream() {
            init_logger!();
            let filename = "FooBar.mkv";
            let (tx, mut rx) = unbounded_channel();
            let mut resource = MockStreamingResource::new();
            resource
                .expect_filename()
                .return_const(filename.to_string());
            resource.expect_stop().times(1).returning(move || {
                let _ = tx.send(());
            });
            let server = StreamServer::new().await.unwrap();

            // subscribe to the server events
            let mut receiver = server.subscribe();

            // start the stream
            let result = server.start_stream(resource).await;
            assert!(result.is_ok(), "expected Ok, but got {:?}", result);
            let event = recv_timeout!(&mut receiver, Duration::from_millis(250));
            match &*event {
                StreamServerEvent::StreamStarted(stream) => {
                    assert_eq!(
                        filename, stream.filename,
                        "expected the stream to have been started"
                    );
                }
                _ => assert!(
                    false,
                    "expected StreamServerEvent::StreamStopped, but got {:?}",
                    event
                ),
            }

            // stop the stream
            server.stop_stream(filename).await;
            let event = recv_timeout!(&mut receiver, Duration::from_millis(250));
            match &*event {
                StreamServerEvent::StreamStopped(result) => {
                    assert_eq!(filename, result, "expected the stream to have been stopped");
                }
                _ => assert!(
                    false,
                    "expected StreamServerEvent::StreamStopped, but got {:?}",
                    event
                ),
            }

            // verify that the underlying resource was stopped
            let _ = recv_timeout!(
                &mut rx,
                Duration::from_millis(250),
                "expected stop to have been called on the streaming resource"
            );
        }
    }
}
