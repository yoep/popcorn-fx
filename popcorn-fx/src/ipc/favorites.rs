use crate::fx::PopcornFX;
use crate::ipc::proto::favorites;
use crate::ipc::proto::favorites::{
    favorite_event, AddFavoriteRequest, AddFavoriteResponse, GetIsLikedRequest, GetIsLikedResponse,
    RemoveFavoriteRequest,
};
use crate::ipc::proto::media::media;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use log::error;
use popcorn_fx_core::core::media::favorites::FavoriteEvent;
use popcorn_fx_core::core::media::MediaOverview;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct FavoritesMessageHandler {
    instance: Arc<PopcornFX>,
}

impl FavoritesMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.favorite_service().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let proto_event: favorites::FavoriteEvent;

                match &*event {
                    FavoriteEvent::LikedStateChanged(imdb_id, is_liked) => {
                        proto_event = favorites::FavoriteEvent {
                            event: favorite_event::Event::LIKED_STATE_CHANGED.into(),
                            like_state_changed: MessageField::some(
                                favorite_event::LikedStateChanged {
                                    imdb_id: imdb_id.clone(),
                                    is_liked: *is_liked,
                                    special_fields: Default::default(),
                                },
                            ),
                            special_fields: Default::default(),
                        }
                    }
                }

                if let Err(e) = channel
                    .send(proto_event, favorites::FavoriteEvent::NAME)
                    .await
                {
                    error!("Favorites message handler failed to send event, {}", e);
                    break;
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for FavoritesMessageHandler {
    fn name(&self) -> &str {
        "favorites"
    }

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
            RemoveFavoriteRequest::NAME => {
                let request = RemoveFavoriteRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;

                self.instance.favorite_service().remove(media).await;
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
    use crate::timeout;

    use popcorn_fx_core::core::media::{
        Images, MediaIdentifier, MovieOverview, Rating, ShowOverview,
    };
    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_is_liked_request() {
        init_logger!();
        let media: Box<dyn MediaIdentifier> = create_media();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = FavoritesMessageHandler::new(Arc::new(instance), outgoing.clone());

        let response = incoming
            .get(
                GetIsLikedRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                GetIsLikedRequest::NAME,
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

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = GetIsLikedResponse::parse_from_bytes(&response.payload).unwrap();
        assert_eq!(false, result.is_liked);
    }

    #[tokio::test]
    async fn test_add_favorite_request() {
        init_logger!();
        let media: Box<dyn MediaIdentifier> = create_media();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = FavoritesMessageHandler::new(Arc::new(instance), outgoing.clone());

        let response = incoming
            .get(
                AddFavoriteRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                AddFavoriteRequest::NAME,
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

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = AddFavoriteResponse::parse_from_bytes(&response.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            result.result
        );
    }

    #[tokio::test]
    async fn test_remove_favorite_request() {
        init_logger!();
        let media: Box<dyn MediaIdentifier> = create_media();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = FavoritesMessageHandler::new(Arc::new(instance), outgoing.clone());

        incoming
            .send(
                RemoveFavoriteRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                RemoveFavoriteRequest::NAME,
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
    }

    #[tokio::test]
    async fn test_is_liked_state_changed() {
        init_logger!();
        let imdb_id = "tt12000023";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let _handler = FavoritesMessageHandler::new(instance.clone(), outgoing.clone());

        let _ = instance
            .favorite_service()
            .add(Box::new(MovieOverview {
                title: "MyMovie".to_string(),
                imdb_id: imdb_id.to_string(),
                year: "2019".to_string(),
                rating: None,
                images: Default::default(),
            }))
            .await;

        let response = timeout!(incoming.recv(), Duration::from_millis(250))
            .expect("expected to have received an event message");
        assert_eq!(favorites::FavoriteEvent::NAME, response.type_.as_str());

        let event = favorites::FavoriteEvent::parse_from_bytes(&response.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<favorite_event::Event>>::into(
                favorite_event::Event::LIKED_STATE_CHANGED
            ),
            event.event
        );
        assert_eq!(imdb_id, event.like_state_changed.imdb_id.as_str());
    }

    fn create_media() -> Box<dyn MediaIdentifier> {
        Box::new(ShowOverview {
            imdb_id: "tt000001".to_string(),
            tvdb_id: "".to_string(),
            title: "ShowOverviewExample".to_string(),
            year: "2011".to_string(),
            num_seasons: 1,
            images: Images {
                poster: "http://localhost/my-poster.jpg".to_string(),
                fanart: "http://localhost/my-fanart.jpg".to_string(),
                banner: "http://localhost/my-background.jpg".to_string(),
            },
            rating: Some(Rating {
                percentage: 78,
                watching: 10,
                votes: 29,
                loved: 24,
                hated: 5,
            }),
        })
    }
}
