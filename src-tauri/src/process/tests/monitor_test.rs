#[cfg(test)]
mod tests {
    use crate::process::monitor::{ProcessEntry, ProcessMonitor};

    #[test]
    fn test_process_monitor_new_defaults() {
        let monitor = ProcessMonitor::new();

        assert!(monitor.tracked_processes.is_empty());
        assert!(!monitor.is_active);
    }

    #[test]
    fn test_process_monitor_default_trait() {
        let monitor = ProcessMonitor::default();

        assert!(monitor.tracked_processes.is_empty());
        assert!(!monitor.is_active);
    }

    #[test]
    fn test_track_process_adds_to_map() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(
            100,
            "firefox".to_string(),
            Some("/usr/bin/firefox".to_string()),
        );

        assert_eq!(monitor.tracked_processes.len(), 1);
        assert!(monitor.tracked_processes.contains_key(&100));

        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.pid, 100);
        assert_eq!(entry.name, "firefox");
        assert_eq!(entry.exe_path, Some("/usr/bin/firefox".to_string()));
        assert_eq!(entry.query_count, 0);
    }

    #[test]
    fn test_track_process_multiple() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);
        monitor.track_process(200, "chrome".to_string(), None);
        monitor.track_process(300, "curl".to_string(), None);

        assert_eq!(monitor.tracked_processes.len(), 3);
        assert!(monitor.tracked_processes.contains_key(&100));
        assert!(monitor.tracked_processes.contains_key(&200));
        assert!(monitor.tracked_processes.contains_key(&300));
    }

    #[test]
    fn test_track_process_overwrites_existing() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);
        monitor.track_process(
            100,
            "firefox-new".to_string(),
            Some("/usr/bin/firefox".to_string()),
        );

        assert_eq!(monitor.tracked_processes.len(), 1);
        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.name, "firefox-new");
        assert_eq!(entry.exe_path, Some("/usr/bin/firefox".to_string()));
    }

    #[test]
    fn test_increment_query_count_updates_count() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);

        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.query_count, 0);

        monitor.increment_query_count(100);
        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.query_count, 1);

        monitor.increment_query_count(100);
        monitor.increment_query_count(100);
        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.query_count, 3);
    }

    #[test]
    fn test_increment_query_count_nonexistent_pid() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);
        monitor.increment_query_count(999);

        assert_eq!(monitor.tracked_processes.len(), 1);
        let entry = monitor.tracked_processes.get(&100).unwrap();
        assert_eq!(entry.query_count, 0);
    }

    #[test]
    fn test_increment_query_count_multiple_processes() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);
        monitor.track_process(200, "chrome".to_string(), None);

        monitor.increment_query_count(100);
        monitor.increment_query_count(100);
        monitor.increment_query_count(200);

        let firefox = monitor.tracked_processes.get(&100).unwrap();
        let chrome = monitor.tracked_processes.get(&200).unwrap();

        assert_eq!(firefox.query_count, 2);
        assert_eq!(chrome.query_count, 1);
    }

    #[tokio::test]
    async fn test_stop_clears_and_deactivates() {
        let mut monitor = ProcessMonitor::new();

        monitor.track_process(100, "firefox".to_string(), None);
        monitor.track_process(200, "chrome".to_string(), None);
        monitor.is_active = true;

        assert_eq!(monitor.tracked_processes.len(), 2);
        assert!(monitor.is_active);

        let result = monitor.stop().await;
        assert!(result.is_ok());

        assert!(monitor.tracked_processes.is_empty());
        assert!(!monitor.is_active);
    }

    #[tokio::test]
    async fn test_stop_when_already_stopped() {
        let mut monitor = ProcessMonitor::new();

        assert!(!monitor.is_active);

        let result = monitor.stop().await;
        assert!(result.is_ok());
        assert!(!monitor.is_active);
    }

    #[test]
    fn test_process_entry_debug() {
        let entry = ProcessEntry {
            pid: 100,
            name: "test".to_string(),
            exe_path: Some("/usr/bin/test".to_string()),
            query_count: 5,
        };

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("ProcessEntry"));
        assert!(debug_str.contains("pid: 100"));
        assert!(debug_str.contains("name: \"test\""));
        assert!(debug_str.contains("query_count: 5"));
    }

    #[test]
    fn test_process_entry_clone() {
        let entry = ProcessEntry {
            pid: 100,
            name: "test".to_string(),
            exe_path: Some("/usr/bin/test".to_string()),
            query_count: 5,
        };

        let cloned = entry.clone();
        assert_eq!(cloned.pid, entry.pid);
        assert_eq!(cloned.name, entry.name);
        assert_eq!(cloned.exe_path, entry.exe_path);
        assert_eq!(cloned.query_count, entry.query_count);
    }

    #[test]
    fn test_process_monitor_debug() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, "test".to_string(), None);

        let debug_str = format!("{:?}", monitor);
        assert!(debug_str.contains("ProcessMonitor"));
        assert!(debug_str.contains("tracked_processes"));
    }
}
