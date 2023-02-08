use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use derive_more::Display;
use tokio::sync::Mutex;

use popcorn_fx_core::core::torrent::Torrent;

pub type HasByteCallback = Box<dyn Fn(&[u64]) -> bool + Send>;

#[derive(Display)]
#[display(fmt = "filepath: {:?}", filepath)]
pub struct TorrentWrapper {
    filepath: PathBuf,
    has_byte: Mutex<HasByteCallback>,
}

impl TorrentWrapper {
    pub fn new(filepath: String, has_byte: HasByteCallback) -> Self {
        Self {
            filepath: PathBuf::from(filepath),
            has_byte: Mutex::new(has_byte),
        }
    }
}

impl Debug for TorrentWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filepath: {:?}", self.filepath)
    }
}

impl Torrent for TorrentWrapper {
    fn file(&self) -> PathBuf {
        self.filepath.clone()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        tokio::task::block_in_place(move || {
            let mutex = self.has_byte.blocking_lock();
            mutex(bytes)
        })
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {}
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_has_bytes() {
        let (tx, rx) = channel();
        let callback: HasByteCallback = Box::new(move |byte| {
            tx.send(byte.to_vec()).unwrap();
            true
        });
        let wrapper = TorrentWrapper::new("lorem.txt".to_string(), callback);
        let bytes = vec![2, 3];

        let result = wrapper.has_bytes(&bytes[..]);
        let result_byte = rx.recv_timeout(Duration::from_secs(3)).unwrap();

        assert!(result, "expected true to have been returned");
        assert_eq!(bytes, result_byte)
    }
}