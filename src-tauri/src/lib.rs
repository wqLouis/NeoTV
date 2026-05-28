mod api;
mod cache;
mod commands;
mod config;
mod error;
mod gst_check;
mod http;
mod m3u8;
mod preloader;
mod storage;

use std::fs;
use tauri::Manager;

pub use commands::*;
pub use error::HttpError;
pub use storage::{HistoryItem, FavouriteItem};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix blank white window on Linux with NVIDIA proprietary drivers.
    // The webkit2gtk-nvidia-quirk crate auto-detects NVIDIA GPU and session type.
    #[cfg(target_os = "linux")]
    {
        webkit2gtk_nvidia_quirk::apply_workaround_with_options(
            webkit2gtk_nvidia_quirk::ApplyWorkaroundOptions::default()
        );
        // Enable HLS playback via GStreamer on Linux
        std::env::set_var("WEBKIT_GST_ENABLE_HLS_SUPPORT", "1");
    }

    eprintln!("[NeoTV] App starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            commands::make_http_request,
            commands::fetch_url,
            commands::cache_clear,
            commands::cache_stats,
            commands::test_source_speed,
            commands::fetch_media_url,
            commands::fetch_hls_m3u8,
            commands::fetch_hls_segment,
            commands::preloader_set_workers,
            commands::preloader_stop,
            commands::preloader_stats,
            commands::preloader_set_max_cache_size,
            commands::history_get_all,
            commands::history_add,
            commands::history_remove,
            commands::history_clear,
            commands::favourites_get_all,
            commands::favourites_add,
            commands::favourites_remove,
            commands::favourites_clear,
            commands::speed_cache_load,
            commands::speed_cache_save,
            commands::speed_cache_clear_all,
            commands::get_network_id,
            #[cfg(target_os = "linux")] commands::get_gst_libav_status,
            #[cfg(any(target_os = "windows", target_os = "linux"))] commands::window_minimize,
            #[cfg(any(target_os = "windows", target_os = "linux"))] commands::window_maximize,
            #[cfg(any(target_os = "windows", target_os = "linux"))] commands::window_close,
            #[cfg(any(target_os = "windows", target_os = "linux"))] commands::window_is_maximized
        ])
        .setup(|app| {
            #[cfg(target_os = "linux")]
            eprintln!("[NeoTV] GPU workaround applied: {:?}", webkit2gtk_nvidia_quirk::needs_workaround());

            let window = app.get_webview_window("main").expect("Failed to get main window");
            let url = window.url();
            eprintln!("[NeoTV] WebView URL: {:?}", url);

            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let cache_dir = app_data_dir.join("cache");
            fs::create_dir_all(&cache_dir).ok();
            cache::init_cache_dir(cache_dir.clone());

            tauri::async_runtime::spawn(async move {
                cache::load_cache_from_disk().await;
            });

            let storage = storage::Storage::new(app_data_dir.clone());
            app.manage(storage);

            commands::init_speed_cache_storage(app_data_dir.clone());
            eprintln!("[NeoTV] Tauri setup complete, cache dir: {:?}", cache_dir);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
