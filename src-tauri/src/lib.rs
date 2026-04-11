mod api;
mod cache;
mod commands;
mod config;
mod error;
mod http;
mod m3u8;
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
        .register_uri_scheme_protocol("app-media", |_ctx, request| {
            let uri_str = request.uri().to_string();
            eprintln!("[Protocol] app-media request: {}", uri_str);

            let path = uri_str.strip_prefix("app-media://").unwrap_or(&uri_str);
            let parts: Vec<&str> = path.splitn(2, '/').collect();

            if parts.len() != 2 {
                let body = Vec::from(r#"{"error":"Invalid protocol format"}"#);
                return tauri::http::Response::builder()
                    .status(400)
                    .header(tauri::http::header::CONTENT_TYPE, "application/json")
                    .body(body).unwrap();
            }

            let (scheme, encoded_url) = (parts[0], parts[1]);

            let decoded_url = urlencoding::decode(encoded_url)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| encoded_url.to_string());

            match scheme {
                "segment" => {
                    let result = tauri::async_runtime::block_on(async {
                        http::fetch_bytes(&decoded_url, None).await
                    });

                    match result {
                        Ok(bytes) => {
                            let mime = if decoded_url.ends_with(".m3u8") {
                                "application/vnd.apple.mpegurl"
                            } else if decoded_url.ends_with(".ts") {
                                "video/mp2t"
                            } else if decoded_url.ends_with(".m4s") {
                                "video/mp4"
                            } else {
                                "application/octet-stream"
                            };
                            let mut response = tauri::http::Response::builder()
                                .status(200)
                                .header(tauri::http::header::CONTENT_TYPE, mime)
                                .body(bytes).unwrap();
                            response.headers_mut().insert(
                                tauri::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                tauri::http::header::HeaderValue::from_static("*")
                            );
                            response
                        }
                        Err(e) => {
                            let body = Vec::from(format!(r#"{{"error":"{}"}}"#, e));
                            tauri::http::Response::builder()
                                .status(500)
                                .header(tauri::http::header::CONTENT_TYPE, "application/json")
                                .body(body).unwrap()
                        }
                    }
                }
                "m3u8" => {
                    let result = tauri::async_runtime::block_on(async {
                        http::fetch_text(&decoded_url, None).await
                    });

                    match result {
                        Ok((content, _)) => {
                            let mut response = tauri::http::Response::builder()
                                .status(200)
                                .header(tauri::http::header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
                                .body(content.into_bytes()).unwrap();
                            response.headers_mut().insert(
                                tauri::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                tauri::http::header::HeaderValue::from_static("*")
                            );
                            response
                        }
                        Err(e) => {
                            let body = Vec::from(format!(r#"{{"error":"{}"}}"#, e));
                            tauri::http::Response::builder()
                                .status(500)
                                .header(tauri::http::header::CONTENT_TYPE, "application/json")
                                .body(body).unwrap()
                        }
                    }
                }
                _ => {
                    let body = Vec::from(format!(r#"{{"error":"Unknown scheme: {}"}}"#, scheme));
                    tauri::http::Response::builder()
                        .status(400)
                        .header(tauri::http::header::CONTENT_TYPE, "application/json")
                        .body(body).unwrap()
                }
            }
        })
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
            commands::history_get_all,
            commands::history_add,
            commands::history_remove,
            commands::history_clear,
            commands::favourites_get_all,
            commands::favourites_add,
            commands::favourites_remove,
            commands::favourites_has,
            commands::favourites_clear
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let cache_dir = app_data_dir.join("cache");
            fs::create_dir_all(&cache_dir).ok();
            cache::init_cache_dir(cache_dir);
            let storage = storage::Storage::new(app_data_dir);
            app.manage(storage);
            eprintln!("[LibreTV] Tauri setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
