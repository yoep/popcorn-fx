use crate::fx::PopcornFX;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::tracking::{
    GetTrackingProviderIsAuthorizedRequest, GetTrackingProviderIsAuthorizedResponse,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug)]
pub struct TrackingMessageHandler {
    instance: Arc<PopcornFX>,
}

impl TrackingMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for TrackingMessageHandler {
    fn name(&self) -> &str {
        "tracking"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(message_type, GetTrackingProviderIsAuthorizedRequest::NAME)
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetTrackingProviderIsAuthorizedRequest::NAME => {
                let _request =
                    GetTrackingProviderIsAuthorizedRequest::parse_from_bytes(&message.payload)?;
                let is_authorized = self.instance.tracking_provider().is_authorized().await;

                channel
                    .send_reply(
                        &message,
                        GetTrackingProviderIsAuthorizedResponse {
                            is_authorized,
                            special_fields: Default::default(),
                        },
                        GetTrackingProviderIsAuthorizedResponse::NAME,
                    )
                    .await?;
            }
            _ => {
                return Err(Error::UnsupportedMessage(
                    message.message_type().to_string(),
                ))
            }
        }

        Ok(())
    }
}
