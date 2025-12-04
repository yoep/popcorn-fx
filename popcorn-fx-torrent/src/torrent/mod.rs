pub use compact::*;
pub use errors::*;
pub use file::*;
pub use info_hash::*;
pub use magnet::*;
use peer_pool::*;
pub use piece::*;
use piece_chunk_pool::*;
pub use session::*;
pub use session_cache::*;
pub use torrent::*;
use torrent_config::*;
pub use torrent_flags::*;
pub use torrent_health::*;
pub use torrent_metadata::*;
pub use torrent_metrics::*;
pub use torrent_peer::*;
use torrent_pools::*;

use std::ops::Range;

mod compact;
#[cfg(feature = "dht")]
pub mod dht;
mod errors;
mod file;
mod info_hash;
mod magnet;
mod merkle;
pub mod metrics;
pub mod operation;
pub mod peer;
mod peer_pool;
mod piece;
mod piece_chunk_pool;
mod session;
mod session_cache;
pub mod storage;
mod torrent;
mod torrent_config;
mod torrent_flags;
mod torrent_health;
mod torrent_metadata;
mod torrent_metrics;
mod torrent_peer;
mod torrent_pools;
pub mod tracker;

#[cfg(feature = "extension-donthave")]
use crate::torrent::peer::extension::donthave::DontHaveExtension;
#[cfg(feature = "extension-metadata")]
use crate::torrent::peer::extension::metadata::MetadataExtension;
#[cfg(feature = "extension-pex")]
use crate::torrent::peer::extension::pex::PexExtension;
use crate::torrent::peer::ProtocolExtensionFlags;

const DEFAULT_TORRENT_PROTOCOL_EXTENSIONS: fn() -> ProtocolExtensionFlags = || {
    ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast | ProtocolExtensionFlags::SupportV2
};
const DEFAULT_TORRENT_EXTENSIONS: fn() -> ExtensionFactories = || {
    let mut extensions: ExtensionFactories = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(|| Box::new(MetadataExtension::new()));
    #[cfg(feature = "extension-pex")]
    extensions.push(|| Box::new(PexExtension::new()));
    #[cfg(feature = "extension-donthave")]
    extensions.push(|| Box::new(DontHaveExtension::new()));

    extensions
};

/// Formats the given number of bytes into a human-readable format with appropriate units.
///
/// This function converts a byte size into a more readable format using common storage units (B, KB, MB, GB, TB).
/// The result is rounded to two decimal places for clarity. It ensures that the byte count is represented with
/// the most appropriate unit based on the size of the input. The units scale based on powers of 1024.
///
/// # Arguments
/// - `bytes`: The size in bytes to be formatted.
///
/// # Returns
///
/// It returns the formatted byte size with the corresponding unit.
///
/// # Example
///
/// ```rust,no_run
/// use popcorn_fx_torrent::torrent::format_bytes;
///
/// let formatted = format_bytes(1048576);
/// println!("{}", formatted); // "1.00 MB"
/// ```
///
/// # Notes
/// The function uses the binary system for scaling (i.e., 1024 bytes = 1 KB).
pub fn format_bytes(bytes: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    format!("{:.2} {}", value, units[unit])
}

/// Calculates the data transfer rate in bytes per second.
///
/// This function computes the data transfer rate based on the number of bytes transferred and the
/// elapsed time in microseconds. It returns the rate as bytes per second (B/s). If the elapsed time is less
/// than one second (1,000,000 microseconds), it simply returns the number of bytes as the rate.
///
/// # Arguments
/// - `bytes`: The number of bytes transferred.
/// - `elapsed_micro_secs`: The time elapsed in microseconds.
///
/// # Returns
/// A `u64` representing the data transfer rate in bytes per second (B/s).
///
/// # Example
///
/// ```rust,no_run
/// use popcorn_fx_torrent::torrent::calculate_byte_rate;
///
/// let rate = calculate_byte_rate(1_000_000, 1_500_000);
/// println!("{}", rate); // "666666" (bytes per second);
///
/// let rate = calculate_byte_rate(1_000_000, 2_000_000);
/// println!("{}", rate); // "500000" (bytes per second);
/// ```
///
/// # Notes
/// The function assumes that the elapsed time is given in microseconds. If the elapsed time is very short,
/// it will default to the total byte count as the rate.
pub fn calculate_byte_rate(bytes: usize, elapsed_micro_secs: u128) -> u64 {
    if elapsed_micro_secs <= 1_000_000 {
        return bytes as u64;
    }

    ((bytes as u128 * 1_000_000) / elapsed_micro_secs) as u64
}

