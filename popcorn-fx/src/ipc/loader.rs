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
