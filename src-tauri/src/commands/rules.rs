use crate::db::database::Database;
use crate::state::{AppRule, AppState};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_rules(db: State<'_, Arc<Database>>) -> Result<Vec<AppRule>, String> {
    db.get_rules()
}

#[tauri::command]
pub async fn get_active_rule_sessions(
    state: State<'_, AppState>,
) -> Result<HashMap<i64, u32>, String> {
    Ok(state.rules_engine.get_active_sessions().await)
}

#[tauri::command]
pub async fn add_rule(
    app_name: String,
    app_path: Option<String>,
    dns_server_id: i64,
    use_ld_preload: bool,
    db: State<'_, Arc<Database>>,
) -> Result<AppRule, String> {
    db.add_rule(
        &app_name,
        app_path.as_deref(),
        dns_server_id,
        use_ld_preload,
    )
}

#[tauri::command]
pub async fn update_rule(
    id: i64,
    app_name: String,
    app_path: Option<String>,
    dns_server_id: i64,
    use_ld_preload: bool,
    db: State<'_, Arc<Database>>,
) -> Result<AppRule, String> {
    db.update_rule(
        id,
        &app_name,
        app_path.as_deref(),
        dns_server_id,
        use_ld_preload,
    )
}

#[tauri::command]
pub async fn toggle_rule(
    id: i64,
    enabled: bool,
    db: State<'_, Arc<Database>>,
) -> Result<bool, String> {
    db.toggle_rule(id, enabled)
}

#[tauri::command]
pub async fn delete_rule(id: i64, db: State<'_, Arc<Database>>) -> Result<bool, String> {
    db.delete_rule(id)
}
