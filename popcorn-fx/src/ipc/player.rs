use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::player;
use crate::ipc::proto::player::{
    register_player_response, DiscoverPlayersRequest, GetActivePlayerRequest,
    GetActivePlayerResponse, GetPlayerByIdRequest, GetPlayerByIdResponse, GetPlayerStateRequest,
    GetPlayerStateResponse, GetPlayerVolumeRequest, GetPlayerVolumeResponse, GetPlayersRequest,
    GetPlayersResponse, PlayerPauseRequest, PlayerResumeRequest, PlayerSeekRequest,
    PlayerStopRequest, RegisterPlayerRequest, RegisterPlayerResponse, RemovePlayerRequest,
    StartPlayersDiscoveryRequest, UpdateActivePlayerRequest,
};
use crate::ipc::{proto, Error, IpcChannel, MessageHandler, Result};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{error, trace, warn};
use popcorn_fx_core::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
use protobuf::{Message, MessageField};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct PlayerMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlayerMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.player_manager().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let proto_event = player::PlayerManagerEvent::from(&*event);

                if let Err(e) = channel
                    .send(proto_event, player::PlayerManagerEvent::NAME)
                    .await
                {
                    error!("Failed to send player manager event, {}", e);
                }
            }
        });

        Self { instance }
    }

    async fn execute_player_action<S, F, Fut>(&self, id: S, action: F)
    where
        S: AsRef<str>,
        F: FnOnce(Arc<Box<dyn Player>>) -> Fut,
        Fut: Future<Output = ()>,
    {
        if let Some(player) = self
            .instance
            .player_manager()
            .by_id(id.as_ref())
            .and_then(|e| e.upgrade())
        {
            action(player).await;
        } else {
            warn!(
                "Failed to execute player action, player \"{}\" not found",
                id.as_ref()
            );
        }
    }
}

