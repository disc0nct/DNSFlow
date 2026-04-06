/// Process information lookup utility
/// Kept for potential future use — commands currently use free functions from platform/.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProcessLookup {
    /// Cache of PID to process name mappings
    pub cache: std::collections::HashMap<u32, String>,
}

/// Result of a process lookup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LookupResult {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: Option<String>,
    /// Executable path
    pub exe_path: Option<String>,
    /// Command line arguments
    pub cmdline: Option<String>,
}

impl ProcessLookup {
    /// Create a new process lookup utility
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Look up process information by PID
    pub fn lookup(&mut self, pid: u32) -> Result<LookupResult, String> {
        #[cfg(target_os = "linux")]
        {
            let proc = procfs::process::Process::new(pid as i32).map_err(|e| e.to_string())?;
            let stat = proc.stat().map_err(|e| e.to_string())?;
            let exe_path = proc.exe().ok().map(|p| p.to_string_lossy().to_string());
            let cmdline = proc.cmdline().ok().map(|c| c.join(" "));
            self.cache.insert(pid, stat.comm.clone());
            Ok(LookupResult {
                pid,
                name: Some(stat.comm),
                exe_path,
                cmdline,
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(LookupResult {
                pid,
                name: self.cache.get(&pid).cloned(),
                exe_path: None,
                cmdline: None,
            })
        }
    }

    /// Look up process name by PID
    pub fn get_name(&mut self, pid: u32) -> Option<String> {
        if let Some(name) = self.cache.get(&pid) {
            return Some(name.clone());
        }
        #[cfg(target_os = "linux")]
        {
            let proc = procfs::process::Process::new(pid as i32).ok()?;
            let stat = proc.stat().ok()?;
            self.cache.insert(pid, stat.comm.clone());
            Some(stat.comm)
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Clear the lookup cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for ProcessLookup {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps source port to owning process ID via /proc filesystem
#[cfg(target_os = "linux")]
pub fn lookup_pid_by_socket(local_port: u16) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    use crate::platform::linux::LinuxSocketLookup;
    use crate::platform::SocketLookup;
    LinuxSocketLookup.pid_by_port(local_port)
}

/// Maps source port to owning process ID (non-Linux stub)
#[cfg(not(target_os = "linux"))]
pub fn lookup_pid_by_socket(_local_port: u16) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    Ok(None)
}
