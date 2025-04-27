use crate::fx::PopcornFX;
use crate::ipc::proto::media::media;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::watched;
use crate::ipc::proto::watched::{
    AddToWatchlistRequest, AddToWatchlistResponse, GetIsWatchedRequest, GetIsWatchedResponse,
    RemoveFromWatchlistRequest,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use log::error;
use popcorn_fx_core::core::media::watched::WatchedEvent;
use popcorn_fx_core::core::media::MediaIdentifier;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct WatchedMessageHandler {
    instance: Arc<PopcornFX>,
}

impl WatchedMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.watched_service().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let mut proto_event = watched::WatchedEvent::new();

                if let WatchedEvent::WatchedStateChanged(imdb_id, state) = &*event {
                    proto_event.event = watched::watched_event::Event::STATE_CHANGED.into();
                    proto_event.watched_state_changed =
                        MessageField::some(watched::watched_event::WatchedStateChanged {
                            imdb_id: imdb_id.clone(),
                            new_state: *state,
                            special_fields: Default::default(),
                        });
                }

                if let Err(e) = channel.send(proto_event, watched::WatchedEvent::NAME).await {
                    error!("Failed to send watched event to channel, {}", e);
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for WatchedMessageHandler {
    fn name(&self) -> &str {
        "watched"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetIsWatchedRequest::NAME
                | AddToWatchlistRequest::NAME
                | RemoveFromWatchlistRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetIsWatchedRequest::NAME => {
                let request = GetIsWatchedRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaIdentifier>::try_from(
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
            AddToWatchlistRequest::NAME => {
                let request = AddToWatchlistRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaIdentifier>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;
                let response: AddToWatchlistResponse;

                match self.instance.watched_service().add(media) {
                    Ok(_) => {
                        response = AddToWatchlistResponse {
                            result: response::Result::OK.into(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = AddToWatchlistResponse {
                            result: response::Result::ERROR.into(),
                            error: MessageField::some(media::Error::from(&e)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, AddToWatchlistResponse::NAME)
                    .await?;
            }
            RemoveFromWatchlistRequest::NAME => {
                let request = RemoveFromWatchlistRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaIdentifier>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;

                self.instance.watched_service().remove(media);
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