/// Retrieves an available port on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Arguments
///
/// * `lower_bound` - The lower bound of the available port range (optional, default = 1000).
/// * `upper_bound` - The upper bound of the available port range (optional, default = [u16::MAX]).
///
/// # Returns
///
/// Returns an available port if one is found, else `None`.
#[macro_export]
macro_rules! available_port {
    ($lower_bound:expr, $upper_bound:expr) => {
        crate::torrent::available_port($lower_bound, $upper_bound)
    };
    ($lower_bound:expr) => {
        crate::torrent::available_port($lower_bound, u16::MAX)
    };
    () => {
        crate::torrent::available_port(1000, u16::MAX)
    };
}

/// Get the overlapping range of two ranges.
/// It returns the overlapping range if there is one, else [None].
#[inline]
pub(crate) fn overlapping_range<T>(r1: Range<T>, r2: &Range<T>) -> Option<Range<T>>
where
    T: Ord + Copy,
{
    let start = r1.start.max(r2.start);
    let end = r1.end.min(r2.end);

    if start < end {
        Some(start..end)
    } else {
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::torrent::peer::tests::new_tcp_peer_discovery;
    use crate::torrent::peer::{BitTorrentPeer, PeerDiscovery, PeerId, PeerStream};

    use log::trace;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Once;
    use std::time::Duration;
    use std::{env, fs};
    use tokio::net::TcpStream;
    use tokio::sync::mpsc::unbounded_channel;

    pub static INIT: Once = Once::new();

    /// Create the torrent metadata from the given uri.
    /// The uri can either point to a `.torrent` file or a magnet link.
    pub fn create_metadata(uri: &str) -> TorrentMetadata {
        if uri.starts_with("magnet:") {
            let magnet = Magnet::from_str(uri).unwrap();
            TorrentMetadata::try_from(magnet).unwrap()
        } else {
            let torrent_info_data = read_test_file_to_bytes(uri);
            TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap()
        }
    }

    #[macro_export]
    macro_rules! create_torrent {
        ($uri:expr, $temp_dir:expr, $options:expr) => {{
            create_torrent!(
                $uri,
                $temp_dir,
                $options,
                crate::torrent::TorrentConfig::builder()
                    .path($temp_dir)
                    .build()
            )
        }};
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr) => {{
            use crate::torrent::operation::{
                TorrentConnectPeersOperation, TorrentCreateFilesOperation,
                TorrentCreatePiecesOperation, TorrentDhtNodesOperation, TorrentDhtPeersOperation,
                TorrentFileValidationOperation, TorrentMetadataOperation, TorrentTrackersOperation,
            };

            create_torrent!(
                $uri,
                $temp_dir,
                $options,
                $config,
                vec![
                    || Box::new(TorrentTrackersOperation::new()),
                    || Box::new(TorrentDhtNodesOperation::new()),
                    || Box::new(TorrentDhtPeersOperation::new()),
                    || Box::new(TorrentConnectPeersOperation::new()),
                    || Box::new(TorrentMetadataOperation::new()),
                    || Box::new(TorrentCreatePiecesOperation::new()),
                    || Box::new(TorrentCreateFilesOperation::new()),
                    || Box::new(TorrentFileValidationOperation::new()),
                ]
            )
        }};
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr) => {{
            use crate::torrent::peer::{PeerDiscovery, TcpPeerDiscovery, UtpPeerDiscovery};

            let tcp_discovery = TcpPeerDiscovery::new()
                .await
                .expect("expected a new tcp peer discovery");
            let utp_discovery = UtpPeerDiscovery::new()
                .await
                .expect("expected a new utp peer discovery");
            let discoveries: Vec<Box<dyn PeerDiscovery>> =
                vec![Box::new(tcp_discovery), Box::new(utp_discovery)];

            create_torrent!($uri, $temp_dir, $options, $config, $operations, discoveries)
        }};
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $discoveries:expr) => {{
            create_torrent!(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $discoveries,
                |params| {
                    Box::new(crate::torrent::storage::DiskStorage::new(
                        params.info_hash,
                        params.path,
                        params.files,
                    ))
                }
            )
        }};
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $discoveries:expr, $storage:expr) => {{
            use crate::torrent::dht::DhtTracker;
            use crate::torrent::storage::DiskStorage;

            create_torrent!(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $discoveries,
                |params| {
                    Box::new(DiskStorage::new(
                        params.info_hash,
                        params.path,
                        params.files,
                    ))
                },
                Some(DhtTracker::builder().build().await.unwrap())
            )
        }};
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $discoveries:expr, $storage:expr, $dht:expr) => {{
            use crate::torrent::peer::PeerDiscovery;
            use crate::torrent::tests::create_metadata;
            use crate::torrent::tracker::TrackerClient;
            use crate::torrent::{Torrent, TorrentConfig, TorrentFlags, TorrentOperationFactory};
            use std::time::Duration;

            let uri: &str = $uri;
            let options: TorrentFlags = $options;
            let config: TorrentConfig = $config;
            let operations: Vec<TorrentOperationFactory> = $operations;
            let discoveries: Vec<Box<dyn PeerDiscovery>> = $discoveries;
            let dht: Option<DhtTracker> = $dht;
            let torrent_info = create_metadata(uri);
            let tracker_manager = TrackerClient::new(Duration::from_secs(2));
            let config = TorrentConfig::builder()
                .path($temp_dir)
                .peer_connection_timeout(config.peer_connection_timeout)
                .max_in_flight_pieces(config.max_in_flight_pieces)
                .peers_upper_limit(config.peers_upper_limit)
                .peers_lower_limit(config.peers_lower_limit)
                .build();

            Torrent::request()
                .metadata(torrent_info)
                .peer_discoveries(discoveries)
                .options(options)
                .config(config)
                .operations(operations.iter().map(|e| e()).collect())
                .storage($storage)
                .tracker_manager(tracker_manager)
                .dht_option(dht)
                .build()
                .unwrap()
        }};
    }

    /// A macro wrapper for [`tokio::time::timeout`] that awaits a future with a timeout duration.
    ///
    /// # Returns
    ///
    /// It returns the future result or timeout.
    #[macro_export]
    macro_rules! timeout {
        ($future:expr, $duration:expr) => {{
            use std::io;
            use tokio::time::timeout;
            let future = $future;
            let duration = $duration;

            timeout(duration, future)
                .await
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("after {}.{:03}s", duration.as_secs(), duration.as_millis()),
                    )
                })
                .expect("operation timed-out")
        }};
        ($future:expr, $duration:expr, $message:expr) => {{
            use std::io;
            use tokio::time::timeout;
            let future = $future;
            let duration = $duration;

            timeout(duration, future)
                .await
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("after {}.{:03}s", duration.as_secs(), duration.as_millis()),
                    )
                })
                .expect($message)
        }};
    }

    /// Receive a message result from the given receiver, or panic if the timeout is reached.
    /// Accepts an optional custom (panic) error message.
    #[macro_export]
    macro_rules! recv_timeout {
        ($receiver:expr, $duration:expr) => {{
            use crate::timeout;
            use tokio::pin;
            let receiver = $receiver;
            let future = pin!(receiver.recv());

            timeout!(future, $duration).unwrap()
        }};
        ($receiver:expr, $duration:expr, $message:expr) => {{
            use crate::timeout;
            use tokio::pin;
            let receiver = $receiver;
            let future = pin!(receiver.recv());

            timeout!(future, $duration, $message).unwrap()
        }};
    }

    #[macro_export]
    macro_rules! create_peer_pair {
        ($torrent:expr) => {
            crate::torrent::tests::create_tcp_peer_pair(
                $torrent,
                $torrent,
                $torrent
                    .instance()
                    .expect("expected a valid torrent context")
                    .protocol_extensions()
                    .clone(),
            )
            .await
        };
        ($torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair($torrent, $torrent, $protocols).await
        };
        ($incoming_torrent:expr, $outgoing_torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair(
                $incoming_torrent,
                $outgoing_torrent,
                $protocols,
            )
            .await
        };
    }

    pub async fn create_tcp_peer_pair(
        incoming_torrent: &Torrent,
        outgoing_torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (BitTorrentPeer, BitTorrentPeer) {
        let incoming_context = incoming_torrent.instance().unwrap();
        let outgoing_context = outgoing_torrent.instance().unwrap();
        let (tx, mut rx) = unbounded_channel();

        let incoming_context = incoming_context.clone();
        let extensions = incoming_context.extensions();
        let listener = new_tcp_peer_discovery().await.unwrap();
        let listener_port = listener.port();
        tokio::spawn(async move {
            if let Some(peer) = listener.recv().await {
                if let PeerStream::Tcp(stream) = peer.stream {
                    tx.send(
                        BitTorrentPeer::new_inbound(
                            PeerId::new(),
                            peer.socket_addr,
                            PeerStream::Tcp(stream),
                            incoming_context,
                            protocols.clone(),
                            extensions,
                            Duration::from_secs(5),
                        )
                        .await,
                    )
                    .unwrap()
                }
            }
        });

        let peer_context = outgoing_context.clone();
        let outgoing_extensions = outgoing_context.extensions();
        let addr = SocketAddr::new([127, 0, 0, 1].into(), listener_port);
        let stream = TcpStream::connect(addr).await.unwrap();
        let outgoing_peer = BitTorrentPeer::new_outbound(
            PeerId::new(),
            addr,
            PeerStream::Tcp(stream),
            peer_context,
            protocols,
            outgoing_extensions,
            Duration::from_secs(5),
        )
        .await
        .expect("expected the outgoing connection to succeed");

        let incoming_peer = timeout!(
            rx.recv(),
            Duration::from_secs(1),
            "expected an incoming peer"
        )
        .unwrap()
        .expect("expected an incoming peer");
        (incoming_peer, outgoing_peer)
    }

    /// Retrieve the path to the testing resource directory.
    ///
    /// It returns the [PathBuf] to the testing resources directory.
    pub fn test_resource_directory() -> PathBuf {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let mut source = PathBuf::from(root_dir);
        source.push("test");

        source
    }

    /// Retrieve the filepath of a testing resource file.
    /// These are files located within the "test" directory of the crate.
    ///
    /// It returns the created [PathBuf] for the given filename.
    pub fn test_resource_filepath(filename: &str) -> PathBuf {
        let mut source = test_resource_directory();
        source.push(filename);

        source
    }

    pub fn read_test_file_to_bytes(filename: &str) -> Vec<u8> {
        let source = test_resource_filepath(filename);

        fs::read(&source).unwrap()
    }

    pub fn copy_test_file(temp_dir: &str, filename: &str, output_filename: Option<&str>) -> String {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let source = PathBuf::from(root_dir).join("test").join(filename);
        let destination = PathBuf::from(temp_dir).join(output_filename.unwrap_or(filename));

        // make sure the parent dir exists
        fs::create_dir_all(destination.parent().unwrap()).unwrap();

        trace!("Copying test file {} to {:?}", filename, destination);
        fs::copy(&source, &destination).unwrap();

        destination.to_str().unwrap().to_string()
    }

    /// Initializes the logger with the specified log level.
    #[macro_export]
    macro_rules! init_logger {
        () => {{
            init_logger!(log::LevelFilter::Trace)
        }};
        ($level:expr) => {{
            use log4rs::config::runtime::{Appender, Config, Logger, Root};
            use log4rs::append::console::ConsoleAppender;
            use log4rs::encode::pattern::PatternEncoder;
            use log::LevelFilter;

            let level: LevelFilter = $level;

            crate::torrent::tests::INIT.call_once(|| {
                log4rs::init_config(Config::builder()
                    .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new("\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<60.60}\x1B[0m \x1B[37m:\x1B[0m {m}{n}")))
                        .build())))
                    .logger(Logger::builder().build("axum", LevelFilter::Info))
                    .logger(Logger::builder().build("fx_callback", LevelFilter::Info))
                    .logger(Logger::builder().build("hyper_util", LevelFilter::Info))
                    .logger(Logger::builder().build("mio", LevelFilter::Info))
                    .logger(Logger::builder().build("reqwest", LevelFilter::Info))
                    .build(Root::builder().appender("stdout").build(level))
                    .unwrap())
                    .unwrap();
            })
        }};
    }

    #[macro_export]
    macro_rules! assert_timeout {
        ($timeout:expr, $condition:expr) => {{
            assert_timeout!($timeout, $condition, "")
        }};
        ($timeout:expr, $condition:expr, $message:expr) => {{
            use std::time::Duration;
            use tokio::select;
            use tokio::time;

            let result = select! {
                _ = time::sleep($timeout) => false,
                result = async {
                    loop {
                        if $condition {
                            return true;
                        }

                        time::sleep(Duration::from_millis(10)).await;
                    }
                } => result,
            };

            if !result {
                assert!(
                    false,
                    concat!("Timeout assertion failed after {:?}: ", $message),
                    $timeout
                );
            }
        }};
    }

    mod overlapping_range {
        use super::*;

        #[test]
        fn test_overlap_range() {
            let r1 = 0..10;
            let r2 = 5..15;
            let result = overlapping_range(r1, &r2);
            assert_eq!(Some(5..10), result);

            let r1 = 16..32;
            let r2 = 30..64;
            let result = overlapping_range(r1, &r2);
            assert_eq!(Some(30..32), result);

            let r1 = 128..256;
            let r2 = 512..1024;
            let result = overlapping_range(r1, &r2);
            assert_eq!(None, result);
        }
    }
}
