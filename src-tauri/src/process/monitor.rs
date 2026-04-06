use crate::state::ProcessInfo;
use std::collections::HashMap;

/// Process monitor for tracking DNS-related processes
/// Kept for potential future use — commands currently use free functions from platform/.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProcessMonitor {
    /// Map of PID to process information
    pub tracked_processes: HashMap<u32, ProcessEntry>,
    /// Whether monitoring is active
    pub is_active: bool,
}

/// Entry for a tracked process
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessEntry {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Executable path
    pub exe_path: Option<String>,
    /// Number of DNS queries made
    pub query_count: u64,
}

impl ProcessMonitor {
    /// Create a new process monitor
    pub fn new() -> Self {
        Self {
            tracked_processes: HashMap::new(),
            is_active: false,
        }
    }

    /// Start monitoring processes
    pub async fn start(&mut self) -> Result<(), String> {
        match get_running_processes() {
            Ok(processes) => {
                for proc in processes {
                    self.tracked_processes.insert(
                        proc.pid as u32,
                        ProcessEntry {
                            pid: proc.pid as u32,
                            name: proc.name,
                            exe_path: proc.exe_path,
                            query_count: 0,
                        },
                    );
                }
            }
            Err(e) => {
                return Err(format!("Failed to scan processes: {}", e));
            }
        }
        self.is_active = true;
        Ok(())
    }

    /// Stop monitoring processes
    pub async fn stop(&mut self) -> Result<(), String> {
        self.tracked_processes.clear();
        self.is_active = false;
        Ok(())
    }

    /// Track a new process
    pub fn track_process(&mut self, pid: u32, name: String, exe_path: Option<String>) {
        let entry = ProcessEntry {
            pid,
            name,
            exe_path,
            query_count: 0,
        };
        self.tracked_processes.insert(pid, entry);
    }

    /// Increment query count for a process
    pub fn increment_query_count(&mut self, pid: u32) {
        if let Some(entry) = self.tracked_processes.get_mut(&pid) {
            entry.query_count += 1;
        }
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get all running processes from /proc
#[cfg(target_os = "linux")]
pub fn get_running_processes() -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    use crate::platform::linux::LinuxProcessEnumerator;
    use crate::platform::ProcessEnumerator;
    LinuxProcessEnumerator.enumerate()
}

/// Get all running processes (non-Linux stub)
#[cfg(not(target_os = "linux"))]
pub fn get_running_processes() -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    Ok(Vec::new())
}

/// Get a specific process by PID
#[cfg(target_os = "linux")]
pub fn get_process_by_pid(pid: i32) -> Result<Option<ProcessInfo>, Box<dyn std::error::Error>> {
    use crate::platform::linux::LinuxProcessEnumerator;
    use crate::platform::ProcessEnumerator;
    LinuxProcessEnumerator.by_pid(pid as i64)
}

/// Get a specific process by PID (non-Linux stub)
#[cfg(not(target_os = "linux"))]
pub fn get_process_by_pid(_pid: i32) -> Result<Option<ProcessInfo>, Box<dyn std::error::Error>> {
    Ok(None)
}

/// Detect system DNS servers from /etc/resolv.conf or resolvectl
#[cfg(target_os = "linux")]
pub fn detect_system_dns() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use crate::platform::linux::LinuxDnsDetector;
    use crate::platform::DnsDetector;
    LinuxDnsDetector.detect_system_dns()
}

/// Detect system DNS servers (non-Linux stub)
#[cfg(not(target_os = "linux"))]
pub fn detect_system_dns() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(Vec::new())
}
