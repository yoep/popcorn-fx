use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Announcement, ConnectionMetrics, Result, ScrapeResult,
    ServerRequest, TrackerClientConnection, TrackerError, TrackerHandle, TrackerListener,
};
use crate::torrent::{CompactIpv4Addr, CompactIpv4Addrs, InfoHash};
use async_trait::async_trait;
use axum::extract::{ConnectInfo, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use derive_more::Display;
use itertools::Itertools;
use log::{debug, error, trace, warn};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use reqwest::redirect::Policy;
use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex, Notify};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

const URL_ENCODE_RESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'~')
    .remove(b'.');

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    #[serde(rename = "failure reason", default)]
    pub failure_reason: Option<String>,
    #[serde(default)]
    pub interval: Option<u32>,
    /// The tracker id
    #[allow(dead_code)]
    #[serde(rename = "tracker id", default)]
    pub tracker_id: Option<String>,
    /// The total number of peers which have completed the torrent
    #[serde(default)]
    pub complete: Option<u64>,
    /// The total number of peers which have not yet completed the torrent
    #[serde(default)]
    pub incomplete: Option<u64>,
    #[serde(default)]
    pub peers: CompactIpv4Addrs,
}

impl Into<AnnounceEntryResponse> for HttpResponse {
    fn into(self) -> AnnounceEntryResponse {
        AnnounceEntryResponse {
            interval_seconds: self.interval.unwrap_or(0) as u64,
            leechers: self.incomplete.unwrap_or(0),
            seeders: self.complete.unwrap_or(0),
            peers: self.peers.into_iter().map(|e| e.into()).collect(),
        }
    }
}

/// The HTTP/HTTPS tracker connection protocol implementation.
#[derive(Debug, Display)]
#[display(fmt = "{} ({})", handle, url)]
pub struct HttpClient {
    /// The handle of the tracker
    handle: TrackerHandle,
    /// The base url of the http tracker
    url: Url,
    client: Client,
    metrics: ConnectionMetrics,
    cancellation_token: CancellationToken,
}

impl HttpClient {
    pub fn new(handle: TrackerHandle, url: Url, timeout: Duration) -> Self {
        let client = Client::builder()
            .redirect(Policy::limited(3))
            .timeout(timeout)
            .build()
            .expect("expected a valid http client");

        Self {
            handle,
            url,
            client,
            metrics: Default::default(),
            cancellation_token: Default::default(),
        }
    }

    fn create_announce_url(&self, announce: Announcement) -> Result<Url> {
        let mut url = self.url.clone();
        let info_hash = announce.info_hash.short_info_hash_bytes();
        let event = announce.event;
        let url_encoded_hash =
            percent_encoding::percent_encode(info_hash.as_slice(), URL_ENCODE_RESERVED).to_string();

        let base_url = url
            .query_pairs_mut()
            .append_pair("peer_id", &announce.peer_id.to_string())
            .append_pair("port", &announce.peer_port.to_string())
            .append_pair("uploaded", "0")
            .append_pair("downloaded", announce.bytes_completed.to_string().as_str())
            .append_pair("left", announce.bytes_remaining.to_string().as_str())
            .append_pair("event", event.to_string().as_str())
            .append_pair("key", "0")
            .append_pair("compact", "1")
            .append_pair("numwant", "200")
            .finish()
            .to_string();
        let url = format!("{}&info_hash={}", base_url, url_encoded_hash);

        Ok(Url::parse(&url)?)
    }

    fn create_scrape_url(&self, hashes: &[InfoHash]) -> Result<Url> {
        // replace the announce path segment with the scrape
        let url = self.url.clone().as_str().replace("announce", "scrape");
        let hashes = hashes
            .into_iter()
            .map(|e| e.short_info_hash_bytes())
            .map(|e| {
                percent_encoding::percent_encode(e.as_slice(), URL_ENCODE_RESERVED).to_string()
            })
            .map(|e| format!("info_hash={}", e))
            .join("&");

        let url_value = format!("{}?{}", url, hashes);
        Ok(Url::parse(url_value.as_str())?)
    }

