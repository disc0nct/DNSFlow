use crate::db::database::Database;
use crate::db::query_logger::QueryLogger;
use crate::dns::proxy::DnsProxyServer;
use crate::dns::rules::RulesEngine;
use crate::platform::{create_platform, Platform};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsServer {
    pub id: Option<i64>,
    pub name: String,
    pub address: String,
    pub secondary_address: Option<String>,
    pub protocol: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppRule {
    pub id: Option<i64>,
    pub app_name: String,
    pub app_path: Option<String>,
    pub dns_server_id: i64,
    pub enabled: bool,
    pub use_ld_preload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsQueryLog {
    pub id: Option<i64>,
    pub domain: String,
    pub pid: Option<i64>,
    pub app_name: Option<String>,
    pub dns_server_id: Option<i64>,
    pub resolved_ip: Option<String>,
    pub latency_ms: Option<i64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessInfo {
    pub pid: i64,
    pub ppid: Option<i64>,
    pub name: String,
    pub exe_path: Option<String>,
    pub cmdline: Option<String>,
    pub rule_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub proxy_port: u16,
    pub log_enabled: bool,
    pub auto_start: bool,
    pub debug: bool,
}

// TODO: Consolidate with dnsflow_common::DnsEvent to avoid duplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEvent {
    pub pid: i64,
    pub tgid: i64,
    pub uid: i64,
    pub gid: i64,
    pub comm: String,
    pub daddr: String,
    pub dport: i64,
    pub is_dns: bool,
}

pub struct AppState {
    pub config: AppConfig,
    pub proxy: Arc<RwLock<Option<DnsProxyServer>>>,
    pub rules_engine: Arc<RulesEngine>,
    pub query_logger: Arc<QueryLogger>,
    pub query_log_receiver: Arc<Mutex<Option<mpsc::Receiver<DnsQueryLog>>>>,
    pub platform: Arc<Platform>,
    pub event_tx: mpsc::Sender<DnsEvent>,
    pub event_rx: Arc<Mutex<mpsc::Receiver<DnsEvent>>>,
    pub db: Option<Arc<Database>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("proxy", &self.proxy)
            .field("rules_engine", &self.rules_engine)
            .field("platform", &"<Platform>")
            .finish()
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (logger, receiver) = QueryLogger::new(1000);
        let (tx, rx) = mpsc::channel(100);
        Self {
            config: AppConfig::default(),
            proxy: Arc::new(RwLock::new(None)),
            rules_engine: Arc::new(RulesEngine::new(60)),
            query_logger: Arc::new(logger),
            query_log_receiver: Arc::new(Mutex::new(Some(receiver))),
            platform: Arc::new(create_platform()),
            event_tx: tx,
            event_rx: Arc::new(Mutex::new(rx)),
            db: None,
        }
    }
}
