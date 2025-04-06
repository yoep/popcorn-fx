use std::net::{IpAddr, SocketAddr, TcpListener};

use local_ip_address::local_ip;

/// Retrieves a non-localhost (127.0.0.1) IP address from one of the machine's network interfaces.
///
/// # Returns
///
/// The local IP address found on one of the network interfaces.
pub fn ip_addr() -> IpAddr {
    local_ip().expect("expected an ip address from a network interface")
}

/// Retrieves an available socket address on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Returns
///
/// Returns an available `SocketAddr` with the local IP address and an automatically selected available port.
pub fn available_socket() -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0").expect("expected a TCP address to be bound");
    let socket_addr = listener.local_addr().expect("expected a valid socket");

    SocketAddr::new(ip_addr(), socket_addr.port())
}

/// Retrieves an available port on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Arguments
///
/// * `lower_bound` - The lower bound of the available port range.
/// * `upper_bound` - The upper bound of the available port range.
///
/// # Returns
///
/// Returns an available port if one is found, else `None`.
pub fn available_port(lower_bound: u16, upper_bound: u16) -> Option<u16> {
    let supported_ports: Vec<u16> = (lower_bound..=upper_bound).collect();

    for port in supported_ports {
        let socket: SocketAddr = ([0, 0, 0, 0], port).into();
        if TcpListener::bind(socket).is_ok() {
            return Some(port);
        }
    }

    None
}

/// Retrieves an available port on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Arguments
///
/// * `lower_bound` - The lower bound of the available port range (optional, default = 1000).
/// * `upper_bound` - The upper bound of the available port range (optional, default = [u16::MAX]).
///
/// # Returns
///
/// Returns an available port if one is found, else `None`.
#[macro_export]
macro_rules! available_port {
    ($lower_bound:expr, $upper_bound:expr) => {
        popcorn_fx_core::core::utils::network::available_port($lower_bound, $upper_bound)
    };
    ($lower_bound:expr) => {
        popcorn_fx_core::core::utils::network::available_port($lower_bound, u16::MAX)
    };
    () => {
        popcorn_fx_core::core::utils::network::available_port(1000, u16::MAX)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_addr() {
        let localhost: IpAddr = "172.0.0.1".parse().unwrap();

        let result = ip_addr();

        assert_ne!(localhost, result, "expected no localhost ip address");
    }

    #[test]
    fn test_available_socket() {
        let localhost: IpAddr = "172.0.0.1".parse().unwrap();

        let result = available_socket();

        assert_ne!(localhost, result.ip(), "expected no localhost ip address");
        assert_ne!(0, result.port());
    }
}
