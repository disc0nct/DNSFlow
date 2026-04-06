#[cfg(test)]
mod tests {
    use crate::process::lookup::{LookupResult, ProcessLookup};

    #[test]
    fn test_process_lookup_new_is_empty() {
        let lookup = ProcessLookup::new();

        assert!(lookup.cache.is_empty());
    }

    #[test]
    fn test_process_lookup_default_trait() {
        let lookup = ProcessLookup::default();

        assert!(lookup.cache.is_empty());
    }

    #[test]
    fn test_clear_cache_empties_map() {
        let mut lookup = ProcessLookup::new();

        lookup.cache.insert(100, "firefox".to_string());
        lookup.cache.insert(200, "chrome".to_string());

        assert_eq!(lookup.cache.len(), 2);

        lookup.clear_cache();

        assert!(lookup.cache.is_empty());
    }

    #[test]
    fn test_clear_cache_when_already_empty() {
        let mut lookup = ProcessLookup::new();

        assert!(lookup.cache.is_empty());

        lookup.clear_cache();

        assert!(lookup.cache.is_empty());
    }

    #[test]
    fn test_get_name_returns_cached_name() {
        let mut lookup = ProcessLookup::new();

        lookup.cache.insert(100, "firefox".to_string());

        let name = lookup.get_name(100);
        assert_eq!(name, Some("firefox".to_string()));
    }

    #[test]
    fn test_get_name_returns_none_for_nonexistent_pid() {
        let mut lookup = ProcessLookup::new();

        lookup.cache.insert(100, "firefox".to_string());

        // Use a guaranteed-nonexistent PID (u32::MAX)
        let name = lookup.get_name(u32::MAX);
        assert_eq!(name, None);
    }

    #[test]
    fn test_get_name_after_clear_cache() {
        let mut lookup = ProcessLookup::new();

        lookup.cache.insert(100, "firefox".to_string());
        lookup.clear_cache();

        let name = lookup.get_name(100);
        assert_eq!(name, None);
    }

    #[test]
    fn test_lookup_result_debug() {
        let result = LookupResult {
            pid: 100,
            name: Some("test".to_string()),
            exe_path: Some("/usr/bin/test".to_string()),
            cmdline: Some("test --arg".to_string()),
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("LookupResult"));
        assert!(debug_str.contains("pid: 100"));
        assert!(debug_str.contains("name: Some(\"test\")"));
    }

    #[test]
    fn test_lookup_result_clone() {
        let result = LookupResult {
            pid: 100,
            name: Some("test".to_string()),
            exe_path: Some("/usr/bin/test".to_string()),
            cmdline: Some("test --arg".to_string()),
        };

        let cloned = result.clone();
        assert_eq!(cloned.pid, result.pid);
        assert_eq!(cloned.name, result.name);
        assert_eq!(cloned.exe_path, result.exe_path);
        assert_eq!(cloned.cmdline, result.cmdline);
    }

    #[test]
    fn test_lookup_result_with_none_fields() {
        let result = LookupResult {
            pid: 100,
            name: None,
            exe_path: None,
            cmdline: None,
        };

        assert_eq!(result.pid, 100);
        assert!(result.name.is_none());
        assert!(result.exe_path.is_none());
        assert!(result.cmdline.is_none());
    }

    #[test]
    fn test_process_lookup_debug() {
        let mut lookup = ProcessLookup::new();
        lookup.cache.insert(100, "test".to_string());

        let debug_str = format!("{:?}", lookup);
        assert!(debug_str.contains("ProcessLookup"));
        assert!(debug_str.contains("cache"));
    }

    #[test]
    fn test_cache_operations_multiple_pids() {
        let mut lookup = ProcessLookup::new();

        lookup.cache.insert(100, "firefox".to_string());
        lookup.cache.insert(200, "chrome".to_string());
        lookup.cache.insert(300, "curl".to_string());

        assert_eq!(lookup.cache.len(), 3);
        assert_eq!(lookup.get_name(100), Some("firefox".to_string()));
        assert_eq!(lookup.get_name(200), Some("chrome".to_string()));
        assert_eq!(lookup.get_name(300), Some("curl".to_string()));

        lookup.cache.remove(&200);
        assert_eq!(lookup.cache.len(), 2);
        assert_eq!(lookup.get_name(200), None);
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_lookup_returns_cached_name_on_non_linux() {
        let mut lookup = ProcessLookup::new();
        lookup.cache.insert(100, "cached_process".to_string());

        let result = lookup.lookup(100).unwrap();
        assert_eq!(result.pid, 100);
        assert_eq!(result.name, Some("cached_process".to_string()));
        assert!(result.exe_path.is_none());
        assert!(result.cmdline.is_none());
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_lookup_returns_none_name_when_not_cached_on_non_linux() {
        let mut lookup = ProcessLookup::new();

        let result = lookup.lookup(999).unwrap();
        assert_eq!(result.pid, 999);
        assert!(result.name.is_none());
        assert!(result.exe_path.is_none());
        assert!(result.cmdline.is_none());
    }
}
