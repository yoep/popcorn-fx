use crate::fx::PopcornFX;
use crate::ipc::proto::message;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::playlist::{
    PlayNextPlaylistItemRequest, PlayNextPlaylistItemResponse, PlayPlaylistRequest,
    PlayPlaylistResponse, StopPlaylistRequest,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::playlist::Playlist;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Playlist message handler")]
pub struct PlaylistMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlaylistMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for PlaylistMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            PlayPlaylistRequest::NAME
                | PlayNextPlaylistItemRequest::NAME
                | StopPlaylistRequest::NAME
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
            _ => {
                return Err(Error::UnsupportedMessage(
                    message.message_type().to_string(),
                ))
            }
        }

        Ok(())
    }
}
