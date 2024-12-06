use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Result, TrackerConnection, TrackerError,
};
use crate::torrent::InfoHash;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::{debug, trace};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use reqwest::redirect::Policy;
use reqwest::{Client, Error, Response};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::net::{Ipv4Addr, SocketAddr};
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
    #[serde(default, deserialize_with = "deserialize_peers_type")]
    pub peers: Vec<HttpPeer>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct HttpPeer {
    pub ip: String,
    pub port: u16,
    #[serde(rename = "peer id", default)]
    pub id: Vec<u8>,
}

impl Into<SocketAddr> for HttpPeer {
    fn into(self) -> SocketAddr {
        let ip_addr = self.ip.parse().expect("expected a valid ip address");
        SocketAddr::new(ip_addr, self.port)
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

struct HttpPeerVisitor {}

impl HttpPeerVisitor {
    fn parse_peers_bytes(bytes: &[u8]) -> Vec<HttpPeer> {
        let mut peers = Vec::new();
        let peer_count = bytes.len() / 6;

        for i in 0..peer_count {
            let start = i * 6;
            let end = start + 6;
            let peer = &bytes[start..end];

            peers.push(HttpPeer {
                ip: Ipv4Addr::new(peer[0], peer[1], peer[2], peer[3]).to_string(),
                port: u16::from_be_bytes([peer[4], peer[5]]),
                id: Vec::with_capacity(0),
            });
        }

        trace!("Parsed {} peers from http tracker", peers.len());
        peers
    }
}

impl<'de> Visitor<'de> for HttpPeerVisitor {
    type Value = Vec<HttpPeer>;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a string, sequence or byte array of http peers")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let value = BASE64_STANDARD
            .decode(v)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        Ok(Self::parse_peers_bytes(value.as_ref()))
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut peers = Vec::new();

        while let Ok(Some(peer)) = seq.next_element::<HttpPeer>() {
            peers.push(peer);
        }

        Ok(peers)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::parse_peers_bytes(v))
    }
}

fn deserialize_peers_type<'de, D>(deserializer: D) -> std::result::Result<Vec<HttpPeer>, D::Error>
where
    D: Deserializer<'de>,
{
    D::deserialize_any(deserializer, HttpPeerVisitor {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::TorrentInfo;
    use log::info;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_start() {
        init_logger();
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let peer_id = PeerId::new();
        let url = torrent_info.trackers().get(0).cloned().unwrap();
        let mut connection = HttpConnection::new(url, peer_id, Duration::from_secs(2));

        let result = connection.start().await;

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_create_announce_url() {
        init_logger();
        let expected_hash_value = "A%E6%CDP%CC%ECU%CDW%04%C5%E3%D1v%E7%B5%93%17%A3%FB";
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
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
        init_logger();
        let runtime = Runtime::new().expect("expected a runtime");
        let torrent_info_data = read_test_file_to_bytes("debian.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
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
