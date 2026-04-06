use super::*;
use std::error::Error;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use windivert::{WinDivert, WinDivertAddress, WinDivertFlags, WinDivertLayer};

use crate::state::{DnsEvent, ProcessInfo};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedUdpTable, GetNetworkParams, FIXED_INFO, MIB_UDPTABLE_OWNER_PID, UDP_TABLE_OWNER_PID,
};
use windows::Win32::Networking::WinSock::AF_INET;
use windows::Win32::Security::{
    GetTokenInformation, OpenProcessToken, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows::Win32::System::ProcessStatus::QueryFullProcessImageNameW;
use windows::Win32::System::Threading::{
    CreateToolhelp32Snapshot, GetCurrentProcess, OpenProcess, Process32FirstW, Process32NextW,
    PROCESSENTRY32W, PROCESS_QUERY_LIMITED_INFORMATION, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};

pub struct WindowsProcessEnumerator;

impl ProcessEnumerator for WindowsProcessEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        let mut processes = Vec::new();

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let name_len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                    let exe_path = get_exe_path(entry.th32ProcessID);

                    processes.push(ProcessInfo {
                        pid: entry.th32ProcessID as i64,
                        ppid: Some(entry.th32ParentProcessID as i64),
                        name,
                        exe_path,
                        cmdline: None,
                        rule_id: None,
                    });

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

        processes.sort_by_key(|p| p.pid);
        Ok(processes)
    }

    fn by_pid(&self, pid: i64) -> Result<Option<ProcessInfo>, Box<dyn Error>> {
        let processes = self.enumerate()?;
        Ok(processes.into_iter().find(|p| p.pid == pid))
    }
}

fn get_exe_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;

        let result =
            QueryFullProcessImageNameW(handle, 0, windows::core::PWSTR(buf.as_mut_ptr()), &mut len);

        let _ = CloseHandle(handle);

        if result.is_ok() {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}

pub struct WindowsSocketLookup;

impl SocketLookup for WindowsSocketLookup {
    fn pid_by_port(&self, local_port: u16) -> Result<Option<i32>, Box<dyn Error>> {
        unsafe {
            let mut buf_size: u32 = 0;

            let _ = GetExtendedUdpTable(
                None,
                &mut buf_size,
                false,
                AF_INET.0 as u32,
                UDP_TABLE_OWNER_PID,
                0,
            );

            if buf_size == 0 {
                return Ok(None);
            }

            let mut buf = vec![0u8; buf_size as usize];

            let result = GetExtendedUdpTable(
                Some(buf.as_mut_ptr() as *mut _),
                &mut buf_size,
                false,
                AF_INET.0 as u32,
                UDP_TABLE_OWNER_PID,
                0,
            );

            if result.is_err() {
                return Ok(None);
            }

            let table = &*(buf.as_ptr() as *const MIB_UDPTABLE_OWNER_PID);
            let rows =
                std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);

            for row in rows {
                let port = u16::from_be(row.dwLocalPort as u16);
                if port == local_port {
                    return Ok(Some(row.dwOwningPid as i32));
                }
            }

            Ok(None)
        }
    }
}

pub struct WindowsDnsDetector;

impl DnsDetector for WindowsDnsDetector {
    fn detect_system_dns(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut dns_servers = Vec::new();

        unsafe {
            let mut buf_size: u32 = 0;

            let _ = GetNetworkParams(None, &mut buf_size);

            if buf_size == 0 {
                return Ok(dns_servers);
            }

            let mut buf = vec![0u8; buf_size as usize];
            let result = GetNetworkParams(Some(buf.as_mut_ptr() as *mut _), &mut buf_size);
            if result.is_err() {
                return Ok(dns_servers);
            }

            let fixed_info = &*(buf.as_ptr() as *const FIXED_INFO);

            let mut current = &fixed_info.DnsServerList as *const _;
            while !current.is_null() {
                let addr = &*current;

                let ip_bytes: &[i8] = &addr.IpAddress.String;
                let ip_cstr = std::ffi::CStr::from_ptr(ip_bytes.as_ptr());
                let ip = ip_cstr.to_string_lossy().to_string();

                if !ip.is_empty() && ip != "0.0.0.0" {
                    dns_servers.push(ip);
                }

                current = addr.Next;
            }
        }

        Ok(dns_servers)
    }
}

