use crate::fx::PopcornFX;
use crate::ipc::errors::{Error, Result};
use crate::ipc::protobuf::application_args::ApplicationArgs;
use crate::ipc::protobuf::log::log::LogLevel;
use crate::ipc::protobuf::log::Log;
use crate::ipc::protobuf::message::fx_message::MessageType;
use crate::ipc::protobuf::message::FxMessage;
use crate::ipc::protobuf::settings::ApplicationSettings;
use byteorder::{BigEndian, ByteOrder};
use interprocess::local_socket;
use interprocess::local_socket::tokio::{RecvHalf, SendHalf, Stream};
use log::{debug, error, info, trace, warn};
use protobuf::{EnumOrUnknown, Message};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct IpcChannel {
    inner: Arc<InnerIpcChannel>,
}

impl IpcChannel {
    pub fn new(stream: Stream, popcorn_fx: PopcornFX) -> Self {
        let (reader, writer) = local_socket::traits::tokio::Stream::split(stream);
        let (sender, receiver) = unbounded_channel();
        let inner = Arc::new(InnerIpcChannel {
            instance: Mutex::new(Some(popcorn_fx)),
            sequence_id: Default::default(),
            writer: Mutex::new(writer),
            cancellation_token: Default::default(),
        });

        let mut reader = IpcChannelReader {
            reader,
            sender,
            cancellation_token: inner.cancellation_token.clone(),
        };
        tokio::spawn(async move {
            reader.start().await;
        });

        let mut processor = IpcChannelProcessor {
            channel: inner.clone(),
            receiver,
            cancellation_token: inner.cancellation_token.clone(),
        };
        tokio::spawn(async move {
            processor.start().await;
        });

        Self { inner }
    }

    /// Execute the IPC channel communication.
    /// It will gracefully exit when the channel has been closed by the server.
    pub async fn execute(&self) {
        self.inner.cancellation_token.cancelled().await
    }

