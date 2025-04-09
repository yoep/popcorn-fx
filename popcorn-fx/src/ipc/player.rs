use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::player;
use crate::ipc::proto::player::{
    register_player_response, GetActivePlayerRequest, GetActivePlayerResponse,
    GetPlayerByIdRequest, GetPlayerByIdResponse, GetPlayersRequest, GetPlayersResponse,
    RegisterPlayerRequest, RegisterPlayerResponse, RemovePlayerRequest,
    StartPlayersDiscoveryRequest, UpdateActivePlayerRequest,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::warn;
use popcorn_fx_core::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
use protobuf::{Message, MessageField};
use std::sync::{Arc, Weak};

#[derive(Debug, Display)]
#[display(fmt = "Player message handler")]
pub struct PlayerMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlayerMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
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
                    let player = ProtoPlayerWrapper::new(*proto_player);

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
    callbacks: MultiThreadedCallback<PlayerEvent>,
}

impl ProtoPlayerWrapper {
    pub fn new(player: player::Player) -> Self {
        Self {
            proto: player,
            callbacks: MultiThreadedCallback::new(),
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
        // TODO: request state from the actual proto channel
        PlayerState::Unknown
    }

    async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        // TODO: request the play request from the actual proto channel
        None
    }

    async fn play(&self, _request: Box<dyn PlayRequest>) {
        todo!()
    }

    fn pause(&self) {
        todo!()
    }

    fn resume(&self) {
        todo!()
    }

    fn seek(&self, time: u64) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }
}