    async fn process_announce_response(
        &self,
        response: std::result::Result<Response, Error>,
    ) -> Result<AnnounceEntryResponse> {
        let response = response?;
        let status_code = response.status();
        let bytes = response.bytes().await?;
        self.metrics.bytes_in.inc_by(bytes.len() as u64);

        // check the response status code from the http tracker
        // if it's unsuccessful, we don't try to parse the response body
        if !status_code.is_success() {
            debug!(
                "Http tracker {} received invalid status code {}",
                self, status_code
            );
            trace!(
                "Http tracker {} response: {}",
                self,
                String::from_utf8_lossy(bytes.as_ref())
            );
            return Err(TrackerError::AnnounceError(format!(
                "received invalid status code {}",
                status_code
            )));
        }

        trace!(
            "Http tracker {} received {} bytes, {}",
            self,
            bytes.len(),
            String::from_utf8_lossy(&bytes)
        );
        let message = serde_bencode::from_bytes::<HttpResponse>(bytes.as_ref())?;
        debug!(
            "Http tracker {} received announce response, {:?}",
            self, message
        );

        // check if the response message contains an error
        if let Some(failure) = message.failure_reason {
            return Err(TrackerError::AnnounceError(failure));
        }

        Ok(message.into())
    }

    async fn process_scrape_response(
        &self,
        response: std::result::Result<Response, Error>,
    ) -> Result<ScrapeResult> {
        let bytes = response?.bytes().await?;
        trace!(
            "Http tracker {} received {} bytes, {}",
            self,
            bytes.len(),
            String::from_utf8_lossy(&bytes)
        );
        let message = serde_bencode::from_bytes::<ScrapeResult>(bytes.as_ref())?;
        debug!(
            "Http tracker {} received scrape response, {:?}",
            self, message
        );

        Ok(message)
    }
}

#[async_trait]
impl TrackerClientConnection for HttpClient {
    async fn start(&mut self) -> Result<()> {
        let url = self.url.clone();

        // check if we're able to connect
        trace!("Http tracker {} is trying to connect with the server", self);
        self.client.head(url).send().await?;
        Ok(())
    }

    async fn announce(&self, announce: Announcement) -> Result<AnnounceEntryResponse> {
        let url = self.create_announce_url(announce)?;

        trace!("Http tracker {} is sending request {}", self, url);
        select! {
            _ = self.cancellation_token.cancelled() => {
                self.metrics.timeouts.inc();
                Err(TrackerError::Timeout)
            },
            response = self.client.get(url.clone()).send() => self.process_announce_response(response).await,
        }
    }

    async fn scrape(&self, hashes: &[InfoHash]) -> Result<ScrapeResult> {
        let url = self.create_scrape_url(hashes)?;

        trace!("Http tracker {} is sending request to {}", self, url);
        select! {
            _ = self.cancellation_token.cancelled() => {
                self.metrics.timeouts.inc();
                Err(TrackerError::Timeout)
            },
            response = self.client.get(url.clone()).send() => self.process_scrape_response(response).await,
        }
    }

    fn metrics(&self) -> &ConnectionMetrics {
        &self.metrics
    }

    fn close(&self) {
        self.cancellation_token.cancel()
    }
}

#[derive(Debug)]
pub struct HttpServer {
    inner: Arc<InnerServer>,
}

impl HttpServer {
    pub async fn with_port(port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
        let addr = listener.local_addr()?;
        let inner = Arc::new(InnerServer {
            handle: Default::default(),
            addr,
            queue: Default::default(),
            waker: Default::default(),
            timeout: Duration::from_secs(15),
            metrics: Default::default(),
            cancellation_token: Default::default(),
        });

        let state = inner.clone();
        tokio::spawn(async move {
            let router = Router::new()
                .route("/announce", get(Self::do_announce))
                .route("/scrape", get(Self::do_scrape))
                .with_state(state.clone());

            if let Err(e) = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(state.cancellation_token.clone().cancelled_owned())
            .await
            {
                error!("Http tracker {} failed to start, {}", state, e)
            }
        });

        Ok(Self { inner })
    }

