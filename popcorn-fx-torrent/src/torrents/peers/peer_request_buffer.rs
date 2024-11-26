use crate::torrents::peers::bt_protocol::Request;
use std::collections::VecDeque;
use tokio::sync::{Notify, RwLock};

/// A buffer implementation for peer requests.
/// It uses an internal [VecDeque] to store the request data.
#[derive(Debug)]
pub struct PeerRequestBuffer {
    data: RwLock<VecDeque<Request>>,
    is_running: RwLock<bool>,
    queued_notifications: RwLock<usize>,
    notify: Notify,
}

impl PeerRequestBuffer {
    /// Create a new buffer for peer requests.
    /// It starts in the state given by the `running` parameter.
    pub fn new(running: bool) -> Self {
        Self {
            data: RwLock::new(VecDeque::with_capacity(0)),
            is_running: RwLock::new(running),
            queued_notifications: RwLock::new(0),
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

        for _ in 0..*self.queued_notifications.read().await {
            self.notify.notify_one();
        }

        *self.queued_notifications.write().await = 0;
    }

    /// Get the number of requests in the buffer.
    pub async fn len(&self) -> usize {
        self.data.read().await.len()
    }

    /// Push a new request into the buffer.
    pub async fn push(&self, value: Request) {
        {
            let mut mutex = self.data.write().await;
            let buffer_len = mutex.len();
            mutex.insert(buffer_len, value);
        }

        if *self.is_running.read().await {
            self.notify.notify_one();
        } else {
            *self.queued_notifications.write().await += 1;
        }
    }

    /// Retrieve a value from the buffer.
    /// This method waits for a new request to become available.
    pub async fn next(&self) -> Request {
        let mut is_queued = false;

        loop {
            self.notify.notified().await;

            if *self.is_running.read().await {
                return self
                    .data
                    .write()
                    .await
                    .pop_front()
                    .expect("expected a request to have been present");
            } else if !is_queued {
                *self.queued_notifications.write().await += 1;
                is_queued = true;
            }
        }
    }

    /// Clear all requests from the buffer.
    pub async fn clear(&self) {
        let mut mutex = self.data.write().await;
        mutex.clear();
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
