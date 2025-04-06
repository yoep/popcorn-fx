use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, error, info, trace};
use reqwest::Url;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use warp::http::header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_DISPOSITION, CONTENT_TYPE,
};
use warp::http::{HeaderValue, Response};
use warp::{Filter, Rejection};

use crate::core::subtitles;
use crate::core::subtitles::model::{Subtitle, SubtitleType};
use crate::core::subtitles::{SubtitleError, SubtitleProvider};
use crate::core::utils::network::available_socket;

const SERVER_PROTOCOL: &str = "http";
const SERVER_SUBTITLE_PATH: &str = "subtitle";

/// The state of the server serving subtitles.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServerState {
    Stopped,
    Running,
    Error,
}

/// The events of the subtitle server.
#[derive(Debug, Clone, PartialEq)]
pub enum SubtitleServerEvent {
    /// Indicates that the server state has been changed
    StateChanged(ServerState),
}

/// The subtitle server is responsible for serving [Subtitle]'s over http.
#[derive(Debug, Clone)]
pub struct SubtitleServer {
    inner: Arc<InnerServer>,
}

impl SubtitleServer {
    /// Create a new subtitle server.
    pub fn new(provider: Arc<Box<dyn SubtitleProvider>>) -> Self {
        let socket = available_socket();
        let inner = Arc::new(InnerServer {
            socket,
            provider,
            subtitles: Default::default(),
            state: Mutex::new(ServerState::Stopped),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(&inner_main).await;
        });

        Self { inner }
    }

    /// Serve the given [Subtitle] as a raw format over HTTP.
    ///
    /// It returns the served url on success, else the error.
    pub async fn serve(
        &self,
        subtitle: Subtitle,
        serving_type: SubtitleType,
    ) -> subtitles::Result<String> {
        trace!(
            "Trying to service subtitle type {} for {}",
            &serving_type,
            &subtitle
        );
        let filename = Path::new(subtitle.file())
            .file_stem()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string());

