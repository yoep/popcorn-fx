use crate::torrents::peers::PeerHandle;
use crate::torrents::{PartIndex, PieceIndex, PiecePart};
use derive_more::Display;
use log::{debug, trace, warn};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, Notify, OwnedSemaphorePermit, RwLock, Semaphore};

/// The timeout after which a request will be retried for execution
const REQUEST_TIMEOUT_MILLIS: u128 = 60 * 1000; // 60 seconds

/// The pending request for a piece part of a torrent.
#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", inner)]
pub struct PendingRequest {
    inner: Arc<InnerPendingRequest>,
}

impl PendingRequest {
    pub fn new(piece: PieceIndex, parts: Vec<PiecePart>) -> Self {
        Self {
            inner: Arc::new(InnerPendingRequest {
                piece,
                parts,
                pending_requests: std::sync::RwLock::new(Default::default()),
                permit: std::sync::RwLock::new(None),
            }),
        }
    }

    /// Get the piece index of the pending request.
    pub fn piece(&self) -> PieceIndex {
        self.inner.piece
    }

    /// Get the piece parts of the pending request.
    pub fn parts(&self) -> Vec<PiecePart> {
        self.inner.parts.clone()
    }

    /// Get the piece parts as a slice of the pending request.
    pub fn parts_slice(&self) -> &[PiecePart] {
        &self.inner.parts
    }

    /// Get the list of parts which still need to be requested from a peer.
    /// This is a list of piece parts which are either not yet requested from a peer or have timed-out.
    pub fn parts_to_request(&self) -> Vec<PiecePart> {
        self.inner
            .parts
            .iter()
            .filter(|e| {
                self.inner
                    .pending_requests
                    .read()
                    .unwrap()
                    .get(&e.part)
                    .filter(|e| e.completed)
                    .filter(|e| e.requested_at.elapsed().as_millis() < REQUEST_TIMEOUT_MILLIS)
                    .is_none()
            })
            .cloned()
            .collect()
    }

    /// Check if this pending request has been completed.
    /// It returns true if all parts have been completed, else false.
    pub fn is_completed(&self) -> bool {
        let mutex = self.inner.pending_requests.read().unwrap();

        mutex.len() == self.inner.parts.len() && mutex.iter().all(|(_, e)| e.completed)
    }

    /// Release the permit for the pending request.
    /// This can be used when the pending request is completed or the request couldn't be processed by any peer.
    pub fn release_permit(&self) {
        let mut mutex = self.inner.permit.write().unwrap();
        let _ = mutex.take();
    }

    /// Mark the given part index as completed within the request.
    fn mark_part_as_completed(&self, part: PartIndex) {
        let mut mutex = self.inner.pending_requests.write().unwrap();
        if let Some(request_from) = mutex.get_mut(&part) {
            request_from.completed = true;
        }
    }

    /// Add the given part index as requested from the given peer.
    fn add_part_requested_from(&self, part: PartIndex, peer: PeerHandle) {
        let mut mutex = self.inner.pending_requests.write().unwrap();
        mutex.insert(
            part,
            RequestFrom {
                peer,
                completed: false,
                requested_at: Instant::now(),
            },
        );
    }
}

impl PartialEq for PendingRequest {
    fn eq(&self, other: &Self) -> bool {
        self.inner.piece == other.inner.piece
    }
}

#[derive(Debug, Clone)]
pub struct RequestFrom {
    /// The peer the request was sent to
    pub peer: PeerHandle,
    /// Indicates if the request has been completed
    pub completed: bool,
    /// The time the request was made
    pub requested_at: Instant,
}

#[derive(Debug, Display)]
#[display(fmt = "{} ({} parts)", piece, "parts.len()")]
struct InnerPendingRequest {
    /// The piece that is being requested
    piece: PieceIndex,
    /// The parts that are being requested
    parts: Vec<PiecePart>,
    /// The currently pending requests
    pending_requests: std::sync::RwLock<HashMap<PartIndex, RequestFrom>>,
    /// The pending permit of the request indicating that this request is in flight
    permit: std::sync::RwLock<Option<OwnedSemaphorePermit>>,
}

#[derive(Debug)]
pub struct PendingRequestBuffer {
    requests: RwLock<Vec<PendingRequest>>,
    semaphore: Arc<Semaphore>,
    sender: UnboundedSender<PendingRequest>,
    receiver: Mutex<UnboundedReceiver<PendingRequest>>,
}

