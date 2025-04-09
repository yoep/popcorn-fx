use crate::fx::PopcornFX;
use crate::ipc::channel::IpcChannel;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::events::Event;
use crate::ipc::proto::message::FxMessage;
use async_trait::async_trait;
use derive_more::Display;
use log::error;
use popcorn_fx_core::core::event::LOWEST_ORDER;
use protobuf::Message;
use std::sync::Arc;

#[derive(Debug, Display)]
#[display(fmt = "Event message handler")]
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
                    if let Err(e) = channel.send(Event::from(&event), Event::NAME).await {
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
        message_type == Event::NAME
    }

    async fn process(
        &self,
        message: FxMessage,
        channel: &IpcChannel,
    ) -> crate::ipc::errors::Result<()> {
        Ok(())
    }
}
