use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::{Error, Result};
use crate::ipc::proto::application::ApplicationTerminationRequest;
use crate::ipc::proto::message::FxMessage;
use async_trait::async_trait;
use log::{debug, error, warn};
use protobuf::{Enum, EnumOrUnknown, Message};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

/// A message handler is able to process a receiver [FxMessage].
#[async_trait]
pub trait MessageHandler: Debug + Send + Sync {
    /// Get the unique name of the message handler.
    fn name(&self) -> &str;

    /// Check if this handler is able to process the given message type.
    fn is_supported(&self, message_type: &str) -> bool;

    /// Process the given support message.
    ///
    /// # Arguments
    ///
    /// * `message` - The received message.
    /// * `channel` - The channel on which the message was received.
    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> Result<()>;
}

#[derive(Debug)]
pub struct IpcChannelProcessor {
    inner: Arc<InnerProcessor>,
}

impl IpcChannelProcessor {
    pub fn new(channel: IpcChannel, handlers: Vec<Box<dyn MessageHandler>>) -> Self {
        let inner = Arc::new(InnerProcessor {
            channel,
            handlers,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(&inner_main).await;
        });

        Self { inner }
    }

    /// Get a future which resolves when the processor is stopped.
    /// The future might immediately resolve if the processor has already stopped.
    pub fn stopped(&self) -> WaitForCancellationFuture<'_> {
        self.inner.cancellation_token.cancelled()
    }

    /// Stop the processor from processing any new messages.
    pub fn stop(&self) {
        self.inner.cancellation_token.cancel();
        self.inner.channel.close();
    }
}

#[derive(Debug)]
struct InnerProcessor {
    channel: IpcChannel,
    handlers: Vec<Box<dyn MessageHandler>>,
    cancellation_token: CancellationToken,
}

impl InnerProcessor {
    async fn start(&self, processor: &Arc<InnerProcessor>) {
        loop {
            select! {
                _ = self.channel.closed() => break,
                Some(message) = self.channel.recv() => self.do_safe_process(message, processor).await,
            }
        }
        self.cancellation_token.cancel();
        debug!("IPC channel processor main loop ended");
    }

    async fn do_safe_process(&self, message: FxMessage, processor: &Arc<InnerProcessor>) {
        let processor = processor.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.handle_message(message).await {
                warn!("IPC channel processor failed to process message, {}", e);
            }
        });
    }

    async fn handle_message(&self, message: FxMessage) -> Result<()> {
        let message_type = message.type_.as_str();
        if message_type == ApplicationTerminationRequest::NAME {
            debug!("IPC channel processor is being terminated");
            self.cancellation_token.cancel();
            self.channel.close();
            return Ok(());
        } else if message_type.is_empty() {
            return Err(Error::MissingMessageType);
        }

        let handler = self
            .handlers
            .iter()
            .find(|e| e.is_supported(&message_type))
            .ok_or(Error::UnsupportedMessage(
                message.message_type().to_string(),
            ))?;

        if let Err(e) = handler.process(message, &self.channel).await {
            error!(
                "Message handler \"{}\" encountered an error, {}",
                handler.name(),
                e
            );
        }

        Ok(())
    }
}

impl FxMessage {
    pub fn message_type(&self) -> &str {
        self.type_.as_str()
    }
}

pub fn enum_into<E: Enum>(value: EnumOrUnknown<E>) -> Result<E> {
    value.enum_value().map_err(|_| Error::UnsupportedEnum)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::proto::application::GetApplicationVersionRequest;
    use crate::ipc::proto::log::LogRequest;
    use crate::ipc::test::create_channel_pair;
    use crate::timeout;
    use mockall::mock;

    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    mock! {
        #[derive(Debug)]
        pub MessageHandler {}

        #[async_trait]
        impl MessageHandler for MessageHandler {
            fn name(&self) -> &str;
            fn is_supported(&self, message_type: &str) -> bool;
            async fn process(&self, message: FxMessage, channel: &IpcChannel) -> Result<()>;
        }
    }

    #[test]
    fn test_message_type() {
        let message_type = LogRequest::NAME;
        let mut message = FxMessage::new();
        message.type_ = message_type.to_string();

        let result = message.message_type();

        assert_eq!(result, message_type);
    }

    #[tokio::test]
    async fn test_ipc_channel_processor_handle_message() {
        init_logger!();
        let (tx, mut rx) = unbounded_channel();
        let mut handle = MockMessageHandler::new();
        handle
            .expect_name()
            .return_const("MockMessageHandler".to_string());
        handle.expect_is_supported().times(1).return_const(true);
        handle
            .expect_process()
            .times(1)
            .returning(move |message, _| {
                tx.send(message).unwrap();
                Ok::<(), Error>(())
            });
        let (incoming, outgoing) = create_channel_pair().await;
        let _processor = IpcChannelProcessor::new(incoming, vec![Box::new(handle)]);

        // trigger the processor by sending a message to it's channel
        outgoing
            .send(
                GetApplicationVersionRequest::new(),
                GetApplicationVersionRequest::NAME,
            )
            .await
            .expect("expected to have send the request message");

        let message = timeout!(rx.recv(), Duration::from_millis(250))
            .expect("expected process to have been invoked");
        assert_eq!(GetApplicationVersionRequest::NAME, message.type_.as_str());
    }
}
