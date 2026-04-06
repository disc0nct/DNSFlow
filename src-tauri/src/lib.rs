pub mod commands;
pub mod db;
pub mod debug;
pub mod dns;
#[cfg(target_os = "linux")]
pub mod ebpf;
pub mod platform;
pub mod process;
pub mod state;

use commands::*;
use db::database::Database;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::platform::windows::ensure_elevated() {
            eprintln!("Failed to elevate: {}", e);
            std::process::exit(1);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            eprintln!("[DNSFlow] Starting setup...");
            let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            eprintln!("[DNSFlow] App data dir: {:?}", app_data_dir);
            std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

            let db_path = app_data_dir.join("dnsflow.db");
            eprintln!("[DNSFlow] DB path: {:?}", db_path);
            let database = Database::new(db_path.to_str().unwrap()).map_err(|e| e.to_string())?;
            eprintln!("[DNSFlow] Database created");
            database.initialize().map_err(|e| e.to_string())?;
            eprintln!("[DNSFlow] Database initialized");
            database.seed_if_empty().map_err(|e| e.to_string())?;
            eprintln!("[DNSFlow] Database seeded");

            let config = database.get_config().map_err(|e| e.to_string())?;
            debug::init(config.debug);

            let rules = database.get_rules().unwrap_or_default();
            let servers = database.get_dns_servers().unwrap_or_default();

            let db_arc = std::sync::Arc::new(database);

            let mut app_state = AppState::default();
            app_state.config = config.clone();
            app_state.db = Some(db_arc.clone());

            // Load rules/servers synchronously before runtime starts
            let rules_engine = app_state.rules_engine.clone();
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .map_err(|e| e.to_string())?;
            rt.block_on(async {
                rules_engine.load_rules(rules, None).await;
                rules_engine.load_dns_servers(servers).await;
            });

            app.manage(db_arc);
            eprintln!("[DNSFlow] Database managed");
            app.manage(app_state);
            eprintln!("[DNSFlow] AppState managed");
            eprintln!("[DNSFlow] Setup complete, app ready");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dns_servers,
            add_dns_server,
            update_dns_server,
            set_default_dns_server,
            delete_dns_server,
            start_dns_proxy,
            stop_dns_proxy,
            get_dns_status,
            get_running_processes,
            get_process_info,
            kill_process,
            get_system_dns,
            get_config,
            update_config,
            export_config,
            import_config,
            reset_config,
            save_config_file,
            launch_with_shim,
            get_rules,
            get_active_rule_sessions,
            add_rule,
            update_rule,
            toggle_rule,
            delete_rule,
            get_query_logs,
            clear_query_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
