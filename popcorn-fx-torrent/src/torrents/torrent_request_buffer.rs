use crate::torrents::peers::PeerHandle;
use crate::torrents::{PieceIndex, PiecePart};
use popcorn_fx_core::core::block_in_place;
use std::time::Instant;
use tokio::sync::{Mutex, Notify, RwLock};

/// The timeout after which a request will be retried for execution
const REQUEST_TIMEOUT_MILLIS: u128 = 60 * 1000; // 60 seconds

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRequest {
    /// The piece part that is being requested.
    pub part: PiecePart,
    /// The peers that are retrieving this part
    pub pending_peers: Vec<PeerHandle>,
    /// The time the request was made
    pub requested_at: Option<Instant>,
}

impl PendingRequest {
    pub fn new(part: PiecePart) -> Self {
        Self {
            part,
            pending_peers: Vec::with_capacity(0),
            requested_at: None,
        }
    }
}

#[derive(Debug)]
pub struct PendingRequestBuffer {
    requests: RwLock<Vec<PendingRequest>>,
    in_flight: Mutex<usize>,
    max_in_flight: usize,
    cursor: RwLock<usize>,
    notifier: Notify,
}

impl PendingRequestBuffer {
    /// Get a new request buffer.
    /// It will have a maximum of `max_in_flight` requests in flight at any given time.
    pub fn new(max_in_flight: usize) -> Self {
        Self {
            requests: RwLock::new(Vec::with_capacity(0)),
            in_flight: Mutex::new(0),
            max_in_flight,
            cursor: Default::default(),
            notifier: Default::default(),
        }
    }

    /// Get the number of pending requests.
    pub async fn len(&self) -> usize {
        self.requests.read().await.len()
    }

    /// Get the pending requested parts stored in the buffer.
    pub async fn pending_parts(&self) -> Vec<PiecePart> {
        self.requests
            .read()
            .await
            .iter()
            .map(|e| e.part.clone())
            .collect()
    }

    /// Push a new request onto the buffer.
    pub async fn push(&self, request: PendingRequest) {
        self.requests.write().await.push(request);
        self.notifier.notify_one();
    }

    /// Push multiple requests onto the buffer.
    pub async fn push_all(&self, requests: Vec<PendingRequest>) {
        let request_len = requests.len();
        self.requests.write().await.extend(requests);

        // notify as many times as there are requests
        for _ in 0..request_len {
            self.notifier.notify_one();
        }
    }

    /// Check if the given piece is currently being requested.
    pub fn has_piece(&self, piece: PieceIndex) -> bool {
        block_in_place(self.has_piece_async(piece))
    }

    /// Check if the given piece part is currently being requested.
    pub fn is_pending(&self, part: &PiecePart) -> bool {
        block_in_place(self.is_pending_async(part))
    }

    /// Check if the given piece is currently being requested.
    pub async fn has_piece_async(&self, piece: PieceIndex) -> bool {
        self.requests
            .read()
            .await
            .iter()
            .find(|e| e.part.piece == piece)
            .is_some()
    }

    /// Check if the given piece part is currently being requested.
    pub async fn is_pending_async(&self, part: &PiecePart) -> bool {
        self.requests
            .read()
            .await
            .iter()
            .find(|e| &e.part == part)
            .is_some()
    }

    /// Remove the pending piece part request from the buffer.
    /// This should be called when a piece is completed.
    pub async fn remove_by_part(&self, part: &PiecePart) {
        {
            let mut mutex = self.requests.write().await;
            if let Some(position) = mutex.iter().position(|e| &e.part == part) {
                mutex.remove(position);
                self.decrease_in_flight_counter().await;
            }
        }

        self.notifier.notify_one();
    }

