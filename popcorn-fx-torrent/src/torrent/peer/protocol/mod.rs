pub use bt::*;
pub use utp_socket::*;
pub use utp_stream::*;

mod bt;
mod utils;
mod utp_socket;
mod utp_stream;

/// The maximum size of a single uTP packet (= max UDP size).
const MAX_PACKET_SIZE: usize = 65_535;
/// The maximum size of a payload in a single uTP packet (= max UDP size - max uTP header size).
const MAX_PACKET_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - 26;

#[cfg(test)]
pub mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::{Mutex, MutexGuard};

    #[derive(Debug, Clone)]
    pub struct UtpPacketCaptureExtension {
        inner: Arc<InnerUtpPacketCaptureExtension>,
    }

    impl UtpPacketCaptureExtension {
        pub fn new() -> Self {
            Self {
                inner: Arc::new(InnerUtpPacketCaptureExtension {
                    incoming_packets: Default::default(),
                    outgoing_packets: Default::default(),
                }),
            }
        }

        pub async fn incoming_packets(&self) -> MutexGuard<'_, Vec<Packet>> {
            self.inner.incoming_packets.lock().await
        }

        pub async fn outgoing_packets(&self) -> MutexGuard<'_, Vec<Packet>> {
            self.inner.outgoing_packets.lock().await
        }
    }

    #[async_trait]
    impl UtpSocketExtension for UtpPacketCaptureExtension {
        async fn incoming(&self, packet: &mut Packet, _: &UtpStreamContext) {
            self.inner
                .incoming_packets
                .lock()
                .await
                .push(packet.clone());
        }

        async fn outgoing(&self, packet: &mut Packet, _: &UtpStreamContext) {
            self.inner
                .outgoing_packets
                .lock()
                .await
                .push(packet.clone());
        }
    }

    #[derive(Debug)]
    struct InnerUtpPacketCaptureExtension {
        incoming_packets: Mutex<Vec<Packet>>,
        outgoing_packets: Mutex<Vec<Packet>>,
    }
}
