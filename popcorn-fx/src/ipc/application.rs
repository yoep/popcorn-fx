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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::ipc::FxMessageBuilder;
    use crate::tests::default_args;

    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_application_args_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ApplicationMessageHandler::new(Arc::new(instance));

        let request = ApplicationArgsRequest {
            special_fields: Default::default(),
        };

        let result = handler
            .process(
                FxMessageBuilder::new()
                    .type_(ApplicationArgsRequest::NAME)
                    .sequence_id(1)
                    .payload(request.write_to_bytes().unwrap())
                    .build(),
                &outgoing,
            )
            .await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = incoming.recv().await.unwrap();
        let result = ApplicationArgsResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            true, result.args.is_youtube_player_enabled,
            "expected the youtube player to have been enabled"
        );
        assert_eq!(
            true, result.args.is_fx_player_enabled,
            "expected the FX player to have been enabled"
        );
        assert_eq!(
            true, result.args.is_vlc_video_player_enabled,
            "expected the vlc player to have been enabled"
        );
    }
}
