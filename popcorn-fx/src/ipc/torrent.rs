use crate::fx::PopcornFX;
use crate::ipc::proto::message;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::torrent::{
    torrent, CalculateTorrentHealthRequest, CalculateTorrentHealthResponse,
    CleanTorrentsDirectoryRequest, GetTorrentCollectionRequest, GetTorrentCollectionResponse,
    IsMagnetUriStoredRequest, IsMagnetUriStoredResponse, TorrentHealthRequest,
    TorrentHealthResponse,
};
use crate::ipc::{proto, Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use log::warn;
use popcorn_fx_core::core::torrents::TorrentManagerEvent;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TorrentMessageHandler {
    instance: Arc<PopcornFX>,
}

impl TorrentMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut reciever = instance.torrent_manager().subscribe();
        let instance = Self { instance };

        let handler = instance.clone();
        tokio::spawn(async move {
            while let Some(event) = reciever.recv().await {
                handler
                    .handle_torrent_manager_event(&*event, &channel)
                    .await;
            }
        });

        instance
    }

    async fn handle_torrent_manager_event(
        &self,
        event: &TorrentManagerEvent,
        channel: &IpcChannel,
    ) {
        if let TorrentManagerEvent::TorrentAdded(handle) = &*event {
            if let Some(mut torrent_receiver) = self
                .instance
                .torrent_manager()
                .find_by_handle(handle)
                .await
                .map(|e| e.subscribe())
            {
                let channel = channel.clone();
                let handle = handle.clone();
                tokio::spawn(async move {
                    while let Some(event) = torrent_receiver.recv().await {
                        let mut proto_event = proto::torrent::TorrentEvent::from(&*event);
                        proto_event.torrent_handle =
                            MessageField::some(message::Handle::from(&handle));

                        if let Err(e) = channel
                            .send(proto_event, proto::torrent::TorrentEvent::NAME)
                            .await
                        {
                            warn!("Failed to send torrent event to channel, {}", e);
                        }
                    }
                });
            }
        }
    }
}

#[async_trait]
impl MessageHandler for TorrentMessageHandler {
    fn name(&self) -> &str {
        "torrent"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            TorrentHealthRequest::NAME
                | CalculateTorrentHealthRequest::NAME
                | IsMagnetUriStoredRequest::NAME
                | GetTorrentCollectionRequest::NAME
                | CleanTorrentsDirectoryRequest::NAME
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
            IsMagnetUriStoredRequest::NAME => {
                let request = IsMagnetUriStoredRequest::parse_from_bytes(&message.payload)?;

                let is_stored = self
                    .instance
                    .torrent_collection()
                    .is_stored(request.magnet_uri.as_str())
                    .await;

                channel
                    .send_reply(
                        &message,
                        IsMagnetUriStoredResponse {
                            is_stored,
                            special_fields: Default::default(),
                        },
                        IsMagnetUriStoredResponse::NAME,
                    )
                    .await?;
            }
            GetTorrentCollectionRequest::NAME => {
                let response: GetTorrentCollectionResponse;

                match self.instance.torrent_collection().all().await {
                    Ok(collection) => {
                        response = GetTorrentCollectionResponse {
                            result: response::Result::OK.into(),
                            torrents: collection
                                .iter()
                                .map(proto::torrent::MagnetInfo::from)
                                .collect(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(err) => {
                        response = GetTorrentCollectionResponse {
                            result: response::Result::ERROR.into(),
                            torrents: Vec::with_capacity(0),
                            error: MessageField::some(torrent::Error::from(&err)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetTorrentCollectionResponse::NAME)
                    .await?;
            }
            CleanTorrentsDirectoryRequest::NAME => {
                self.instance.torrent_manager().cleanup().await;
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
