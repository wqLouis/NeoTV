use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const DEBUG_LOG_PATH: &str = "/home/wqlouis/Documents/code/LibreTV-App/debug.log";

pub fn get_log_path() -> PathBuf {
    PathBuf::from(DEBUG_LOG_PATH)
}

pub fn write_log(level: &str, tag: &str, msg: &str) -> Result<(), String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let log_line = format!("[{}][{}][{}] {}", timestamp, level, tag, msg);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&get_log_path())
        .map_err(|e| e.to_string())?;

    writeln!(file, "{}", log_line).map_err(|e| e.to_string())?;
    eprintln!("{}", log_line);
    Ok(())
}

pub fn clear_log() -> Result<(), String> {
    std::fs::remove_file(&get_log_path()).map_err(|e| e.to_string())
}

pub fn read_log() -> Result<String, String> {
    std::fs::read_to_string(&get_log_path()).map_err(|e| e.to_string())
}

#[macro_export]
macro_rules! log_debug {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("DEBUG", $tag, &msg).ok();
    };
}

#[macro_export]
macro_rules! log_info {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("INFO", $tag, &msg).ok();
    };
}

#[macro_export]
macro_rules! log_warn {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("WARN", $tag, &msg).ok();
    };
}

#[macro_export]
macro_rules! log_error {
    ($tag:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        crate::logging::write_log("ERROR", $tag, &msg).ok();
    };
}
