use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::tracking;
use crate::ipc::proto::tracking::{
    tracking_provider, GetTrackingProviderIsAuthorizedRequest,
    GetTrackingProviderIsAuthorizedResponse, TrackingProviderAuthorizeRequest,
    TrackingProviderAuthorizeResponse, TrackingProviderDisconnectRequest, TrackingProviderEvent,
};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use log::error;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct TrackingMessageHandler {
    instance: Arc<PopcornFX>,
}

impl TrackingMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.tracking_provider().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let proto_event = tracking::TrackingProviderEvent::from(&*event);
                if let Err(e) = channel.send(proto_event, TrackingProviderEvent::NAME).await {
                    error!(
                        "Tracking provider event couldn't be sent to the channel, {}",
                        e
                    );
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for TrackingMessageHandler {
    fn name(&self) -> &str {
        "tracking"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetTrackingProviderIsAuthorizedRequest::NAME
                | TrackingProviderAuthorizeRequest::NAME
                | TrackingProviderDisconnectRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetTrackingProviderIsAuthorizedRequest::NAME => {
                let _request =
                    GetTrackingProviderIsAuthorizedRequest::parse_from_bytes(&message.payload)?;
                let is_authorized = self.instance.tracking_provider().is_authorized().await;

                channel
                    .send_reply(
                        &message,
                        GetTrackingProviderIsAuthorizedResponse {
                            is_authorized,
                            special_fields: Default::default(),
                        },
                        GetTrackingProviderIsAuthorizedResponse::NAME,
                    )
                    .await?;
            }
            TrackingProviderAuthorizeRequest::NAME => {
                let _request =
                    TrackingProviderAuthorizeRequest::parse_from_bytes(&message.payload)?;
                let response: TrackingProviderAuthorizeResponse;

                match self.instance.tracking_provider().authorize().await {
                    Ok(_) => {
                        response = TrackingProviderAuthorizeResponse {
                            result: response::Result::OK.into(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(err) => {
                        response = TrackingProviderAuthorizeResponse {
                            result: response::Result::ERROR.into(),
                            error: MessageField::some(tracking_provider::AuthorizationError::from(
                                &err,
                            )),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, TrackingProviderAuthorizeResponse::NAME)
                    .await?;
            }
            TrackingProviderDisconnectRequest::NAME => {
                let _request =
                    TrackingProviderDisconnectRequest::parse_from_bytes(&message.payload)?;

                self.instance.tracking_provider().disconnect().await;
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

    use crate::ipc::proto::message::response;
    use crate::ipc::proto::tracking::{tracking_provider_event, TrackingProviderAuthorizeResponse};
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::core::media::tracking::TrackingEvent;
    use popcorn_fx_core::init_logger;
    use protobuf::MessageField;
    use reqwest::Client;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::time::Duration;
    use tempfile::tempdir;
    use url::Url;

    #[tokio::test]
    async fn test_tracking_event() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let _handler = TrackingMessageHandler::new(instance.clone(), outgoing.clone());

        tokio::spawn(async move {
            let _ = instance.tracking_provider().authorize().await;
        });

        let message = timeout!(incoming.recv(), Duration::from_millis(250))
            .expect("expected to have received an event message");
        let result = TrackingProviderEvent::parse_from_bytes(&message.payload).unwrap();
        assert_eq!(
            tracking_provider_event::Event::OPEN_AUTHORIZATION_URI,
            result.event.unwrap(),
        );
        assert_ne!(MessageField::none(), result.open_authorization_uri);
    }

    #[tokio::test]
    async fn test_is_supported() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (_incoming, outgoing) = create_channel_pair().await;
        let handler = TrackingMessageHandler::new(instance.clone(), outgoing.clone());

        assert_eq!(
            true,
            handler.is_supported(GetTrackingProviderIsAuthorizedRequest::NAME)
        );
        assert_eq!(
            true,
            handler.is_supported(TrackingProviderAuthorizeRequest::NAME)
        );
        assert_eq!(
            true,
            handler.is_supported(TrackingProviderDisconnectRequest::NAME)
        );
    }

    #[tokio::test]
    async fn test_process_get_is_authorized_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TrackingMessageHandler::new(instance.clone(), outgoing.clone());

        let response = incoming
            .get(
                GetTrackingProviderIsAuthorizedRequest::new(),
                GetTrackingProviderIsAuthorizedRequest::NAME,
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
        let result =
            GetTrackingProviderIsAuthorizedResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(false, result.is_authorized);
    }

    #[tokio::test]
    async fn test_process_authorize_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TrackingMessageHandler::new(instance.clone(), outgoing.clone());

        let mut receiver = instance.tracking_provider().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let TrackingEvent::OpenAuthorization(url) = &*event {
                    let client = Client::new();
                    let query_params: HashMap<String, String> = url
                        .query_pairs()
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();

                    if let Some(state) = query_params.get("state") {
                        if let Some(redirect_uri) = query_params.get("redirect_uri") {
                            let uri = Url::from_str(redirect_uri)
                                .unwrap()
                                .query_pairs_mut()
                                .append_pair("code", "SomeCode")
                                .append_pair("state", state.as_ref())
                                .finish()
                                .to_string();

                            if let Err(e) = client.get(uri).send().await {
                                assert!(false, "expected the callback to have succeeded, {}", e)
                            }
                        } else {
                            assert!(false, "failed to find redirect_uri parameter");
                        }
                    } else {
                        assert!(false, "failed to find state parameter");
                    }
                }
            }
        });

        let response = incoming
            .get(
                TrackingProviderAuthorizeRequest {
                    tracking_provider_id: "trakt".to_string(),
                    special_fields: Default::default(),
                },
                TrackingProviderAuthorizeRequest::NAME,
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
        let result =
            TrackingProviderAuthorizeResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(response::Result::ERROR, result.result.unwrap());
        assert_ne!(
            MessageField::none(),
            result.error,
            "expected an error to have been returned"
        );
        assert_eq!(
            tracking_provider::authorization_error::Type::TOKEN,
            result.error.type_.unwrap()
        );
    }

    #[tokio::test]
    async fn test_process_disconnect_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = TrackingMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(
                TrackingProviderDisconnectRequest {
                    tracking_provider_id: "trakt".to_string(),
                    special_fields: Default::default(),
                },
                TrackingProviderDisconnectRequest::NAME,
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
