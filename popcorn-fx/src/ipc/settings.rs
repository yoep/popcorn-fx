use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::Error;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::settings::{
    ApplicationSettings, ApplicationSettingsRequest, ApplicationSettingsResponse,
    UpdateTorrentSettingsRequest, UpdateUISettingsRequest,
};
use async_trait::async_trait;
use popcorn_fx_core::core::config::{TorrentSettings, UiSettings};
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct SettingsMessageHandler {
    instance: Arc<PopcornFX>,
}

impl SettingsMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for SettingsMessageHandler {
    fn name(&self) -> &str {
        "settings"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            ApplicationSettingsRequest::NAME
                | UpdateUISettingsRequest::NAME
                | UpdateTorrentSettingsRequest::NAME
        )
    }

    async fn process(
        &self,
        message: FxMessage,
        channel: &IpcChannel,
    ) -> crate::ipc::errors::Result<()> {
        match message.message_type() {
            ApplicationSettingsRequest::NAME => {
                let settings = self
                    .instance
                    .settings()
                    .user_settings_ref(|e| ApplicationSettings::from(e))
                    .await;

                channel
                    .send_reply(
                        &message,
                        ApplicationSettingsResponse {
                            settings: MessageField::some(settings),
                            special_fields: Default::default(),
                        },
                        ApplicationSettingsResponse::NAME,
                    )
                    .await?;
            }
            UpdateUISettingsRequest::NAME => {
                let mut request = UpdateUISettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = UiSettings::try_from(&proto_settings)?;
                self.instance.settings().update_ui(settings).await;
            }
            UpdateTorrentSettingsRequest::NAME => {
                let mut request = UpdateTorrentSettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = TorrentSettings::try_from(&proto_settings)?;
                self.instance.settings().update_torrent(settings).await;
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
