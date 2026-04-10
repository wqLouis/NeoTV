mod api;
mod cache;
mod commands;
mod config;
mod m3u8;
mod transcoder;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("[LibreTV] App starting");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::make_http_request,
            commands::fetch_url,
            commands::search_videos,
            commands::get_video_detail,
            commands::cache_clear,
            commands::cache_stats,
            commands::test_source_speed,
            commands::fetch_media_url,
            commands::fetch_media_segment,
            commands::fetch_hls_m3u8,
            commands::fetch_hls_segment,
            commands::check_transcoder,
            commands::start_transcoded_stream,
            commands::stop_transcoded_stream
        ])
        .setup(move |_app| {
            eprintln!("[LibreTV] Tauri setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}