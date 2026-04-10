mod api;
mod commands;
mod config;
mod logging;
mod m3u8;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_path = logging::get_log_path();
    let start_msg = format!("[LibreTV] App starting, log path: {:?}", log_path);
    eprintln!("{}", start_msg);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", start_msg);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::make_http_request,
            commands::search_videos,
            commands::get_video_detail,
            commands::cache_fetch_image,
            commands::cache_clear,
            commands::cache_stats,
            commands::test_source_speed,
            commands::tauri_write_log,
            commands::clear_debug_log,
            commands::read_debug_log
        ])
        .setup(move |_app| {
            let setup_msg = "[LibreTV] Tauri setup complete";
            eprintln!("{}", setup_msg);
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                use std::io::Write;
                let _ = writeln!(file, "{}", setup_msg);
            }
            Ok(())
        })
        .plugin(tauri_plugin_immersive_android::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}