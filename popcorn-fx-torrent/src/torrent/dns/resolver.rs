use crate::torrent::dns::{Error, Result};
use log::trace;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::net::lookup_host;
use url::Url;

const DEFAULT_PORT: u16 = 80;

#[derive(Debug, Clone)]
pub struct DnsResolver(String, u16);

impl DnsResolver {
    /// Create a new DNS resolver for the given url.
    pub fn new(url: String) -> Self {
        Self(url, DEFAULT_PORT)
    }

    /// Create a new DNS resolver for the given url with a default port number if one is not specified within the url.
    pub fn new_with_port(url: String, port: u16) -> Self {
        Self(url, port)
    }

    /// Resolve the socket addresses for the stored url.
    pub async fn resolve(&self) -> Result<Vec<SocketAddr>> {
        let host = self.host()?;

        trace!("DNS resolver is resolving {}", host);
        lookup_host(host)
            .await
            .map(|e| e.collect())
            .map_err(|e| Error::from(e))
    }

    /// Resolve the ip addresses for the stored url.
    pub async fn resolve_ip(&self) -> Result<Vec<IpAddr>> {
        self.resolve()
            .await
            .map(|e| e.into_iter().map(|addr| addr.ip()).collect())
    }

    fn host(&self) -> Result<String> {
        let url = Url::parse(&self.0)?;
        if let Some(host) = url.host_str() {
            let port = url.port().unwrap_or(self.1);
            return Ok(format!("{}:{}", host, port));
        }

        Ok(self.0.clone())
    }
}

impl FromStr for DnsResolver {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Ok(Self::new(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_from_str_with_scheme() {
        let value = "udp://google.com";
        let resolver = DnsResolver::from_str(value).unwrap();

        let result = resolver.resolve().await;

        match &result {
            Ok(addrs) => {
                assert_ne!(
                    0,
                    addrs.len(),
                    "expected to have resolved at least one address"
                );
            }
            Err(_) => assert!(
                false,
                "expected the url to have been resolved, but got {:?} instead",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_from_str_without_scheme() {
        let value = "router.utorrent.com:6881";
        let resolver = DnsResolver::from_str(value).unwrap();

        let result = resolver.resolve().await;

        match &result {
            Ok(addrs) => {
                assert_ne!(
                    0,
                    addrs.len(),
                    "expected to have resolved at least one address"
                );
            }
            Err(_) => assert!(
                false,
                "expected the url to have been resolved, but got {:?} instead",
                result
            ),
        }
    }
}
