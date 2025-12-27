pub use discovery::*;
pub use errors::*;
pub use player::*;

mod discovery;
mod errors;
mod models;
mod player;

#[cfg(test)]
mod tests {
    use std::io;
    use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
    use std::sync::Arc;

    use log::{debug, error};
    use socket2::{Protocol, SockAddr};
    use tokio::select;
    use tokio::sync::Mutex;
    use tokio_util::sync::CancellationToken;

    pub const DEFAULT_SSDP_DESCRIPTION_RESPONSE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
        <root xmlns="urn:schemas-upnp-org:device-1-0">
            <specVersion>
                <major>1</major>
                <minor>0</minor>
            </specVersion>
            <device>
                <deviceType>urn:schemas-upnp-org:device:MediaRenderer:1</deviceType>
                <friendlyName>test</friendlyName>
                <manufacturer>MediaTech Inc.</manufacturer>
                <manufacturerURL>http://www.mediatech.example.com</manufacturerURL>
                <modelDescription>Media Renderer Device</modelDescription>
                <modelName>MR-5000</modelName>
                <modelNumber>5000</modelNumber>
                <UDN>uuid:87654321-4321-4321-4321-210987654321</UDN>
                <serviceList>
                  <service>
                    <serviceType>urn:schemas-upnp-org:service:AVTransport:1</serviceType>
                    <serviceId>urn:upnp-org:serviceId:AVTransport</serviceId>
                    <controlURL>/AVTransport/control</controlURL>
                    <eventSubURL>/AVTransport/event</eventSubURL>
                    <SCPDURL>/AVTransport/scpd.xml</SCPDURL>
                  </service>
                </serviceList>
            </device>
        </root>"#;

    pub struct SsdpServer {
        inner: Arc<InnerSsdpServer>,
    }

    impl SsdpServer {
        pub fn start(&self) {
            let inner = self.inner.clone();
            tokio::spawn(async move {
                loop {
                    if inner.cancellation_token.is_cancelled() {
                        break;
                    }

                    select! {
                        _ = inner.cancellation_token.cancelled() => break,
                        (resp, temp_buf) = inner.receive() => {
                            inner.handle_socket_packet(resp, temp_buf).await;
                        },
                    }
                }
            });
        }
    }

    impl Drop for SsdpServer {
        fn drop(&mut self) {
            self.inner.cancel();
        }
    }

    struct InnerSsdpServer {
        socket: UdpSocket,
        upnp_server_addr: SocketAddr,
        invocations: Arc<Mutex<u32>>,
        cancellation_token: CancellationToken,
    }

    impl InnerSsdpServer {
        fn cancel(&self) {
            self.cancellation_token.cancel();
        }

        async fn receive(&self) -> (io::Result<(usize, SocketAddr)>, [u8; 1024]) {
            let mut temp_buf = [0u8; 1024];
            (self.socket.recv_from(&mut temp_buf), temp_buf)
        }

        async fn handle_socket_packet(
            &self,
            resp: io::Result<(usize, SocketAddr)>,
            msg_buf: [u8; 1024],
        ) {
            match resp {
                Ok((msg_size, src_addr)) => {
                    if let Ok(msg) = std::str::from_utf8(&msg_buf[..msg_size]) {
                        debug!("Received SSDP packet: {}", msg);
                    }

                    self.send_response(src_addr).await;
                    let mut mutex = self.invocations.lock().await;
                    *mutex += 1;
                }
                Err(e) => {
                    error!("Failed to receive UDP packet: {}", e);
                }
            }
        }

        async fn send_response(&self, src_addr: SocketAddr) {
            let resp_msg = format!(
                "HTTP/1.1 200 OK\r
CACHE-CONTROL: max-age=100\r
LOCATION: http://{}/description.xml\r
SERVER: Unix/5.0 UPnP/1.1 TestDevice/1.0\r
ST: urn:schemas-upnp-org:service:MediaRenderer:1\r
USN: uuid:TEST-DEVICE-001::urn:schemas-upnp-org:device:TestDevice:1\r
EXT:\r
\r
",
                self.upnp_server_addr
            );

            if let Err(e) = self.socket.send_to(resp_msg.as_bytes(), src_addr) {
                error!("Failed to send SSDP response: {}", e);
                panic!("{}", e);
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct MockUdpServer {
        device_name: Option<String>,
        upnp_server_addr: Option<SocketAddr>,
    }

    impl MockUdpServer {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn device_name<S: Into<String>>(mut self, device_name: S) -> Self {
            self.device_name = Some(device_name.into());
            self
        }

        pub fn upnp_server_addr(mut self, upnp_server_addr: SocketAddr) -> Self {
            self.upnp_server_addr = Some(upnp_server_addr);
            self
        }

        pub fn build(self) -> SsdpServer {
            let cancellation_token = CancellationToken::new();
            let invocations = Arc::new(Mutex::new(0));
            let upnp_server_addr = self
                .upnp_server_addr
                .expect("expected an upnp server address to have been set");
            let addr = SockAddr::from(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 1900)));
            let socket = socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::DGRAM,
                Some(Protocol::UDP),
            )
            .expect("failed to create socket");
            #[cfg(not(target_os = "windows"))]
            socket.set_reuse_port(true).unwrap();
            socket.bind(&addr).expect("failed to bind socket");
            let socket = UdpSocket::from(socket);
            socket
                .join_multicast_v4(
                    &"239.255.255.250".parse().unwrap(),
                    &"0.0.0.0".parse().unwrap(),
                )
                .unwrap();

            let instance = SsdpServer {
                inner: Arc::new(InnerSsdpServer {
                    socket,
                    upnp_server_addr,
                    invocations,
                    cancellation_token,
                }),
            };

            instance.start();
            instance
        }
    }
}