    /// Update the peers that are currently trying to retrieve the given piece part.
    pub async fn update_pending_peers(&self, part: PiecePart, peers: &[PeerHandle]) {
        let mut mutex = self.requests.write().await;
        if let Some(request) = mutex.iter_mut().find(|e| e.part == part) {
            request.pending_peers = peers.to_vec();
            request.requested_at = Some(Instant::now());
        }

        if peers.len() > 0 {
            self.increase_in_flight_counter().await;
        }
    }

    /// Clear the current buffer
    pub async fn clear(&self) {
        self.requests.write().await.clear();
        *self.in_flight.lock().await = 0;
    }

    /// Get the next pending request from the buffer.
    ///
    /// When no requests are available, this will wait until a request is available.
    pub async fn next(&self) -> Option<PendingRequest> {
        loop {
            if *self.in_flight.lock().await >= self.max_in_flight {
                self.notifier.notified().await;
            } else {
                break;
            }
        }

        if let Some(e) = self.internal_get_next().await {
            return Some(e);
        }

        self.internal_get_next().await
    }

    pub async fn retry_timed_out_requests(&self) {
        let max_retries = self.max_in_flight - *self.in_flight.lock().await;
        let num_of_timeout_triggers = self
            .requests
            .read()
            .await
            .iter()
            .filter(|e| e.requested_at.is_some())
            .filter(|e| {
                e.requested_at.as_ref().unwrap().elapsed().as_millis() > REQUEST_TIMEOUT_MILLIS
            })
            .take(max_retries)
            .count();

        for _ in 0..num_of_timeout_triggers {
            self.notifier.notify_one();
        }
    }

    async fn internal_get_next(&self) -> Option<PendingRequest> {
        let cursor = *self.cursor.read().await;

        // check if the cursor reached the end of the requests
        // if so, then we move the cursor to the oldest request
        if cursor >= self.requests.read().await.len() {
            // reorder all requests by oldest first
            {
                let mut mutex = self.requests.write().await;
                mutex.sort_by_key(|e| e.requested_at);
            }

            // reset the cursor
            *self.cursor.write().await = 0;
        }

        match self.requests.read().await.get(cursor) {
            Some(e) => {
                *self.cursor.write().await += 1;
                Some(e.clone())
            }
            None => None,
        }
    }

    async fn decrease_in_flight_counter(&self) {
        *self.in_flight.lock().await -= 1;
        self.notifier.notify_one();
    }

    async fn increase_in_flight_counter(&self) {
        *self.in_flight.lock().await += 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_len() {
        let runtime = Runtime::new().unwrap();
        let buffer = PendingRequestBuffer::new(1);

        runtime.block_on(buffer.push(PendingRequest::new(PiecePart {
            piece: 1,
            part: 0,
            begin: 0,
            length: 16 * 1024,
        })));
        runtime.block_on(buffer.push(PendingRequest::new(PiecePart {
            piece: 1,
            part: 1,
            begin: 16 * 1024,
            length: 2048,
        })));
        let result = runtime.block_on(buffer.len());

        assert_eq!(2, result, "expected 2 pending requests");
    }

    #[test]
    fn test_has_piece() {
        let runtime = Runtime::new().unwrap();
        let piece_index = 8;
        let part = PiecePart {
            piece: piece_index,
            part: 3,
            begin: 3072,
            length: 16 * 1024,
        };
        let request = PendingRequest::new(part.clone());
        let buffer = PendingRequestBuffer::new(1);

        runtime.block_on(buffer.push(request));
        let result = buffer.has_piece(piece_index);

        assert_eq!(true, result, "expected the piece to be pending");
    }

    #[test]
    fn test_is_pending() {
        let runtime = Runtime::new().unwrap();
        let part = PiecePart {
            piece: 1,
            part: 2,
            begin: 2048,
            length: 1024,
        };
        let request = PendingRequest::new(part.clone());
        let buffer = PendingRequestBuffer::new(1);

        runtime.block_on(buffer.push(request));
        let result = buffer.is_pending(&part);

        assert_eq!(true, result, "expected the piece part to be pending");
    }
}
