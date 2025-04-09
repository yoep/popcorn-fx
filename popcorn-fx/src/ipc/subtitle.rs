use crate::fx::PopcornFX;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::subtitle::{
    subtitle, GetSubtitleCustomRequest, GetSubtitleCustomResponse, GetSubtitleNoneRequest,
    GetSubtitleNoneResponse, GetSubtitlePreferenceRequest, GetSubtitlePreferenceResponse,
};
use crate::ipc::{proto, Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Subtitle message handler")]
pub struct SubtitleMessageHandler {
    instance: Arc<PopcornFX>,
}

impl SubtitleMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for SubtitleMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetSubtitleNoneRequest::NAME
                | GetSubtitleCustomRequest::NAME
                | GetSubtitlePreferenceRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetSubtitleNoneRequest::NAME => {
                let mut response = GetSubtitleNoneResponse::new();
                response.info = MessageField::some(subtitle::Info::from(&SubtitleInfo::none()));

                channel
                    .send_reply(&message, response, GetSubtitleNoneResponse::NAME)
                    .await?;
            }
            GetSubtitleCustomRequest::NAME => {
                let mut response = GetSubtitleCustomResponse::new();
                response.info = MessageField::some(subtitle::Info::from(&SubtitleInfo::custom()));

                channel
                    .send_reply(&message, response, GetSubtitleCustomResponse::NAME)
                    .await?;
            }
            GetSubtitlePreferenceRequest::NAME => {
                let mut response = GetSubtitlePreferenceResponse::new();
                let preference = self.instance.subtitle_manager().preference().await;
                response.preference =
                    MessageField::some(proto::subtitle::SubtitlePreference::from(&preference));

                channel
                    .send_reply(&message, response, GetSubtitlePreferenceResponse::NAME)
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
