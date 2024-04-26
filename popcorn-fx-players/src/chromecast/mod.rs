pub use discovery::*;
pub use errors::*;
pub use models::*;
pub use player::*;

mod discovery;
mod errors;
mod models;
mod player;
pub mod transcode;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use std::net::SocketAddr;
    use std::sync::Arc;

    use log::{debug, error, info};
    use mdns_sd::{ServiceDaemon, ServiceInfo};
    use protobuf::CodedInputStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, sink, split};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::runtime::Runtime;
    use tokio_rustls::{rustls, TlsAcceptor};
    use tokio_rustls::rustls::pki_types::PrivateKeyDer;
    use tokio_rustls::server::TlsStream;

    use popcorn_fx_core::core::utils::network::available_socket;

    use super::*;

    pub struct TestInstance {
        pub addr: SocketAddr,
        pub mdns: Option<ServiceDaemon>,
        pub player: Option<ChromecastPlayer>,
        pub runtime: Arc<Runtime>,
    }

    impl TestInstance {
        pub fn new_mdns() -> Self {
            let mut instance = Self::new();
            let mdns = ServiceDaemon::new().expect("Failed to create daemon");
            let service = ServiceInfo::new(
                SERVICE_TYPE,
                "chromecast_test_device",
                format!("{}.local.", instance.addr.ip()).as_str(),
                instance.addr.ip(),
                instance.addr.port(),
                &[("fn", "Chromecast test device"), ("md", "Chromecast")][..],
            ).unwrap();

            mdns.register(service).expect("Failed to register service");

            instance.mdns = Some(mdns);
            instance
        }

        pub fn new_player() -> Self {
            let mut instance = Self::new();
            let player = ChromecastPlayer::builder()
                .id("MyChromecastId")
                .name("MyChromecastName")
                .cast_model("Chromecast")
                .cast_address(instance.addr.ip().to_string())
                .cast_port(instance.addr.port())
                .heartbeat_millis(500)
                .build()
                .unwrap();

            instance.player = Some(player);
            instance
        }

        fn new() -> Self {
            let addr = available_socket();
            let cert = rcgen::generate_simple_self_signed([]).unwrap();
            let runtime = Arc::new(Runtime::new().unwrap());

            let server_addr = addr.clone();
            runtime.spawn(async move {
                let config = rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(
                        vec![cert.cert.der().clone()],
                        PrivateKeyDer::try_from(cert.key_pair.serialize_der()).unwrap())
                    .unwrap();

                let acceptor = TlsAcceptor::from(Arc::new(config));
                let listener = TcpListener::bind(&server_addr).await.unwrap();

                loop {
                    match listener.accept().await {
                        Ok((stream, socket)) => handle_socket_connection(stream, socket, acceptor.clone()),
                        Err(e) => error!("Failed to establish connection with client, {}", e),
                    }
                }
            });

            Self {
                addr,
                mdns: None,
                player: None,
                runtime,
            }
        }
    }

    fn handle_socket_connection(stream: TcpStream, socket: SocketAddr, acceptor: TlsAcceptor) {
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(stream) => {
                    debug!("Client TLS connection stream has been established for {}", socket);
                    let (mut stream, conn) = stream.into_inner();
                    let (mut reader, mut writer) = stream.split();

                    loop {
                        // Read the length prefix of the message
                        let mut len_buf = [0u8; 4];
                        if let Err(e) = reader.read_exact(&mut len_buf).await {
                            error!("Failed to read message length, {}", e);
                            continue;
                        }
                        let len = u32::from_be_bytes(len_buf) as usize;

                        // Read the protobuf message
                        let mut buf = vec![0u8; len];
                        if let Err(e) = reader.read_exact(&mut buf).await {
                            error!("Failed to read message, {}", e);
                            continue;
                        }

                        // Parse the protobuf message
                        let mut input_stream = CodedInputStream::from_bytes(&buf);
                        let mut message = String::new();
                        input_stream.read_to_string(&mut message);
                        info!("Received proto message {}", message);
                    }
                }
                Err(e) => error!("Failed to accept client TLS connection, {}", e),
            }
        });
    }
}