pub struct WinDivertInterceptor {
    is_active: Arc<Mutex<bool>>,
}

impl WinDivertInterceptor {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(Mutex::new(false)),
        }
    }
}

impl DnsInterceptor for WinDivertInterceptor {
    fn start(&mut self, event_tx: mpsc::Sender<DnsEvent>) -> Result<(), Box<dyn Error>> {
        let filter = "outbound and udp.DstPort == 53";
        let divert = WinDivert::open(filter, WinDivertLayer::Network, 0, WinDivertFlags::None)?;

        let is_active = self.is_active.clone();
        {
            let mut guard = is_active.lock().unwrap();
            *guard = true;
        }

        std::thread::spawn(move || {
            let mut packet = vec![0u8; 65535];
            let mut addr = WinDivertAddress::default();

            loop {
                {
                    let guard = is_active.lock().unwrap();
                    if !*guard {
                        break;
                    }
                }

                if let Ok(recv_len) = divert.recv(Some(&mut packet), Some(&mut addr)) {
                    let buf = &mut packet[..recv_len];

                    if let Ok(info) = dns_packet::parse_dns_packet(buf) {
                        let lookup = WindowsSocketLookup;
                        let pid = lookup.pid_by_port(info.src_port).ok().flatten();

                        let event = DnsEvent {
                            pid: pid.unwrap_or(0) as i64,
                            tgid: 0,
                            uid: 0,
                            gid: 0,
                            comm: String::new(),
                            daddr: info.dst_addr.to_string(),
                            dport: info.dst_port as i64,
                            is_dns: true,
                        };

                        let _ = event_tx.blocking_send(event);

                        buf[16] = 127;
                        buf[17] = 0;
                        buf[18] = 0;
                        buf[19] = 53;

                        divert.calc_checksums(buf, &addr, WinDivertFlags::None);
                    }

                    let _ = divert.send(buf, &addr);
                }
            }
        });

        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let mut guard = self.is_active.lock().unwrap();
        *guard = false;
        Ok(())
    }

    fn is_active(&self) -> bool {
        *self.is_active.lock().unwrap()
    }
}

pub struct WindowsAppLauncher;

impl AppLauncher for WindowsAppLauncher {
    fn launch(&self, app_path: &str, _rule_id: Option<i64>) -> Result<u32, Box<dyn Error>> {
        let child = Command::new(app_path).spawn()?;
        Ok(child.id())
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token_handle = INVALID_HANDLE_VALUE;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );

        let _ = CloseHandle(token_handle);

        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

pub fn relaunch_elevated() -> Result<(), Box<dyn Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_wide: Vec<u16> = exe_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let verb: Vec<u16> = "runas\0".encode_utf16().collect();

    unsafe {
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: PCWSTR(verb.as_ptr()),
            lpFile: PCWSTR(exe_path_wide.as_ptr()),
            nShow: 1,
            ..Default::default()
        };

        ShellExecuteExW(&mut sei)?;
    }

    Ok(())
}

pub fn ensure_elevated() -> Result<(), Box<dyn Error>> {
    if !is_elevated() {
        relaunch_elevated()?;
        std::process::exit(0);
    }
    Ok(())
}

pub fn create_windows_platform() -> Platform {
    Platform {
        process_enumerator: Box::new(WindowsProcessEnumerator),
        socket_lookup: Box::new(WindowsSocketLookup),
        dns_detector: Box::new(WindowsDnsDetector),
        dns_interceptor: Box::new(WinDivertInterceptor::new()),
        app_launcher: Box::new(WindowsAppLauncher),
    }
}
