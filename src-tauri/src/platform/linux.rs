use super::*;
use std::error::Error;

use crate::debug;
use aya::maps::perf::AsyncPerfEventArray;
use aya::programs::kprobe::KProbe;
#[allow(deprecated)]
use aya::Ebpf;
use bytes::BytesMut;
use std::net::Ipv4Addr;

pub struct LinuxProcessEnumerator;
pub struct LinuxSocketLookup;
pub struct LinuxDnsDetector;
pub struct LinuxDnsInterceptor {
    is_active: bool,
}
pub struct LinuxAppLauncher;

pub fn create_linux_platform() -> Platform {
    Platform {
        process_enumerator: Box::new(LinuxProcessEnumerator),
        socket_lookup: Box::new(LinuxSocketLookup),
        dns_detector: Box::new(LinuxDnsDetector),
        dns_interceptor: Box::new(LinuxDnsInterceptor { is_active: false }),
        app_launcher: Box::new(LinuxAppLauncher),
    }
}

impl ProcessEnumerator for LinuxProcessEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        use procfs::process::all_processes;

        let mut processes = Vec::new();

        for proc_result in all_processes()? {
            let proc = match proc_result {
                Ok(p) => p,
                Err(_) => continue,
            };

            let stat = match proc.stat() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let exe_path = proc.exe().ok().map(|p| p.to_string_lossy().to_string());
            let cmdline = proc.cmdline().ok().map(|c| c.join(" "));

            let rule_id = proc.environ().ok().and_then(|env| {
                env.get(std::ffi::OsStr::new("DNSFLOW_RULE_ID"))
                    .and_then(|s| s.to_string_lossy().parse::<i64>().ok())
            });

            processes.push(ProcessInfo {
                pid: proc.pid() as i64,
                ppid: Some(stat.ppid as i64),
                name: stat.comm,
                exe_path,
                cmdline,
                rule_id,
            });
        }

        processes.sort_by_key(|p| p.pid);
        Ok(processes)
    }

    fn by_pid(&self, pid: i64) -> Result<Option<ProcessInfo>, Box<dyn Error>> {
        let proc = procfs::process::Process::new(pid as i32)?;
        let stat = proc.stat()?;

        let exe_path = proc.exe().ok().map(|p| p.to_string_lossy().to_string());
        let cmdline = proc.cmdline().ok().map(|c| c.join(" "));

        let rule_id = proc.environ().ok().and_then(|env| {
            env.get(std::ffi::OsStr::new("DNSFLOW_RULE_ID"))
                .and_then(|s| s.to_string_lossy().parse::<i64>().ok())
        });

        Ok(Some(ProcessInfo {
            pid: proc.pid() as i64,
            ppid: Some(stat.ppid as i64),
            name: stat.comm,
            exe_path,
            cmdline,
            rule_id,
        }))
    }
}

