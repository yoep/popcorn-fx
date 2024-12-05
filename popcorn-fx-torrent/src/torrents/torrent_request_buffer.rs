use crate::torrents::peers::PeerHandle;
use crate::torrents::{PartIndex, PieceIndex, PiecePart};
use derive_more::Display;
use log::trace;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// The timeout after which a request will be retried for execution
const REQUEST_TIMEOUT_MILLIS: u128 = 20 * 1000;

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
    pub fn parts_as_slice(&self) -> &[PiecePart] {
        &self.inner.parts
    }

    /// Get the list of parts which still need to be requested from a peer.
    /// This is a list of piece parts which are either not yet requested from a peer or have timed-out.
    pub fn parts_to_request(&self) -> Vec<&PiecePart> {
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

    /// Release the given part index as being processed by a peer.
    /// This is called when a peer has rejected a part of the pending request.
    fn release_part(&self, part: &PartIndex) {
        let mut mutex = self.inner.pending_requests.write().unwrap();
        mutex.remove(part);
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
    pub requests: Vec<PendingRequest>,
    pub semaphore: Arc<Semaphore>,
}

impl PendingRequestBuffer {
    /// Get a new request buffer.
    /// It will have a maximum of `max_in_flight` requests in flight at any given time.
    pub fn new(max_in_flight: usize) -> Self {
        Self {
            requests: Vec::with_capacity(0),
            semaphore: Arc::new(Semaphore::new(max_in_flight)),
        }
    }

    /// Get the number of pending requests.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Get the pending requested pieces stored in the buffer.
    pub fn pending_pieces(&self) -> Vec<PieceIndex> {
        self.requests.iter().map(|e| e.piece()).collect()
    }

    /// Check if the given piece is currently queued for being requested.
    pub fn is_pending(&self, piece: &PieceIndex) -> bool {
        self.requests.iter().any(|e| e.piece() == *piece)
    }

    /// Get all pending requests that have not yet been requested from a peer or have timed-out.
    /// It returns the list of pending requests.
    pub fn pending_requests(&self) -> Vec<&PendingRequest> {
        self.requests
            .iter()
            .filter(|e| !e.is_completed())
            .filter(|e| e.parts_to_request().len() > 0)
            .collect()
    }

    /// Push a new request into the buffer.
    pub fn push(&mut self, request: PendingRequest) {
        // check if the pending piece request is already in the buffer
        // if so, ignore this request
        if self.requests.iter().any(|e| e.piece() == request.piece()) {
            return;
        }

        self.requests.push(request);
    }

    /// Get the amount of available requests that are allowed to be processed/requested by peers.
    /// This will count the remaining "in-flight" slots of the buffer.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Push multiple requests onto the buffer.
    pub fn push_all(&mut self, requests: Vec<PendingRequest>) {
        // filter out any duplicate requests
        let requests: Vec<_> = requests
            .into_iter()
            .filter(|e| !self.requests.contains(e))
            .collect();

        self.requests.extend(requests);
    }

    /// Update the peers that are currently trying to retrieve the given piece part.
    pub fn update_request_from(&self, piece: PieceIndex, part: PartIndex, peer: PeerHandle) {
        if let Some(request) = self.requests.iter().find(|e| e.piece() == piece) {
            request.add_part_requested_from(part, peer);
        }
    }

    /// Update a pending piece part request as being completed.
    pub fn update_request_part_completed(&mut self, piece: PieceIndex, part: PartIndex) {
        let mut is_piece_completed = false;

        if let Some(request) = self.requests.iter().find(|e| e.piece() == piece) {
            request.mark_part_as_completed(part);
            is_piece_completed = request.is_completed();
        }

        if is_piece_completed {
            if let Some(position) = self.requests.iter().position(|e| e.piece() == piece) {
                let request = self.requests.remove(position);
                request.release_permit();
                trace!("Removed pending request of piece {}", piece);
            }
        }
    }

    /// Accept a pending request as "in-flight".
    /// This will mark the request as being requested by one-or-more peers.
    pub async fn accept(&self, request: &PendingRequest) {
        // check if the request already has a permit
        if request.inner.permit.read().unwrap().is_some() {
            return;
        }

        if let Ok(permit) = self.semaphore.clone().acquire_owned().await {
            if let Ok(mut mutex) = request.inner.permit.write() {
                *mutex = Some(permit);
            }
        }
    }

    /// Release the pending request from in-flight as it was unable to be processed by any peer.
    pub fn release(&self, request: &PendingRequest) {
        request.release_permit();
    }

    /// Release the given part from the pending request as being processed.
    /// This should be called when a peer has rejected a part of the pending request.
    pub fn release_part(&self, request: &PendingRequest, part: &PartIndex) {
        request.release_part(part);
    }

    /// Clear the current buffer
    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::core::Handle;
    use popcorn_fx_core::testing::init_logger;

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
        let mut buffer = PendingRequestBuffer::new(1);

        buffer.push_all(vec![
            create_pending_request(1, 2),
            create_pending_request(2, 1),
        ]);
        let result = buffer.len();

        assert_eq!(2, result, "expected 2 pending requests");
    }

    #[test]
    fn test_buffer_is_pending() {
        let piece_index = 1;
        let request = create_pending_request(piece_index, 3);
        let mut buffer = PendingRequestBuffer::new(1);

        buffer.push_all(vec![request]);
        let result = buffer.is_pending(&piece_index);

        assert_eq!(true, result, "expected the piece part to be pending");
    }

    #[test]
    fn test_buffer_clear() {
        init_logger();
        let requests = vec![create_pending_request(1, 1), create_pending_request(2, 6)];
        let mut buffer = PendingRequestBuffer::new(1);

        buffer.push_all(requests);
        buffer.clear();

        assert_eq!(0, buffer.requests.len());
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
