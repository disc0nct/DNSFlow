#[cfg(test)]
mod tests {
    use crate::dns::rules::RulesEngine;
    use crate::state::{AppRule, DnsServer};

    #[tokio::test]
    async fn test_rules_engine_lookup() {
        let engine = RulesEngine::new(60);

        let server = DnsServer {
            id: Some(1),
            name: "Test DNS".to_string(),
            address: "8.8.8.8".to_string(),
            secondary_address: None,
            protocol: "udp".to_string(),
            is_default: false,
        };

        let rule = AppRule {
            id: Some(1),
            app_name: "test_app".to_string(),
            app_path: Some("/usr/bin/test".to_string()),
            dns_server_id: 1,
            enabled: true,
            use_ld_preload: false,
        };

        engine.load_dns_servers(vec![server]).await;
        engine.load_rules(vec![rule], None).await;

        let result = engine.lookup_by_app_name("test_app").await;
        assert!(result.is_some(), "Should find matching rule");
        assert_eq!(result.unwrap().address, "8.8.8.8");
    }

    #[tokio::test]
    async fn test_rules_engine_disabled_rule() {
        let engine = RulesEngine::new(60);

        let server = DnsServer {
            id: Some(1),
            name: "Test DNS".to_string(),
            address: "8.8.8.8".to_string(),
            secondary_address: None,
            protocol: "udp".to_string(),
            is_default: false,
        };

        let rule = AppRule {
            id: Some(1),
            app_name: "test_app".to_string(),
            app_path: None,
            dns_server_id: 1,
            enabled: false,
            use_ld_preload: false,
        };

        engine.load_dns_servers(vec![server]).await;
        engine.load_rules(vec![rule], None).await;

        let result = engine.lookup_by_app_name("test_app").await;
        assert!(result.is_none(), "Disabled rule should not match");
    }
}