impl DnsDetector for LinuxDnsDetector {
    fn detect_system_dns(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut dns_servers = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("nameserver") {
                    if let Some(ip) = line.split_whitespace().nth(1) {
                        dns_servers.push(ip.to_string());
                    }
                }
            }
        }

        if dns_servers.is_empty() {
            if let Ok(output) = std::process::Command::new("resolvectl")
                .arg("status")
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("DNS Servers:") {
                        if let Some(servers) = line.split(':').nth(1) {
                            for server in servers.split_whitespace() {
                                dns_servers.push(server.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(dns_servers)
    }
}

impl SocketLookup for LinuxSocketLookup {
    fn pid_by_port(&self, local_port: u16) -> Result<Option<i32>, Box<dyn Error>> {
        use procfs::net::{tcp, udp};
        use procfs::process::{all_processes, FDTarget};
        use std::collections::HashMap;

        let mut inode_map: HashMap<u64, i32> = HashMap::new();

        for proc_result in all_processes()? {
            let proc = match proc_result {
                Ok(p) => p,
                Err(_) => continue,
            };

            if let Ok(fds) = proc.fd() {
                for fd in fds.flatten() {
                    if let FDTarget::Socket(inode) = fd.target {
                        inode_map.insert(inode, proc.pid());
                    }
                }
            }
        }

        if let Ok(entries) = udp() {
            for entry in entries {
                if entry.local_address.port() == local_port {
                    if let Some(&pid) = inode_map.get(&entry.inode) {
                        return Ok(Some(pid));
                    }
                }
            }
        }

        if let Ok(entries) = tcp() {
            for entry in entries {
                if entry.local_address.port() == local_port {
                    if let Some(&pid) = inode_map.get(&entry.inode) {
                        return Ok(Some(pid));
                    }
                }
            }
        }

        Ok(None)
    }
}

impl DnsInterceptor for LinuxDnsInterceptor {
    fn start(&mut self, event_tx: mpsc::Sender<DnsEvent>) -> Result<(), Box<dyn Error>> {
        let bpf_path = "target/bpfel-unknown-none/debug/dnsflow-ebpf";

        let mut bpf = match Ebpf::load_file(bpf_path) {
            Ok(bpf) => bpf,
            Err(e) => {
                debug!("Failed to load BPF: {}", e);
                return Err(e.into());
            }
        };

        let program: &mut KProbe = bpf
            .program_mut("dnsflow_kprobe")
            .ok_or("BPF program not found")?
            .try_into()?;
        program.load()?;
        program.attach("udp_sendmsg", 0)?;

        let mut events: AsyncPerfEventArray<_> = bpf
            .take_map("EVENTS")
            .ok_or("BPF map not found")?
            .try_into()?;

        let mut buf = events.open(0, None)?;

        tokio::spawn(async move {
            let mut buffers: Vec<BytesMut> =
                (0..4).map(|_| BytesMut::with_capacity(1024)).collect();

            loop {
                match buf.read_events(&mut buffers).await {
                    Ok(event_data) => {
                        for i in 0..event_data.read {
                            let buf = &buffers[i];
                            if buf.len() < std::mem::size_of::<dnsflow_common::DnsEvent>() {
                                continue;
                            }

                            let ptr = buf.as_ptr() as *const dnsflow_common::DnsEvent;
                            let raw = unsafe { ptr.read_unaligned() };

                            let event = DnsEvent {
                                pid: raw.pid as i64,
                                tgid: raw.tgid as i64,
                                uid: raw.uid as i64,
                                gid: raw.gid as i64,
                                comm: String::from_utf8_lossy(&raw.comm)
                                    .trim_end_matches('\0')
                                    .to_string(),
                                daddr: Ipv4Addr::new(
                                    raw.daddr[0],
                                    raw.daddr[1],
                                    raw.daddr[2],
                                    raw.daddr[3],
                                )
                                .to_string(),
                                dport: raw.dport as i64,
                                is_dns: raw.is_dns,
                            };

                            if event_tx.send(event).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        debug!("BPF event error: {}", e);
                        continue;
                    }
                }
            }
        });

        self.is_active = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.is_active = false;
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl AppLauncher for LinuxAppLauncher {
    fn launch(&self, app_path: &str, rule_id: Option<i64>) -> Result<u32, Box<dyn Error>> {
        use crate::dns::proxy::PROXY_RESOLV_CONF;
        use std::env;

        let shim_path = Self::find_shim();

        let uid: u32 = env::var("SUDO_UID")
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or("SUDO_UID not set — must run via sudo")?;
        let _gid: u32 = env::var("SUDO_GID")
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or("SUDO_GID not set — must run via sudo")?;
        let user = env::var("SUDO_USER").map_err(|_| "SUDO_USER not set — must run via sudo")?;

        let home = format!("/home/{}", user);
        let runtime_dir = format!("/run/user/{}", uid);

        debug!("Launching '{}' as user {} ({})", app_path, user, uid);

        let mut env_parts: Vec<String> = vec![
            format!("HOME={}", home),
            format!("USER={}", user),
            format!("LOGNAME={}", user),
            format!("XDG_RUNTIME_DIR={}", runtime_dir),
            format!("DBUS_SESSION_BUS_ADDRESS=unix:path={}/bus", runtime_dir),
        ];

        if let Some(shim) = &shim_path {
            env_parts.push(format!("LD_PRELOAD={}", shim.to_string_lossy()));
        } else {
            debug!("Warning: libdnsflow_shim.so not found");
        }

        if let Ok(val) = env::var("DISPLAY") {
            env_parts.push(format!("DISPLAY={}", val));
        }
        if let Ok(val) = env::var("WAYLAND_DISPLAY") {
            env_parts.push(format!("WAYLAND_DISPLAY={}", val));
        }
        if let Ok(xauth) = env::var("XAUTHORITY") {
            if xauth.starts_with("/root/") {
                env_parts.push(format!("XAUTHORITY={}/.Xauthority", home));
            } else {
                env_parts.push(format!("XAUTHORITY={}", xauth));
            }
        } else {
            env_parts.push(format!("XAUTHORITY={}/.Xauthority", home));
        }

        let rules_path = crate::dns::rules::shim_rules_path();
        env_parts.push(format!("DNSFLOW_RULES={}", rules_path.display()));

        if let Some(id) = rule_id {
            env_parts.push(format!("DNSFLOW_RULE_ID={}", id));
        }

        Self::ensure_firefox_doh_disabled(app_path);

        let mut user_cmd = format!("exec setsid sudo -u {} env", user);
        for e in &env_parts {
            user_cmd.push_str(&format!(" {}", e));
        }
        user_cmd.push_str(&format!(" '{}'", app_path.replace('\'', "'\\''")));
        for arg in Self::browser_dns_args(app_path) {
            user_cmd.push_str(&format!(" '{}'", arg));
        }

        // Mount namespace: bind-mount our custom resolv.conf so ALL apps
        // (including browsers with internal DNS resolvers) use our proxy.
        // mount --make-rprivate / prevents leaking the bind mount outside the namespace.
        let inner_cmd = format!(
            "mount --make-rprivate / && mount --bind {} /etc/resolv.conf && {}",
            PROXY_RESOLV_CONF, user_cmd
        );

        debug!("Exec (mount ns): {}", inner_cmd);

        let child = std::process::Command::new("unshare")
            .arg("--mount")
            .arg("--propagation")
            .arg("private")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(&inner_cmd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", app_path, e))?;

        let pid = child.id();
        debug!("Launched PID {} (mount namespace, session leader)", pid);
        Ok(pid)
    }
}

impl LinuxAppLauncher {
    fn find_shim() -> Option<std::path::PathBuf> {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let candidates = [
            exe_dir.join("libdnsflow_shim.so"),
            exe_dir.join("../lib/dnsflow/libdnsflow_shim.so"),
            std::path::PathBuf::from("/usr/lib/dnsflow/libdnsflow_shim.so"),
            std::path::PathBuf::from("/usr/local/lib/dnsflow/libdnsflow_shim.so"),
            current_dir.join("libdnsflow_shim.so"),
            current_dir.join("../libdnsflow_shim.so"),
            current_dir.join("target/debug/libdnsflow_shim.so"),
            current_dir.join("target/release/libdnsflow_shim.so"),
            current_dir.join("../target/debug/libdnsflow_shim.so"),
            current_dir.join("../target/release/libdnsflow_shim.so"),
            current_dir.join("../../libdnsflow_shim.so"),
        ];

        candidates.iter().find(|p| p.exists()).cloned()
    }

    /// Detect browser type from executable path and return DNS-forcing args.
    /// Returns an empty vec for non-browser apps.
    fn browser_dns_args(app_path: &str) -> Vec<&'static str> {
        let lower = app_path.to_lowercase();
        let basename = lower.rsplit('/').next().unwrap_or(&lower);

        if basename.contains("chrome")
            || basename.contains("chromium")
            || basename.contains("brave")
            || basename.contains("edge")
            || basename.contains("vivaldi")
            || basename.contains("opera")
        {
            return vec![
                "--disable-features=AsyncDns,DnsOverHttps,SecureDns",
                "--dns-result-order=ipv4first",
            ];
        }

        // Firefox/derivatives: DoH disabled via policies.json (no valid CLI flag exists)
        vec![]
    }

    fn ensure_firefox_doh_disabled(app_path: &str) {
        let lower = app_path.to_lowercase();
        let basename = lower.rsplit('/').next().unwrap_or(&lower);

        let is_firefox = basename.contains("firefox")
            || basename.contains("librewolf")
            || basename.contains("waterfox")
            || basename.contains("palemoon");

        if !is_firefox {
            return;
        }

        let mut policy_dirs: Vec<std::path::PathBuf> = Vec::new();

        if basename.contains("firefox-esr") {
            policy_dirs.push(std::path::PathBuf::from("/etc/firefox-esr/policies"));
        }
        if basename.contains("firefox") && !basename.contains("firefox-esr") {
            policy_dirs.push(std::path::PathBuf::from("/etc/firefox/policies"));
        }
        if basename.contains("librewolf") {
            policy_dirs.push(std::path::PathBuf::from("/etc/librewolf/policies"));
        }
        if basename.contains("waterfox") {
            policy_dirs.push(std::path::PathBuf::from("/etc/waterfox/policies"));
        }
        if basename.contains("palemoon") {
            policy_dirs.push(std::path::PathBuf::from("/etc/palemoon/policies"));
        }

        if policy_dirs.is_empty() && is_firefox {
            policy_dirs.push(std::path::PathBuf::from("/etc/firefox/policies"));
            policy_dirs.push(std::path::PathBuf::from("/etc/firefox-esr/policies"));
        }

        // Full DoH/TRR disable policy:
        // - DNSOverHTTPS: enterprise policy to disable DoH UI and lock it
        // - Preferences: set network.trr.mode=5 (completely off, no fallback probing)
        //   and disable canary domain checks that trigger DoH connectivity tests
        let doh_policy = serde_json::json!({
            "DNSOverHTTPS": {
                "Enabled": false,
                "Locked": true
            },
            "Preferences": {
                "network.trr.mode": {
                    "Value": 5,
                    "Status": "locked"
                },
                "network.trr.bootstrapAddr": {
                    "Value": "",
                    "Status": "locked"
                },
                "network.connectivity-service.enabled": {
                    "Value": false,
                    "Status": "locked"
                },
                "doh-rollout.enabled": {
                    "Value": false,
                    "Status": "locked"
                }
            }
        });

        for dir in &policy_dirs {
            if let Err(e) = std::fs::create_dir_all(dir) {
                debug!("Failed to create policy dir {:?}: {}", dir, e);
                continue;
            }

            let policy_path = dir.join("policies.json");

            let final_policy = if let Ok(content) = std::fs::read_to_string(&policy_path) {
                if let Ok(mut existing) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(policies) =
                        existing.get_mut("policies").and_then(|p| p.as_object_mut())
                    {
                        if let Some(our_policies) = doh_policy.as_object() {
                            for (key, val) in our_policies {
                                policies.insert(key.clone(), val.clone());
                            }
                        }
                    } else {
                        existing = serde_json::json!({ "policies": doh_policy });
                    }
                    existing
                } else {
                    serde_json::json!({ "policies": doh_policy })
                }
            } else {
                serde_json::json!({ "policies": doh_policy })
            };

            match serde_json::to_string_pretty(&final_policy) {
                Ok(json_str) => match std::fs::write(&policy_path, json_str) {
                    Ok(_) => debug!("Wrote Firefox DoH-disable policy to {:?}", policy_path),
                    Err(e) => debug!("Failed to write policy {:?}: {}", policy_path, e),
                },
                Err(e) => debug!("Failed to serialize policy: {}", e),
            }
        }
    }
}
