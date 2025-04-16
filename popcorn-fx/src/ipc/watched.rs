use crate::fx::PopcornFX;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::watched::{GetIsWatchedRequest, GetIsWatchedResponse};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::media::MediaOverview;
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Watched message handler")]
pub struct WatchedMessageHandler {
    instance: Arc<PopcornFX>,
}

impl WatchedMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for WatchedMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(message_type, GetIsWatchedRequest::NAME)
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetIsWatchedRequest::NAME => {
                let request = GetIsWatchedRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;

                let is_watched = self
                    .instance
                    .watched_service()
                    .is_watched(media.imdb_id())
                    .await;

                channel
                    .send_reply(
                        &message,
                        GetIsWatchedResponse {
                            is_watched,
                            special_fields: Default::default(),
                        },
                        GetIsWatchedResponse::NAME,
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
