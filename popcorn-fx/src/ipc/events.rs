use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::events;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::Error;
use async_trait::async_trait;
use derive_more::Display;
use log::error;
use popcorn_fx_core::core::event::{Event, LOWEST_ORDER};
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Event message handler")]
pub struct EventMessageHandler {
    instance: Arc<PopcornFX>,
}

impl EventMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: &IpcChannel) -> Self {
        let mut receiver = instance
            .event_publisher()
            .subscribe(LOWEST_ORDER)
            .expect("expected a subscription");

        let channel = channel.clone();
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
