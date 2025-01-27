use crate::torrent::peer::{
    ConnectionDirection, Error, Peer, PeerClientInfo, PeerEvent, PeerHandle, PeerId, PeerState,
    PeerStats, Result,
};
use crate::torrent::{PieceIndex, TorrentContext, TorrentFileInfo, TorrentMetadata};
use async_trait::async_trait;
use bit_vec::BitVec;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, warn};
use percent_encoding::{percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::redirect::Policy;
use reqwest::Client;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use url::Url;

const URL_ENCODE_RESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'~')
    .remove(b'.');

/// The HTTP peer, also known as webseed, implementation that exchanges data with a HTTP server.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct HttpPeer {
    inner: Arc<HttpPeerContext>,
}

impl HttpPeer {
    pub fn new(url: Url, torrent: Arc<TorrentContext>, runtime: Arc<Runtime>) -> Result<Self> {
        let handle = Handle::new();
        let client = Client::builder()
            .redirect(Policy::limited(3))
            .build()
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        let addr = url
            .socket_addrs(|| match url.scheme() {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            })
            .unwrap_or(Vec::new())
            .pop()
            .unwrap_or(SocketAddr::from(([120, 0, 0, 1], 80)));
        let inner = Arc::new(HttpPeerContext {
            handle,
            client,
            client_info: PeerClientInfo {
                handle,
                id: PeerId::new(),
                addr,
                connection_type: ConnectionDirection::Outbound,
            },
            url,
            addr,
            stats: RwLock::new(Default::default()),
            torrent,
            callbacks: MultiThreadedCallback::new(runtime.clone()),
            cancellation_token: Default::default(),
        });

        let main_inner = inner.clone();
        runtime.spawn(async move { main_inner.start().await });

        Ok(Self { inner })
    }
}

#[async_trait]
impl Peer for HttpPeer {
    fn handle(&self) -> PeerHandle {
        self.inner.handle
    }

    fn handle_as_ref(&self) -> &PeerHandle {
        &self.inner.handle
    }

    fn client(&self) -> PeerClientInfo {
        self.inner.client_info.clone()
    }

    fn addr(&self) -> SocketAddr {
        self.inner.addr
    }

    fn addr_as_ref(&self) -> &SocketAddr {
        &self.inner.addr
    }

    async fn state(&self) -> PeerState {
        PeerState::Idle
    }

    async fn stats(&self) -> PeerStats {
        self.inner.stats.read().await.clone()
    }

    async fn remote_piece_bitfield(&self) -> BitVec {
        let total_pieces = self.inner.torrent.total_pieces().await;
        BitVec::from_elem(total_pieces, true)
    }

    fn notify_piece_availability(&self, _: Vec<PieceIndex>) {
        // no-op
    }

    async fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

impl Callback<PeerEvent> for HttpPeer {
    fn subscribe(&self) -> Subscription<PeerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PeerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for HttpPeer {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", client_info)]
struct HttpPeerContext {
    handle: PeerHandle,
    client: Client,
    client_info: PeerClientInfo,
    url: Url,
    addr: SocketAddr,
    stats: RwLock<PeerStats>,
    torrent: Arc<TorrentContext>,
    callbacks: MultiThreadedCallback<PeerEvent>,
    cancellation_token: CancellationToken,
}

impl HttpPeerContext {
    async fn start(&self) {
        let mut interval = interval(Duration::from_secs(3));

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = interval.tick() => self.check_for_wanted_pieces().await,
            }
        }

        debug!("Http peer {} main loop ended", self);
    }

    async fn check_for_wanted_pieces(&self) {
        let wanted_pieces = self
            .torrent
            .wanted_request_pieces()
            .await
            .into_iter()
            .take(3);
        let metadata = self.torrent.metadata().await;

        for piece in wanted_pieces {
            // request a permit and release it after requesting the piece, don't release it while requesting
            if let Some(_permit) = self.torrent.request_download_permit(&piece).await {
                if let Err(e) = self.request_piece(piece, &metadata).await {
                    warn!(
                        "Torrent {} failed to request webseed data from {}, {}",
                        self.torrent, self, e
                    );
                    break;
                }
            }
        }
    }