    async fn do_announce(
        State(state): State<Arc<InnerServer>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        query: RawQuery,
    ) -> (StatusCode, Vec<u8>) {
        let status_code: StatusCode;
        let response: HttpResponse;

        if let Some(query) = query.0 {
            match AnnounceParams::from_str(query.as_str()) {
                Ok(params) => match state.announce(addr, params).await {
                    Ok(e) => {
                        status_code = StatusCode::OK;
                        response = HttpResponse {
                            failure_reason: None,
                            tracker_id: None,
                            interval: Some(e.interval_seconds as u32),
                            complete: Some(e.seeders),
                            incomplete: Some(e.leechers),
                            peers: e
                                .peers
                                .into_iter()
                                .filter_map(|e| CompactIpv4Addr::try_from(e).ok())
                                .collect::<Vec<_>>()
                                .into(),
                        };
                    }
                    Err(e) => {
                        status_code = StatusCode::BAD_REQUEST;
                        response = HttpResponse {
                            failure_reason: Some(e.to_string()),
                            interval: None,
                            tracker_id: None,
                            complete: None,
                            incomplete: None,
                            peers: Vec::with_capacity(0).into(),
                        }
                    }
                },
                Err(e) => {
                    debug!(
                        "Http tracker {} failed to parse announce request, {}",
                        state, e
                    );
                    status_code = StatusCode::BAD_REQUEST;
                    response = HttpResponse {
                        failure_reason: Some(e.to_string()),
                        interval: None,
                        tracker_id: None,
                        complete: None,
                        incomplete: None,
                        peers: Vec::with_capacity(0).into(),
                    }
                }
            }
        } else {
            status_code = StatusCode::BAD_REQUEST;
            response = HttpResponse {
                failure_reason: Some("missing announcement information".to_string()),
                interval: None,
                tracker_id: None,
                complete: None,
                incomplete: None,
                peers: Vec::with_capacity(0).into(),
            }
        }

        match serde_bencode::to_bytes(&response) {
            Ok(bytes) => (status_code, bytes),
            Err(e) => {
                error!("Http tracker {} failed to serialize response, {}", state, e);
                (StatusCode::INTERNAL_SERVER_ERROR, Vec::with_capacity(0))
            }
        }
    }

    async fn do_scrape(
        State(state): State<Arc<InnerServer>>,
        query: RawQuery,
    ) -> (StatusCode, Vec<u8>) {
        let status_code: StatusCode;
        let response: ScrapeResult;

        if let Some(query) = query.0 {
            let mut info_hashes = vec![];
            for keypair in query.split("&") {
                let mut key_value = keypair.splitn(2, "=");
                let key = key_value.next().unwrap_or_default();
                let value = key_value.next().unwrap_or_default();

                if key.to_lowercase().trim() == "info_hash" {
                    let bytes =
                        percent_encoding::percent_decode(value.as_bytes()).collect::<Vec<_>>();
                    match InfoHash::try_from_bytes(bytes.as_slice()) {
                        Ok(e) => info_hashes.push(e),
                        Err(e) => {
                            debug!("Http tracker {} failed to parse info hash, {}", state, e);
                            continue;
                        }
                    }
                } // otherwise, ignore the query parameter
            }

            match state.scrape(info_hashes).await {
                Ok(e) => {
                    status_code = StatusCode::OK;
                    response = e;
                }
                Err(e) => {
                    warn!("Http tracker {} failed process request, {}", state, e);
                    status_code = StatusCode::INTERNAL_SERVER_ERROR;
                    response = ScrapeResult::default();
                }
            }
        } else {
            status_code = StatusCode::BAD_REQUEST;
            response = ScrapeResult::default();
        }

        match serde_bencode::to_bytes(&response) {
            Ok(bytes) => (status_code, bytes),
            Err(e) => {
                error!("Http tracker {} failed to serialize response, {}", state, e);
                (StatusCode::INTERNAL_SERVER_ERROR, Vec::with_capacity(0))
            }
        }
    }
}

#[async_trait]
impl TrackerListener for HttpServer {
    async fn accept(&self) -> Option<ServerRequest> {
        loop {
            if self.inner.cancellation_token.is_cancelled() {
                return None;
            }
            if let Some(request) = self.inner.queue.lock().await.pop_front() {
                return Some(request);
            }

            self.inner.waker.notified().await;
        }
    }

    fn addr(&self) -> &SocketAddr {
        &self.inner.addr
    }

    fn metrics(&self) -> &ConnectionMetrics {
        &self.inner.metrics
    }

