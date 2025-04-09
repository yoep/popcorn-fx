use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::Error;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::settings::{
    ApplicationSettings, ApplicationSettingsRequest, ApplicationSettingsResponse,
    UpdateTorrentSettingsRequest,
};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::config::TorrentSettings;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Settings message handler")]
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
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            ApplicationSettingsRequest::NAME | UpdateTorrentSettingsRequest::NAME
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
            UpdateTorrentSettingsRequest::NAME => {
                let mut request = UpdateTorrentSettingsRequest::parse_from_bytes(&message.payload)?;

                if let Some(settings) = request.settings.take() {
                    let settings = TorrentSettings::try_from(&settings)?;
                    self.instance.settings().update_torrent(settings).await;
                }
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
