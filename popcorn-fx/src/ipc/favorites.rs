use crate::fx::PopcornFX;
use crate::ipc::proto::favorites::{
    AddFavoriteRequest, AddFavoriteResponse, GetIsLikedRequest, GetIsLikedResponse,
    RemoveFavoriteRequest,
};
use crate::ipc::proto::media::media;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::media::MediaOverview;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Favorites message handler")]
pub struct FavoritesMessageHandler {
    instance: Arc<PopcornFX>,
}

impl FavoritesMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for FavoritesMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetIsLikedRequest::NAME | AddFavoriteRequest::NAME | RemoveFavoriteRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetIsLikedRequest::NAME => {
                let request = GetIsLikedRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;

                let is_liked = self
                    .instance
                    .favorite_service()
                    .is_liked(media.imdb_id())
                    .await;

                channel
                    .send_reply(
                        &message,
                        GetIsLikedResponse {
                            is_liked,
                            special_fields: Default::default(),
                        },
                        GetIsLikedResponse::NAME,
                    )
                    .await?;
            }
            AddFavoriteRequest::NAME => {
                let request = AddFavoriteRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;
                let response: AddFavoriteResponse;

                match self.instance.favorite_service().add(media).await {
                    Ok(_) => {
                        response = AddFavoriteResponse {
                            result: response::Result::OK.into(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = AddFavoriteResponse {
                            result: response::Result::ERROR.into(),
                            error: MessageField::some(media::Error::from(&e).into()),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, AddFavoriteResponse::NAME)
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