    /// Try to request the given piece.
    /// It returns an error if the piece couldn't be requested from the webseed.
    async fn request_piece(&self, piece: PieceIndex, metadata: &TorrentMetadata) -> Result<()> {
        if let Some(piece) = self.torrent.pieces_lock().read().await.get(piece).cloned() {
            let file = self.torrent.find_relevant_files_for_piece(&piece).await;

            if let Some(file) = file.get(0) {
                let url = self.create_request_url(metadata, &file.info)?;
                let range_start = piece.offset;
                let range_end = piece.offset + piece.length;

                let response = self
                    .client
                    .get(url)
                    .header("Range", format!("bytes={}-{}", range_start, range_end))
                    .send()
                    .await
                    .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?;
                let mut stats = self.stats.write().await;
                stats.download += response.content_length().unwrap_or(0) as usize;

                if response.status().is_success() {
                    let body = response
                        .bytes()
                        .await
                        .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?;
                    stats.download_useful += body.len();

                    // loop over each part that needs to be completed and fetch it from the body
                    for part in piece.parts_to_request() {
                        let data_len = body.len();
                        let part_end = part.begin + part.length;
                        if part_end > data_len {
                            return Err(Error::Io(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "part end {} is out of bound for response data length {}",
                                    part_end, data_len
                                ),
                            )));
                        }

                        let data = &body[part.begin..part_end];
                        self.torrent
                            .piece_part_completed(part.clone(), data.to_vec());
                    }
                } else {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "expected status 200, but got {:?} instead",
                            response.status()
                        ),
                    )));
                }
            }

            Ok(())
        } else {
            Err(Error::InvalidPiece(piece))
        }
    }

    fn create_request_url(
        &self,
        metadata: &TorrentMetadata,
        file: &TorrentFileInfo,
    ) -> Result<Url> {
        let path = Self::create_filepath(metadata, file)?;
        let mut encoded_path_segments = Vec::new();
        let mut url = self.url.clone();

        for segment in path.iter() {
            encoded_path_segments.push(
                percent_encode(segment.to_string_lossy().as_bytes(), URL_ENCODE_RESERVED)
                    .to_string(),
            )
        }

        // remove trailing slash from the base URL if it exists
        if url.path().ends_with('/') {
            let path = url.path().to_string();
            url.set_path(&path[..url.path().len() - 1]);
        }

        // update the path segments of the url
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| Error::Parsing("invalid base url".to_string()))?;

            for segment in encoded_path_segments {
                segments.push(&segment);
            }
        }

        Ok(url)
    }

    fn create_filepath(metadata: &TorrentMetadata, file: &TorrentFileInfo) -> Result<PathBuf> {
        if let Some(name) = metadata.info.as_ref().map(|e| e.name()) {
            let mut path = PathBuf::from(name);

            if file.path_segments().len() > 0 {
                path.push(file.path());
            }

            return Ok(path);
        }

        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("unable to create filepath for {:?}", file),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use log::LevelFilter;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use tempfile::tempdir;

    #[test]
    fn test_http_peer_create_request_url() {
        init_logger!(LevelFilter::Debug);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let url = Url::parse("https://mirror.com/pub/").unwrap();
        let expected_result =
            Url::parse("https://mirror.com/pub/debian-11.6.0-amd64-netinst.iso/README%25201.md")
                .unwrap();
        let torrent = create_torrent!(
            "debian.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let metadata = runtime.block_on(context.metadata());
        let file = TorrentFileInfo {
            length: 0,
            path: Some(vec!["README 1.md".to_string()]),
            path_utf8: None,
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let peer =
            HttpPeer::new(url, context.clone(), runtime.clone()).expect("expected an http peer");

        let result = peer
            .inner
            .create_request_url(&metadata, &file)
            .expect("expected the request url to be created");

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_http_peer_create_filepath() {
        let expected_result = PathBuf::from("debian-11.6.0-amd64-netinst.iso");
        let torrent = read_test_file_to_bytes("debian.torrent");
        let metadata = TorrentMetadata::try_from(torrent.as_slice()).unwrap();
        let files = metadata.info.as_ref().unwrap().files();
        let file = files.get(0).expect("expected a file to have been present");

        let result = HttpPeerContext::create_filepath(&metadata, file)
            .expect("expected a filepath to have been returned");
        assert_eq!(expected_result, result);

        let expected_result = PathBuf::from("debian-11.6.0-amd64-netinst.iso/README.md");
        let file = TorrentFileInfo {
            length: 0,
            path: Some(vec!["README.md".to_string()]),
            path_utf8: None,
            md5sum: None,
            attr: None,
            symlink_path: None,
            sha1: None,
        };
        let result = HttpPeerContext::create_filepath(&metadata, &file)
            .expect("expected a filepath to have been returned");
        assert_eq!(expected_result, result);
    }
}
