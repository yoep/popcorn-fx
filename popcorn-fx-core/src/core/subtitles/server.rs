use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use local_ip_address::local_ip;
use log::{debug, info, trace};
use tokio::sync::{Mutex, MutexGuard};
use warp::{Filter, Rejection};
use warp::http::{HeaderValue, Response};
use warp::http::header::CONTENT_TYPE;

use crate::core::subtitles;
use crate::core::subtitles::{SubtitleError, SubtitleProvider};
use crate::core::subtitles::model::{Subtitle, SubtitleType};

pub struct SubtitleServer {
    runtime: tokio::runtime::Runtime,
    socket: Arc<SocketAddr>,
    subtitles: Arc<Mutex<HashMap<String, DataHolder>>>,
    provider: Arc<Box<dyn SubtitleProvider>>,
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
        };

        instance.start_subtitle_server();
        instance
    }

    /// Serve the given [Subtitle] as a raw format over HTTP.
    ///
    /// It returns the served url on success, else the error.
    pub fn serve(&self, subtitle: Subtitle, serving_type: SubtitleType) -> subtitles::Result<String> {
        let filename = Path::new(subtitle.file()).file_stem()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string());

        match filename {
            None => Err(SubtitleError::InvalidFile(subtitle.file().clone(), "no extension".to_string())),
            Some(base_name) => self.subtitle_to_serving_url(base_name, subtitle, serving_type)
        }
    }

    fn start_subtitle_server(&self) {
        debug!("Starting subtitle server");
        let subtitles = self.subtitles.clone();
        let socket = self.socket.clone();

        self.runtime.spawn(async move {
            let routes = warp::get()
                .and(warp::path!("subtitle" / String))
                .and_then(move |subtitle: String| {
                    let subtitles = subtitles.clone();
                    async move {
                        let subtitles = subtitles.lock().await;
                        Self::handle_subtitle_request(subtitles, subtitle)
                    }
                });
            let socket = socket.clone();

            trace!("Serving subtitle server on {}:{}", socket.ip(), socket.port());
            warp::serve(routes).bind((socket.ip(), socket.port())).await;
        });
    }

    fn subtitle_to_serving_url(&self, filename_base: String, subtitle: Subtitle, serving_type: SubtitleType) -> subtitles::Result<String> {
        match self.provider.convert(subtitle, serving_type.clone()) {
            Ok(data) => {
                debug!("Converted subtitle for serving");
                let mutex = self.subtitles.clone();
                let filename_full = format!("{}.{}", filename_base, &serving_type.extension());
                let url = self.build_url(&filename_full);
                let execute = async move {
                    let mut subtitles = mutex.lock().await;
                    subtitles.insert(filename_full.clone(), DataHolder::new(data, serving_type.clone()));
                    debug!("Registered new subtitle entry {}", filename_full);
                };

                self.runtime.block_on(execute);

                info!("Serving new subtitle url {}", &url);
                Ok(url)
            }
            Err(e) => Err(e),
        }
    }

    fn build_url(&self, filename_full: &str) -> String {
        format!("http://{}/subtitle/{}", self.socket.to_string(), filename_full)
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
                let header_value = HeaderValue::from_bytes( content_type.as_bytes()).expect("expected a valid header value");
                let mut response = Response::new(e.data());

                response.headers_mut().insert(CONTENT_TYPE, header_value);
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
    use reqwest::{Client, Url};
    use reqwest::header::CONTENT_TYPE;

    use crate::core::subtitles::MockSubtitleProvider;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_subtitle_is_served() {
        init_logger();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut provider: Box<MockSubtitleProvider> = Box::new(MockSubtitleProvider::new());
        let subtitle = Subtitle::new(
            vec![],
            None,
            "my-subtitle.srt".to_string(),
        );
        let client = Client::builder().build().expect("Client should have been created");
        provider.expect_convert()
            .returning(|subtitle: Subtitle, output_type: SubtitleType| -> subtitles::Result<String> {
                Ok("lorem ipsum".to_string())
            });
        let arc = Arc::new(provider as Box<dyn SubtitleProvider>);
        let server = SubtitleServer::new(&arc);

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
        let serving_url = server.build_url(filename);

        let status_code = runtime.block_on(async move {
            client.get(Url::parse(serving_url.as_str()).expect("expected a valid url"))
                .send()
                .await
                .expect("expected a response")
                .status()
        });

        assert_eq!(404, status_code.as_u16())
    }
}