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
