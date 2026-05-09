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
    eprintln!("[LibreTV] App starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            #[cfg(target_os = "linux")] commands::get_gst_libav_status
        ])
        .setup(|app| {
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
            eprintln!("[LibreTV] Tauri setup complete, cache dir: {:?}", cache_dir);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
