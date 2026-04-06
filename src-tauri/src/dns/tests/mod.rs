#[cfg(test)]
mod tests {
    use super::super::proxy::DnsProxyServer;

    #[tokio::test]
    async fn test_dns_proxy_can_query() {
        let proxy = DnsProxyServer::new("127.0.0.1:5353").await.unwrap();
        let result = proxy.query("google.com").await;
        assert!(result.is_ok(), "Should be able to query DNS");
        let ips = result.unwrap();
        assert!(!ips.is_empty(), "Should get at least one IP");
    }
}

#[cfg(test)]
mod rules_test;

#[cfg(test)]
mod resolver_test;
