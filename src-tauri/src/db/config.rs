use crate::state::{AppConfig, AppRule, DnsServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsFlowConfig {
    pub version: String,
    pub app_config: AppConfig,
    pub dns_servers: Vec<DnsServer>,
    pub app_rules: Vec<AppRule>,
}

impl DnsFlowConfig {
    pub fn new(
        app_config: AppConfig,
        dns_servers: Vec<DnsServer>,
        app_rules: Vec<AppRule>,
    ) -> Self {
        Self {
            version: "1.0".to_string(),
            app_config,
            dns_servers,
            app_rules,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
