use crate::api::{self, HttpRequestOptions, HttpError};
use crate::config;
use crate::logging;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::api::HttpResponse;

// === HTTP Command ===

#[tauri::command]
pub async fn make_http_request(options: HttpRequestOptions) -> Result<HttpResponse, HttpError> {
    api::http_request(options).await
}

// === Search Commands ===

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponse {
    code: u16,
    msg: Option<String>,
    list: serde_json::Value,
}

#[tauri::command]
pub async fn search_videos(
    query: String,
    source_id: String,
    custom_api_url: Option<String>,
) -> Result<String, HttpError> {
    let source_info: config::ApiSourceInfo;
    let base_url: String;

    if source_id == "custom" {
        match custom_api_url {
            Some(url) => {
                base_url = url;
                source_info = config::ApiSourceInfo {
                    api_base_url: base_url.clone(),
                    name: "Custom".to_string(),
                    detail_base_url: None,
                    api_type: config::ApiType::Json,
                    search_path: None,
                    detail_path: None,
                };
            }
            None => {
                return Err(HttpError {
                    error: "Custom source selected but no API URL provided".to_string(),
                    details: None,
                });
            }
        }
    } else {
        match config::get_api_source(&source_id) {
            Some(info) => {
                source_info = info;
                base_url = source_info.api_base_url.clone();
            }
            None => {
                return Err(HttpError {
                    error: format!("Unknown source_id: {}", source_id),
                    details: None,
                });
            }
        }
    }

    let search_path = config::get_search_path(&source_info);
    let encoded_query = urlencoding::encode(&query);
    let full_url = format!("{}{}{}", base_url, search_path, encoded_query);

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let http_options = HttpRequestOptions {
        url: full_url,
        method: Some("GET".to_string()),
        headers: Some(headers),
        body: None,
        timeout_secs: Some(20),
    };

    match api::http_request(http_options).await {
        Ok(http_response) => {
            if http_response.status >= 200 && http_response.status < 300 {
                Ok(http_response.body)
            } else {
                Err(HttpError {
                    error: format!("API request failed with status: {}", http_response.status),
                    details: Some(http_response.body),
                })
            }
        }
        Err(e) => Err(e),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoDetailResult {
    pub list: Vec<serde_json::Value>,
}

#[tauri::command]
pub async fn get_video_detail(video_id: String, source_id: String) -> Result<String, HttpError> {
    let source_info = match config::get_api_source(&source_id) {
        Some(info) => info,
        None => {
            return Err(HttpError {
                error: format!("Unknown source_id: {}", source_id),
                details: None,
            });
        }
    };

    let base_url = source_info.api_base_url.clone();
    let detail_path = source_info.detail_path.unwrap_or_else(|| "/api.php/provide/vod/?ac=videolist&ids=".to_string());
    let full_url = format!("{}{}{}", base_url, detail_path.replace("{id}", &video_id), video_id);

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let options = HttpRequestOptions {
        url: full_url,
        method: Some("GET".to_string()),
        headers: Some(headers),
        body: None,
        timeout_secs: Some(20),
    };

    match api::http_request(options).await {
        Ok(resp) => {
            if resp.status >= 200 && resp.status < 300 {
                Ok(resp.body)
            } else {
                Err(HttpError {
                    error: format!("HTTP {}", resp.status),
                    details: Some(resp.body),
                })
            }
        }
        Err(e) => Err(e),
    }
}

// === Cache Commands ===

use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static IMAGE_MEMORY_CACHE: OnceLock<Mutex<std::collections::HashMap<String, String>>> = OnceLock::new();

fn get_image_cache() -> &'static Mutex<std::collections::HashMap<String, String>> {
    IMAGE_MEMORY_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn get_image_cache_dir() -> PathBuf {
    let cache_dir = PathBuf::from("/home/wqlouis/Documents/code/LibreTV-App/image_cache");
    if !cache_dir.exists() {
        let _ = fs::create_dir_all(&cache_dir);
    }
    cache_dir
}

fn hash_url(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    format!("{:x}", s.finish())
}

fn get_image_cache_path(url: &str) -> PathBuf {
    get_image_cache_dir().join(format!("img_{}", hash_url(url)))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn detect_mime(data: &[u8]) -> &str {
    if data.len() >= 3 {
        match (data[0], data[1], data[2]) {
            (0x89, 0x50, 0x4E) => return "image/png",
            (0xFF, 0xD8, 0xFF) => return "image/jpeg",
            (0x47, 0x49, 0x46) => return "image/gif",
            (0x57, 0x45, 0x42) => return "image/webp",
            _ => {}
        }
    }
    "image/jpeg"
}

#[tauri::command]
pub async fn cache_fetch_image(url: String) -> Result<String, String> {
    let cache_key = hash_url(&url);
    let cache_path = get_image_cache_path(&url);

    // Check memory cache first
    {
        let cache = get_image_cache().lock().map_err(|e| e.to_string())?;
        if let Some(data) = cache.get(&cache_key) {
            return Ok(data.clone());
        }
    }

    // Check disk cache
    if cache_path.exists() {
        match fs::read(&cache_path) {
            Ok(data) => {
                let base64_data = base64_encode(&data);
                let mime = detect_mime(&data);
                let data_url = format!("data:{};base64,{}", mime, base64_data);

                let mut cache = get_image_cache().lock().map_err(|e| e.to_string())?;
                cache.insert(cache_key, data_url.clone());

                return Ok(data_url);
            }
            Err(e) => return Err(format!("Failed to read cache: {}", e)),
        }
    }

    // Fetch from network
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"));

    if url.contains("doubanio.com") {
        headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://movie.douban.com/"));
    }

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Read failed: {}", e))?;

    // Save to disk cache
    if let Err(e) = fs::write(&cache_path, &bytes) {
        eprintln!("[cache] Failed to write cache: {}", e);
    }

    // Store in memory cache
    let base64_data = base64_encode(&bytes);
    let mime = detect_mime(&bytes);
    let data_url = format!("data:{};base64,{}", mime, base64_data);

    let mut cache = get_image_cache().lock().map_err(|e| e.to_string())?;
    cache.insert(cache_key, data_url.clone());

    Ok(data_url)
}

#[tauri::command]
pub async fn cache_clear() -> Result<(), String> {
    // Clear memory cache
    {
        let mut cache = get_image_cache().lock().map_err(|e| e.to_string())?;
        cache.clear();
    }

    // Clear disk cache
    let cache_dir = get_image_cache_dir();
    if cache_dir.exists() {
        match fs::read_dir(&cache_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = fs::remove_file(path);
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read cache dir: {}", e)),
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CacheStats {
    count: usize,
    size_bytes: u64,
}

#[tauri::command]
pub async fn cache_stats() -> Result<CacheStats, String> {
    let cache_dir = get_image_cache_dir();
    let mut count = 0usize;
    let mut size = 0u64;

    // Memory cache count
    {
        let cache = get_image_cache().lock().map_err(|e| e.to_string())?;
        count += cache.len();
    }

    // Disk cache
    if cache_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                    if let Ok(meta) = fs::metadata(&path) {
                        size += meta.len();
                    }
                }
            }
        }
    }

    Ok(CacheStats { count, size_bytes: size })
}

// === Logging Commands ===

#[tauri::command]
pub fn tauri_write_log(level: String, tag: String, msg: String) -> Result<(), String> {
    logging::write_log(&level, &tag, &msg)
}

#[tauri::command]
pub fn clear_debug_log() -> Result<(), String> {
    logging::clear_log()
}

#[tauri::command]
pub fn read_debug_log() -> Result<String, String> {
    logging::read_log()
}

// === Speed Test ===

#[derive(Serialize, Deserialize)]
pub struct SpeedTestResult {
    source_id: String,
    source_name: String,
    latency_ms: u64,
    download_speed_kbps: f64,
    status: String,
    error: Option<String>,
}

#[tauri::command]
pub async fn test_source_speed(source_id: String, custom_url: Option<String>) -> SpeedTestResult {
    use std::time::Instant;

    let api_base_url: String;
    let source_name: String;

    if source_id == "custom" {
        match custom_url {
            Some(url) => {
                api_base_url = url;
                source_name = "自定义源".to_string();
            }
            None => {
                return SpeedTestResult {
                    source_id,
                    source_name: "未知".to_string(),
                    latency_ms: 0,
                    download_speed_kbps: 0.0,
                    status: "error".to_string(),
                    error: Some("No URL provided".to_string()),
                };
            }
        }
    } else {
        match config::get_api_source(&source_id) {
            Some(info) => {
                api_base_url = info.api_base_url.clone();
                source_name = info.name.clone();
            }
            None => {
                return SpeedTestResult {
                    source_id,
                    source_name: "未知".to_string(),
                    latency_ms: 0,
                    download_speed_kbps: 0.0,
                    status: "error".to_string(),
                    error: Some("Unknown source".to_string()),
                };
            }
        }
    }

    let test_url = format!("{}/api.php/provide/vod/?ac=videolist&wd=test", api_base_url);

    let start = Instant::now();
    match api::http_request(HttpRequestOptions {
        url: test_url,
        method: Some("GET".to_string()),
        headers: Some({
            let mut h = HashMap::new();
            h.insert("User-Agent".to_string(), "Mozilla/5.0".to_string());
            h.insert("Accept".to_string(), "application/json".to_string());
            h
        }),
        body: None,
        timeout_secs: Some(10),
    }).await {
        Ok(resp) => {
            let elapsed = start.elapsed();
            let latency_ms = elapsed.as_millis() as u64;
            let body_size_kb = (resp.body.len() as f64) / 1024.0;
            let elapsed_secs = elapsed.as_secs_f64();
            let download_speed_kbps = if elapsed_secs > 0.0 { body_size_kb / elapsed_secs } else { 0.0 };

            SpeedTestResult {
                source_id,
                source_name,
                latency_ms,
                download_speed_kbps,
                status: "ok".to_string(),
                error: None,
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            SpeedTestResult {
                source_id,
                source_name,
                latency_ms: elapsed.as_millis() as u64,
                download_speed_kbps: 0.0,
                status: "error".to_string(),
                error: Some(e.error),
            }
        }
    }
}