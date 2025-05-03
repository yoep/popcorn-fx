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

impl PartialEq for ProtoPlayerWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.proto == other.proto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::proto::player::PlayerPlayRequest;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::try_recv;

    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::MockPlayer;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_is_supported() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (_incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance, outgoing);

        assert_eq!(true, handler.is_supported(GetPlayerByIdRequest::NAME));
        assert_eq!(true, handler.is_supported(GetPlayersRequest::NAME));
    }

    #[tokio::test]
    async fn test_process_get_player_by_id() {
        init_logger!();
        let player_id = "mock-player-id";
        let player = create_mock_player(player_id);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        let result = instance.player_manager().add_player(Box::new(player));
        assert_eq!(Ok(()), result, "expected the player to have been added");

        let response = incoming
            .get(
                GetPlayerByIdRequest {
                    id: player_id.to_string(),
                    special_fields: Default::default(),
                },
                GetPlayerByIdRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = GetPlayerByIdResponse::parse_from_bytes(&response.payload).unwrap();

        assert_ne!(
            MessageField::none(),
            result.player,
            "expected the player to have been returned"
        );
    }

    #[tokio::test]
    async fn test_process_get_players_request() {
        init_logger!();
        let player_id = "mock-player-id";
        let player = create_mock_player(player_id);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        let result = instance.player_manager().add_player(Box::new(player));
        assert_eq!(Ok(()), result, "expected the player to have been added");

        let response = incoming
            .get(GetPlayersRequest::default(), GetPlayersRequest::NAME)
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = GetPlayersResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            1,
            result.players.len(),
            "expected the player to have been returned"
        );
        assert_eq!(player_id, result.players.get(0).unwrap().id);
    }

    #[tokio::test]
    async fn test_process_get_active_player_request() {
        init_logger!();
        let player_id = "active-player-id";
        let player = create_mock_player(player_id);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        let result = instance.player_manager().add_player(Box::new(player));
        assert_eq!(Ok(()), result, "expected the player to have been added");
        instance.player_manager().set_active_player(player_id).await;

        let response = incoming
            .get(
                GetActivePlayerRequest::default(),
                GetActivePlayerRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = GetActivePlayerResponse::parse_from_bytes(&response.payload).unwrap();

        assert_ne!(
            MessageField::none(),
            result.player,
            "expected an active player"
        );
        assert_eq!(player_id, result.player.id.as_str());
    }

    #[tokio::test]
    async fn test_process_update_active_player_request() {
        init_logger!();
        let player_id = "active-player-id";
        let player = create_mock_player(player_id);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        let result = instance.player_manager().add_player(Box::new(player));
        assert_eq!(Ok(()), result, "expected the player to have been added");

        incoming
            .send(
                UpdateActivePlayerRequest {
                    player: MessageField::some(player::Player {
                        id: player_id.to_string(),
                        name: "player-name".to_string(),
                        description: "player-description".to_string(),
                        graphic_resource: vec![],
                        state: Default::default(),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdateActivePlayerRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let player = instance
            .player_manager()
            .active_player()
            .await
            .and_then(|e| e.upgrade())
            .expect("expected an active player");
        assert_eq!(
            player_id,
            player.id(),
            "expected the active player to match"
        );
    }

    #[tokio::test]
    async fn test_process_register_player_request() {
        init_logger!();
        let proto_player = player::Player {
            id: "my-player-id".to_string(),
            name: "player-name".to_string(),
            description: "player-description".to_string(),
            graphic_resource: vec![0, 11, 14],
            state: player::player::State::STOPPED.into(),
            special_fields: Default::default(),
        };
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let player = ProtoPlayerWrapper::new(proto_player.clone(), outgoing.clone());
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        let response = incoming
            .get(
                RegisterPlayerRequest {
                    player: MessageField::some(proto_player),
                    special_fields: Default::default(),
                },
                RegisterPlayerRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = try_recv!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = RegisterPlayerResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            result.result
        );
        let result = instance
            .player_manager()
            .players()
            .get(0)
            .and_then(|e| e.upgrade())
            .unwrap();
        if let Some(result) = result.downcast_ref::<ProtoPlayerWrapper>() {
            assert_eq!(&player, result);
        } else {
            assert!(
                false,
                "expected a ProtoPlayerWrapper, but got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_process_remove_player_request() {
        init_logger!();
        let player_id = "player-to-remove";
        let player = create_mock_player(player_id);
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .player_manager()
            .add_player(Box::new(player))
            .unwrap();
        let players = instance.player_manager().players();
        assert_eq!(1, players.len(), "expected the player to have been added");

        incoming
            .send(
                RemovePlayerRequest {
                    player: MessageField::some(player::Player {
                        id: player_id.to_string(),
                        name: "mock-player".to_string(),
                        description: "player-description".to_string(),
                        graphic_resource: vec![],
                        state: player::player::State::READY.into(),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                RemovePlayerRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let result = instance.player_manager().players();
        assert_eq!(0, result.len(), "expected the player to have been removed");
    }

    #[tokio::test]
    async fn test_process_start_players_discovery_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(
                StartPlayersDiscoveryRequest::default(),
                StartPlayersDiscoveryRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_player_pause_request() {
        init_logger!();
        let player_id = "my-player";
        let mut player = create_mock_player(player_id);
        player.expect_pause().times(1).return_const(());
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .player_manager()
            .add_player(Box::new(player))
            .unwrap();

        incoming
            .send(
                PlayerPauseRequest {
                    player_id: player_id.to_string(),
                    special_fields: Default::default(),
                },
                PlayerPauseRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_player_resume_request() {
        init_logger!();
        let player_id = "my-player";
        let mut player = create_mock_player(player_id);
        player.expect_resume().times(1).return_const(());
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .player_manager()
            .add_player(Box::new(player))
            .unwrap();

        incoming
            .send(
                PlayerResumeRequest {
                    player_id: player_id.to_string(),
                    special_fields: Default::default(),
                },
                PlayerResumeRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_player_stop_request() {
        init_logger!();
        let player_id = "my-player";
        let mut player = create_mock_player(player_id);
        player.expect_stop().times(1).return_const(());
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .player_manager()
            .add_player(Box::new(player))
            .unwrap();

        incoming
            .send(
                PlayerStopRequest {
                    player_id: player_id.to_string(),
                    special_fields: Default::default(),
                },
                PlayerStopRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_player_seek_request() {
        init_logger!();
        let player_id = "my-player";
        let time = 20700;
        let (tx, rx) = oneshot::channel();
        let mut player = create_mock_player(player_id);
        player.expect_seek().times(1).return_once(move |time| {
            tx.send(time).unwrap();
            ()
        });
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlayerMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .player_manager()
            .add_player(Box::new(player))
            .unwrap();

        incoming
            .send(
                PlayerSeekRequest {
                    player_id: player_id.to_string(),
                    time,
                    special_fields: Default::default(),
                },
                PlayerSeekRequest::NAME,
            )
            .await
            .unwrap();
        let message = try_recv!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let result = try_recv!(rx, Duration::from_millis(250))
            .expect("expected the seek to have been invoked");
        assert_eq!(time, result);
    }

    #[tokio::test]
    async fn test_proto_player_state() {
        init_logger!();
        let player_id = "proto-player";
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = GetPlayerStateRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
                incoming
                    .send_reply(
                        &message,
                        GetPlayerStateResponse {
                            state: player::player::State::READY.into(),
                            special_fields: Default::default(),
                        },
                        GetPlayerStateResponse::NAME,
                    )
                    .await
                    .unwrap();
            }
        });

        let result = player.state().await;
        assert_eq!(PlayerState::Ready, result);

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
    }

    #[tokio::test]
    async fn test_proto_player_play() {
        init_logger!();
        let player_id = "proto-player";
        let play_request = PlayRequest::builder()
            .url("http://localhost:3000/my-video.mkv")
            .title("MyPlayRequest")
            .caption("Caption")
            .thumb("ThumbUrl.png")
            .background("BackgroundUrl.png")
            .quality("1080p")
            .subtitles_enabled(true)
            .build();
        let proto_play_request = player::player::PlayRequest::from(&play_request);
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = PlayerPlayRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
            }
        });

        player.play(play_request.clone()).await;

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
        assert_eq!(MessageField::some(proto_play_request), request.request);

        let request = player.request().await;
        assert_eq!(Some(play_request), request);
    }

    #[tokio::test]
    async fn test_proto_player_pause() {
        init_logger!();
        let player_id = "proto-player";
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = PlayerPauseRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
            }
        });

        player.pause().await;

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
    }

    #[tokio::test]
    async fn test_proto_player_resume() {
        init_logger!();
        let player_id = "proto-player";
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = PlayerResumeRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
            }
        });

        player.resume().await;

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
    }

    #[tokio::test]
    async fn test_proto_player_seek() {
        init_logger!();
        let time = 28000;
        let player_id = "proto-player";
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = PlayerSeekRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
            }
        });

        player.seek(time).await;

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
        assert_eq!(time, request.time);
    }

    #[tokio::test]
    async fn test_proto_player_stop() {
        init_logger!();
        let player_id = "proto-player";
        let (tx, rx) = oneshot::channel();
        let (incoming, outgoing) = create_channel_pair().await;
        let player = create_proto_player(player_id, outgoing.clone());

        tokio::spawn(async move {
            if let Some(message) = incoming.recv().await {
                let request = PlayerStopRequest::parse_from_bytes(&message.payload).unwrap();
                tx.send(request).unwrap();
            }
        });

        player.stop().await;

        let request = try_recv!(rx, Duration::from_millis(250)).unwrap();
        assert_eq!(player_id, request.player_id.as_str());
    }

    fn create_mock_player(id: &str) -> MockPlayer {
        let mut player = MockPlayer::new();
        player.expect_id().return_const(id.to_string());
        player.expect_name().return_const("MockPlayer".to_string());
        player
            .expect_description()
            .return_const("FooBar".to_string());
        player.expect_state().return_const(PlayerState::Unknown);
        player.expect_graphic_resource().return_const(Vec::new());
        player.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        player
    }

    fn create_proto_player(player_id: &str, channel: IpcChannel) -> ProtoPlayerWrapper {
        ProtoPlayerWrapper::new(
            player::Player {
                id: player_id.to_string(),
                name: "TestProtoPlayer".to_string(),
                description: "ProtoPlayerDescription".to_string(),
                graphic_resource: vec![],
                state: player::player::State::READY.into(),
                special_fields: Default::default(),
            },
            channel,
        )
    }
}
