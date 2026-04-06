#[cfg(test)]
mod tests {
    use crate::db::config::DnsFlowConfig;
    use crate::state::{AppConfig, AppRule, DnsServer};

    #[test]
    fn test_config_round_trip() {
        let app_config = AppConfig {
            proxy_port: 5353,
            log_enabled: true,
            auto_start: false,
            debug: false,
        };

        let dns_servers = vec![DnsServer {
            id: Some(1),
            name: "Google".to_string(),
            address: "8.8.8.8".to_string(),
            secondary_address: None,
            protocol: "udp".to_string(),
            is_default: false,
        }];

        let app_rules = vec![AppRule {
            id: Some(1),
            app_name: "firefox".to_string(),
            app_path: Some("/usr/bin/firefox".to_string()),
            dns_server_id: 1,
            enabled: true,
            use_ld_preload: false,
        }];

        let config = DnsFlowConfig::new(app_config, dns_servers, app_rules);
        let json = config.to_json().unwrap();
        let parsed = DnsFlowConfig::from_json(&json).unwrap();

        assert_eq!(config.version, parsed.version);
        assert_eq!(config.app_config.proxy_port, parsed.app_config.proxy_port);
        assert_eq!(config.dns_servers.len(), parsed.dns_servers.len());
        assert_eq!(config.app_rules.len(), parsed.app_rules.len());
    }

    #[test]
    fn test_config_invalid_json() {
        let result = DnsFlowConfig::from_json("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_missing_fields() {
        let json = r#"{"version": "1.0"}"#;
        let result = DnsFlowConfig::from_json(json);
        assert!(result.is_err());
    }
}
