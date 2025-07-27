use popcorn_fx_torrent::torrent::peer::ProtocolExtensionFlags;
use popcorn_fx_torrent::torrent::FxTorrentSession;

/// Create a new simple torrent session.
#[tokio::main]
async fn main() {
    let _session = FxTorrentSession::builder()
        .client_name("My session name")
        .base_path("torrents")
        .protocol_extensions(
            ProtocolExtensionFlags::Fast
                | ProtocolExtensionFlags::LTEP
                | ProtocolExtensionFlags::Dht,
        )
        .build()
        .expect("failed to create torrent session");
}
