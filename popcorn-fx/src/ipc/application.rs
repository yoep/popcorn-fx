use crate::fx::PopcornFX;
use crate::ipc::proto::application::{
    ApplicationArgs, ApplicationArgsRequest, ApplicationArgsResponse,
};
use crate::ipc::proto::message::FxMessage;
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use derive_more::Display;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Application message handler")]
pub struct ApplicationMessageHandler {
    instance: Arc<PopcornFX>,
}

impl ApplicationMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for ApplicationMessageHandler {
    fn is_supported(&self, message_type: &str) -> bool {
        message_type == ApplicationArgsRequest::NAME
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            ApplicationArgsRequest::NAME => {
                let opts = self.instance.opts();
                channel
                    .send_reply(
                        &message,
                        ApplicationArgsResponse {
                            args: MessageField::some(ApplicationArgs {
                                is_tv_mode: opts.tv,
                                is_maximized: opts.maximized,
                                is_kiosk_mode: opts.kiosk,
                                is_mouse_disabled: opts.disable_mouse,
                                is_youtube_player_enabled: opts.enable_youtube_video_player,
                                is_vlc_video_player_enabled: opts.enable_vlc_video_player,
                                is_fx_player_enabled: opts.enable_fx_video_player,
                                special_fields: Default::default(),
                            }),
                            special_fields: Default::default(),
                        },
                        ApplicationArgsResponse::NAME,
                    )
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
