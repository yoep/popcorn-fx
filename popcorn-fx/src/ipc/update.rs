use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::update::{
    update, GetUpdateInfoRequest, GetUpdateInfoResponse, GetUpdateStateRequest,
    GetUpdateStateResponse, RefreshUpdateInfoRequest, StartUpdateDownloadRequest,
    StartUpdateDownloadResponse, StartUpdateInstallationRequest, StartUpdateInstallationResponse,
    UpdateEvent,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use fx_callback::Callback;
use log::error;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct UpdateMessageHandler {
    instance: Arc<PopcornFX>,
}

impl UpdateMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.updater().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let proto_event = UpdateEvent::from(&*event);
                if let Err(e) = channel.send(proto_event, UpdateEvent::NAME).await {
                    error!("Failed to send update event to channel, {}", e);
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for UpdateMessageHandler {
    fn name(&self) -> &str {
        "update"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetUpdateStateRequest::NAME
                | GetUpdateInfoRequest::NAME
                | RefreshUpdateInfoRequest::NAME
                | StartUpdateDownloadRequest::NAME
                | StartUpdateInstallationRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetUpdateStateRequest::NAME => {
                let state = update::State::from(&self.instance.updater().state().await);

                channel
                    .send_reply(
                        &message,
                        GetUpdateStateResponse {
                            state: state.into(),
                            special_fields: Default::default(),
                        },
                        GetUpdateStateResponse::NAME,
                    )
                    .await?;
            }
            GetUpdateInfoRequest::NAME => {
                let response: GetUpdateInfoResponse;

                match self.instance.updater().version_info().await {
                    Ok(info) => {
                        let proto_info = update::VersionInfo::from(&info);

                        response = GetUpdateInfoResponse {
                            result: response::Result::OK.into(),
                            info: MessageField::some(proto_info),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = GetUpdateInfoResponse {
                            result: response::Result::ERROR.into(),
                            info: Default::default(),
                            error: MessageField::some(update::Error::from(&e)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetUpdateInfoResponse::NAME)
                    .await?;
            }
            RefreshUpdateInfoRequest::NAME => {
                self.instance.updater().check_for_updates().await;
            }
            StartUpdateDownloadRequest::NAME => {
                let response: StartUpdateDownloadResponse;

                match self.instance.updater().download().await {
                    Ok(_) => {
                        response = StartUpdateDownloadResponse {
                            result: response::Result::OK.into(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = StartUpdateDownloadResponse {
                            result: response::Result::ERROR.into(),
                            error: MessageField::some(update::Error::from(&e)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, StartUpdateDownloadResponse::NAME)
                    .await?;
            }
            StartUpdateInstallationRequest::NAME => {
                let response: StartUpdateInstallationResponse;

                match self.instance.updater().install().await {
                    Ok(_) => {
                        response = StartUpdateInstallationResponse {
                            result: response::Result::OK.into(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = StartUpdateInstallationResponse {
                            result: response::Result::ERROR.into(),
                            error: MessageField::some(update::Error::from(&e)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, StartUpdateInstallationResponse::NAME)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::try_recv;

    use popcorn_fx_core::init_logger;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_update_state_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = UpdateMessageHandler::new(Arc::clone(&instance), outgoing.clone());

        let response = incoming
            .get(GetUpdateStateRequest::new(), GetUpdateStateRequest::NAME)
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
        let result = GetUpdateStateResponse::parse_from_bytes(&response.payload).unwrap();

        assert!(
            result.state.enum_value().is_ok(),
            "expected a valid state to have been returned"
        );
    }

    #[tokio::test]
    async fn test_process_get_update_info_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = UpdateMessageHandler::new(Arc::clone(&instance), outgoing.clone());

        let response = incoming
            .get(GetUpdateInfoRequest::new(), GetUpdateInfoRequest::NAME)
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
        let result = GetUpdateInfoResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            Into::<EnumOrUnknown<response::Result>>::into(response::Result::OK),
            result.result,
            "expected the info to have been returned"
        );
    }

    #[tokio::test]
    async fn test_process_refresh_update_info_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = UpdateMessageHandler::new(Arc::clone(&instance), outgoing.clone());

        incoming
            .send(
                RefreshUpdateInfoRequest::new(),
                RefreshUpdateInfoRequest::NAME,
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
    async fn test_process_start_update_download_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = UpdateMessageHandler::new(Arc::clone(&instance), outgoing.clone());

        let response = incoming
            .get(
                StartUpdateDownloadRequest::new(),
                StartUpdateDownloadRequest::NAME,
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

        let _ = StartUpdateDownloadResponse::parse_from_bytes(&response.payload)
            .expect("expected a valid response");
    }

    #[tokio::test]
    async fn test_process_start_update_installation_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = UpdateMessageHandler::new(Arc::clone(&instance), outgoing.clone());

        let response = incoming
            .get(
                StartUpdateInstallationRequest::new(),
                StartUpdateInstallationRequest::NAME,
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

        let _ = StartUpdateInstallationResponse::parse_from_bytes(&response.payload)
            .expect("expected a valid response");
    }
}
