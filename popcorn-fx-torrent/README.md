# FXTor

A rust native torrent implementation used within the PopcornFX application.
It's based on the `libtorrent` library for functionality and naming convention.

Both V1 & V2 of the Bittorrent protocol specification have been implemented.

## Features

- [x] [BEP3](https://www.bittorrent.org/beps/bep_0003.html) - The BitTorrent Protocol Specification
- [x] [BEP4](https://www.bittorrent.org/beps/bep_0004.html) - Assigned Numbers
- [ ] [BEP5](https://www.bittorrent.org/beps/bep_0005.html) - DHT Protocol
- [x] [BEP6](https://www.bittorrent.org/beps/bep_0006.html) - Fast Extension
- [x] [BEP9](https://www.bittorrent.org/beps/bep_0009.html) - Extension for Peers to Send Metadata Files
- [x] [BEP10](https://www.bittorrent.org/beps/bep_0010.html) - Extension Protocol
- [x] [BEP11](https://www.bittorrent.org/beps/bep_0011.html) - Peer Exchange (PEX)
- [x] [BEP12](https://www.bittorrent.org/beps/bep_0012.html) - Multitracker Metadata Extension
- [x] [BEP15](https://www.bittorrent.org/beps/bep_0015.html) - UDP Tracker Protocol for BitTorrent
- [x] [BEP19](https://www.bittorrent.org/beps/bep_0019.html) - WebSeed - HTTP/FTP Seeding (GetRight style)
- [x] [BEP20](https://www.bittorrent.org/beps/bep_0020.html) - Peer ID Conventions
- [x] [BEP21](https://www.bittorrent.org/beps/bep_0021.html) - Extension for partial seeds
- [ ] [BEP29](https://www.bittorrent.org/beps/bep_0029.html) - uTorrent transport protocol
- [ ] [BEP40](https://www.bittorrent.org/beps/bep_0040.html) - Canonical Peer Priority
- [x] [BEP47](https://www.bittorrent.org/beps/bep_0047.html) - Padding files and extended file attributes
- [x] [BEP48](https://www.bittorrent.org/beps/bep_0048.html) - Tracker Protocol Extension: Scrape
- [ ] [BEP52](https://www.bittorrent.org/beps/bep_0052.html) - The BitTorrent Protocol Specification v2
- [x] [BEP53](https://www.bittorrent.org/beps/bep_0053.html) - Magnets
- [ ] [BEP54](https://www.bittorrent.org/beps/bep_0054.html) - The lt_donthave extension
- [ ] [BEP55](https://www.bittorrent.org/beps/bep_0055.html) - Holepunch extension

## Installation

To install the library, add the following cargo dependency.

```toml
[dependencies]
popcorn-fx-torrent = { path = "../popcorn-fx-torrent" }
```

## Usage

Every interaction with `Torrent`, `Tracker` or `Peer`, requires the use of a `Session` which isolates torrents from each other.
It is however possible to interact with `.torrent` metadata through the `TorrentInfo` without making use of any `Session`.

_Create a new session_
```rust
use tokio::runtime::Runtime;
use popcorn_fx_torrents::torrents::{DefaultSession, Session};

fn main() {
  let runtime = Arc::new(Runtime::new().unwrap());
  // it's recommended, but not required, to provide a shared runtime
  let session: Session = DefaultSession::builder()
    .base_path("/torrent/location/directory")
    .client_name("MyClient")
    .runtime(shared_runtime)
    .build();
}
```

For more examples, check the tests which are present within the crate.