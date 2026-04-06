use crate::state::{DnsEvent, ProcessInfo};
use std::error::Error;
use tokio::sync::mpsc;

/// Trait for enumerating running processes
pub trait ProcessEnumerator: Send + Sync {
    fn enumerate(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>>;
    fn by_pid(&self, pid: i64) -> Result<Option<ProcessInfo>, Box<dyn Error>>;
}

/// Trait for mapping network ports to process IDs
pub trait SocketLookup: Send + Sync {
    fn pid_by_port(&self, local_port: u16) -> Result<Option<i32>, Box<dyn Error>>;
}

/// Trait for detecting system DNS configuration
pub trait DnsDetector: Send + Sync {
    fn detect_system_dns(&self) -> Result<Vec<String>, Box<dyn Error>>;
}

/// Trait for DNS traffic interception
pub trait DnsInterceptor: Send + Sync {
    fn start(&mut self, event_tx: mpsc::Sender<DnsEvent>) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    fn is_active(&self) -> bool;
}

/// Trait for launching applications with DNS interception
pub trait AppLauncher: Send + Sync {
    fn launch(&self, app_path: &str, rule_id: Option<i64>) -> Result<u32, Box<dyn Error>>;
}

/// Platform abstraction container
pub struct Platform {
    pub process_enumerator: Box<dyn ProcessEnumerator>,
    pub socket_lookup: Box<dyn SocketLookup>,
    pub dns_detector: Box<dyn DnsDetector>,
    pub dns_interceptor: Box<dyn DnsInterceptor>,
    pub app_launcher: Box<dyn AppLauncher>,
}

impl Platform {
    pub fn process_enumerator(&self) -> &dyn ProcessEnumerator {
        self.process_enumerator.as_ref()
    }

    pub fn socket_lookup(&self) -> &dyn SocketLookup {
        self.socket_lookup.as_ref()
    }

    pub fn dns_detector(&self) -> &dyn DnsDetector {
        self.dns_detector.as_ref()
    }

    pub fn dns_interceptor(&self) -> &dyn DnsInterceptor {
        self.dns_interceptor.as_ref()
    }

    pub fn launcher(&self) -> &dyn AppLauncher {
        self.app_launcher.as_ref()
    }
}

/// Platform-specific implementations
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "windows")]
pub mod dns_packet;

/// Create platform-specific implementation
pub fn create_platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        linux::create_linux_platform()
    }

    #[cfg(target_os = "windows")]
    {
        windows::create_windows_platform()
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("DNSFlow only supports Linux and Windows");
