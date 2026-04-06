use crate::debug;
use crate::state::{AppRule, DnsServer};
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn shim_rules_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(|d| PathBuf::from(d).join("dnsflow_rules.json"))
        .unwrap_or_else(|| PathBuf::from("/tmp/dnsflow_rules.json"))
}

#[derive(Debug, Clone)]
pub struct RulesEngine {
    rules: Arc<RwLock<Vec<AppRule>>>,
    dns_servers: Arc<RwLock<HashMap<i64, DnsServer>>>,
    cache: Arc<RwLock<HashMap<String, Option<DnsServer>>>>,
    active_sessions: Arc<RwLock<HashMap<u32, i64>>>,
    #[allow(dead_code)]
    cache_ttl: std::time::Duration,
}

impl RulesEngine {
    pub fn new(cache_ttl_secs: u64) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            dns_servers: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(cache_ttl_secs),
        }
    }

    pub async fn track_session(&self, pid: u32, rule_id: i64) {
        debug!("Tracking session: PID {} -> Rule ID {}", pid, rule_id);
        let mut guard = self.active_sessions.write().await;
        guard.insert(pid, rule_id);
    }

    pub async fn untrack_session(&self, pid: u32) {
        let mut guard = self.active_sessions.write().await;
        guard.remove(&pid);
    }

    pub async fn get_active_sessions(&self) -> HashMap<i64, u32> {
        let guard = self.active_sessions.read().await;
        let mut map = HashMap::new();
        for (&pid, &rule_id) in guard.iter() {
            map.insert(rule_id, pid);
        }
        map
    }

    pub async fn load_rules(&self, rules: Vec<AppRule>, current_proxy_addr: Option<&str>) {
        let enabled = rules.iter().filter(|r| r.enabled).count();
        let ld_preload = rules
            .iter()
            .filter(|r| r.enabled && r.use_ld_preload)
            .count();
        debug!(
            "Loading {} rules ({} enabled, {} with LD_PRELOAD)",
            rules.len(),
            enabled,
            ld_preload
        );
        {
            let mut guard = self.rules.write().await;
            *guard = rules;
        }
        self.cache.write().await.clear();

        if let Err(e) = self.write_shim_rules(current_proxy_addr).await {
            debug!("Failed to write shim rules: {}", e);
        }
    }

    async fn write_shim_rules(
        &self,
        current_proxy_addr: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::dns::proxy::{PROXY_LISTEN_IP, PROXY_LISTEN_PORT};

        let rules = self.rules.read().await;

        let any_ld_preload = rules.iter().any(|r| r.enabled && r.use_ld_preload);

        let default_addr = format!("{}:{}", PROXY_LISTEN_IP, PROXY_LISTEN_PORT);
        let proxy_addr = current_proxy_addr.unwrap_or(&default_addr);

        let config = json!({
            "intercept": any_ld_preload,
            "proxy_addr": proxy_addr
        });

        let final_path = shim_rules_path();
        let tmp_path = final_path.with_extension("json.tmp");

        let mut file = File::create(&tmp_path)?;
        file.write_all(config.to_string().as_bytes())?;
        file.sync_all()?;

        std::fs::rename(tmp_path, &final_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o644));
        }

        Ok(())
    }

    pub async fn load_dns_servers(&self, servers: Vec<DnsServer>) {
        debug!("Loading {} DNS servers", servers.len());
        let mut guard = self.dns_servers.write().await;
        for server in &servers {
            if let Some(id) = server.id {
                debug!("  Server {}: {} ({})", id, server.name, server.address);
                guard.insert(id, server.clone());
            }
        }
    }

    pub async fn lookup_by_pid(&self, pid: i32) -> Option<DnsServer> {
        let cache_key = format!("pid:{}", pid);
        if let Some(cached) = self.cache.read().await.get(&cache_key) {
            debug!(
                "Cache hit for {}: {:?}",
                cache_key,
                cached.as_ref().map(|s| &s.name)
            );
            return cached.clone();
        }

        let mut current_pid = pid;
        let mut forced_rule_id = None;

        let sessions = self.active_sessions.read().await;
        let mut depth = 0;
        while current_pid > 1 && depth < 10 {
            if let Some(&rule_id) = sessions.get(&(current_pid as u32)) {
                debug!(
                    "Matched session by ancestor tracking: PID {} (originally {}) -> Rule ID {}",
                    current_pid, pid, rule_id
                );
                forced_rule_id = Some(rule_id);
                break;
            }

            if let Some(info) = crate::process::monitor::get_process_by_pid(current_pid)
                .ok()
                .flatten()
            {
                if let Some(env_rule_id) = info.rule_id {
                    forced_rule_id = Some(env_rule_id);
                    break;
                }

                if let Some(ppid) = info.ppid {
                    current_pid = ppid as i32;
                } else {
                    break;
                }
            } else {
                break;
            }
            depth += 1;
        }
        drop(sessions);

        let proc_info = match crate::process::monitor::get_process_by_pid(pid)
            .ok()
            .flatten()
        {
            Some(info) => info,
            None => {
                debug!("No process info for PID {}", pid);
                self.cache.write().await.insert(cache_key, None);
                return None;
            }
        };
        let exe_path = proc_info.exe_path.unwrap_or_default();
        let app_name = proc_info.name;

        if forced_rule_id.is_none() {
            forced_rule_id = proc_info.rule_id;
        }

        debug!(
            "PID {} -> {} ({}, rule_id: {:?})",
            pid, app_name, exe_path, forced_rule_id
        );

        let rules = self.rules.read().await;
        let dns_servers = self.dns_servers.read().await;

        if let Some(id) = forced_rule_id {
            if let Some(rule) = rules.iter().find(|r| r.id == Some(id)) {
                if rule.enabled {
                    if let Some(server) = dns_servers.get(&rule.dns_server_id) {
                        debug!(
                            "Rule matched (forced by ID {}): {} -> {} ({})",
                            id, app_name, server.name, server.address
                        );
                        self.cache
                            .write()
                            .await
                            .insert(cache_key, Some(server.clone()));
                        return Some(server.clone());
                    }
                }
            }
        }

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let matches = if let Some(ref path) = rule.app_path {
                exe_path.contains(path) || app_name.contains(&rule.app_name)
            } else {
                app_name.contains(&rule.app_name)
            };

            if matches {
                if let Some(server) = dns_servers.get(&rule.dns_server_id) {
                    debug!(
                        "Rule matched: {} -> {} ({})",
                        app_name, server.name, server.address
                    );
                    self.cache
                        .write()
                        .await
                        .insert(cache_key, Some(server.clone()));
                    return Some(server.clone());
                }
            }
        }

        debug!(
            "No rule matched for PID {} ({}), using default",
            pid, app_name
        );
        self.cache.write().await.insert(cache_key, None);
        None
    }

    pub async fn lookup_by_app_name(&self, app_name: &str) -> Option<DnsServer> {
        let cache_key = format!("app:{}", app_name);
        if let Some(cached) = self.cache.read().await.get(&cache_key) {
            return cached.clone();
        }

        let rules = self.rules.read().await;
        let dns_servers = self.dns_servers.read().await;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if rule.app_name == app_name
                || rule
                    .app_path
                    .as_ref()
                    .map(|p| app_name.contains(p))
                    .unwrap_or(false)
            {
                if let Some(server) = dns_servers.get(&rule.dns_server_id) {
                    self.cache
                        .write()
                        .await
                        .insert(cache_key, Some(server.clone()));
                    return Some(server.clone());
                }
            }
        }

        self.cache.write().await.insert(cache_key, None);
        None
    }

    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new(60)
    }
}
