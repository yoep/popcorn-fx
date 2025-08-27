use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::loader::{LoaderCancelRequest, LoaderLoadRequest, LoaderLoadResponse};
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::{loader, message};
use crate::ipc::Error;
use async_trait::async_trait;
use log::error;
use popcorn_fx_core::core::loader::LoadingHandle;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct LoaderMessageHandler {
    instance: Arc<PopcornFX>,
}

impl LoaderMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.media_loader().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let proto_event = loader::LoaderEvent::from(&*event);
                if let Err(e) = channel.send(proto_event, loader::LoaderEvent::NAME).await {
                    error!("Failed to send loader event to channel, {}", e);
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for LoaderMessageHandler {
    fn name(&self) -> &str {
        "loader"
    }

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
        match message.message_type() {
            LoaderLoadRequest::NAME => {
                let request = LoaderLoadRequest::parse_from_bytes(&message.payload)?;
                let handle = self
                    .instance
                    .media_loader()
                    .load_url(request.url.as_str())
                    .await;

                channel
                    .send_reply(
                        &message,
                        LoaderLoadResponse {
                            handle: MessageField::some(message::Handle::from(&handle)),
                            special_fields: Default::default(),
                        },
                        LoaderLoadResponse::NAME,
                    )
                    .await?;
            }
            LoaderCancelRequest::NAME => {
                let request = LoaderCancelRequest::parse_from_bytes(&message.payload)?;
                let handle = request
                    .handle
                    .as_ref()
                    .map(LoadingHandle::from)
                    .ok_or(Error::MissingField)?;

                self.instance.media_loader().cancel(handle);
            }
            _ => {
                return Err(Error::UnsupportedMessage(
                    message.message_type().to_string(),
                ));
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
    use crate::timeout;

    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_loader_load_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = LoaderMessageHandler::new(instance.clone(), outgoing.clone());

        let response = incoming
            .get(
                LoaderLoadRequest {
                    url: "http://localhost/my-video.mp4".to_string(),
                    special_fields: Default::default(),
                },
                LoaderLoadRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = timeout!(response, Duration::from_millis(500))
            .expect("expected to have received a reply");
        let result = LoaderLoadResponse::parse_from_bytes(&response.payload).unwrap();
        assert_ne!(
            MessageField::none(),
            result.handle,
            "expected a handle to have been present"
        );
    }

    #[tokio::test]
    async fn test_loading_event() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = LoaderMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .get(
                LoaderLoadRequest {
                    url: "http://localhost/my-video.mp4".to_string(),
                    special_fields: Default::default(),
                },
                LoaderLoadRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let event_message = timeout!(incoming.recv(), Duration::from_millis(250))
            .expect("expected to have received a reply");
        let event = loader::LoaderEvent::parse_from_bytes(&event_message.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<loader::loader_event::Event>>::into(
                loader::loader_event::Event::LOADING_STARTED
            ),
            event.event
        );
    }
}
