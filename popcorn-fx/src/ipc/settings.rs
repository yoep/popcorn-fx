use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::Error;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::settings;
use crate::ipc::proto::settings::{
    ApplicationSettings, ApplicationSettingsRequest, ApplicationSettingsResponse,
    UpdatePlaybackSettingsRequest, UpdateServerSettingsRequest, UpdateSubtitleSettingsRequest,
    UpdateTorrentSettingsRequest, UpdateUISettingsRequest,
};
use async_trait::async_trait;
use fx_callback::Callback;
use log::error;
use popcorn_fx_core::core::config::{
    ApplicationConfigEvent, PlaybackSettings, ServerSettings, SubtitleSettings, TorrentSettings,
    UiSettings,
};
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct SettingsMessageHandler {
    instance: Arc<PopcornFX>,
}

impl SettingsMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.settings().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = Self::on_event(&event, &channel).await {
                    error!("Failed to send application settings event, {}", e);
                }
            }
        });
        Self { instance }
    }

    async fn on_event(
        event: &ApplicationConfigEvent,
        channel: &IpcChannel,
    ) -> crate::ipc::Result<()> {
        match &*event {
            ApplicationConfigEvent::UiSettingsChanged(settings) => {
                channel
                    .send(
                        settings::ApplicationSettingsEvent {
                            event: settings::application_settings_event::Event::UI_SETTINGS_CHANGED
                                .into(),
                            ui_settings: MessageField::some(settings.into()),
                            subtitle_settings: Default::default(),
                            torrent_settings: Default::default(),
                            server_settings: Default::default(),
                            playback_settings: Default::default(),
                            tracking_settings: Default::default(),
                            special_fields: Default::default(),
                        },
                        settings::ApplicationSettingsEvent::NAME,
                    )
                    .await?;
            }
            ApplicationConfigEvent::PlaybackSettingsChanged(settings) => {
                channel
                    .send(settings::ApplicationSettingsEvent {
                        event:
                        settings::application_settings_event::Event::PLAYBACK_SETTINGS_CHANGED
                            .into(),
                        ui_settings: Default::default(),
                        subtitle_settings: Default::default(),
                        torrent_settings: Default::default(),
                        server_settings: Default::default(),
                        playback_settings: MessageField::some(settings.into()),
                        tracking_settings: Default::default(),
                        special_fields: Default::default(),
                    }, settings::ApplicationSettingsEvent::NAME)
                    .await?;
            }
            ApplicationConfigEvent::SubtitleSettingsChanged(settings) => {
                channel
                    .send(settings::ApplicationSettingsEvent {
                        event:
                        settings::application_settings_event::Event::SUBTITLE_SETTINGS_CHANGED
                            .into(),
                        ui_settings: Default::default(),
                        subtitle_settings: MessageField::some(settings.into()),
                        torrent_settings: Default::default(),
                        server_settings: Default::default(),
                        playback_settings: Default::default(),
                        tracking_settings: Default::default(),
                        special_fields: Default::default(),
                    }, settings::ApplicationSettingsEvent::NAME)
                    .await?;
            }
            _ => {}
        }

        Ok(())
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
                | UpdateSubtitleSettingsRequest::NAME
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
            UpdateSubtitleSettingsRequest::NAME => {
                let mut request =
                    UpdateSubtitleSettingsRequest::parse_from_bytes(&message.payload)?;
                let proto_settings = request.settings.take().ok_or(Error::MissingField)?;

                let settings = SubtitleSettings::try_from(proto_settings)?;
                self.instance.settings().update_subtitle(settings).await;
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
        let handler = SettingsMessageHandler::new(instance, outgoing.clone());

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
        let handler = SettingsMessageHandler::new(instance, outgoing.clone());

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
        let handler = SettingsMessageHandler::new(instance, outgoing.clone());

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
        let handler = SettingsMessageHandler::new(instance, outgoing.clone());

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
        let handler = SettingsMessageHandler::new(instance, outgoing.clone());

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

    mod on_event {
        use super::*;
        use crate::ipc::proto::settings::application_settings;
        use crate::ipc::proto::subtitle;
        use popcorn_fx_core::core::config::{
            DecorationType, PlaybackSettings, Quality, SubtitleFamily, SubtitleSettings, UiScale,
            UiSettings,
        };
        use popcorn_fx_core::core::media::Category;
        use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
        use protobuf::EnumOrUnknown;

        #[tokio::test]
        async fn test_on_ui_settings_changed() {
            init_logger!();
            let default_language = "english";
            let ui_scale = 1.25f32;
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let _handler = SettingsMessageHandler::new(instance.clone(), outgoing.clone());

            // update the settings
            instance
                .settings()
                .update_ui(UiSettings {
                    default_language: default_language.to_string(),
                    ui_scale: UiScale::new(ui_scale).unwrap(),
                    start_screen: Category::Series,
                    maximized: false,
                    native_window_enabled: false,
                })
                .await;

            // wait for the event to be received
            let message =
                timeout!(incoming.recv(), Duration::from_millis(750)).expect("expected a message");
            assert_eq!(
                settings::ApplicationSettingsEvent::NAME,
                message.type_.as_str()
            );

            // parse the message
            let result =
                settings::ApplicationSettingsEvent::parse_from_bytes(message.payload.as_slice())
                    .unwrap();
            assert_eq!(
                EnumOrUnknown::from(
                    settings::application_settings_event::Event::UI_SETTINGS_CHANGED
                ),
                result.event
            );

            // validate the settings
            let result = result
                .ui_settings
                .into_option()
                .expect("expected the ui settings to be present");
            assert_eq!(
                default_language,
                result.default_language.as_str(),
                "expected the default language to match"
            );
            assert_eq!(
                MessageField::some(settings::application_settings::uisettings::Scale {
                    factor: ui_scale,
                    special_fields: Default::default(),
                }),
                result.scale,
                "expected the scale to match"
            );
        }

        #[tokio::test]
        async fn test_on_playback_settings_changed() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let _handler = SettingsMessageHandler::new(instance.clone(), outgoing.clone());

            // update the settings
            instance
                .settings()
                .update_playback(PlaybackSettings {
                    quality: Some(Quality::P1080),
                    fullscreen: true,
                    auto_play_next_episode_enabled: true,
                })
                .await;

            // wait for the event to be received
            let message =
                timeout!(incoming.recv(), Duration::from_millis(750)).expect("expected a message");
            assert_eq!(
                settings::ApplicationSettingsEvent::NAME,
                message.type_.as_str()
            );

            // parse the message
            let result =
                settings::ApplicationSettingsEvent::parse_from_bytes(message.payload.as_slice())
                    .unwrap();
            assert_eq!(
                EnumOrUnknown::from(
                    settings::application_settings_event::Event::PLAYBACK_SETTINGS_CHANGED
                ),
                result.event
            );

            // validate the settings
            let result = result
                .playback_settings
                .into_option()
                .expect("expected the playback settings to be present");
            assert_eq!(
                Some(EnumOrUnknown::from(
                    settings::application_settings::playback_settings::Quality::P1080
                )),
                result.quality,
                "expected the quality to match"
            );
            assert_eq!(
                true, result.fullscreen,
                "expected the fullscreen setting to be enabled"
            );
        }

        #[tokio::test]
        async fn test_on_subtitle_settings_changed() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let _handler = SettingsMessageHandler::new(instance.clone(), outgoing.clone());

            // update the settings
            instance
                .settings()
                .update_subtitle(SubtitleSettings {
                    directory: "".to_string(),
                    auto_cleaning_enabled: true,
                    default_subtitle: SubtitleLanguage::German,
                    font_family: SubtitleFamily::Arial,
                    font_size: 18,
                    decoration: DecorationType::Outline,
                    bold: true,
                })
                .await;

            // wait for the event to be received
            let message =
                timeout!(incoming.recv(), Duration::from_millis(750)).expect("expected a message");
            assert_eq!(
                settings::ApplicationSettingsEvent::NAME,
                message.type_.as_str()
            );

            // parse the message
            let result =
                settings::ApplicationSettingsEvent::parse_from_bytes(message.payload.as_slice())
                    .unwrap();
            assert_eq!(
                EnumOrUnknown::from(
                    settings::application_settings_event::Event::SUBTITLE_SETTINGS_CHANGED
                ),
                result.event
            );

            // validate the settings
            let result = result
                .subtitle_settings
                .into_option()
                .expect("expected the subtitle settings to be present");
            assert_eq!(
                true, result.auto_cleaning_enabled,
                "expected the auto cleaning setting to be enabled"
            );
            assert_eq!(
                EnumOrUnknown::from(subtitle::subtitle::Language::GERMAN),
                result.default_subtitle,
                "expected the default subtitle language to be German"
            );
            assert_eq!(
                EnumOrUnknown::from(application_settings::subtitle_settings::Family::ARIAL),
                result.font_family,
                "expected the font family to be Arial"
            );
            assert_eq!(18, result.font_size, "expected the font size to be 18");
            assert_eq!(
                EnumOrUnknown::from(
                    application_settings::subtitle_settings::DecorationType::OUTLINE
                ),
                result.decoration,
                "expected the decoration to be Outline"
            );
        }
    }
}
