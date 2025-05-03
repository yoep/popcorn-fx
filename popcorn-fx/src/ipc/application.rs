use crate::fx::PopcornFX;
use crate::ipc::proto::application::{
    ApplicationArgs, ApplicationArgsRequest, ApplicationArgsResponse, GetApplicationVersionRequest,
    GetApplicationVersionResponse,
};
use crate::ipc::proto::message::FxMessage;
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use popcorn_fx_core::VERSION;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
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
    fn name(&self) -> &str {
        "application"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            ApplicationArgsRequest::NAME | GetApplicationVersionRequest::NAME
        )
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
            GetApplicationVersionRequest::NAME => {
                channel
                    .send_reply(
                        &message,
                        GetApplicationVersionResponse {
                            version: VERSION.to_string(),
                            special_fields: Default::default(),
                        },
                        GetApplicationVersionResponse::NAME,
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
    use std::time::Duration;

    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;

    use crate::try_recv;
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

        let response = incoming
            .get(ApplicationArgsRequest::new(), ApplicationArgsRequest::NAME)
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

    #[tokio::test]
    async fn test_process_get_application_version() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = ApplicationMessageHandler::new(instance);

        let response = incoming
            .get(
                GetApplicationVersionRequest::new(),
                GetApplicationVersionRequest::NAME,
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
        let result = GetApplicationVersionResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(VERSION, result.version.as_str());
    }
}
