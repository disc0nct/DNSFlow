use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn set(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_enabled() {
            eprintln!("[DNSFlow:debug] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! debug_with_prefix {
    ($prefix:expr, $($arg:tt)*) => {
        if $crate::debug::is_enabled() {
            eprintln!("[DNSFlow:debug:{}] {}", $prefix, format!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_false() {
        init(false);
        assert!(!is_enabled());
    }

    #[test]
    fn test_set_true() {
        set(true);
        assert!(is_enabled());
    }
}
