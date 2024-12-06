use crate::torrent::peer::protocol::Request;
use log::warn;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, Notify, RwLock};

const BUFFER_SIZE: usize = 25;

/// A buffer implementation for peer requests.
/// It uses an internal [VecDeque] to store the request data.
#[derive(Debug)]
pub struct PeerRequestBuffer {
    sender: Sender<Request>,
    receiver: Mutex<Receiver<Request>>,
    is_running: RwLock<bool>,
    notify: Notify,
}

impl PeerRequestBuffer {
    /// Create a new buffer for peer requests.
    /// It starts in the state given by the `running` parameter.
    pub fn new(running: bool) -> Self {
        let (sender, receiver) = channel(BUFFER_SIZE);
        Self {
            sender,
            receiver: Mutex::new(receiver),
            is_running: RwLock::new(running),
            notify: Notify::new(),
        }
    }

    /// Pauses the buffer for processing until it is resumed.
    /// This will make the [PeerRequestBuffer::next] not return any results until it's resumed.
    pub async fn pause(&self) {
        *self.is_running.write().await = false;
    }

    /// Resumes the buffer for being processed.
    pub async fn resume(&self) {
        *self.is_running.write().await = true;
        self.notify.notify_waiters();
    }

    /// Get the number of requests in the buffer.
    pub async fn len(&self) -> usize {
        self.receiver.lock().await.len()
    }

    /// Push a new request into the buffer.
    pub async fn push(&self, value: Request) {
        if let Err(e) = self.sender.send(value).await {
            warn!("Failed to buffer request, {}", e);
        }
    }

    /// Retrieve a value from the buffer.
    /// This method waits for a new request to become available.
    pub async fn next(&self) -> Option<Request> {
        if !*self.is_running.read().await {
            self.notify.notified().await;
        }

        self.receiver.lock().await.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, RecvTimeoutError};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    #[test]
    fn test_len() {
        let runtime = Runtime::new().expect("expected a runtime");
        let request = Request {
            index: 0,
            begin: 0,
            length: 1024,
        };
        let buffer = Arc::new(PeerRequestBuffer::new(false));

        runtime.block_on(buffer.push(request.clone()));
        runtime.block_on(buffer.push(request));
        runtime.block_on(buffer.len());

        let result = runtime.block_on(buffer.len());

        assert_eq!(2, result);
    }

    #[test]
    fn test_next_resume() {
        let runtime = Runtime::new().expect("expected a runtime");
        let expected_request = Request {
            index: 0,
            begin: 0,
            length: 1024,
        };
        let (tx, rx) = channel();
        let buffer = Arc::new(PeerRequestBuffer::new(false));

        let listener = buffer.clone();
        runtime.spawn(async move {
            tx.send(listener.next().await).unwrap();
        });

        runtime.block_on(buffer.push(expected_request.clone()));
        runtime.block_on(buffer.resume());

        let request = rx
            .recv_timeout(Duration::from_millis(50))
            .unwrap()
            .expect("expected a request to have been sent");

        assert_eq!(expected_request, request);
    }

    #[test]
    fn test_next_when_paused() {
        let runtime = Runtime::new().expect("expected a runtime");
        let expected_request = Request {
            index: 0,
            begin: 0,
            length: 1024,
        };
        let (tx, rx) = channel();
        let buffer = Arc::new(PeerRequestBuffer::new(true));

        let listener = buffer.clone();
        runtime.spawn(async move {
            tx.send(listener.next().await).unwrap();
        });

        runtime.block_on(buffer.pause());
        runtime.block_on(buffer.push(expected_request));

        let result = rx.recv_timeout(Duration::from_millis(50));
        assert_eq!(
            Err(RecvTimeoutError::Timeout),
            result,
            "expected the request to not have been sent"
        );
    }
}
