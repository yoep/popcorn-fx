use crate::fx::PopcornFX;
use crate::ipc::proto::images;
use crate::ipc::proto::images::{
    GetArtworkPlaceholderRequest, GetArtworkPlaceholderResponse, GetFanartRequest,
    GetFanartResponse, GetImageRequest, GetImageResponse, GetPosterPlaceholderRequest,
    GetPosterPlaceholderResponse, GetPosterRequest, GetPosterResponse,
};
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::media::MediaOverview;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Images message handler")]
pub struct ImagesMessageHandler {
    instance: Arc<PopcornFX>,
}

impl ImagesMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for ImagesMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetPosterPlaceholderRequest::NAME
                | GetArtworkPlaceholderRequest::NAME
                | GetFanartRequest::NAME
                | GetPosterRequest::NAME
                | GetImageRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetPosterPlaceholderRequest::NAME => {
                let data = self.instance.image_loader().default_poster();
                channel
                    .send_reply(
                        &message,
                        GetPosterPlaceholderResponse {
                            image: MessageField::some(images::Image {
                                data,
                                special_fields: Default::default(),
                            }),
                            special_fields: Default::default(),
                        },
                        GetPosterPlaceholderResponse::NAME,
                    )
                    .await?;
            }
            GetArtworkPlaceholderRequest::NAME => {
                let data = self.instance.image_loader().default_artwork();
                channel
                    .send_reply(
                        &message,
                        GetArtworkPlaceholderResponse {
                            image: MessageField::some(images::Image {
                                data,
                                special_fields: Default::default(),
                            }),
                            special_fields: Default::default(),
                        },
                        GetArtworkPlaceholderResponse::NAME,
                    )
                    .await?;
            }
            GetFanartRequest::NAME => {
                let request = GetFanartRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.media.as_ref().ok_or(Error::MissingField)?,
                )?;
                let data = self.instance.image_loader().load_fanart(&media).await;

                channel
                    .send_reply(
                        &message,
                        GetFanartResponse {
                            result: response::Result::OK.into(),
                            image: MessageField::some(images::Image {
                                data,
                                special_fields: Default::default(),
                            }),
                            error: None,
                            special_fields: Default::default(),
                        },
                        GetFanartResponse::NAME,
                    )
                    .await?;
            }
            GetPosterRequest::NAME => {
                let request = GetPosterRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.media.as_ref().ok_or(Error::MissingField)?,
                )?;
                let data = self.instance.image_loader().load_poster(&media).await;

                channel
                    .send_reply(
                        &message,
                        GetPosterResponse {
                            result: response::Result::OK.into(),
                            image: MessageField::some(images::Image {
                                data,
                                special_fields: Default::default(),
                            }),
                            error: None,
                            special_fields: Default::default(),
                        },
                        GetPosterResponse::NAME,
                    )
                    .await?;
            }
            GetImageRequest::NAME => {
                let request = GetImageRequest::parse_from_bytes(&message.payload)?;
                let response: GetImageResponse;

                match self
                    .instance
                    .image_loader()
                    .load(request.url.as_str())
                    .await
                {
                    None => {
                        response = GetImageResponse {
                            result: response::Result::ERROR.into(),
                            image: MessageField::none(),
                            error: Some(images::image::Error::UNAVAILABLE.into()),
                            special_fields: Default::default(),
                        };
                    }
                    Some(data) => {
                        response = GetImageResponse {
                            result: response::Result::OK.into(),
                            image: MessageField::some(images::Image {
                                data,
                                special_fields: Default::default(),
                            }),
                            error: None,
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetImageResponse::NAME)
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
