use crate::torrent::peer::extension::Extension;
use crate::torrent::peer::{PeerContext, PeerEvent, TcpPeer};
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
        _payload: &'a [u8],
        _peer: &'a PeerContext,
    ) -> crate::torrent::peer::extension::Result<()> {
        todo!()
    }

    async fn on<'a>(&'a self, _event: &'a PeerEvent, _peer: &'a PeerContext) {
        todo!()
    }

    fn clone_boxed(&self) -> Box<dyn Extension> {
        Box::new(Self::new())
    }
}
