use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::player;
use crate::ipc::proto::player::{
    register_player_response, DiscoverPlayersRequest, GetActivePlayerRequest,
    GetActivePlayerResponse, GetPlayerByIdRequest, GetPlayerByIdResponse, GetPlayerStateRequest,
    GetPlayerStateResponse, GetPlayersRequest, GetPlayersResponse, RegisterPlayerRequest,
    RegisterPlayerResponse, RemovePlayerRequest, StartPlayersDiscoveryRequest,
    UpdateActivePlayerRequest,
};
use crate::ipc::{Error, IpcChannel, MessageHandler, Result};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{error, warn};
use popcorn_fx_core::core::players::{
    PlayRequest, Player, PlayerEvent, PlayerManagerEvent, PlayerState,
};
use protobuf::{Message, MessageField};
use std::sync::{Arc, Weak};

#[derive(Debug, Display)]
#[display(fmt = "Player message handler")]
pub struct PlayerMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlayerMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: &IpcChannel) -> Self {
        let mut receiver = instance.player_manager().subscribe();

        let channel = channel.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let mut proto_event = player::PlayerManagerEvent::new();

                match &*event {
                    PlayerManagerEvent::ActivePlayerChanged(change) => {
                        proto_event.event =
                            player::player_manager_event::Event::ACTIVE_PLAYER_CHANGED.into();
                        proto_event.active_player_changed =
                            MessageField::some(player::player_manager_event::ActivePlayerChanged {
                                old_player_id: change.old_player_id.clone(),
                                new_player_id: change.new_player_id.clone(),
                                new_player_name: change.new_player_name.clone(),
                                special_fields: Default::default(),
                            });
                    }
                    PlayerManagerEvent::PlayersChanged => {
                        proto_event.event =
                            player::player_manager_event::Event::PLAYERS_CHANGED.into();
                    }
                    PlayerManagerEvent::PlayerPlaybackChanged(_) => {
                        proto_event.event =
                            player::player_manager_event::Event::PLAYER_PLAYBACK_CHANGED.into();
                    }
                    PlayerManagerEvent::PlayerDurationChanged(duration) => {
                        proto_event.event =
                            player::player_manager_event::Event::PLAYER_DURATION_CHANGED.into();
                        proto_event.player_duration_changed = MessageField::some(
                            player::player_manager_event::PlayerDurationChanged {
                                duration: *duration,
                                special_fields: Default::default(),
                            },
                        );
                    }
                    PlayerManagerEvent::PlayerTimeChanged(time) => {
                        proto_event.event =
                            player::player_manager_event::Event::PLAYER_TIMED_CHANGED.into();
                        proto_event.player_time_changed =
                            MessageField::some(player::player_manager_event::PlayerTimeChanged {
                                time: *time,
                                special_fields: Default::default(),
                            });
                    }
                    PlayerManagerEvent::PlayerStateChanged(state) => {
                        proto_event.event =
                            player::player_manager_event::Event::PLAYER_STATE_CHANGED.into();
                        proto_event.player_state_changed =
                            MessageField::some(player::player_manager_event::PlayerStateChanged {
                                state: player::player::State::from(state).into(),
                                special_fields: Default::default(),
                            });
                    }
                }

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
}

#[async_trait]
impl MessageHandler for PlayerMessageHandler {
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
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
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
                    let mut proto_player = player::Player::from(&*player);
                    proto_player.state = player::player::State::from(&state).into();
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
                    let mut proto_player = player::Player::from(&*player);
                    proto_player.state = player::player::State::from(&state).into();

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
                    let mut proto_player = player::Player::from(&*player);
                    proto_player.state = player::player::State::from(&state).into();
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
    proto: player::Player,
    channel: IpcChannel,
    callbacks: MultiThreadedCallback<PlayerEvent>,
}

impl ProtoPlayerWrapper {
    pub fn new(player: player::Player, channel: IpcChannel) -> Self {
        Self {
            proto: player,
            channel,
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

    async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        // TODO: request the play request from the actual proto channel
        None
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        let request = player::player::PlayRequest::from(&request);
        if let Err(e) = self
            .channel
            .send(
                player::PlayerPlayRequest {
                    player_id: self.proto.id.clone(),
                    request: MessageField::some(request),
                    special_fields: Default::default(),
                },
                player::PlayerPlayRequest::NAME,
            )
            .await
        {
            error!("Player {} failed to send play request, {}", self.id(), e);
        }
    }

    async fn pause(&self) {
        if let Err(e) = self
            .channel
            .send(
                player::PlayerPauseRequest {
                    player_id: self.proto.id.clone(),
                    special_fields: Default::default(),
                },
                player::PlayerPauseRequest::NAME,
            )
            .await
        {
            error!("Player {} failed to send pause request, {}", self.id(), e);
        }
    }

    async fn resume(&self) {
        if let Err(e) = self
            .channel
            .send(
                player::PlayerResumeRequest {
                    player_id: self.proto.id.clone(),
                    special_fields: Default::default(),
                },
                player::PlayerResumeRequest::NAME,
            )
            .await
        {
            error!("Player {} failed to send resume request, {}", self.id(), e);
        }
    }

    fn seek(&self, time: u64) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }
}
