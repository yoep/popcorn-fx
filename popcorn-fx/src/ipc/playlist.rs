use crate::fx::PopcornFX;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::playlist::{
    GetActivePlaylistRequest, GetActivePlaylistResponse, PlayNextPlaylistItemRequest,
    PlayNextPlaylistItemResponse, PlayPlaylistRequest, PlayPlaylistResponse, PlaylistEvent,
    StopPlaylistRequest,
};
use crate::ipc::proto::{message, playlist};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use fx_callback::Callback;
use log::error;
use popcorn_fx_core::core::playlist::Playlist;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct PlaylistMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlaylistMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.playlist_manager().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match playlist::PlaylistEvent::try_from(&*event) {
                    Ok(proto_event) => {
                        if let Err(e) = channel.send(proto_event, PlaylistEvent::NAME).await {
                            error!("Failed to send playlist event to channel, {}", e);
                        }
                    }
                    Err(e) => error!("Failed to parse playlist event, {}", e),
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for PlaylistMessageHandler {
    fn name(&self) -> &str {
        "playlist"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            PlayPlaylistRequest::NAME
                | PlayNextPlaylistItemRequest::NAME
                | StopPlaylistRequest::NAME
                | GetActivePlaylistRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            PlayPlaylistRequest::NAME => {
                let request = PlayPlaylistRequest::parse_from_bytes(&message.payload)?;
                let playlist = request
                    .playlist
                    .as_ref()
                    .map(Playlist::try_from)
                    .transpose()?
                    .ok_or(Error::MissingField)?;

                let handle = self
                    .instance
                    .playlist_manager()
                    .play(playlist)
                    .await
                    .map(|e| message::Handle {
                        handle: e.value(),
                        special_fields: Default::default(),
                    });

                channel
                    .send_reply(
                        &message,
                        PlayPlaylistResponse {
                            handle: MessageField::from(handle),
                            special_fields: Default::default(),
                        },
                        PlayPlaylistResponse::NAME,
                    )
                    .await?;
            }
            PlayNextPlaylistItemRequest::NAME => {
                let handle =
                    self.instance
                        .playlist_manager()
                        .play_next()
                        .await
                        .map(|e| message::Handle {
                            handle: e.value(),
                            special_fields: Default::default(),
                        });

                channel
                    .send_reply(
                        &message,
                        PlayNextPlaylistItemResponse {
                            handle: MessageField::from(handle),
                            special_fields: Default::default(),
                        },
                        PlayNextPlaylistItemResponse::NAME,
                    )
                    .await?;
            }
            StopPlaylistRequest::NAME => {
                self.instance.playlist_manager().stop().await;
            }
            GetActivePlaylistRequest::NAME => {
                let playlist = playlist::Playlist::try_from(
                    &self.instance.playlist_manager().playlist().await,
                )?;

                channel
                    .send_reply(
                        &message,
                        GetActivePlaylistResponse {
                            playlist: MessageField::some(playlist),
                            special_fields: Default::default(),
                        },
                        GetActivePlaylistResponse::NAME,
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