    /// Close the IPC channel communication.
    pub fn close(&self) {
        trace!("IPC channel is being closed by client");
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerIpcChannel {
    instance: Mutex<Option<PopcornFX>>,
    sequence_id: AtomicU32,
    writer: Mutex<SendHalf>,
    cancellation_token: CancellationToken,
}

impl InnerIpcChannel {
    async fn instance(&self) -> MutexGuard<Option<PopcornFX>> {
        self.instance.lock().await
    }

    async fn send_bytes(
        &self,
        payload: Vec<u8>,
        message_type: MessageType,
        reply_id: Option<i32>,
    ) -> Result<()> {
        let mut buffer = vec![0u8; 4];
        let mut message = FxMessage::new();
        let mut writer = self.writer.lock().await;

        message.type_ = EnumOrUnknown::from(message_type);
        message.sequence_id = self.sequence_id.fetch_add(1, Ordering::SeqCst) as i32;
        message.payload = payload;
        message.reply_id = reply_id;

        let mut bytes = message.write_to_bytes()?;

        // write the message length as the first 4 bytes
        BigEndian::write_u32(&mut buffer[..4], bytes.len() as u32);
        buffer.write_all(&bytes).await?;

        let start_time = Instant::now();
        writer.write_all(&buffer).await?;
        writer.flush().await?;
        let elapsed = start_time.elapsed();
        debug!(
            "IPC channel wrote {} bytes in {}.{:03}ms",
            buffer.len(),
            elapsed.as_millis(),
            elapsed.as_micros()
        );
        Ok(())
    }
}

#[derive(Debug)]
struct IpcChannelReader {
    reader: RecvHalf,
    sender: UnboundedSender<Vec<u8>>,
    cancellation_token: CancellationToken,
}

impl IpcChannelReader {
    async fn start(&mut self) {
        loop {
            let mut buffer = [0u8; 4];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                read_result = self.reader.read_exact(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            trace!("IPC channel reader received EOF");
                            break
                        },
                        Ok(len) => {
                            if let Err(e) = self.handle_received_message(&buffer, len).await {
                                error!("IPC channel reader failed to handle message, {}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            error!("IPC channel reader encountered an error, {}", e);
                            break;
                        }
                    }
                },
            }
        }
        debug!("IPC channel reader main loop ended");
    }

    async fn handle_received_message(&mut self, buffer: &[u8], buffer_len: usize) -> Result<()> {
        if buffer_len != 4 {
            return Err(Error::InvalidLength);
        }

        let start_time = Instant::now();
        let len = BigEndian::read_u32(&buffer[..4]) as usize;
        let mut buffer = vec![0u8; len];

        trace!("IPC channel reader is reading message (size {})", len);
        self.reader.read_exact(&mut buffer).await?;
        let elapsed = start_time.elapsed();
        debug!(
            "IPC channel read message (size {}) in {}.{:03}ms",
            len,
            elapsed.as_millis(),
            elapsed.as_micros()
        );

        if let Err(e) = self.sender.send(buffer) {
            warn!(
                "IPC channel reader failed to send payload for processing, {}",
                e
            );
        }

        Ok(())
    }
}

#[derive(Debug)]
struct IpcChannelProcessor {
    channel: Arc<InnerIpcChannel>,
    receiver: UnboundedReceiver<Vec<u8>>,
    cancellation_token: CancellationToken,
}

impl IpcChannelProcessor {
    async fn start(&mut self) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(buffer) = self.receiver.recv() => self.process(buffer).await,
            }
        }
        debug!("IPC channel processor main loop ended");
    }

    async fn process(&self, buffer: Vec<u8>) {
        if let Err(e) = self.do_process(buffer).await {
            error!("IPC channel processor failed, {}", e)
        }
    }

    async fn do_process(&self, buffer: Vec<u8>) -> Result<()> {
        let message = FxMessage::parse_from_bytes(&buffer)?;

        trace!(
            "IPC channel processor is parsing message {:?}",
            message.type_
        );
        let message_type = message
            .type_
            .enum_value()
            .map_err(|e| Error::InvalidMessage(e as u32))?;
        match message_type {
            MessageType::LOG_MESSAGE => {
                let log = Log::parse_from_bytes(message.payload.as_slice())?;
                match log.level.enum_value_or(LogLevel::INFO) {
                    LogLevel::TRACE => trace!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::DEBUG => debug!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::INFO => info!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::WARN => warn!(target: log.target.as_str(), "{}", log.message),
                    LogLevel::ERROR => error!(target: log.target.as_str(), "{}", log.message),
                    _ => {}
                }
            }
            MessageType::APPLICATION_ARGS_REQUEST => {
                if let Some(opts) = self.channel.instance().await.as_ref().map(|e| e.opts()) {
                    let mut args = ApplicationArgs::new();

                    args.is_tv_mode = opts.tv;
                    args.is_maximized = opts.maximized;
                    args.is_kiosk_mode = opts.kiosk;
                    args.is_mouse_disabled = opts.disable_mouse;
                    args.is_youtube_player_enabled = opts.enable_youtube_video_player;
                    args.is_vlc_video_player_enabled = opts.enable_vlc_video_player;
                    args.is_fx_player_enabled = opts.enable_fx_video_player;

                    self.channel
                        .send_bytes(
                            args.write_to_bytes()?,
                            MessageType::APPLICATION_ARGS_RESPONSE,
                            Some(message.sequence_id),
                        )
                        .await?;
                }
            }
            MessageType::APPLICATION_SETTINGS_REQUEST => {
                let mut settings: Option<ApplicationSettings> = None;

                if let Some(instance) = &*self.channel.instance().await {
                    settings = Some(
                        instance
                            .settings()
                            .user_settings_ref(|e| ApplicationSettings::from(e))
                            .await,
                    );
                }

                if let Some(settings) = settings {
                    self.channel
                        .send_bytes(
                            settings.write_to_bytes()?,
                            MessageType::APPLICATION_SETTINGS_RESPONSE,
                            Some(message.sequence_id),
                        )
                        .await?;
                }
            }
            MessageType::TERMINATE => {
                debug!("IPC channel is terminating the FX instance");
                let _ = self.channel.instance().await.take();
            }
            _ => warn!(
                "IPC channel received unsupported message type {:?}",
                message_type
            ),
        }

        Ok(())
    }
}
