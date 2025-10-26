use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) const DEFAULT_PEER_CLIENT_NAME: &str = "PopcornFX";
pub(crate) const DEFAULT_PEER_TIMEOUT: Duration = Duration::from_secs(6);
pub(crate) const DEFAULT_PEER_LOWER_LIMIT: usize = 10;
pub(crate) const DEFAULT_PEER_UPPER_LIMIT: usize = 200;
pub(crate) const DEFAULT_PEER_IN_FLIGHT: usize = 25;
pub(crate) const DEFAULT_PEER_UPLOAD_SLOTS: usize = 50;
pub(crate) const DEFAULT_MAX_IN_FLIGHT_PIECES: usize = 256;

/// The torrent configuration values.
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentConfig {
    client_name: String,
    path: PathBuf,
    pub peers_lower_limit: usize,
    pub peers_upper_limit: usize,
    pub peers_in_flight: usize,
    pub peers_upload_slots: usize,
    pub peer_connection_timeout: Duration,
    pub max_in_flight_pieces: usize,
}

impl TorrentConfig {
    /// Create a new torrent configuration builder.
    pub fn builder() -> TorrentConfigBuilder {
        TorrentConfigBuilder::builder()
    }

    /// Get the client name of the torrent.
    pub fn client_name(&self) -> &str {
        self.client_name.as_str()
    }

    /// Get the path of the torrent data.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Default for TorrentConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Default)]
pub struct TorrentConfigBuilder {
    client_name: Option<String>,
    path: Option<PathBuf>,
    peers_lower_limit: Option<usize>,
    peers_upper_limit: Option<usize>,
    peers_in_flight: Option<usize>,
    peers_upload_slots: Option<usize>,
    peer_connection_timeout: Option<Duration>,
    max_in_flight_pieces: Option<usize>,
}

impl TorrentConfigBuilder {
    /// Create a new torrent configuration builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the name of the client.
    pub fn client_name<S: AsRef<str>>(&mut self, name: S) -> &mut Self {
        self.client_name = Some(name.as_ref().to_string());
        self
    }

    /// Set the torrent data path.
    /// This is the path where the downloaded data will be stored.
    pub fn path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the lower limit for the number of peers.
    pub fn peers_lower_limit(&mut self, limit: usize) -> &mut Self {
        self.peers_lower_limit = Some(limit);
        self
    }

    /// Set the upper limit for the number of peers.
    pub fn peers_upper_limit(&mut self, limit: usize) -> &mut Self {
        self.peers_upper_limit = Some(limit);
        self
    }

    /// Set the max number of peer upload slots.
    pub fn peers_upload_slots(&mut self, slots: usize) -> &mut Self {
        self.peers_upload_slots = Some(slots);
        self
    }

    /// Set the timeout for peer connections.
    pub fn peer_connection_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.peer_connection_timeout = Some(timeout);
        self
    }

    /// Set the maximum number of in flight pieces which can be requested in parallel from peers.
    pub fn max_in_flight_pieces(&mut self, limit: usize) -> &mut Self {
        self.max_in_flight_pieces = Some(limit);
        self
    }

    /// Build the torrent configuration.
    pub fn build(&mut self) -> TorrentConfig {
        let client_name = self
            .client_name
            .take()
            .unwrap_or_else(|| DEFAULT_PEER_CLIENT_NAME.to_string());
        let path = self.path.take().unwrap_or_else(|| PathBuf::new());
        let peers_lower_limit = self
            .peers_lower_limit
            .take()
            .unwrap_or(DEFAULT_PEER_LOWER_LIMIT);
        let peers_upper_limit = self
            .peers_upper_limit
            .take()
            .unwrap_or(DEFAULT_PEER_UPPER_LIMIT);
        let peers_in_flight = self
            .peers_in_flight
            .take()
            .unwrap_or(DEFAULT_PEER_IN_FLIGHT);
        let peers_upload_slots = self
            .peers_upload_slots
            .take()
            .unwrap_or(DEFAULT_PEER_UPLOAD_SLOTS);
        let peer_connection_timeout = self
            .peer_connection_timeout
            .take()
            .unwrap_or(DEFAULT_PEER_TIMEOUT);
        let max_in_flight_pieces = self
            .max_in_flight_pieces
            .take()
            .unwrap_or(DEFAULT_MAX_IN_FLIGHT_PIECES);

        TorrentConfig {
            client_name,
            path,
            peers_lower_limit,
            peers_upper_limit,
            peers_in_flight,
            peers_upload_slots,
            peer_connection_timeout,
            max_in_flight_pieces,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let expected_result = TorrentConfigBuilder::builder().build();

        let result = TorrentConfig::default();

        assert_eq!(expected_result, result);
        assert_eq!(DEFAULT_PEER_CLIENT_NAME, result.client_name);
    }
}
