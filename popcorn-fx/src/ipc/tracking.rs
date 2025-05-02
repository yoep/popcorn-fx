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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::try_recv;

    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_is_authorized_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TrackingMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetTrackingProviderIsAuthorizedRequest::new(),
                GetTrackingProviderIsAuthorizedRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result =
            GetTrackingProviderIsAuthorizedResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(false, result.is_authorized);
    }
}
