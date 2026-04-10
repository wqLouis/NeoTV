pub fn write_log(level: &str, tag: &str, msg: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    eprintln!("[{}][{}][{}] {}", timestamp, level, tag, msg);
}

#[macro_export]
macro_rules! log_debug {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("DEBUG", $tag, &msg);
    };
}

#[macro_export]
macro_rules! log_info {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("INFO", $tag, &msg);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("WARN", $tag, &msg);
    };
}

#[macro_export]
macro_rules! log_error {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("ERROR", $tag, &msg);
    };
}
