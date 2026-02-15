use crate::core::subtitles::model::{Subtitle, SubtitleType};
use crate::core::subtitles::Result;
use crate::core::subtitles::{SubtitleError, SubtitleProvider};
use crate::core::utils::network::ip_addr;
use axum::body::Body;
use axum::extract::Path as AxumPath;
use axum::extract::State;
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_DISPOSITION, CONTENT_TYPE,
};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{http, Router};
use log::{debug, error, info, trace};
use reqwest::Url;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const SERVER_PROTOCOL: &str = "http";
const SERVER_SUBTITLE_PATH: &str = "subtitle";

/// The subtitle server is responsible for serving [Subtitle]'s over http.
#[derive(Debug, Clone)]
pub struct SubtitleServer {
    inner: Arc<InnerServer>,
}

impl SubtitleServer {
    /// Create a new subtitle server for the given provider.
    pub async fn new(provider: Arc<dyn SubtitleProvider>) -> Result<Self> {
        Self::with_port(0, provider).await
    }

    /// Create a new subtitle server for the given provider on the given port.
    pub async fn with_port(port: u16, provider: Arc<dyn SubtitleProvider>) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
        let addr = (ip_addr(), listener.local_addr()?.port()).into();
        let inner = Arc::new(InnerServer {
            addr,
            provider,
            subtitles: Default::default(),
            cancellation_token: Default::default(),
        });

        let state = inner.clone();
        tokio::spawn(async move {
            let cancellation_token = state.cancellation_token.clone();
            let router = Router::new()
                .route("/subtitle/{filename}", get(Self::on_subtitle_request))
                .with_state(state);

            if let Err(e) = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            {
                error!("Failed to start torrent stream server, {}", e);
            }
        });

        Ok(Self { inner })
    }

    /// Serve the given [Subtitle] as a raw format over HTTP.
    ///
    /// It returns the served url on success, else the error.
    pub async fn serve(&self, subtitle: Subtitle, serving_type: SubtitleType) -> Result<String> {
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

    async fn on_subtitle_request(
        State(state): State<Arc<InnerServer>>,
        AxumPath(filename): AxumPath<String>,
    ) -> impl IntoResponse {
        match percent_encoding::percent_decode(filename.as_bytes()).decode_utf8() {
            Err(_) => (StatusCode::BAD_REQUEST, Body::empty()).into_response(),
            Ok(filename) => state.on_subtitle_request(&*filename).await,
        }
    }

    async fn subtitle_to_serving_url(
        &self,
        filename_base: String,
        subtitle: Subtitle,
        serving_type: SubtitleType,
    ) -> Result<String> {
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

    fn build_url(&self, filename_full: &str) -> result::Result<Url, url::ParseError> {
        let host = format!("{}://{}", SERVER_PROTOCOL, self.inner.addr);
        let path = format!("{}/{}", SERVER_SUBTITLE_PATH, filename_full);
        let url = Url::parse(host.as_str())?;

        url.join(path.as_str())
    }
}

impl Drop for SubtitleServer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerServer {
    addr: SocketAddr,
    provider: Arc<dyn SubtitleProvider>,
    subtitles: Mutex<HashMap<String, DataHolder>>,
    cancellation_token: CancellationToken,
}

impl InnerServer {
    /// Handle a request send to the subtitle server for the given filename.
    /// It takes a lock on the subtitles and the filename to verify the validity of the request.
    ///
    /// # Arguments
    ///
    /// * `filename` - the filename which is requested to being served.
    ///
    /// # Returns
    ///
    /// It returns the subtitle filename contents if found, else a `404`.
    async fn on_subtitle_request(&self, filename: &str) -> Response<Body> {
        let subtitles = self.subtitles.lock().await;
        trace!("Handling request for subtitle filename {}", &filename);

        match subtitles.get(filename) {
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap_or_else(Self::handle_internal_error),
            Some(e) => {
                let content_type = format!("{}; charset=utf-8", e.data_type.content_type());
                debug!("Handled subtitle request for {}", filename);
                Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, content_type)
                    .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .header(ACCESS_CONTROL_ALLOW_METHODS, "GET,HEAD")
                    .header(CONTENT_DISPOSITION, "")
                    .body(Body::from(e.data()))
                    .unwrap_or_else(Self::handle_internal_error)
            }
        }
    }

    fn handle_internal_error(err: http::Error) -> Response<Body> {
        error!("Subtitle server request failed, {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
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
    use super::*;
    use crate::core::subtitles::MockSubtitleProvider;
    use crate::init_logger;
    use reqwest::header::CONTENT_TYPE;
    use reqwest::{Client, Url};

    #[tokio::test]
    async fn test_subtitle_is_served() {
        init_logger!();
        let mut provider = MockSubtitleProvider::new();
        let subtitle = Subtitle::new(vec![], None, "my-subtitle - heavy.srt".to_string());
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        provider
            .expect_convert()
            .returning(|_: Subtitle, _: SubtitleType| -> Result<String> {
                Ok("lorem ipsum".to_string())
            });
        let server = SubtitleServer::new(Arc::new(provider)).await.unwrap();

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
        let client = Client::builder()
            .build()
            .expect("Client should have been created");
        let server = SubtitleServer::new(Arc::new(MockSubtitleProvider::new()))
            .await
            .unwrap();
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
        let server = SubtitleServer::new(Arc::new(MockSubtitleProvider::new()))
            .await
            .unwrap();
        let expected_result = format!(
            "{}://{}/{}/Lorem.S01E16%20720p%20-%20Heavy.vtt",
            SERVER_PROTOCOL,
            server.inner.addr.to_string(),
            SERVER_SUBTITLE_PATH
        );

        let result = server.build_url("Lorem.S01E16 720p - Heavy.vtt").unwrap();

        assert_eq!(expected_result, result.to_string())
    }
}