impl PendingRequestBuffer {
    /// Get a new request buffer.
    /// It will have a maximum of `max_in_flight` requests in flight at any given time.
    pub fn new(max_in_flight: usize) -> Self {
        let (sender, receiver) = unbounded_channel();

        Self {
            requests: RwLock::new(Vec::with_capacity(0)),
            semaphore: Arc::new(Semaphore::new(max_in_flight)),
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// Get the number of pending requests.
    pub async fn len(&self) -> usize {
        self.requests.read().await.len()
    }

    /// Get the pending requested pieces stored in the buffer.
    pub async fn pending_pieces(&self) -> Vec<PieceIndex> {
        self.requests
            .read()
            .await
            .iter()
            .map(|e| e.piece())
            .collect()
    }

    /// Push a new request into the buffer.
    pub async fn push(&self, request: PendingRequest) {
        {
            let mut mutex = self.requests.write().await;
            if !mutex.iter().any(|e| e.piece() == request.piece()) {
                let index = mutex.len();
                mutex.insert(index, request.clone());
                let _ = self.sender.send(request);
            }
        }
    }

    /// Push multiple requests onto the buffer.
    pub async fn push_all(&self, mut requests: Vec<PendingRequest>) {
        {
            let mut mutex = self.requests.write().await;
            // filter out any duplicate requests
            requests = requests
                .into_iter()
                .filter(|pending_request| {
                    !mutex
                        .iter()
                        .any(|existing_request| existing_request.piece() == pending_request.piece())
                })
                .collect();

            mutex.extend(requests.clone());
        }

        for request in requests {
            let _ = self.sender.send(request);
        }
    }

    /// Check if the given piece part is currently being requested.
    pub async fn is_pending_async(&self, piece: PieceIndex) -> bool {
        self.requests
            .read()
            .await
            .iter()
            .find(|e| e.piece() == piece)
            .is_some()
    }

    /// Update the peers that are currently trying to retrieve the given piece part.
    pub async fn update_request_from(&self, piece: PieceIndex, part: PartIndex, peer: PeerHandle) {
        let mut mutex = self.requests.write().await;
        if let Some(request) = mutex.iter_mut().find(|e| e.piece() == piece) {
            request.add_part_requested_from(part, peer);
        }
    }

    /// Update a pending piece part request as being completed.
    pub async fn update_request_part_completed(&self, piece: PieceIndex, part: PartIndex) {
        let mut mutex = self.requests.write().await;
        if let Some(request) = mutex.iter_mut().find(|e| e.piece() == piece) {
            request.mark_part_as_completed(part);

            if request.is_completed() {
                request.release_permit();
                mutex.retain(|e| e.piece() != piece);
            }
        }
    }

    /// Clear the current buffer
    pub async fn clear(&self) {
        self.requests.write().await.clear();

        {
            let mut mutex = self.receiver.lock().await;
            while mutex.try_recv().is_ok() {}
        }
    }

    /// Get the next pending request from the buffer.
    ///
    /// When no requests are available, this will wait until a request is available.
    pub async fn next(&self) -> Option<PendingRequest> {
        if let Ok(permit) = self.semaphore.clone().acquire_owned().await {
            if let Some(request) = self.receiver.lock().await.recv().await {
                request.inner.permit.write().unwrap().replace(permit);
                return Some(request);
            }
        }

        None
    }

    /// Retry the requests which are in flight and have not yet been completed within the expected deadline.
    /// This will push the request back on the buffer channel to be processed again.
    pub async fn retry_timed_out_requests(&self) {
        let retry_requests: Vec<PendingRequest>;

        {
            let mutex = self.requests.read().await;
            retry_requests = mutex
                .iter()
                .filter(|e| e.inner.pending_requests.read().unwrap().len() > 0)
                .filter(|e| {
                    e.inner
                        .pending_requests
                        .read()
                        .unwrap()
                        .iter()
                        .any(|(_, from)| {
                            !from.completed
                                && from.requested_at.elapsed().as_millis() > REQUEST_TIMEOUT_MILLIS
                        })
                })
                .take(self.semaphore.available_permits())
                .cloned()
                .collect();
        }

        if retry_requests.len() > 0 {
            trace!("Retrying a total of {} requests", retry_requests.len());
            for request in retry_requests {
                let _ = self.sender.send(request);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::core::Handle;
    use popcorn_fx_core::testing::init_logger;
    use std::sync::mpsc::{channel, RecvTimeoutError};
    use std::time::Duration;
    use tokio::runtime::Runtime;

    #[test]
    fn test_pending_request_is_completed() {
        let request = create_pending_request(1, 2);
        assert_eq!(false, request.is_completed());

        request.add_part_requested_from(1, Handle::new());
        request.add_part_requested_from(2, Handle::new());
        request.mark_part_as_completed(1);
        request.mark_part_as_completed(2);

        assert_eq!(
            true,
            request.is_completed(),
            "expected the request to be completed"
        );
    }

    #[test]
    fn test_buffer_len() {
        let runtime = Runtime::new().unwrap();
        let buffer = PendingRequestBuffer::new(1);

        runtime.block_on(buffer.push_all(vec![
            create_pending_request(1, 2),
            create_pending_request(2, 1),
        ]));
        let result = runtime.block_on(buffer.len());

        assert_eq!(2, result, "expected 2 pending requests");
    }

    #[tokio::test]
    async fn test_buffer_is_pending() {
        let piece_index = 1;
        let request = create_pending_request(piece_index, 3);
        let buffer = PendingRequestBuffer::new(1);

        buffer.push_all(vec![request]).await;
        let result = buffer.is_pending_async(piece_index).await;

        assert_eq!(true, result, "expected the piece part to be pending");
    }

    #[test]
    fn test_buffer_next() {
        let runtime = Runtime::new().unwrap();
        let request = create_pending_request(1, 3);
        let (tx, rx) = channel();
        let buffer = PendingRequestBuffer::new(1);

        runtime.block_on(buffer.push_all(vec![request]));
        runtime.spawn(async move {
            tx.send(buffer.next().await).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_millis(50)).unwrap();

        assert_ne!(None, result, "expected a pending request");
    }

    #[test]
    fn test_buffer_next_max_in_flight() {
        init_logger();
        let runtime = Runtime::new().unwrap();
        let requests = vec![create_pending_request(1, 1), create_pending_request(2, 6)];
        let (tx, rx) = channel();
        let buffer = Arc::new(PendingRequestBuffer::new(1));

        runtime.block_on(buffer.push_all(requests));
        let inner_recv = buffer.clone();
        runtime.spawn(async move {
            loop {
                if let Some(request) = inner_recv.next().await {
                    if let Err(_) = tx.send(request) {
                        debug!("Closing pending request channel");
                        break;
                    }
                }
            }
        });

        let _ = rx
            .recv_timeout(Duration::from_millis(50))
            .expect("expected a pending request");

        let result = rx.recv_timeout(Duration::from_millis(100));
        assert_eq!(
            Err(RecvTimeoutError::Timeout),
            result,
            "expected no next item to be returned"
        );

        // mark the piece as completed
        let inner = buffer.clone();
        runtime.block_on(async move {
            inner.update_request_from(1, 1, Handle::new()).await;
            inner.update_request_part_completed(1, 1).await;
        });

        let _ = rx
            .recv_timeout(Duration::from_millis(100))
            .expect("expected a pending request");
    }

    #[test]
    fn test_buffer_clear() {
        init_logger();
        let runtime = Runtime::new().unwrap();
        let requests = vec![create_pending_request(1, 1), create_pending_request(2, 6)];
        let (tx, rx) = channel();
        let buffer = Arc::new(PendingRequestBuffer::new(1));

        runtime.block_on(buffer.push_all(requests));
        runtime.block_on(buffer.clear());

        runtime.spawn(async move {
            let _ = tx.send(buffer.next().await);
        });

        let result = rx.recv_timeout(Duration::from_millis(100));
        assert_eq!(
            Err(RecvTimeoutError::Timeout),
            result,
            "expected no next item to be returned"
        );
    }

    fn create_pending_request(piece: PieceIndex, num_of_parts: usize) -> PendingRequest {
        let mut parts = Vec::new();
        let mut begin = 0;
        let length = 16 * 1024;

        for i in 0..num_of_parts {
            parts.push(PiecePart {
                piece,
                part: i,
                begin,
                length,
            });

            begin += length;
        }

        PendingRequest::new(piece, parts)
    }
}
