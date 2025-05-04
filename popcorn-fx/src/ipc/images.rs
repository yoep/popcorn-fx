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
use popcorn_fx_core::core::media::MediaOverview;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
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
    fn name(&self) -> &str {
        "images"
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::proto::media::media;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::try_recv;

    use popcorn_fx_core::core::media::{Images, MediaIdentifier, MovieOverview};
    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_poster_placeholder_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ImagesMessageHandler::new(instance.clone());

        let expected_image_data = instance.image_loader().default_poster();

        let response = incoming
            .get(
                GetPosterPlaceholderRequest::default(),
                GetPosterPlaceholderRequest::NAME,
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

        let message = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let placeholder_response =
            GetPosterPlaceholderResponse::parse_from_bytes(&message.payload).unwrap();
        assert_ne!(MessageField::none(), placeholder_response.image);
        assert_eq!(
            expected_image_data, placeholder_response.image.data,
            "expected the poster image placeholder data"
        );
    }

    #[tokio::test]
    async fn test_process_get_artwork_placeholder_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ImagesMessageHandler::new(instance.clone());

        let expected_image_data = instance.image_loader().default_artwork();

        let response = incoming
            .get(
                GetArtworkPlaceholderRequest::default(),
                GetArtworkPlaceholderRequest::NAME,
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

        let message = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let placeholder_response =
            GetArtworkPlaceholderResponse::parse_from_bytes(&message.payload).unwrap();
        assert_ne!(MessageField::none(), placeholder_response.image);
        assert_eq!(
            expected_image_data, placeholder_response.image.data,
            "expected the artwork image placeholder data"
        );
    }

    #[tokio::test]
    async fn test_process_get_fanart_request() {
        init_logger!();
        let media = Box::new(MovieOverview {
            imdb_id: "tt11198330".to_string(),
            title: "MovieTitle".to_string(),
            year: "2011".to_string(),
            images: Images {
                poster: "http://image.tmdb.org/t/p/w500/t9XkeE7HzOsdQcDDDapDYh8Rrmt.jpg"
                    .to_string(),
                fanart: "http://image.tmdb.org/t/p/w500/etj8E2o0Bud0HkONVQPjyCkIvpv.jpg"
                    .to_string(),
                banner: "http://image.tmdb.org/t/p/w500/t9XkeE7HzOsdQcDDDapDYh8Rrmt.jpg"
                    .to_string(),
            },
            rating: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ImagesMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetFanartRequest {
                    media: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                GetFanartRequest::NAME,
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

        let message = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let response = GetFanartResponse::parse_from_bytes(&message.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            response.result
        );
        assert_ne!(
            MessageField::none(),
            response.image,
            "expected the image data to have been present"
        );
    }

    #[tokio::test]
    async fn test_process_get_poster_request() {
        init_logger!();
        let media = Box::new(MovieOverview {
            imdb_id: "tt11198330".to_string(),
            title: "MovieTitle".to_string(),
            year: "2011".to_string(),
            images: Images {
                poster: "http://image.tmdb.org/t/p/w500/t9XkeE7HzOsdQcDDDapDYh8Rrmt.jpg"
                    .to_string(),
                fanart: "http://image.tmdb.org/t/p/w500/etj8E2o0Bud0HkONVQPjyCkIvpv.jpg"
                    .to_string(),
                banner: "http://image.tmdb.org/t/p/w500/t9XkeE7HzOsdQcDDDapDYh8Rrmt.jpg"
                    .to_string(),
            },
            rating: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ImagesMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetPosterRequest {
                    media: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                GetPosterRequest::NAME,
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

        let message = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let response = GetPosterResponse::parse_from_bytes(&message.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            response.result
        );
        assert_ne!(
            MessageField::none(),
            response.image,
            "expected the image data to have been present"
        );
    }

    #[tokio::test]
    async fn test_process_get_image_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ImagesMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetImageRequest {
                    url: "https://c.media-amazon.com/images/G/01/gno/sprites/nav-sprite-global-2x-reorg-privacy._CB546805360_.png".to_string(),
                    special_fields: Default::default(),
                },
                GetImageRequest::NAME,
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

        let message = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let response = GetImageResponse::parse_from_bytes(&message.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            response.result
        );
        assert_ne!(
            MessageField::none(),
            response.image,
            "expected the image data to have been present"
        );
    }
}
