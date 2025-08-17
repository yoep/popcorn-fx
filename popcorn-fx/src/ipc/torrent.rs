use crate::fx::PopcornFX;
use crate::ipc::proto::message;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::torrent::{
    torrent, AddTorrentCollectionRequest, AddTorrentCollectionResponse,
    CalculateTorrentHealthRequest, CalculateTorrentHealthResponse, CleanTorrentsDirectoryRequest,
    GetTorrentCollectionRequest, GetTorrentCollectionResponse, IsMagnetUriStoredRequest,
    IsMagnetUriStoredResponse, RemoveTorrentCollectionRequest, TorrentHealthRequest,
    TorrentHealthResponse,
};
use crate::ipc::{proto, Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use log::warn;
use popcorn_fx_core::core::torrents;
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
                | AddTorrentCollectionRequest::NAME
                | RemoveTorrentCollectionRequest::NAME
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
            AddTorrentCollectionRequest::NAME => {
                let mut request = AddTorrentCollectionRequest::parse_from_bytes(&message.payload)?;
                let info = request
                    .magnet_info
                    .take()
                    .map(|magnet| (magnet.name, magnet.magnet_uri))
                    .or_else(|| {
                        request
                            .torrent_info
                            .take()
                            .map(|torrent| (torrent.name, torrent.uri))
                    });
                let response: AddTorrentCollectionResponse = if let Some((name, magnet_uri)) = info
                {
                    self.instance
                        .torrent_collection()
                        .insert(name.as_str(), magnet_uri.as_str())
                        .await;
                    AddTorrentCollectionResponse {
                        result: response::Result::OK.into(),
                        error: Default::default(),
                        special_fields: Default::default(),
                    }
                } else {
                    warn!("Torrent message handler failed to add torrent to collection, missing field \"magnet_info\"");
                    AddTorrentCollectionResponse {
                        result: response::Result::ERROR.into(),
                        error: MessageField::some(torrent::Error::from(
                            &torrents::Error::InvalidUrl("".to_string()),
                        )),
                        special_fields: Default::default(),
                    }
                };

                channel
                    .send_reply(&message, response, AddTorrentCollectionResponse::NAME)
                    .await?;
            }
            RemoveTorrentCollectionRequest::NAME => {
                let request = RemoveTorrentCollectionRequest::parse_from_bytes(&message.payload)?;
                let magnet_uri: &str;

                if let Some(info) = request.magnet_info.as_ref() {
                    magnet_uri = info.magnet_uri.as_str();
                } else if let Some(torrent) = request.torrent_info.as_ref() {
                    magnet_uri = torrent.uri.as_str();
                } else {
                    warn!("Torrent message handler failed to remove torrent, field \"magnet_info\" or \"torrent_info\" are not present");
                    return Err(Error::MissingField);
                }

                self.instance.torrent_collection().remove(magnet_uri).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::proto::torrent::{MagnetInfo, TorrentEvent};
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_torrent_health_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance, outgoing.clone());

        let response = incoming
            .get(
                TorrentHealthRequest {
                    uri: magnet_uri().to_string(),
                    special_fields: Default::default(),
                },
                TorrentHealthRequest::NAME,
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
        let result = TorrentHealthResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            result.result
        );
        assert_ne!(MessageField::none(), result.health);
    }

    #[tokio::test]
    async fn test_process_calculate_health_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance, outgoing.clone());

        let response = incoming
            .get(
                CalculateTorrentHealthRequest {
                    seeds: 20,
                    leechers: 8,
                    special_fields: Default::default(),
                },
                CalculateTorrentHealthRequest::NAME,
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
        let result = CalculateTorrentHealthResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            MessageField::some(torrent::Health {
                state: torrent::health::State::GOOD.into(),
                ratio: 2.5,
                seeds: 20,
                leechers: 8,
                special_fields: Default::default(),
            }),
            result.health
        );
    }

    #[tokio::test]
    async fn test_process_is_magnet_uri_stored_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .torrent_collection()
            .insert("test", magnet_uri())
            .await;

        let response = incoming
            .get(
                IsMagnetUriStoredRequest {
                    magnet_uri: magnet_uri().to_string(),
                    special_fields: Default::default(),
                },
                IsMagnetUriStoredRequest::NAME,
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
        let result = IsMagnetUriStoredResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            true, result.is_stored,
            "expected the magnet uri to have been stored"
        );
    }

    #[tokio::test]
    async fn test_process_get_torrent_collection_request() {
        init_logger!();
        let magnet_info = MagnetInfo {
            name: "MyMagnetName".to_string(),
            magnet_uri: magnet_uri().to_string(),
            special_fields: Default::default(),
        };
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .torrent_collection()
            .insert(magnet_info.name.as_str(), magnet_info.magnet_uri.as_str())
            .await;

        let response = incoming
            .get(
                GetTorrentCollectionRequest::new(),
                GetTorrentCollectionRequest::NAME,
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
        let result = GetTorrentCollectionResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            result.result
        );
        assert_eq!(vec![magnet_info], result.torrents);
    }

    mod add_torrent_collection {
        use super::*;

        #[tokio::test]
        async fn test_process_magnet_info() {
            init_logger!();
            let name = "FooBar";
            let magnet_uri = "magnet:?xt=SomeRandomMagnet";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

            let response = incoming
                .get(
                    AddTorrentCollectionRequest {
                        type_: MagnetInfo::NAME.to_string(),
                        magnet_info: MessageField::some(MagnetInfo {
                            name: name.to_string(),
                            magnet_uri: magnet_uri.to_string(),
                            special_fields: Default::default(),
                        }),
                        torrent_info: Default::default(),
                        special_fields: Default::default(),
                    },
                    AddTorrentCollectionRequest::NAME,
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
            let result = AddTorrentCollectionResponse::parse_from_bytes(&response.payload).unwrap();
            assert_eq!(
                EnumOrUnknown::<response::Result>::from(response::Result::OK),
                result.result
            );

            let result = instance.torrent_collection().is_stored(magnet_uri).await;
            assert_eq!(true, result, "expected the magnet to have been stored")
        }

        #[tokio::test]
        async fn test_process_torrent_info() {
            init_logger!();
            let name = "LoremIpsumDolorTor";
            let magnet_uri = "magnet:?xt=SomeRandomMagnet";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

            let response = incoming
                .get(
                    AddTorrentCollectionRequest {
                        type_: torrent::Info::NAME.to_string(),
                        magnet_info: Default::default(),
                        torrent_info: MessageField::some(torrent::Info {
                            handle: Default::default(),
                            info_hash: "".to_string(),
                            uri: magnet_uri.to_string(),
                            name: name.to_string(),
                            directory_name: None,
                            total_files: 0,
                            files: vec![],
                            special_fields: Default::default(),
                        }),
                        special_fields: Default::default(),
                    },
                    AddTorrentCollectionRequest::NAME,
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
            let result = AddTorrentCollectionResponse::parse_from_bytes(&response.payload).unwrap();
            assert_eq!(
                EnumOrUnknown::<response::Result>::from(response::Result::OK),
                result.result
            );

            let result = instance.torrent_collection().is_stored(magnet_uri).await;
            assert_eq!(true, result, "expected the magnet to have been stored")
        }

        #[tokio::test]
        async fn test_process_no_info() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

            let response = incoming
                .get(
                    AddTorrentCollectionRequest {
                        type_: String::default(),
                        magnet_info: Default::default(),
                        torrent_info: Default::default(),
                        special_fields: Default::default(),
                    },
                    AddTorrentCollectionRequest::NAME,
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
            let result = AddTorrentCollectionResponse::parse_from_bytes(&response.payload).unwrap();
            assert_eq!(
                EnumOrUnknown::<response::Result>::from(response::Result::ERROR),
                result.result,
                "expected an error to have been returned"
            );
        }
    }

    #[tokio::test]
    async fn test_torrent_process_remove_torrent_collection_magnet_info() {
        init_logger!();
        let name = "FooBar";
        let magnet_uri = "magnet:?xt=SomeRandomMagnet";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(
                RemoveTorrentCollectionRequest {
                    type_: MagnetInfo::NAME.to_string(),
                    magnet_info: MessageField::some(MagnetInfo {
                        name: name.to_string(),
                        magnet_uri: magnet_uri.to_string(),
                        special_fields: Default::default(),
                    }),
                    torrent_info: Default::default(),
                    special_fields: Default::default(),
                },
                RemoveTorrentCollectionRequest::NAME,
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
    async fn test_torrent_process_remove_torrent_collection_torrent_info() {
        init_logger!();
        let name = "FooBar";
        let magnet_uri = "magnet:?xt=SomeRandomMagnet";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(
                RemoveTorrentCollectionRequest {
                    type_: torrent::Info::NAME.to_string(),
                    magnet_info: Default::default(),
                    torrent_info: MessageField::some(torrent::Info {
                        handle: Default::default(),
                        info_hash: "".to_string(),
                        uri: magnet_uri.to_string(),
                        name: name.to_string(),
                        directory_name: None,
                        total_files: 0,
                        files: vec![],
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                RemoveTorrentCollectionRequest::NAME,
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
    async fn test_process_clean_torrents_directory_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(
                CleanTorrentsDirectoryRequest::new(),
                CleanTorrentsDirectoryRequest::NAME,
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
    async fn test_torrent_added_event() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let _handler = TorrentMessageHandler::new(instance.clone(), outgoing.clone());

        let result = instance
            .torrent_manager()
            .create(magnet_uri())
            .await
            .unwrap();
        result.prioritize_pieces(&vec![0, 1]).await;
        let handle = result.handle();

        let message = timeout!(incoming.recv(), Duration::from_secs(1)).unwrap();
        let result = TorrentEvent::parse_from_bytes(&message.payload).unwrap();

        let _ = result.event.enum_value().expect("expected a valid event");
        assert_eq!(
            handle.value(),
            result.torrent_handle.handle,
            "expected the handle to match"
        );
    }

    fn magnet_uri() -> &'static str {
        "magnet:?xt=urn:btih:2C6B6858D61DA9543D4231A71DB4B1C9264B0685&dn=Ubuntu%2022.04%20LTS&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce"
    }
}
