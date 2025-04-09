use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::loader::{LoaderCancelRequest, LoaderLoadRequest};
use crate::ipc::proto::message::FxMessage;
use async_trait::async_trait;
use derive_more::Display;
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Loader message handler")]
pub struct LoaderMessageHandler {
    instance: Arc<PopcornFX>,
}

impl LoaderMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for LoaderMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            LoaderLoadRequest::NAME | LoaderCancelRequest::NAME
        )
    }

    async fn process(
        &self,
        message: FxMessage,
        channel: &IpcChannel,
    ) -> crate::ipc::errors::Result<()> {
        Ok(())
    }
}
