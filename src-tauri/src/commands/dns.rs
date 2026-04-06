use crate::db::database::Database;
use crate::debug;
use crate::dns::proxy::DnsProxyServer;
use crate::state::{AppState, DnsQueryLog, DnsServer};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_dns_servers(db: State<'_, Arc<Database>>) -> Result<Vec<DnsServer>, String> {
    db.get_dns_servers()
}

#[tauri::command]
pub async fn add_dns_server(
    name: String,
    address: String,
    secondary_address: Option<String>,
    protocol: String,
    db: State<'_, Arc<Database>>,
) -> Result<DnsServer, String> {
    let result = db.add_dns_server(&name, &address, secondary_address.as_deref(), &protocol);
    debug!(
        "add_dns_server: name={}, address={}, secondary={:?}, protocol={}, result={:?}",
        name, address, secondary_address, protocol, result
    );
    result
}

#[tauri::command]
pub async fn update_dns_server(
    id: i64,
    name: String,
    address: String,
    secondary_address: Option<String>,
    protocol: String,
    db: State<'_, Arc<Database>>,
) -> Result<DnsServer, String> {
    db.update_dns_server(id, &name, &address, secondary_address.as_deref(), &protocol)
}

#[tauri::command]
pub async fn set_default_dns_server(id: i64, db: State<'_, Arc<Database>>) -> Result<bool, String> {
    db.set_default_dns_server(id)
}

#[tauri::command]
pub async fn delete_dns_server(id: i64, db: State<'_, Arc<Database>>) -> Result<bool, String> {
    db.delete_dns_server(id)
}

pub async fn internal_start_dns_proxy(state: &AppState) -> Result<bool, String> {
    use crate::dns::proxy::{write_proxy_resolv_conf, PROXY_FALLBACK_IPS, PROXY_LISTEN_PORT};

    let mut proxy_guard = state.proxy.write().await;

    if proxy_guard.is_some() {
        return Ok(true);
    }

    let db = state.db.clone().ok_or("Database not available")?;

    let mut last_error = String::new();

    for &ip in PROXY_FALLBACK_IPS {
        if let Err(e) = write_proxy_resolv_conf(ip) {
            last_error = format!("Failed to write resolv.conf for IP {}: {}", ip, e);
            continue;
        }

        let addr = format!("{}:{}", ip, PROXY_LISTEN_PORT);
        let proxy = match DnsProxyServer::new(&addr).await {
            Ok(p) => p,
            Err(e) => {
                crate::dns::proxy::cleanup_proxy_resolv_conf();
                last_error = format!("Failed to create proxy on {}: {}", addr, e);
                continue;
            }
        };

        match proxy
            .start_with_rules(
                state.rules_engine.clone(),
                state.query_logger.clone(),
                db.clone(),
            )
            .await
        {
            Ok(_) => {
                *proxy_guard = Some(proxy);
                return Ok(true);
            }
            Err(e) => {
                crate::dns::proxy::cleanup_proxy_resolv_conf();
                last_error = format!("Failed to start proxy on {}: {}", addr, e);
                continue;
            }
        }
    }

    Err(last_error)
}

#[tauri::command]
pub async fn start_dns_proxy(state: State<'_, AppState>) -> Result<bool, String> {
    internal_start_dns_proxy(&state).await
}

#[tauri::command]
pub async fn stop_dns_proxy(state: State<'_, AppState>) -> Result<bool, String> {
    let mut proxy_guard = state.proxy.write().await;

    if let Some(proxy) = proxy_guard.take() {
        proxy.stop().await.map_err(|e| e.to_string())?;
    }

    crate::dns::proxy::cleanup_proxy_resolv_conf();

    Ok(true)
}

#[tauri::command]
pub async fn get_dns_status(state: State<'_, AppState>) -> Result<String, String> {
    let proxy_guard = state.proxy.read().await;

    if proxy_guard.is_some() {
        Ok("running".to_string())
    } else {
        Ok("stopped".to_string())
    }
}

#[tauri::command]
pub async fn get_query_logs(
    limit: Option<i64>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<DnsQueryLog>, String> {
    db.get_query_logs(limit.unwrap_or(50))
}

#[tauri::command]
pub async fn clear_query_logs(db: State<'_, Arc<Database>>) -> Result<bool, String> {
    db.clear_query_logs()
}
