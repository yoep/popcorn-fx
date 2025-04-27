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
