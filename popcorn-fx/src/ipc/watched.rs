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

                match &*event {
                    WatchedEvent::WatchedStateChanged(imdb_id, state) => {
                        proto_event.event = watched::watched_event::Event::STATE_CHANGED.into();
                        proto_event.watched_state_changed =
                            MessageField::some(watched::watched_event::WatchedStateChanged {
                                imdb_id: imdb_id.clone(),
                                new_state: *state,
                                special_fields: Default::default(),
                            });
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::core::media::{Episode, MovieOverview};
    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_is_watched_request() {
        init_logger!();
        let media = Box::new(Episode {
            season: 1,
            episode: 2,
            first_aired: 0,
            title: "MyEpisodeTitle".to_string(),
            overview: "MyEpisodeOverview".to_string(),
            tvdb_id: 128777777,
            tvdb_id_value: "128777777".to_string(),
            thumb: Some("EpisodeThumb.png".to_string()),
            torrents: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = WatchedMessageHandler::new(instance, outgoing.clone());

        let response = incoming
            .get(
                GetIsWatchedRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                GetIsWatchedRequest::NAME,
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
        let result = GetIsWatchedResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(false, result.is_watched);
    }

    #[tokio::test]
    async fn test_process_add_watchlist_request() {
        init_logger!();
        let media = Box::new(MovieOverview {
            imdb_id: "tt220000000".to_string(),
            title: "MyMovie".to_string(),
            year: "2010".to_string(),
            rating: None,
            images: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = WatchedMessageHandler::new(instance, outgoing.clone());

        let response = incoming
            .get(
                AddToWatchlistRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                AddToWatchlistRequest::NAME,
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
        let result = AddToWatchlistResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(EnumOrUnknown::from(response::Result::OK), result.result);
    }

    #[tokio::test]
    async fn test_process_remove_from_watchlist_request() {
        init_logger!();
        let media = Box::new(MovieOverview {
            imdb_id: "tt003".to_string(),
            title: "MyMovie".to_string(),
            year: "2010".to_string(),
            rating: None,
            images: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = WatchedMessageHandler::new(instance, outgoing.clone());

        incoming
            .send(
                RemoveFromWatchlistRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                RemoveFromWatchlistRequest::NAME,
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
}