    fn close(&self) {
        self.inner.cancellation_token.cancel();
        self.inner.waker.notify_waiters();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerServer {
    handle: TrackerHandle,
    addr: SocketAddr,
    queue: Mutex<VecDeque<ServerRequest>>,
    waker: Notify,
    timeout: Duration,
    metrics: ConnectionMetrics,
    cancellation_token: CancellationToken,
}

impl InnerServer {
    async fn announce(
        &self,
        addr: SocketAddr,
        params: AnnounceParams,
    ) -> Result<AnnounceEntryResponse> {
        let info_hash = InfoHash::try_from_bytes(params.info_hash.as_slice())
            .map_err(|e| TrackerError::Parse(e.to_string()))?;
        let peer_id = PeerId::try_from(params.peer_id.as_bytes())
            .map_err(|e| TrackerError::Parse(format!("failed to parse peer id, {}", e)))?;
        let (tx, rx) = oneshot::channel();
        let announcement = Announcement {
            info_hash,
            peer_id,
            peer_port: params.port,
            event: params.event,
            bytes_completed: params.downloaded,
            bytes_remaining: params.left,
        };

        {
            let mut queue = self.queue.lock().await;
            queue.push_back(ServerRequest::Announcement {
                addr,
                request: announcement,
                response: tx,
            });
        }

        self.await_response(rx).await
    }

    async fn scrape(&self, info_hashes: Vec<InfoHash>) -> Result<ScrapeResult> {
        let (tx, rx) = oneshot::channel();

        {
            let mut queue = self.queue.lock().await;
            queue.push_back(ServerRequest::Scrape {
                request: info_hashes,
                response: tx,
            });
        }

        self.await_response(rx).await
    }

    /// Waits for a response from the tracker server that manages this listener.
    ///
    /// If the tracker server does not respond within the specified timeout,
    /// an error is returned.
    async fn await_response<T>(&self, rx: oneshot::Receiver<T>) -> Result<T> {
        self.waker.notify_waiters();

        select! {
            _ = time::sleep(self.timeout) => Err(TrackerError::Timeout),
            response = rx => response.map_err(|e| TrackerError::Io(io::Error::new(io::ErrorKind::BrokenPipe, e))),
        }
    }
}

struct AnnounceParams {
    info_hash: Vec<u8>,
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: AnnounceEvent,
    key: u32,
    compact: bool,
    numwant: u32,
}

impl Default for AnnounceParams {
    fn default() -> Self {
        Self {
            info_hash: vec![],
            peer_id: String::new(),
            port: 0,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            event: AnnounceEvent::None,
            key: 0,
            compact: false,
            numwant: 0,
        }
    }
}

impl FromStr for AnnounceParams {
    type Err = TrackerError;

    fn from_str(value: &str) -> Result<Self> {
        let mut params = Self::default();
        let query_pairs = value.split("&").map(|pair| {
            let mut split = pair.splitn(2, "=");
            (
                split.next().unwrap_or_default(),
                split.next().unwrap_or_default(),
            )
        });

        for (key, value) in query_pairs {
            match key.to_lowercase().trim() {
                "info_hash" => {
                    params.info_hash = percent_encoding::percent_decode(value.as_bytes()).collect();
                }
                "peer_id" => {
                    params.peer_id = value.to_string();
                }
                "port" => {
                    params.port = value.parse::<u16>().unwrap_or(0);
                }
                "uploaded" => {
                    params.uploaded = value.parse::<u64>().unwrap_or(0);
                }
                "downloaded" => {
                    params.downloaded = value.parse::<u64>().unwrap_or(0);
                }
                "left" => {
                    params.left = value.parse::<u64>().unwrap_or(0);
                }
                "event" => {
                    params.event = AnnounceEvent::from_str(value)?;
                }
                "key" => {
                    params.key = value.parse::<u32>().unwrap_or(0);
                }
                "compact" => match value.parse::<u8>() {
                    Ok(value) => params.compact = value == 1,
                    Err(_) => {
                        return Err(TrackerError::Parse(format!(
                            "invalid compact value {}",
                            value
                        )))
                    }
                },
                "numwant" => match value.parse::<u32>() {
                    Ok(value) => params.numwant = value,
                    Err(_) => {
                        return Err(TrackerError::Parse(format!(
                            "invalid numwant value {}",
                            value
                        )))
                    }
                },
                _ => {}
            }
        }

        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::init_logger;
    use crate::torrent::peer::PeerId;
    use crate::torrent::tracker::{AnnounceEvent, TrackerServer};

    use log::info;

    #[tokio::test]
    async fn test_start() {
        init_logger!();
        let http_server = HttpServer::with_port(0).await.unwrap();
        let server = TrackerServer::with_listeners(vec![Box::new(http_server)])
            .await
            .unwrap();
        let url =
            Url::parse(format!("http://localhost:{}/announce", server.addr().port()).as_str())
                .unwrap();
        let mut connection = HttpClient::new(TrackerHandle::new(), url, Duration::from_secs(2));

        let result = connection.start().await;

        assert_eq!(Ok(()), result);
    }

    #[tokio::test]
    async fn test_create_announce_url() {
        init_logger!();
        let info_hash = InfoHash::from_str("a1dfefec1a9dd7fa8a041ebeeea271db55126d2f").unwrap();
        let http_server = HttpServer::with_port(0).await.unwrap();
        let url =
            Url::parse(format!("http://localhost:{}/announce", http_server.addr().port()).as_str())
                .unwrap();
        let expected_hash_value =
            "info_hash=%A1%DF%EF%EC%1A%9D%D7%FA%8A%04%1E%BE%EE%A2q%DBU%12m%2F";
        let tracker_handle = TrackerHandle::new();
        let peer_id = PeerId::new();
        let announce = Announcement {
            info_hash,
            peer_id,
            peer_port: 0,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let connection = HttpClient::new(tracker_handle, url, Duration::from_secs(2));

        let url = connection.create_announce_url(announce).unwrap();
        let result = url.query().unwrap();

        info!("Got url parameters {}", result);
        assert!(
            result.contains(expected_hash_value),
            "expected query parameter \"{}\"",
            expected_hash_value
        );
    }

    #[tokio::test]
    async fn test_http_tracker_announce() {
        init_logger!();
        let info_hash = InfoHash::from_str("a1dfefec1a9dd7fa8a041ebeeea271db55126d2f").unwrap();
        let http_server = HttpServer::with_port(0).await.unwrap();
        let server = TrackerServer::with_listeners(vec![Box::new(http_server)])
            .await
            .unwrap();
        let url =
            Url::parse(format!("http://localhost:{}/announce", server.addr().port()).as_str())
                .unwrap();
        let tracker_handle = TrackerHandle::new();
        let peer_id = PeerId::new();
        let announce = Announcement {
            info_hash: info_hash.clone(),
            peer_id,
            peer_port: 6881,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let mut connection = HttpClient::new(tracker_handle, url, Duration::from_secs(2));

        // test the tracker connection
        let result = connection.start().await;
        assert_eq!(Ok(()), result);

        // add dummies to the tracker server
        server
            .add_peer(
                info_hash.clone(),
                (Ipv4Addr::LOCALHOST, 9000).into(),
                PeerId::new(),
                6881,
                false,
            )
            .await;
        server
            .add_peer(
                info_hash,
                (Ipv4Addr::LOCALHOST, 9000).into(),
                PeerId::new(),
                6882,
                true,
            )
            .await;

        // make a new announcement
        let result = connection
            .announce(announce)
            .await
            .expect("expected an announcement response");
        assert_ne!(
            0, result.interval_seconds,
            "expected the interval to be greater than 0"
        );
        assert_eq!(
            2,
            result.peers.len(),
            "expected 2 peers to have been returned"
        );
    }

    #[tokio::test]
    async fn test_http_tracker_scrape() {
        init_logger!();
        let info_hash = InfoHash::from_str("a1dfefec1a9dd7fa8a041ebeeea271db55126d2f").unwrap();
        let http_server = HttpServer::with_port(0).await.unwrap();
        let server = TrackerServer::with_listeners(vec![Box::new(http_server)])
            .await
            .unwrap();
        let url =
            Url::parse(format!("http://localhost:{}/announce", server.addr().port()).as_str())
                .unwrap();
        let mut connection = HttpClient::new(TrackerHandle::new(), url, Duration::from_secs(2));

        // test the tracker connection
        let result = connection.start().await;
        assert_eq!(Ok(()), result);

        // add a dummy peer to the server
        server
            .add_peer(
                info_hash.clone(),
                (Ipv4Addr::LOCALHOST, 1010).into(),
                PeerId::new(),
                6881,
                true,
            )
            .await;

        // scrape the torrent results
        match connection.scrape(&vec![info_hash.clone()]).await {
            Ok(scrape_result) => match scrape_result.files.get(&info_hash) {
                Some(scrape_file) => assert_eq!(1, scrape_file.complete),
                None => assert!(false, "expected a scrape file result"),
            },
            Err(e) => assert!(false, "expected Result::Ok, got {:?}", e),
        }
    }
}