        match filename {
            None => Err(SubtitleError::InvalidFile(
                subtitle.file().to_string(),
                "no extension".to_string(),
            )),
            Some(base_name) => {
                self.subtitle_to_serving_url(base_name, subtitle, serving_type)
                    .await
            }
        }
    }

    /// Retrieve the current state of the subtitle server.
    ///
    /// It returns the state of the server.
    pub async fn state(&self) -> ServerState {
        *self.inner.state.lock().await
    }

    async fn subtitle_to_serving_url(
        &self,
        filename_base: String,
        subtitle: Subtitle,
        serving_type: SubtitleType,
    ) -> subtitles::Result<String> {
        match self.inner.provider.convert(subtitle, serving_type.clone()) {
            Ok(data) => {
                debug!("Converted subtitle for serving");
                let filename_full = format!("{}.{}", filename_base, &serving_type.extension());
                let url = self.build_url(&filename_full);

                match url {
                    Ok(result) => {
                        let mut subtitles = self.inner.subtitles.lock().await;
                        subtitles.insert(
                            filename_full.clone(),
                            DataHolder::new(data, serving_type.clone()),
                        );
                        debug!("Registered new subtitle entry {}", filename_full);

                        info!("Serving new subtitle url {}", &result);
                        Ok(result.to_string())
                    }
                    Err(e) => Err(SubtitleError::ParseUrlError(e.to_string())),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn build_url(&self, filename_full: &str) -> Result<Url, url::ParseError> {
        let host = format!("{}://{}", SERVER_PROTOCOL, self.inner.socket);
        let path = format!("{}/{}", SERVER_SUBTITLE_PATH, filename_full);
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }
}

impl Callback<SubtitleServerEvent> for SubtitleServer {
    fn subscribe(&self) -> Subscription<SubtitleServerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<SubtitleServerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for SubtitleServer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerServer {
    socket: SocketAddr,
    provider: Arc<Box<dyn SubtitleProvider>>,
    subtitles: Mutex<HashMap<String, DataHolder>>,
    state: Mutex<ServerState>,
    callbacks: MultiThreadedCallback<SubtitleServerEvent>,
    cancellation_token: CancellationToken,
}

impl InnerServer {
    async fn start(&self, subtitle_server: &Arc<InnerServer>) {
        let route_server = subtitle_server.clone();
        let routes = warp::get()
            .and(warp::path!("subtitle" / String))
            .and_then(move |subtitle: String| {
                let subtitle = percent_encoding::percent_decode(subtitle.as_bytes())
                    .decode_utf8()
                    .expect("expected a valid utf8 value")
                    .to_string();
                trace!("Handling request for subtitle filename {}", &subtitle);

                let route_server = route_server.clone();
                let cancellation_token = route_server.cancellation_token.clone();
                async move {
                    select! {
                        _ = cancellation_token.cancelled() => Err(warp::reject()),
                        result = route_server.handle_subtitle_request(subtitle) => result,
                    }
                }
            })
            .with(warp::cors().allow_any_origin());

        trace!(
            "Starting subtitle server on {}:{}",
            self.socket.ip(),
            self.socket.port()
        );
        let server = warp::serve(routes);

        trace!("Binding subtitle server to socket {:?}", self.socket);
        match server.try_bind_ephemeral((self.socket.ip(), self.socket.port())) {
            Ok((socket, execution)) => {
                info!("Subtitle server is running on {}", socket);
                self.update_state(ServerState::Running).await;

                select! {
                    _ = self.cancellation_token.cancelled() => {},
                    _ = execution => {},
                }

                self.update_state(ServerState::Stopped).await;
            }
            Err(e) => {
                error!("Failed to start subtitle server, {}", e);
                self.update_state(ServerState::Error).await;
            }
        }
    }

    /// Update the server to the given state.
    /// This will result in a no-op if the current state is the same.
    async fn update_state(&self, state: ServerState) {
        let mut mutex = self.state.lock().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        debug!("Subtitle server state changed to {:?}", state);
        self.callbacks
            .invoke(SubtitleServerEvent::StateChanged(state));
    }

    /// Handle a request send to the subtitle server for the given filename.
    /// It takes a lock on the subtitles and the filename to verify the validity of the request.
    ///
    /// # Arguments
    ///
    /// * `subtitles`   - the locked subtitles
    /// * `filename`    - the filename which is requested to being served.
    ///
    /// # Returns
    ///
    /// It returns the subtitle filename contents if found, else a `404`.
    async fn handle_subtitle_request(
        &self,
        filename: String,
    ) -> Result<Response<String>, Rejection> {
        let subtitles = self.subtitles.lock().await;

        match subtitles.get(filename.as_str()) {
            None => Err(warp::reject()),
            Some(e) => {
                let content_type = format!("{}; charset=utf-8", e.data_type.content_type());
                let header_value = HeaderValue::from_bytes(content_type.as_bytes())
                    .expect("expected a valid header value");
                let mut response = Response::new(e.data());
                let headers = response.headers_mut();

                headers.insert(CONTENT_TYPE, header_value);
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
                headers.insert(
                    ACCESS_CONTROL_ALLOW_METHODS,
                    HeaderValue::from_static("GET,HEAD"),
                );
                headers.insert(
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    HeaderValue::from_static(CONTENT_TYPE.as_str()),
                );
                headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static(""));

                debug!("Handled subtitle request for {}", filename);
                Ok(response)
            }
        }
    }
}

/// Holds the raw format data of a [Subtitle] with additional information.
#[derive(Debug)]
pub struct DataHolder {
    data: String,
    data_type: SubtitleType,
}

impl DataHolder {
    fn new(data: String, data_type: SubtitleType) -> Self {
        Self { data, data_type }
    }

    /// Retrieve a copy of the raw data.
    pub fn data(&self) -> String {
        self.data.clone()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::core::subtitles::MockSubtitleProvider;
    use crate::{init_logger, recv_timeout};
    use reqwest::header::CONTENT_TYPE;
    use reqwest::{Client, Url};
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    #[tokio::test]
    async fn test_state() {
        init_logger!();
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let server = SubtitleServer::new(Arc::new(provider as Box<dyn SubtitleProvider>));

        let result = server.state().await;

        assert_eq!(ServerState::Stopped, result)
    }

    #[tokio::test]
    async fn test_subtitle_is_served() {
        init_logger!();
        let mut provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let subtitle = Subtitle::new(vec![], None, "my-subtitle - heavy.srt".to_string());
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        provider.expect_convert().returning(
            |_: Subtitle, _: SubtitleType| -> subtitles::Result<String> {
                Ok("lorem ipsum".to_string())
            },
        );
        let (tx, mut rx) = unbounded_channel();
        let server = SubtitleServer::new(Arc::new(provider as Box<dyn SubtitleProvider>));

        let mut receiver = server.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(
            SubtitleServerEvent::StateChanged(ServerState::Running),
            result
        );
        let serving_url = server
            .serve(subtitle, SubtitleType::Vtt)
            .await
            .expect("expected the subtitle to be served");

        let (content_type, body) = async {
            let response = client
                .get(Url::parse(serving_url.as_str()).unwrap())
                .send()
                .await
                .expect("expected a valid response");

            if response.status().is_success() {
                let headers = response.headers().clone();
                let content_type = headers
                    .get(CONTENT_TYPE)
                    .expect("expected the content type within the response");
                let body = response.text().await.expect("expected a string body");

                (content_type.clone(), body)
            } else {
                panic!(
                    "invalid response received with status {}",
                    response.status().as_u16()
                )
            }
        }
        .await;

        assert_eq!(String::from("lorem ipsum"), body);
        assert_eq!("text/vtt; charset=utf-8", content_type.to_str().unwrap())
    }

    #[tokio::test]
    async fn test_subtitle_not_being_served() {
        init_logger!();
        let filename = "lorem.srt";
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let (tx, mut rx) = unbounded_channel();
        let server = SubtitleServer::new(Arc::new(provider as Box<dyn SubtitleProvider>));

        let mut receiver = server.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(
            SubtitleServerEvent::StateChanged(ServerState::Running),
            result
        );
        let serving_url = server.build_url(filename).unwrap();

        let status_code = client
            .get(serving_url)
            .send()
            .await
            .expect("expected a response")
            .status();

        assert_eq!(
            404,
            status_code.as_u16(),
            "expected the subtitle to not have been found"
        )
    }

    #[tokio::test]
    async fn test_build_url_escape_characters() {
        init_logger!();
        let provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let server = SubtitleServer::new(Arc::new(provider as Box<dyn SubtitleProvider>));
        let expected_result = format!(
            "{}://{}/{}/Lorem.S01E16%20720p%20-%20Heavy.vtt",
            SERVER_PROTOCOL,
            server.inner.socket.to_string(),
            SERVER_SUBTITLE_PATH
        );

        let result = server.build_url("Lorem.S01E16 720p - Heavy.vtt").unwrap();

        assert_eq!(expected_result, result.to_string())
    }
}
