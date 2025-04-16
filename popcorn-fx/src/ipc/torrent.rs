use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::torrent::{
    torrent, CalculateTorrentHealthRequest, CalculateTorrentHealthResponse, TorrentHealthRequest,
    TorrentHealthResponse,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use log::warn;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Torrent message handler")]
pub struct TorrentMessageHandler {
    instance: Arc<PopcornFX>,
}

impl TorrentMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for TorrentMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            TorrentHealthRequest::NAME | CalculateTorrentHealthRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            TorrentHealthRequest::NAME => {
                let request = TorrentHealthRequest::parse_from_bytes(&message.payload)?;
                let response: TorrentHealthResponse;

                match self
                    .instance
                    .torrent_manager()
                    .health_from_uri(request.uri.as_str())
                    .await
                {
                    Ok(health) => {
                        response = TorrentHealthResponse {
                            result: response::Result::OK.into(),
                            health: MessageField::some(torrent::Health::from(&health)),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        warn!("Failed to retrieve torrent health, {}", e);
                        response = TorrentHealthResponse {
                            result: response::Result::ERROR.into(),
                            health: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, TorrentHealthResponse::NAME)
                    .await?;
            }
            CalculateTorrentHealthRequest::NAME => {
                let request = CalculateTorrentHealthRequest::parse_from_bytes(&message.payload)?;

                let health = self
                    .instance
                    .torrent_manager()
                    .calculate_health(request.seeds, request.leechers);

                channel
                    .send_reply(
                        &message,
                        CalculateTorrentHealthResponse {
                            health: MessageField::some(torrent::Health::from(&health)),
                            special_fields: Default::default(),
                        },
                        CalculateTorrentHealthResponse::NAME,
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
