#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use dnsflow_lib::platform::windows::*;
    use dnsflow_lib::platform::*;

    #[test]
    fn test_enumerate_includes_self() {
        let enumerator = WindowsProcessEnumerator;
        let processes = enumerator.enumerate().unwrap();
        let current_pid = std::process::id() as i64;
        assert!(processes.iter().any(|p| p.pid == current_pid));
    }

    #[test]
    fn test_socket_lookup_finds_bound_port() {
        let lookup = WindowsSocketLookup;
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        let pid = lookup.pid_by_port(port).unwrap();
        assert_eq!(pid, Some(std::process::id() as i32));
    }

    #[test]
    fn test_dns_detector_returns_servers() {
        let detector = WindowsDnsDetector;
        let servers = detector.detect_system_dns().unwrap();
        assert!(!servers.is_empty());
    }
}
