use crate::ipc::channel::IpcChannel;
use crate::ipc::errors::Error;
use crate::ipc::message::MessageHandler;
use crate::ipc::proto::log::log_request::LogLevel;
use crate::ipc::proto::log::LogRequest;
use crate::ipc::proto::message::FxMessage;
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use protobuf::Message;

#[derive(Debug, Default)]
pub struct LogMessageHandler;

impl LogMessageHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MessageHandler for LogMessageHandler {
    fn name(&self) -> &str {
        "log"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        message_type == LogRequest::NAME
    }

    async fn process(&self, message: FxMessage, _: &IpcChannel) -> crate::ipc::errors::Result<()> {
        match message.message_type() {
            LogRequest::NAME => {
                let log = LogRequest::parse_from_bytes(message.payload.as_slice())?;
                match log.level.enum_value_or(LogLevel::INFO) {
                    LogLevel::TRACE => trace!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::DEBUG => debug!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::INFO => info!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::WARN => warn!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::ERROR => error!(target: log.target.as_str(), "{}", log.message),
                    _ => {}
                }
                Ok(())
            }
            _ => Err(Error::UnsupportedMessage(
                message.message_type().to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::timeout;

    use popcorn_fx_core::init_logger;
    use std::time::Duration;

    #[tokio::test]
    async fn test_process_log_request() {
        init_logger!();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = LogMessageHandler::new();

        incoming
            .send(
                LogRequest {
                    level: LogLevel::TRACE.into(),
                    target: "jvm::package".to_string(),
                    message: "Lorem ipsum".to_string(),
                    special_fields: Default::default(),
                },
                LogRequest::NAME,
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

        incoming
            .send(
                LogRequest {
                    level: LogLevel::DEBUG.into(),
                    target: "jvm::package".to_string(),
                    message: "Foo".to_string(),
                    special_fields: Default::default(),
                },
                LogRequest::NAME,
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

        incoming
            .send(
                LogRequest {
                    level: LogLevel::INFO.into(),
                    target: "jvm::package".to_string(),
                    message: "Bar".to_string(),
                    special_fields: Default::default(),
                },
                LogRequest::NAME,
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
