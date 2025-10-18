use crate::torrent::tracker::{
    AnnounceEntryResponse, Announcement, ConnectionMetrics, Result, ScrapeResult,
    TrackerConnection, TrackerError, TrackerHandle,
};
use crate::torrent::{CompactIpv4Addrs, InfoHash};
use async_trait::async_trait;
use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use reqwest::redirect::Policy;
use reqwest::{Client, Error, Response};
use serde::Deserialize;
use std::time::Duration;
use tokio::select;
use tokio_util::sync::CancellationToken;
use url::Url;

const URL_ENCODE_RESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'~')
    .remove(b'.');

#[derive(Debug, Clone, Deserialize)]
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
    pub complete: Option<u32>,
    /// The total number of peers which have not yet completed the torrent
    #[serde(default)]
    pub incomplete: Option<u32>,
    #[serde(default, with = "crate::torrent::compact::compact_ipv4")]
    pub peers: CompactIpv4Addrs,
}

impl Into<AnnounceEntryResponse> for HttpResponse {
    fn into(self) -> AnnounceEntryResponse {
        AnnounceEntryResponse {
            interval_seconds: self.interval.unwrap_or(0) as u64,
            leechers: self.incomplete.unwrap_or(0) as u64,
            seeders: self.complete.unwrap_or(0) as u64,
            peers: self.peers.into_iter().map(|e| e.into()).collect(),
        }
    }
}

/// The HTTP/HTTPS tracker connection protocol implementation.
#[derive(Debug, Display)]
#[display(fmt = "{} ({})", handle, url)]
pub struct HttpConnection {
    /// The handle of the tracker
    handle: TrackerHandle,
    /// The base url of the http tracker
    url: Url,
    client: Client,
    metrics: ConnectionMetrics,
    cancellation_token: CancellationToken,
}

impl HttpConnection {
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
impl TrackerConnection for HttpConnection {
    async fn start(&mut self) -> Result<()> {
        let url = self.url.clone();

        // check if we're able to connect
        trace!("Http tracker {} is trying to connect with the server", self);
        self.client.head(url).send().await?;
        Ok(())
    }

    async fn announce(&self, announce: Announcement) -> Result<AnnounceEntryResponse> {
        let url = self.create_announce_url(announce)?;

        trace!("Http tracker {} is sending request to {}", self, url);
        select! {
            _ = self.cancellation_token.cancelled() => {
                self.metrics.timeouts.inc();
                Err(TrackerError::Timeout(url.clone()))
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
                Err(TrackerError::Timeout(url.clone()))
            },
            response = self.client.get(url.clone()).send() => self.process_scrape_response(response).await,
        }
    }

    fn metrics(&self) -> &ConnectionMetrics {
        &self.metrics
    }

    fn close(&mut self) {
        self.cancellation_token.cancel()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::peer::PeerId;
    use crate::torrent::tests::create_metadata;
    use crate::torrent::tracker::AnnounceEvent;

    use crate::init_logger;
    use log::info;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_start() {
        init_logger!();
        let torrent_info = create_metadata("ubuntu-https.torrent");
        let tracker_handle = TrackerHandle::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let mut connection = HttpConnection::new(tracker_handle, url, Duration::from_secs(2));

        let result = connection.start().await;

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_create_announce_url() {
        init_logger!();
        let expected_hash_value = "info_hash=.%8ED%06%8B%25H%14%EA%1A%7DIi%A9%AF%1Dx%E0%F5%1F";
        let torrent_info = create_metadata("ubuntu-https.torrent");
        let tracker_handle = TrackerHandle::new();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let announce = Announcement {
            info_hash: torrent_info.info_hash,
            peer_id,
            peer_port: 0,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let connection = HttpConnection::new(tracker_handle, url, Duration::from_secs(2));

        let url = connection.create_announce_url(announce).unwrap();
        let result = url.query().unwrap();

        info!("Got url parameters {}", result);
        assert!(
            result.contains(expected_hash_value),
            "expected the info hash to be present"
        );
    }

    #[test]
    fn test_http_tracker_announce() {
        init_logger!();
        let runtime = Runtime::new().expect("expected a runtime");
        let torrent_info = create_metadata("ubuntu-https.torrent");
        let tracker_handle = TrackerHandle::new();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let announce = Announcement {
            info_hash: torrent_info.info_hash,
            peer_id,
            peer_port: 6881,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let mut connection = HttpConnection::new(tracker_handle, url, Duration::from_secs(2));

        runtime.block_on(connection.start()).unwrap();
        let result = runtime.block_on(connection.announce(announce)).unwrap();

        assert_ne!(
            0, result.interval_seconds,
            "expected the interval to be greater than 0"
        );
        assert_ne!(
            0,
            result.peers.len(),
            "expected the number of peers to be greater than 0"
        );
    }

    #[test]
    fn test_http_tracker_scrape() {
        init_logger!();
        let runtime = Runtime::new().expect("expected a runtime");
        let torrent_info = create_metadata("ubuntu-https.torrent");
        let tracker_handle = TrackerHandle::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let mut connection = HttpConnection::new(tracker_handle, url, Duration::from_secs(2));

        runtime.block_on(connection.start()).unwrap();
        let result = runtime
            .block_on(connection.scrape(&vec![torrent_info.info_hash]))
            .unwrap();

        assert_eq!(
            1,
            result.files.len(),
            "expected the scrape result to match the torrent files"
        );
    }
}
