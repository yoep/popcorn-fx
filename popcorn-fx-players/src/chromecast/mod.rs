pub use discovery::*;
pub use errors::*;
pub use models::*;
pub use player::*;

mod device;
mod discovery;
mod errors;
mod models;
mod player;
pub mod transcode;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::io::Cursor;
    use std::net::SocketAddr;
    use std::sync::Arc;

    use log::{debug, error};
    use mdns_sd::{ServiceDaemon, ServiceInfo};
    use protobuf::{EnumOrUnknown, Message};
    use rust_cast::cast::cast_channel;
    use rust_cast::cast::cast_channel::cast_message::{PayloadType, ProtocolVersion};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::tcp::WriteHalf;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::runtime::Runtime;
    use tokio_rustls::rustls::pki_types::PrivateKeyDer;
    use tokio_rustls::{rustls, TlsAcceptor};
    use tokio_util::sync::CancellationToken;

    use popcorn_fx_core::core::subtitles::{
        MockSubtitleProvider, SubtitleProvider, SubtitleServer,
    };
    use popcorn_fx_core::core::utils::network::{available_socket, ip_addr};

    use crate::chromecast::device::{MockFxCastDevice, DEFAULT_RECEIVER};
    use crate::chromecast::transcode::{MockTranscoder, Transcoder};

    use super::*;

    pub struct MdnsInstance {
        pub addr: SocketAddr,
        pub daemon: ServiceDaemon,
    }

    impl Debug for MdnsInstance {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MdnsInstance")
                .field("addr", &self.addr)
                .finish()
        }
    }

    #[derive(Debug)]
    pub struct TestInstance {
        pub mdns: Option<MdnsInstance>,
        pub player: Option<ChromecastPlayer<MockFxCastDevice>>,
        pub runtime: Arc<Runtime>,
        pub cancel_token: CancellationToken,
    }

    impl TestInstance {
        pub fn new_mdns() -> Self {
            let mut instance = Self::new();
            let listener = instance
                .runtime
                .block_on(TcpListener::bind("0.0.0.0:0"))
                .expect("expected a TCP address to be bound");
            let socket_addr = listener.local_addr().expect("expected a valid socket");
            let addr = SocketAddr::new(ip_addr(), socket_addr.port());
            let cert = rcgen::generate_simple_self_signed([]).unwrap();

            let thread_cancel = instance.cancel_token.clone();
            instance.runtime.spawn(async move {
                let config = rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(
                        vec![cert.cert.der().clone()],
                        PrivateKeyDer::try_from(cert.key_pair.serialize_der()).unwrap(),
                    )
                    .unwrap();

                let acceptor = TlsAcceptor::from(Arc::new(config));

                loop {
                    tokio::select! {
                        _ = thread_cancel.cancelled() => break,
                        result = listener.accept() => {
                            match result {
                                Ok((stream, socket)) => {
                                    handle_socket_connection(stream, socket, acceptor.clone())
                                }
                                Err(e) => error!("Failed to establish connection with client, {}", e),
                            }
                        }
                    }
                }
            });
            let mdns = ServiceDaemon::new().expect("Failed to create daemon");
            let service = ServiceInfo::new(
                SERVICE_TYPE,
                "chromecast_test_device",
                format!("{}.local.", addr.ip()).as_str(),
                addr.ip(),
                addr.port(),
                &[("fn", "Chromecast test device"), ("md", "Chromecast")][..],
            )
            .unwrap();

            mdns.register(service).expect("Failed to register service");

            instance.mdns = Some(MdnsInstance { addr, daemon: mdns });
            instance
        }

        pub fn new_player(device: Box<dyn Fn() -> MockFxCastDevice + Send + Sync>) -> Self {
            let mut transcoder = MockTranscoder::new();
            transcoder.expect_stop().return_const(());
            Self::new_player_with_additions(
                device,
                Box::new(MockSubtitleProvider::new()),
                Box::new(transcoder),
            )
        }

        pub fn new_player_with_additions(
            device: Box<dyn Fn() -> MockFxCastDevice + Send + Sync>,
            subtitle_provider: Box<dyn SubtitleProvider>,
            transcoder: Box<dyn Transcoder>,
        ) -> Self {
            let mut instance = Self::new();
            let addr = available_socket();
            let subtitle_server = SubtitleServer::new(Arc::new(subtitle_provider));
            let player = ChromecastPlayer::builder()
                .id("MyChromecastId")
                .name("MyChromecastName")
                .cast_model("Chromecast")
                .cast_address(addr.ip().to_string())
                .cast_port(addr.port())
                .subtitle_server(Arc::new(subtitle_server))
                .cast_device_factory(Box::new(move |_, _| Ok(device())))
                .heartbeat_millis(500)
                .build()
                .unwrap();

            instance.player = Some(player);
            instance
        }

        fn new() -> Self {
            let runtime = Arc::new(Runtime::new().unwrap());

            Self {
                mdns: None,
                player: None,
                runtime,
                cancel_token: Default::default(),
            }
        }

        pub fn mdns(&self) -> Option<&MdnsInstance> {
            self.mdns.as_ref()
        }
    }

    impl Drop for TestInstance {
        fn drop(&mut self) {
            debug!("Dropping {:?}", self);
            self.cancel_token.cancel();
        }
    }

    fn handle_socket_connection(stream: TcpStream, socket: SocketAddr, acceptor: TlsAcceptor) {
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(stream) => {
                    debug!(
                        "Client TLS connection stream has been established for {}",
                        socket
                    );
                    let (mut stream, _conn) = stream.into_inner();
                    let (mut reader, mut writer) = stream.split();

                    loop {
                        // Read the length prefix of the message
                        let mut len_buf = [0u8; 4];
                        if let Err(e) = reader.read_exact(&mut len_buf).await {
                            error!("Failed to read message length, {}", e);
                            break;
                        }
                        let len = u32::from_be_bytes(len_buf) as usize;

                        if len == 0 {
                            debug!("Stopping TLS connection stream");
                            break;
                        }

                        // read the protobuf message by filling the buffer
                        // based on the determined length
                        let mut buf = vec![0u8; len];
                        if let Err(e) = reader.read_exact(&mut buf).await {
                            error!("Failed to read message, {}", e);
                            continue;
                        }

                        let mut cursor = Cursor::new(buf.as_slice());
                        match <cast_channel::CastMessage as protobuf::Message>::parse_from_reader(
                            &mut cursor,
                        ) {
                            Ok(message) => {
                                debug!("Received cast message {:?}", message);
                                let response = create_response(message);
                                write_response(&mut writer, response).await;
                            }
                            Err(e) => {
                                error!("Failed to parse message, {}", e);
                                let response = create_ping_response();
                                write_response(&mut writer, response).await;
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to accept client TLS connection, {}", e),
            }
        });
    }

    async fn write_response<'a>(
        writer: &'a mut WriteHalf<'_>,
        response: cast_channel::CastMessage,
    ) {
        let mut response_len = vec![];
        let _ = response_len
            .write_u32(response.payload_utf8().len() as u32)
            .await;
        match writer.write_all(response_len.as_slice()).await {
            Ok(()) => {
                let mut response_buf = Vec::<u8>::new();
                let _ = response.write_to_writer(&mut response_buf);
                writer.write_all(response_buf.as_slice()).await.unwrap();
            }
            Err(e) => error!("Failed to write length message, {}", e),
        }
    }

    fn create_response(message: cast_channel::CastMessage) -> cast_channel::CastMessage {
        match message.namespace() {
            "urn:x-cast:com.google.cast.tp.connection" => cast_channel::CastMessage {
                protocol_version: Some(EnumOrUnknown::new(ProtocolVersion::CASTV2_1_2)),
                source_id: Some(DEFAULT_RECEIVER.to_string()),
                destination_id: Some("sender-0".to_string()),
                namespace: Some("urn:x-cast:com.google.cast.tp.connection".to_string()),
                payload_type: Some(EnumOrUnknown::new(PayloadType::STRING)),
                payload_utf8: Some(r#"{"type": "CLOSE"}"#.to_string()),
                payload_binary: None,
                continued: None,
                remaining_length: None,
                special_fields: Default::default(),
            },
            _ => create_ping_response(),
        }
    }

    fn create_ping_response() -> cast_channel::CastMessage {
        cast_channel::CastMessage {
            protocol_version: Some(EnumOrUnknown::new(ProtocolVersion::CASTV2_1_2)),
            source_id: Some(DEFAULT_RECEIVER.to_string()),
            destination_id: Some("sender-0".to_string()),
            namespace: Some("urn:x-cast:com.google.cast.tp.heartbeat".to_string()),
            payload_type: Some(EnumOrUnknown::new(PayloadType::STRING)),
            payload_utf8: Some(r#"{"type": "PING"}"#.to_string()),
            payload_binary: None,
            continued: None,
            remaining_length: None,
            special_fields: Default::default(),
        }
    }
}
