# FXTor

A rust native torrent implementation used within the PopcornFX application.
It's based on the `libtorrent` library for functionality and naming convention.

Both V1 & V2 of the Bittorrent protocol specification have been implemented.

## Features

- [ ] [BEP3](https://www.bittorrent.org/beps/bep_0003.html)
  - [x] UDP Trackers
  - [ ] HTTP Trackers
  - [ ] HTTPS Trackers
- [ ] [BEP5](https://www.bittorrent.org/beps/bep_0005.html)
  - [ ] DHT
- [ ] [BEP9](https://www.bittorrent.org/beps/bep_0009.html)
  - [ ] Extension for Peers to Send Metadata Files
- [x] [BEP10](https://www.bittorrent.org/beps/bep_0010.html)
  - [x] Extension Protocol
- [x] [BEP53](https://www.bittorrent.org/beps/bep_0053.html)
  - [x] Magnets

## Usage

Every interaction with `Torrents`, `Trackers` or `Peers`, requires the use of a `Session` which isolates torrents from each other.
It is however possible to interact with `.torrent` metadata through the `TorrentInfo` without making use of any `Session`.

_Create a new session_
```rust
use tokio::runtime::Runtime;
use popcorn_fx_torrents::torrents::{DefaultSession, Session};

fn main() {
  let runtime = Arc::new(Runtime::new().unwrap());
  // always provide a tokio runtime which is used to run all async operation in the background
  let session : Session = DefaultSession::new(runtime);
}
```

For more examples, check the tests which are present within the crate.