use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::stream;
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use fx_callback::Callback;
use log::{trace, warn};
use popcorn_fx_core::core::stream::{StreamEvent, StreamServerEvent};
use protobuf::{EnumOrUnknown, Message, MessageField};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StreamMessageHandler {
    instance: Arc<PopcornFX>,
    channel: IpcChannel,
}

impl StreamMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.stream_server().subscribe();
        let instance = Self { instance, channel };

        let handler = instance.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                handler.handle_event(&*event).await;
            }
        });

        instance
    }

    async fn handle_event(&self, event: &StreamServerEvent) {
        match event {
            StreamServerEvent::StreamStarted(stream) => {
                let filename = stream.filename().to_string();
                let mut receiver = match self
                    .instance
                    .stream_server()
                    .subscribe_stream(filename.as_str())
                    .await
                {
                    Err(_) => return,
                    Ok(e) => e,
                };

                let channel = self.channel.clone();
                tokio::spawn(async move {
                    while let Some(event) = receiver.recv().await {
                        Self::handle_stream_event(filename.as_str(), &*event, &channel).await;
                    }
                });
            }
            StreamServerEvent::StreamStopped(filename) => {
                trace!("Stream {} has been stopped", filename)
            }
        }
    }

    async fn handle_stream_event(filename: &str, event: &StreamEvent, channel: &IpcChannel) {
        let mut proto_event = stream::stream::StreamEvent::from(event);
        proto_event.filename = filename.to_string();

        if let Err(e) = channel
            .send(proto_event, stream::stream::StreamEvent::NAME)
            .await
        {
            warn!("Failed to send stream event to channel, {}", e);
        }
    }
}

#[async_trait]
impl MessageHandler for StreamMessageHandler {
    fn name(&self) -> &str {
        "stream"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(message_type, stream::StreamStateRequest::NAME)
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            stream::StreamStateRequest::NAME => {
                let request = stream::StreamStateRequest::parse_from_bytes(&message.payload)?;
                let response = match self
                    .instance
                    .stream_server()
                    .state(request.filename.as_str())
                    .await
                {
                    Ok(state) => stream::StreamStateResponse {
                        result: response::Result::OK.into(),
                        state: Some(EnumOrUnknown::from(stream::stream::StreamState::from(
                            state,
                        ))),
                        error: MessageField::none(),
                        special_fields: Default::default(),
                    },
                    Err(e) => stream::StreamStateResponse {
                        result: response::Result::ERROR.into(),
                        state: None,
                        error: MessageField::some(stream::stream::Error::from(e)),
                        special_fields: Default::default(),
                    },
                };

                channel
                    .send_reply(&message, response, stream::StreamStateResponse::NAME)
                    .await?;
            }
            _ => {
                return Err(Error::UnsupportedMessage(
                    message.message_type().to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;
    use popcorn_fx_core::core::stream::tests::MockStreamingResource;
    use popcorn_fx_core::core::stream::StreamState;
    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    mod stream_state {
        use super::*;

        #[tokio::test]
        async fn test_stream_state_request() {
            init_logger!();
            let filename = "lorem.mp4";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let handler = StreamMessageHandler::new(instance.clone(), outgoing.clone());

            // start a new stream
            let mut resource = MockStreamingResource::new();
            resource
                .expect_filename()
                .return_const(filename.to_string());
            resource.expect_state().return_const(StreamState::Streaming);
            instance
                .stream_server()
                .start_stream(resource)
                .await
                .expect("expected the stream to have started");

            // request the state of the stream
            let response = incoming
                .get(
                    stream::StreamStateRequest {
                        filename: filename.to_string(),
                        special_fields: Default::default(),
                    },
                    stream::StreamStateRequest::NAME,
                )
                .await
                .unwrap();
            let message = timeout!(outgoing.recv(), Duration::from_millis(250))
                .expect("expected to have received an incoming message");

            // handle the request message
            let result = handler.process(message, &outgoing).await;
            assert_eq!(Ok(()), result);

            // validate the response
            let response = timeout!(response, Duration::from_millis(250))
                .expect("expected to have received a reply");
            let result = stream::StreamStateResponse::parse_from_bytes(&response.payload).unwrap();
            assert_eq!(EnumOrUnknown::from(response::Result::OK), result.result);
            assert_eq!(
                Some(EnumOrUnknown::from(stream::stream::StreamState::STREAMING)),
                result.state
            );
        }

        #[tokio::test]
        async fn test_stream_not_found() {
            init_logger!();
            let filename = "bar.mp4";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let handler = StreamMessageHandler::new(instance.clone(), outgoing.clone());

            // request the state of the stream
            let response = incoming
                .get(
                    stream::StreamStateRequest {
                        filename: filename.to_string(),
                        special_fields: Default::default(),
                    },
                    stream::StreamStateRequest::NAME,
                )
                .await
                .unwrap();
            let message = timeout!(outgoing.recv(), Duration::from_millis(250))
                .expect("expected to have received an incoming message");

            // handle the request message
            let result = handler.process(message, &outgoing).await;
            assert_eq!(Ok(()), result);

            // validate the response
            let response = timeout!(response, Duration::from_millis(250))
                .expect("expected to have received a reply");
            let result = stream::StreamStateResponse::parse_from_bytes(&response.payload).unwrap();
            assert_eq!(EnumOrUnknown::from(response::Result::ERROR), result.result);
            assert_eq!(
                EnumOrUnknown::from(stream::stream::error::Type::NOT_FOUND),
                result.error.type_,
            );
        }
    }

    mod events {
        use super::*;
        use popcorn_fx_core::recv_timeout;

        #[tokio::test]
        async fn test_stream_event() {
            init_logger!();
            let filename = "ipsum.mkv";
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (tx, mut rx) = unbounded_channel();
            let (sender, receiver) = unbounded_channel();
            let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
            let (incoming, outgoing) = create_channel_pair().await;
            let _handler = StreamMessageHandler::new(instance.clone(), outgoing.clone());

            // start a new stream
            let mut resource = MockStreamingResource::new();
            resource
                .expect_filename()
                .return_const(filename.to_string());
            resource.expect_state().return_const(StreamState::Streaming);
            resource.expect_subscribe().return_once(move || {
                let _ = tx.send(());
                receiver
            });
            instance
                .stream_server()
                .start_stream(resource)
                .await
                .expect("expected the stream to have started");

            // wait for the handler to subscribe to the stream
            recv_timeout!(
                &mut rx,
                Duration::from_millis(250),
                "expected the handler to subscribe to the stream"
            );

            // invoke a new event
            sender
                .send(Arc::new(StreamEvent::StateChanged(StreamState::Stopped)))
                .unwrap();

            // wait for the event to be received by the channel
            let message = timeout!(incoming.recv(), Duration::from_millis(250))
                .expect("expected to have received an incoming message");
            let result = stream::stream::StreamEvent::parse_from_bytes(&message.payload).unwrap();
            assert_eq!(
                EnumOrUnknown::from(stream::stream::stream_event::Type::STATE_CHANGED),
                result.type_
            );
            assert_eq!(
                Some(EnumOrUnknown::from(stream::stream::StreamState::STOPPED)),
                result.state
            );
        }
    }
}
