use std::{fs, thread};
use std::borrow::BorrowMut;
use std::future::Future;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use derive_more::Display;
use futures::Stream;
use log::{debug, error, trace};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
use tokio::runtime;
use url::Url;

use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{StreamBytesResult, Torrent, TorrentStream, TorrentStreamingResource};

#[derive(Debug)]
pub struct DefaultTorrentStream {
    /// The backing torrent of this stream.
    torrent: Arc<Box<dyn Torrent>>,
    /// The url on which this stream is being hosted.
    url: Url,
}

impl DefaultTorrentStream {
    pub fn new(url: Url, torrent: Box<dyn Torrent>) -> Self {
        Self {
            torrent: Arc::new(torrent),
            url,
        }
    }
}

impl Torrent for DefaultTorrentStream {
    fn file(&self) -> PathBuf {
        self.torrent.file()
    }

    fn has_byte(&self, byte: u64) -> bool {
        self.torrent.has_byte(byte)
    }

    fn prioritize_byte(&self, byte: u64) {
        self.torrent.prioritize_byte(byte)
    }
}

impl TorrentStream for DefaultTorrentStream {
    fn url(&self) -> Url {
        self.url.clone()
    }

    fn stream(&self) -> torrent::Result<TorrentStreamingResource> {
        DefaultTorrentStreamingResource::new(&self.torrent)
            .map(|e| TorrentStreamingResource::new(e))
    }
}

/// The default implementation of a [Stream] for torrents.
#[derive(Debug, Display)]
#[display(fmt = "torrent: {:?}, file: {:?}, cursor: {}", torrent, filepath, cursor)]
pub struct DefaultTorrentStreamingResource {
    torrent: Arc<Box<dyn Torrent>>,
    runtime: runtime::Runtime,
    /// The open reader handle to the torrent file
    file: fs::File,
    filepath: PathBuf,
    /// The current reading cursor for the stream
    cursor: u64,
}

impl DefaultTorrentStreamingResource {
    pub fn new(torrent: &Arc<Box<dyn Torrent>>) -> torrent::Result<Self> {
        debug!("Creating a new streaming resource for torrent {:?}", torrent);
        futures::executor::block_on(async {
            let filepath = torrent.file();

            trace!("Opening torrent file {:?}", &filepath);
            fs::OpenOptions::new()
                .read(true)
                .open(&filepath)
                .map(|file| {
                    Self {
                        torrent: torrent.clone(),
                        runtime: runtime::Runtime::new().expect("expected a new runtime"),
                        file,
                        filepath: filepath.clone(),
                        cursor: 0,
                    }
                })
                .map_err(|e| {
                    let file = filepath;
                    let filepath = file.as_path().to_str().expect("expected a valid path");
                    torrent::TorrentError::FileNotFound(filepath.to_string())
                })
        })
    }

    /// Wait for the current cursor to become available.
    fn wait_for(&self, cx: &mut Context) -> Poll<Option<StreamBytesResult>> {
        let torrent = self.torrent.clone();
        let expected_byte = self.cursor.clone();
        let waker = cx.waker().clone();
        let stream_info = self.to_string();

        torrent.prioritize_byte(self.cursor);

        self.runtime.spawn(async move {
            while !torrent.has_byte(expected_byte) {
                trace!("Waiting for byte {} to be available", &expected_byte);
                thread::sleep(Duration::from_millis(5))
            }

            trace!("Awakening torrent stream {{{}}}", stream_info);
            waker.wake();
        });

        return Poll::Pending;
    }

    /// Read the data of the stream at the current cursor.
    fn read_data(&mut self) -> Option<StreamBytesResult> {
        let mut reader = &mut self.file;
        let cursor = self.cursor.clone();
        let current_pos = reader.stream_position().unwrap();
        let mut buffer = vec![0];

        match reader.seek(SeekFrom::Start(cursor)) {
            Err(e) => {
                error!("Failed to modify the file cursor to {}, {}", &self.cursor, e);
                return None;
            }
            Ok(_) => {}
        }

        match reader.read(&mut buffer) {
            Err(e) => {
                error!("Failed to read the file cursor data, {}", e);
                None
            }
            Ok(size) => {
                if size == 0 {
                    trace!("Reached EOF for {:?}", &self.filepath);
                    return None;
                }

                self.cursor += 1;
                Some(Ok(Bytes::from(buffer)))
            }
        }
    }
}

impl Stream for DefaultTorrentStreamingResource {
    type Item = StreamBytesResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.torrent.has_byte(self.cursor) {
            return self.wait_for(cx);
        }

        Poll::Ready(self.as_mut().read_data())
    }
}

#[cfg(test)]
mod test {
    use futures::{StreamExt, TryStreamExt};
    use tokio::runtime;

    use popcorn_fx_core::core::torrent::{MockTorrent, StreamBytes};
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_poll_next_byte_not_present() {
        init_logger();
        let filename = "simple.txt";
        let runtime = runtime::Runtime::new().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join(filename);
        let mut a = Some(true);
        let mut mock = MockTorrent::new();
        mock.expect_file()
            .returning(move || temp_path.clone());
        mock.expect_has_byte()
            .returning(move |e| {
                if a.is_some() {
                    a.take();
                    return false;
                }

                true
            });
        mock.expect_prioritize_byte()
            .times(1)
            .return_const(());
        let torrent = Arc::new(Box::new(mock) as Box<dyn Torrent>);
        copy_test_file(temp_dir.path().to_str().unwrap(), filename);
        let mut stream = DefaultTorrentStreamingResource::new(&torrent).unwrap();

        let result = runtime.block_on(async {
            let mut data: Option<StreamBytes>;
            let mut result: Vec<u8> = vec![];

            loop {
                data = stream.try_next().await.unwrap();
                if data.is_some() {
                    result.append(&mut data.unwrap().to_vec());
                } else {
                    break;
                }
            }

            String::from_utf8(result)
        }).expect("expected a valid string");

        assert_eq!("Lorem ipsum dolor".to_string(), result)
    }
}