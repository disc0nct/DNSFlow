use crate::db::config::DnsFlowConfig;
use crate::db::database::Database;
use crate::debug;
use crate::state::AppConfig;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_config(db: State<'_, Arc<Database>>) -> Result<AppConfig, String> {
    db.get_config()
}

#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    db: State<'_, Arc<Database>>,
) -> Result<bool, String> {
    db.update_config(&config)?;
    debug::set(config.debug);
    Ok(true)
}

#[tauri::command]
pub async fn export_config(db: State<'_, Arc<Database>>) -> Result<String, String> {
    let app_config = db.get_config()?;
    let dns_servers = db.get_dns_servers()?;
    let app_rules = db.get_rules()?;

    let config = DnsFlowConfig::new(app_config, dns_servers, app_rules);
    config.to_json().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_config(json: String, db: State<'_, Arc<Database>>) -> Result<bool, String> {
    let config = DnsFlowConfig::from_json(&json).map_err(|e| e.to_string())?;

    if config.version != "1.0" {
        return Err(format!("Unsupported config version: {}", config.version));
    }

    db.clear_all_data()?;
    db.update_config(&config.app_config)?;

    let mut server_id_map = std::collections::HashMap::new();

    for server in &config.dns_servers {
        let old_id = server.id;
        let new_server = db.add_dns_server(
            &server.name,
            &server.address,
            server.secondary_address.as_deref(),
            &server.protocol,
        )?;
        if let (Some(old), Some(new)) = (old_id, new_server.id) {
            server_id_map.insert(old, new);
        }
    }

    for rule in &config.app_rules {
        let new_server_id = server_id_map
            .get(&rule.dns_server_id)
            .copied()
            .unwrap_or(rule.dns_server_id);

        db.add_rule(
            &rule.app_name,
            rule.app_path.as_deref(),
            new_server_id,
            rule.use_ld_preload,
        )?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn save_config_file(json: String, app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;

    let mut save_dir = match app.path().download_dir() {
        Ok(p) => p,
        Err(_) => std::path::PathBuf::from("/tmp"),
    };

    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let user_download = std::path::PathBuf::from(format!("/home/{}/Downloads", sudo_user));
        if user_download.exists() {
            save_dir = user_download;
        } else {
            let user_home = std::path::PathBuf::from(format!("/home/{}", sudo_user));
            if user_home.exists() {
                save_dir = user_home;
            }
        }
    }

    let filename = format!(
        "dnsflow-config-{}.json",
        chrono::Local::now().format("%Y-%m-%d-%H%M%S")
    );
    let file_path = save_dir.join(filename);

    std::fs::write(&file_path, json)
        .map_err(|e| format!("Failed to write to {:?}: {}", file_path, e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));

        if let Ok(uid_str) = std::env::var("SUDO_UID") {
            if let (Ok(uid), Ok(gid_str)) = (uid_str.parse::<u32>(), std::env::var("SUDO_GID")) {
                if let Ok(gid) = gid_str.parse::<u32>() {
                    let _ = std::process::Command::new("chown")
                        .arg(format!("{}:{}", uid, gid))
                        .arg(&file_path)
                        .status();
                }
            }
        }
    }

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn reset_config(db: State<'_, Arc<Database>>) -> Result<bool, String> {
    db.reset_to_defaults()?;
    Ok(true)
}
