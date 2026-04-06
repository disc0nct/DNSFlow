#[cfg(test)]
mod tests {
    use crate::dns::resolver::DnsResolver;
    use std::net::SocketAddr;

    #[test]
    fn test_resolver_new_creates_with_correct_defaults() {
        let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
        let resolver = DnsResolver::new(addr);

        assert_eq!(resolver.upstream_addr, addr);
        assert_eq!(resolver.timeout_ms, 5000);
    }

    #[test]
    fn test_resolver_set_timeout_updates_timeout_ms() {
        let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
        let mut resolver = DnsResolver::new(addr);

        assert_eq!(resolver.timeout_ms, 5000);

        resolver.set_timeout(10000);
        assert_eq!(resolver.timeout_ms, 10000);

        resolver.set_timeout(100);
        assert_eq!(resolver.timeout_ms, 100);
    }

    #[tokio::test]
    async fn test_resolve_with_mock_udp_server() {
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let mut buf = [0u8; 512];
            let (len, src) = server_socket.recv_from(&mut buf).await.unwrap();

            let mut response = vec![0u8; len];
            response.copy_from_slice(&buf[..len]);
            response[2] = 0x81;
            response[3] = 0x80;

            server_socket.send_to(&response, src).await.unwrap();
        });

        let mut resolver = DnsResolver::new(server_addr);
        resolver.set_timeout(1000);

        let query = vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = resolver.resolve(&query).await;

        server_handle.await.unwrap();

        assert!(
            result.is_ok(),
            "Should successfully resolve with mock server"
        );
        let response = result.unwrap();
        assert!(!response.is_empty(), "Response should not be empty");
        assert_eq!(response[2], 0x81, "Response should have QR bit set");
        assert_eq!(response[3], 0x80, "Response should have RD bit set");
    }

    #[tokio::test]
    async fn test_resolve_timeout() {
        let addr: SocketAddr = "192.0.2.1:53".parse().unwrap();
        let mut resolver = DnsResolver::new(addr);
        resolver.set_timeout(100);

        let query = vec![0u8; 12];
        let result = resolver.resolve(&query).await;

        assert!(result.is_err(), "Should timeout with unreachable server");
        let err = result.unwrap_err();
        assert!(err.contains("timed out"), "Error should mention timeout");
    }

    #[test]
    fn test_resolver_debug_trait() {
        let addr: SocketAddr = "1.1.1.1:53".parse().unwrap();
        let resolver = DnsResolver::new(addr);

        let debug_str = format!("{:?}", resolver);
        assert!(debug_str.contains("DnsResolver"));
        assert!(debug_str.contains("upstream_addr"));
        assert!(debug_str.contains("timeout_ms"));
    }

    #[test]
    fn test_resolver_with_different_addresses() {
        let addrs = [
            "8.8.8.8:53",
            "1.1.1.1:53",
            "9.9.9.9:53",
            "208.67.222.222:53",
        ];

        for addr_str in addrs {
            let addr: SocketAddr = addr_str.parse().unwrap();
            let resolver = DnsResolver::new(addr);
            assert_eq!(resolver.upstream_addr, addr);
        }
    }
}
