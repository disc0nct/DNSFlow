use crate::commands::dns::internal_start_dns_proxy;
use crate::db::database::Database;
use crate::debug;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn launch_with_shim(
    app_path: String,
    rule_id: Option<i64>,
    state: State<'_, AppState>,
    db: State<'_, Arc<Database>>,
) -> Result<u32, String> {
    let rules = db.get_rules().map_err(|e| e.to_string())?;
    let proxy_addr = state
        .proxy
        .read()
        .await
        .as_ref()
        .map(|p| p.bound_addr().to_string());
    state
        .rules_engine
        .load_rules(rules, proxy_addr.as_deref())
        .await;

    let servers = db.get_dns_servers().map_err(|e| e.to_string())?;
    state.rules_engine.load_dns_servers(servers).await;

    internal_start_dns_proxy(&state).await?;

    let pid = state
        .platform
        .launcher()
        .launch(&app_path, rule_id)
        .map_err(|e| format!("Failed to launch '{}': {}", app_path, e))?;

    if let Some(id) = rule_id {
        state.rules_engine.track_session(pid, id).await;
    }

    debug!("Launched PID {} with rule_id {:?}", pid, rule_id);
    Ok(pid)
}
