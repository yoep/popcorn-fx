use crate::torrent::peer::extension::Extension;
use crate::torrent::peer::{Peer, PeerEvent};
use async_trait::async_trait;

const HOLEPUNCH_EXTENSION_NAME: &str = "ut_holepunch";

/// The holepunch extension as defined in BEP55
#[derive(Debug)]
pub struct HolepunchExtension {}

impl HolepunchExtension {
    /// Creates a new holepunch extension
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Extension for HolepunchExtension {
    fn name(&self) -> &str {
        HOLEPUNCH_EXTENSION_NAME
    }

    async fn handle<'a>(
        &'a self,
        payload: &'a [u8],
        peer: &'a Peer,
    ) -> crate::torrent::peer::extension::Result<()> {
        todo!()
    }

    async fn on<'a>(&'a self, event: PeerEvent, peer: &'a Peer) {
        todo!()
    }

    fn clone_boxed(&self) -> Box<dyn Extension> {
        Box::new(Self::new())
    }
}
