use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::events;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::Error;
use async_trait::async_trait;
use log::error;
use popcorn_fx_core::core::event::{Event, LOWEST_ORDER};
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug)]
pub struct EventMessageHandler {
    instance: Arc<PopcornFX>,
}

impl EventMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance
            .event_publisher()
            .subscribe(LOWEST_ORDER)
            .expect("expected a subscription");
        tokio::spawn(async move {
            while let Some(mut handler) = receiver.recv().await {
                if let Some(event) = handler.take() {
                    if let Err(e) = channel
                        .send(events::Event::from(&event), events::Event::NAME)
                        .await
                    {
                        error!("Event bridge failed to send message, {}", e)
                    }
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for EventMessageHandler {
    fn name(&self) -> &str {
        "event"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        message_type == events::Event::NAME
    }

    async fn process(&self, message: FxMessage, _: &IpcChannel) -> crate::ipc::errors::Result<()> {
        match message.message_type() {
            events::Event::NAME => {
                let request = events::Event::parse_from_bytes(&message.payload)?;
                let event = Event::try_from(&request)?;

                self.instance.event_publisher().publish(event);
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

    use crate::ipc::proto::player::player;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::core::event::HIGHEST_ORDER;
    use popcorn_fx_core::core::playback::PlaybackState;
    use popcorn_fx_core::init_logger;
    use protobuf::{EnumOrUnknown, MessageField};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process_event() {
        init_logger!();
        let (tx, mut rx) = unbounded_channel();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handle = EventMessageHandler::new(Arc::new(instance), outgoing.clone());

        // listen to events published by the event publisher
        let mut receiver = handle
            .instance
            .event_publisher()
            .subscribe(HIGHEST_ORDER)
            .unwrap();
        tokio::spawn(async move {
            while let Some(mut event_handler) = receiver.recv().await {
                if let Some(event) = event_handler.take() {
                    tx.send(event).unwrap();
                }
            }
        });

        // process the close player event
        incoming
            .send(
                events::Event {
                    type_: events::event::EventType::PLAYBACK_STATE_CHANGED.into(),
                    playback_state_changed: MessageField::some(
                        events::event::PlaybackStateChanged {
                            new_state: player::State::PLAYING.into(),
                            special_fields: Default::default(),
                        },
                    ),
                    torrent_details_loaded: Default::default(),
                    special_fields: Default::default(),
                },
                events::Event::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handle.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been processed"
        );

        let event = timeout!(rx.recv(), Duration::from_millis(250))
            .expect("expected to have received an event");
        assert_eq!(Event::PlaybackStateChanged(PlaybackState::PLAYING), event);
    }

    #[tokio::test]
    async fn test_on_event() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let _handle = EventMessageHandler::new(instance.clone(), outgoing.clone());

        instance
            .event_publisher()
            .publish(Event::PlaybackStateChanged(PlaybackState::PLAYING));

        let message =
            timeout!(incoming.recv(), Duration::from_millis(750)).expect("expected a message");
        assert_eq!(events::Event::NAME, message.type_.as_str());

        let event = events::Event::parse_from_bytes(&message.payload).unwrap();
        assert_eq!(
            Into::<EnumOrUnknown<events::event::EventType>>::into(
                events::event::EventType::PLAYBACK_STATE_CHANGED
            ),
            event.type_
        );
        assert_eq!(
            Into::<EnumOrUnknown<player::State>>::into(player::State::PLAYING),
            event.playback_state_changed.new_state
        );
    }
}
