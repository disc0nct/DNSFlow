use crate::state::{AppState, ProcessInfo};
use tauri::State;

#[tauri::command]
pub async fn get_running_processes(state: State<'_, AppState>) -> Result<Vec<ProcessInfo>, String> {
    state
        .platform
        .process_enumerator()
        .enumerate()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_process_info(
    state: State<'_, AppState>,
    pid: i64,
) -> Result<Option<ProcessInfo>, String> {
    state
        .platform
        .process_enumerator()
        .by_pid(pid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_system_dns(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .platform
        .dns_detector()
        .detect_system_dns()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn kill_process(state: State<'_, AppState>, pid: i64) -> Result<bool, String> {
    use std::process::Command;

    state.rules_engine.untrack_session(pid as u32).await;

    // Get the session ID of the process
    let sid_output = Command::new("ps")
        .arg("-o")
        .arg("sid=")
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to get session ID: {}", e))?;

    let sid = String::from_utf8_lossy(&sid_output.stdout)
        .trim()
        .to_string();

    if !sid.is_empty() {
        // Kill entire session (all processes in the same session group)
        let _ = Command::new("kill")
            .arg("-9")
            .arg(format!("-{}", sid))
            .status();
    }

    // Also kill the specific PID as fallback
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .map_err(|e| format!("Failed to execute kill command: {}", e))?;

    Ok(status.success())
}
