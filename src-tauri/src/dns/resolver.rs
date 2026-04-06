use std::net::SocketAddr;

/// DNS resolver for forwarding queries to upstream servers
/// Kept for potential future use — currently the proxy uses TokioAsyncResolver directly.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DnsResolver {
    /// Upstream DNS server address
    pub upstream_addr: SocketAddr,
    /// Query timeout in milliseconds
    pub timeout_ms: u64,
}

impl DnsResolver {
    /// Create a new DNS resolver
    pub fn new(upstream_addr: SocketAddr) -> Self {
        Self {
            upstream_addr,
            timeout_ms: 5000,
        }
    }

    /// Resolve a DNS query by forwarding to upstream
    pub async fn resolve(&self, query: &[u8]) -> Result<Vec<u8>, String> {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| e.to_string())?;
        socket
            .send_to(query, self.upstream_addr)
            .await
            .map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; 4096];
        let timeout_duration = std::time::Duration::from_millis(self.timeout_ms);
        let (len, _addr) = tokio::time::timeout(timeout_duration, socket.recv_from(&mut buf))
            .await
            .map_err(|_| "DNS query timed out".to_string())?
            .map_err(|e| e.to_string())?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Set query timeout
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
}
