use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::Error;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::settings::{
    ApplicationSettings, ApplicationSettingsRequest, ApplicationSettingsResponse,
    UpdatePlaybackSettingsRequest, UpdateServerSettingsRequest, UpdateTorrentSettingsRequest,
    UpdateUISettingsRequest,
};
use async_trait::async_trait;
use popcorn_fx_core::core::config::{
    PlaybackSettings, ServerSettings, TorrentSettings, UiSettings,
};
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
                | UpdateServerSettingsRequest::NAME
                | UpdateTorrentSettingsRequest::NAME
                | UpdatePlaybackSettingsRequest::NAME
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
            UpdateServerSettingsRequest::NAME => {
                let mut request = UpdateServerSettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = ServerSettings::try_from(proto_settings)?;
                self.instance.settings().update_server(settings).await;
            }
            UpdateTorrentSettingsRequest::NAME => {
                let mut request = UpdateTorrentSettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = TorrentSettings::try_from(&proto_settings)?;
                self.instance.settings().update_torrent(settings).await;
            }
            UpdatePlaybackSettingsRequest::NAME => {
                let mut request =
                    UpdatePlaybackSettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = PlaybackSettings::try_from(proto_settings)?;
                self.instance.settings().update_playback(settings).await;
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

    use crate::ipc::proto::media::media;
    use crate::ipc::proto::settings::application_settings;
    use crate::ipc::proto::settings::application_settings::torrent_settings;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_application_settings_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SettingsMessageHandler::new(instance);

        let response = incoming
            .get(
                ApplicationSettingsRequest {
                    special_fields: Default::default(),
                },
                ApplicationSettingsRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = ApplicationSettingsResponse::parse_from_bytes(&response.payload).unwrap();
        assert_ne!(MessageField::none(), result.settings);
    }

    #[tokio::test]
    async fn test_process_update_ui_settings() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SettingsMessageHandler::new(instance);

        incoming
            .send(
                UpdateUISettingsRequest {
                    settings: MessageField::some(application_settings::UISettings {
                        default_language: "en".to_string(),
                        scale: Default::default(),
                        start_screen: media::Category::SERIES.into(),
                        maximized: true,
                        native_window_enabled: false,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdateUISettingsRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_update_server_settings() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SettingsMessageHandler::new(instance);

        incoming
            .send(
                UpdateServerSettingsRequest {
                    settings: MessageField::some(application_settings::ServerSettings {
                        movie_api_servers: vec!["https://api-v2.com".to_string()],
                        serie_api_servers: vec![],
                        update_api_servers_automatically: false,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdateServerSettingsRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_update_torrent_settings() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SettingsMessageHandler::new(instance);

        incoming
            .send(
                UpdateTorrentSettingsRequest {
                    settings: MessageField::some(application_settings::TorrentSettings {
                        directory: "".to_string(),
                        cleaning_mode: torrent_settings::CleaningMode::ON_SHUTDOWN.into(),
                        connections_limit: 200,
                        download_rate_limit: 1024,
                        upload_rate_limit: 0,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdateTorrentSettingsRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_update_playback_settings() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SettingsMessageHandler::new(instance);

        incoming
            .send(
                UpdatePlaybackSettingsRequest {
                    settings: MessageField::some(application_settings::PlaybackSettings {
                        quality: Some(
                            application_settings::playback_settings::Quality::P2160.into(),
                        ),
                        fullscreen: false,
                        auto_play_next_episode_enabled: true,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdatePlaybackSettingsRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }
}
