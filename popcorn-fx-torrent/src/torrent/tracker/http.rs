use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Result, TrackerConnection, TrackerError,
};
use crate::torrent::{CompactIpv4Addrs, InfoHash};
use async_trait::async_trait;
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
#[derive(Debug)]
pub struct HttpConnection {
    peer_id: PeerId,
    url: Url,
    client: Client,
    cancellation_token: CancellationToken,
}

impl HttpConnection {
    pub fn new(url: Url, peer_id: PeerId, timeout: Duration) -> Self {
        let client = Client::builder()
            .redirect(Policy::limited(3))
            .timeout(timeout)
            .build()
            .expect("expected a valid http client");

        Self {
            peer_id,
            url,
            client,
            cancellation_token: Default::default(),
        }
    }

    fn create_announce_url(&self, info_hash: InfoHash, event: AnnounceEvent) -> Result<Url> {
        let mut url = self.url.clone();
        let url_encoded_hash = percent_encoding::percent_encode(
            info_hash.short_info_hash_bytes().as_slice(),
            URL_ENCODE_RESERVED,
        )
        .to_string();

        let base_url = url
            .query_pairs_mut()
            .append_pair("peer_id", &self.peer_id.to_string())
            .append_pair("port", "0")
            .append_pair("uploaded", "0")
            .append_pair("downloaded", "0")
            .append_pair("left", u64::MAX.to_string().as_str())
            .append_pair("event", event.to_string().as_str())
            .append_pair("key", "0")
            .append_pair("compact", "1")
            .append_pair("numwant", "100")
            .finish()
            .to_string();
        let url = format!("{}&info_hash={}", base_url, url_encoded_hash);

        Ok(Url::parse(&url)?)
    }

    async fn process_response(
        &self,
        response: std::result::Result<Response, Error>,
        url: Url,
    ) -> Result<AnnounceEntryResponse> {
        let bytes = response?.bytes().await?;
        trace!(
            "Received {} bytes from {}, {}",
            bytes.len(),
            url,
            String::from_utf8_lossy(&bytes)
        );
        let message = serde_bencode::from_bytes::<HttpResponse>(bytes.as_ref())?;
        debug!("Received tracker {} response, {:?}", self.url, message);

        // check if the response message contains an error
        if let Some(failure) = message.failure_reason {
            return Err(TrackerError::AnnounceError(failure));
        }

        Ok(message.into())
    }
}

#[async_trait]
impl TrackerConnection for HttpConnection {
    async fn start(&mut self) -> Result<()> {
        let url = self.url.clone();

        // check if we're able to connect
        trace!("Trying to connect to {}", url);
        self.client.head(url).send().await?;
        Ok(())
    }

    async fn announce(
        &self,
        info_hash: InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse> {
        let url = self.create_announce_url(info_hash, event)?;

        trace!("Sending announce request to {}", url);
        select! {
            _ = self.cancellation_token.cancelled() => Err(TrackerError::Timeout(url.clone())),
            response = self.client.get(url.clone()).send() => self.process_response(response, url).await,
        }
    }

    fn close(&mut self) {
        self.cancellation_token.cancel()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::TorrentMetadata;
    use log::info;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_start() {
        init_logger!();
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let mut connection = HttpConnection::new(url, peer_id, Duration::from_secs(2));

        let result = connection.start().await;

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_create_announce_url() {
        init_logger!();
        let expected_hash_value = "info_hash=.%8ED%06%8B%25H%14%EA%1A%7DIi%A9%AF%1Dx%E0%F5%1F";
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let connection = HttpConnection::new(url, peer_id, Duration::from_secs(2));

        let url = connection
            .create_announce_url(torrent_info.info_hash, AnnounceEvent::Started)
            .unwrap();
        let result = url.query().unwrap();

        info!("Got url parameters {}", result);
        assert!(
            result.contains(expected_hash_value),
            "expected the info hash to be present"
        );
    }

    #[test]
    fn test_announce() {
        init_logger!();
        let runtime = Runtime::new().expect("expected a runtime");
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let mut connection = HttpConnection::new(url, peer_id, Duration::from_secs(2));

        runtime.block_on(connection.start()).unwrap();
        let result = runtime
            .block_on(connection.announce(torrent_info.info_hash, AnnounceEvent::Started))
            .unwrap();

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
}