#[async_trait]
impl MessageHandler for PlayerMessageHandler {
    fn name(&self) -> &str {
        "player"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetPlayerByIdRequest::NAME
                | GetPlayersRequest::NAME
                | GetActivePlayerRequest::NAME
                | UpdateActivePlayerRequest::NAME
                | RegisterPlayerRequest::NAME
                | RemovePlayerRequest::NAME
                | DiscoverPlayersRequest::NAME
                | StartPlayersDiscoveryRequest::NAME
                | PlayerPauseRequest::NAME
                | PlayerResumeRequest::NAME
                | PlayerStopRequest::NAME
                | PlayerSeekRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> Result<()> {
        match message.message_type() {
            GetPlayerByIdRequest::NAME => {
                let request = GetPlayerByIdRequest::parse_from_bytes(&message.payload)?;
                let mut response = GetPlayerByIdResponse::new();

                if let Some(player) = self
                    .instance
                    .player_manager()
                    .by_id(request.id.as_str())
                    .and_then(|e| e.upgrade())
                {
                    let state = player.state().await;
                    let mut proto_player = proto::player::Player::from(&*player);
                    proto_player.state = proto::player::player::State::from(&state).into();
                    response.player = MessageField::some(proto_player);
                }

                channel
                    .send_reply(&message, response, GetPlayerByIdResponse::NAME)
                    .await?;
            }
            GetPlayersRequest::NAME => {
                let mut response = GetPlayersResponse::new();
                response.players = vec![];

                for player in self
                    .instance
                    .player_manager()
                    .players()
                    .iter()
                    .flat_map(|e| e.upgrade())
                {
                    let state = player.state().await;
                    let mut proto_player = proto::player::Player::from(&*player);
                    proto_player.state = proto::player::player::State::from(&state).into();

                    response.players.push(proto_player);
                }

                channel
                    .send_reply(&message, response, GetPlayersResponse::NAME)
                    .await?;
            }
            GetActivePlayerRequest::NAME => {
                let mut response = GetActivePlayerResponse::new();

                if let Some(player) = self
                    .instance
                    .player_manager()
                    .active_player()
                    .await
                    .and_then(|e| e.upgrade())
                {
                    let state = player.state().await;
                    let mut proto_player = proto::player::Player::from(&*player);
                    proto_player.state = proto::player::player::State::from(&state).into();
                    response.player = MessageField::some(proto_player);
                }

                channel
                    .send_reply(&message, response, GetActivePlayerResponse::NAME)
                    .await?;
            }
            UpdateActivePlayerRequest::NAME => {
                let request = UpdateActivePlayerRequest::parse_from_bytes(&message.payload)?;

                self.instance
                    .player_manager()
                    .set_active_player(request.player.id.as_str())
                    .await;
            }
            RegisterPlayerRequest::NAME => {
                let request = RegisterPlayerRequest::parse_from_bytes(&message.payload)?;
                let mut response = RegisterPlayerResponse::new();

                if let Some(proto_player) = request.player.0 {
                    let player = ProtoPlayerWrapper::new(*proto_player, channel.clone());

                    if let Err(err) = self.instance.player_manager().add_player(Box::new(player)) {
                        warn!("Failed to register new player, {}", err);
                        response.result = response::Result::ERROR.into();
                        response.error = register_player_response::Error::DUPLICATE_PLAYER.into();
                    } else {
                        response.result = response::Result::OK.into();
                    }
                } else {
                    warn!("Invalid register player request received, no player defined");
                };

                channel
                    .send_reply(&message, response, RegisterPlayerResponse::NAME)
                    .await?;
            }
            RemovePlayerRequest::NAME => {
                let request = RemovePlayerRequest::parse_from_bytes(&message.payload)?;
                self.instance
                    .player_manager()
                    .remove_player(request.player.id.as_str());
            }
            DiscoverPlayersRequest::NAME => {
                // TODO
            }
            StartPlayersDiscoveryRequest::NAME => {
                let request = StartPlayersDiscoveryRequest::parse_from_bytes(&message.payload)?;
                self.instance
                    .start_discovery_external_players(request.interval_seconds.unwrap_or(60));
            }
            PlayerPauseRequest::NAME => {
                let request = PlayerPauseRequest::parse_from_bytes(&message.payload)?;
                let player_id = request.player_id.as_str();

                self.execute_player_action(player_id, |player| async move { player.pause().await })
                    .await;
            }
            PlayerResumeRequest::NAME => {
                let request = PlayerResumeRequest::parse_from_bytes(&message.payload)?;
                let player_id = request.player_id.as_str();

                self.execute_player_action(
                    player_id,
                    |player| async move { player.resume().await },
                )
                .await;
            }
            PlayerStopRequest::NAME => {
                let request = PlayerStopRequest::parse_from_bytes(&message.payload)?;
                let player_id = request.player_id.as_str();

                self.execute_player_action(player_id, |player| async move { player.stop().await })
                    .await;
            }
            PlayerSeekRequest::NAME => {
                let request = PlayerSeekRequest::parse_from_bytes(&message.payload)?;
                let player_id = request.player_id.as_str();

                self.execute_player_action(player_id, |player| async move {
                    player.seek(request.time).await
                })
                .await;
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

#[derive(Debug, Display)]
#[display(fmt = "{}", "proto.name")]
struct ProtoPlayerWrapper {
    proto: proto::player::Player,
    channel: IpcChannel,
    request: Mutex<Option<PlayRequest>>,
    callbacks: MultiThreadedCallback<PlayerEvent>,
}

impl ProtoPlayerWrapper {
    pub fn new(player: proto::player::Player, channel: IpcChannel) -> Self {
        Self {
            proto: player,
            channel,
            request: Default::default(),
            callbacks: MultiThreadedCallback::new(),
        }
    }

    async fn internal_player_state(&self) -> Result<PlayerState> {
        let receiver = self
            .channel
            .get(
                GetPlayerStateRequest {
                    player_id: self.proto.id.clone(),
                    special_fields: Default::default(),
                },
                GetPlayerStateRequest::NAME,
            )
            .await?;
        let message = receiver.await?;
        let response = GetPlayerStateResponse::parse_from_bytes(&message.payload)?;

        match response.state.enum_value() {
            Ok(state) => Ok(PlayerState::from(&state)),
            Err(_) => Err(Error::UnsupportedEnum),
        }
    }

    async fn send_channel_message(&self, message: impl Message, message_type: &str) {
        trace!(
            "Proto player {} is sending message \"{}\"",
            self,
            message_type
        );
        if let Err(e) = self.channel.send(message, message_type).await {
            error!(
                "Proto player {} failed to send message \"{}\", {}",
                self, message_type, e
            );
        }
    }
}

impl Callback<PlayerEvent> for ProtoPlayerWrapper {
    fn subscribe(&self) -> Subscription<PlayerEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>) {
        self.callbacks.subscribe_with(subscriber)
    }
}

#[async_trait]
impl Player for ProtoPlayerWrapper {
    fn id(&self) -> &str {
        self.proto.id.as_str()
    }

    fn name(&self) -> &str {
        self.proto.name.as_str()
    }

    fn description(&self) -> &str {
        self.proto.description.as_str()
    }

    fn graphic_resource(&self) -> Vec<u8> {
        self.proto.graphic_resource.clone()
    }

    async fn state(&self) -> PlayerState {
        self.internal_player_state().await.unwrap_or_else(|e| {
            error!("Proto player failed to retrieve state, {}", e);
            PlayerState::Unknown
        })
    }

    async fn request(&self) -> Option<PlayRequest> {
        self.request.lock().await.clone()
    }

    async fn current_volume(&self) -> Option<u32> {
        let response = self
            .channel
            .get(
                GetPlayerVolumeRequest {
                    player_id: self.proto.id.clone(),
                    special_fields: Default::default(),
                },
                GetPlayerVolumeResponse::NAME,
            )
            .await
            .map_err(|e| {
                error!("Proto player failed to retrieve the volume, {}", e);
                e
            })
            .ok();

        if let Some(response) = response {
            match response
                .await
                .map_err(|e| Error::from(e))
                .and_then(|message| {
                    GetPlayerVolumeResponse::parse_from_bytes(&message.payload)
                        .map_err(|e| Error::from(e))
                }) {
                Ok(response) => {
                    return response.volume;
                }
                Err(e) => {
                    error!("Proto player failed to retrieve the volume, {}", e);
                }
            }
        }

        None
    }

    async fn play(&self, request: PlayRequest) {
        let proto_request = proto::player::player::PlayRequest::from(&request);
        self.send_channel_message(
            proto::player::PlayerPlayRequest {
                player_id: self.proto.id.clone(),
                request: MessageField::some(proto_request),
                special_fields: Default::default(),
            },
            proto::player::PlayerPlayRequest::NAME,
        )
        .await;
        self.request.lock().await.replace(request);
    }

    async fn pause(&self) {
        self.send_channel_message(
            proto::player::PlayerPauseRequest {
                player_id: self.proto.id.clone(),
                special_fields: Default::default(),
            },
            proto::player::PlayerPauseRequest::NAME,
        )
        .await;
    }

    async fn resume(&self) {
        self.send_channel_message(
            proto::player::PlayerResumeRequest {
                player_id: self.proto.id.clone(),
                special_fields: Default::default(),
            },
            proto::player::PlayerResumeRequest::NAME,
        )
        .await;
    }

    async fn seek(&self, time: u64) {
        self.send_channel_message(
            proto::player::PlayerSeekRequest {
                player_id: self.proto.id.clone(),
                time,
                special_fields: Default::default(),
            },
            proto::player::PlayerSeekRequest::NAME,
        )
        .await;
    }

    async fn stop(&self) {
        self.send_channel_message(
            proto::player::PlayerStopRequest {
                player_id: self.proto.id.clone(),
                special_fields: Default::default(),
            },
            proto::player::PlayerStopRequest::NAME,
        )
        .await;
    }
}
