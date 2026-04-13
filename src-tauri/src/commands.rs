use crate::api::{self, HttpRequestOptions};
use crate::cache::{self, SpeedTestResult};
use crate::config;
use crate::error::HttpError;
use crate::m3u8;
use crate::preloader;
use crate::storage::{self, HistoryItem, FavouriteItem, SpeedCacheStorage};
use std::collections::HashMap;
use std::sync::OnceLock;

static SPEED_CACHE_STORAGE: OnceLock<SpeedCacheStorage> = OnceLock::new();

pub fn init_speed_cache_storage(data_dir: std::path::PathBuf) {
    let _ = SPEED_CACHE_STORAGE.set(SpeedCacheStorage::new(data_dir));
}

pub use crate::api::HttpResponse;

#[tauri::command]
pub async fn make_http_request(options: HttpRequestOptions) -> Result<HttpResponse, HttpError> {
    const CACHE_TTL_SECS: u64 = 600;
    const MAX_CACHE_SIZE: usize = 5 * 1024 * 1024;

    let url = options.url.clone();

    if let Some(cached) = cache::get_cached(&url, CACHE_TTL_SECS).await {
        let body = match String::from_utf8(cached.data.clone()) {
            Ok(s) => s,
            Err(_) => return api::http_request(options).await,
        };
        return Ok(HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body,
            cached: true,
        });
    }

    let response = api::http_request(options).await?;

    let content_type = response.headers.get("content-type")
        .map(|h| h.as_str())
        .unwrap_or("");

    if !content_type.starts_with("video/")
        && !content_type.starts_with("audio/")
        && response.body.len() < MAX_CACHE_SIZE
    {
        cache::set_cached(&url, response.body.clone().into_bytes(), content_type.to_string()).await;
    }

    Ok(response)
}

#[tauri::command]
pub async fn fetch_url(url: String, referer: Option<String>) -> Result<String, String> {
    if let Some(cached) = cache::get_cached(&url, 3600).await {
        return Ok(format!("data:{};base64,{}", cached.content_type, base64_encode(&cached.data)));
    }

    let referer_str = referer.as_deref();
    let (data, content_type) = cache::fetch_url(&url, referer_str).await?;

    cache::set_cached(&url, data.clone(), content_type.clone()).await;

    if content_type.starts_with("image/") {
        Ok(format!("data:{};base64,{}", content_type, base64_encode(&data)))
    } else {
        String::from_utf8(data).map_err(|e| format!("UTF-8 error: {}", e))
    }
}

#[tauri::command]
pub async fn search_videos(
    query: String,
    source_id: String,
    custom_api_url: Option<String>,
) -> Result<String, HttpError> {
    let source_info = if source_id == "custom" {
        let url = custom_api_url.ok_or_else(|| HttpError::new("Custom source selected but no API URL provided"))?;
        config::ApiSourceInfo {
            api_base_url: url.clone(),
            name: "Custom".to_string(),
            detail_base_url: None,
            api_type: config::ApiType::Json,
            search_path: None,
            detail_path: None,
        }
    } else {
        config::get_api_source(&source_id).ok_or_else(|| HttpError::new(format!("Unknown source_id: {}", source_id)))?
    };

    let base_url = source_info.api_base_url.clone();
    let search_path = config::get_search_path(&source_info);
    let full_url = format!("{}{}{}", base_url, search_path, urlencoding::encode(&query));

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let resp = api::http_request(HttpRequestOptions {
        url: full_url,
        method: Some("GET".to_string()),
        headers: Some(headers),
        body: None,
        timeout_secs: Some(20),
    }).await?;

    if resp.status >= 200 && resp.status < 300 {
        Ok(resp.body)
    } else {
        Err(HttpError::with_details(format!("API request failed: {}", resp.status), resp.body))
    }
}

#[tauri::command]
pub async fn get_video_detail(video_id: String, source_id: String) -> Result<String, HttpError> {
    let source_info = config::get_api_source(&source_id).ok_or_else(|| HttpError::new(format!("Unknown source_id: {}", source_id)))?;

    let base_url = source_info.api_base_url.clone();
    let detail_path = source_info.detail_path.unwrap_or_else(|| "/api.php/provide/vod/?ac=videolist&ids=".to_string());
    let full_url = format!("{}{}{}", base_url, detail_path.replace("{id}", &video_id), video_id);

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let resp = api::http_request(HttpRequestOptions {
        url: full_url,
        method: Some("GET".to_string()),
        headers: Some(headers),
        body: None,
        timeout_secs: Some(20),
    }).await?;

    if resp.status >= 200 && resp.status < 300 {
        Ok(resp.body)
    } else {
        Err(HttpError::with_details(format!("HTTP {}", resp.status), resp.body))
    }
}

#[tauri::command]
pub async fn fetch_media_url(url: String, ad_filtering: Option<bool>) -> Result<m3u8::MediaInfo, String> {
    m3u8::fetch_and_process_m3u8(&url, ad_filtering.unwrap_or(true))
        .await
        .map_err(|HttpError { error, .. }| error)
}

#[tauri::command]
pub async fn fetch_media_segment(url: String) -> Result<Vec<u8>, HttpError> {
    let actual_url = if url.starts_with("app-media://") {
        url.strip_prefix("app-media://")
            .and_then(|e| urlencoding::decode(e).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| url.clone())
    } else {
        url.clone()
    };
    m3u8::fetch_media_segment(&actual_url).await
}

#[tauri::command]
pub async fn fetch_hls_m3u8(url: String, ad_filtering: Option<bool>) -> Result<String, String> {
    let content = m3u8::fetch_m3u8_content(&url, ad_filtering.unwrap_or(true))
        .await
        .map_err(|e| e.error)?;

    preloader::PRELOADER.start(&content, &url).await;

    Ok(content)
}

#[tauri::command]
pub async fn fetch_hls_segment(url: String) -> Result<Vec<u8>, String> {
    let actual_url = if url.starts_with("app-media://") {
        url.strip_prefix("app-media://")
            .and_then(|e| urlencoding::decode(e).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| url.clone())
    } else {
        url.clone()
    };

    if let Some(data) = preloader::PRELOADER.get_segment(&actual_url).await {
        return Ok(data);
    }

    preloader::PRELOADER.get_segment_or_fetch(&actual_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preloader_set_workers(count: usize) {
    preloader::PRELOADER.set_worker_count(count);
}

#[tauri::command]
pub async fn preloader_stop() {
    preloader::PRELOADER.stop().await;
}

#[tauri::command]
pub async fn preloader_stats() -> (usize, usize) {
    preloader::PRELOADER.get_cache_stats().await
}

#[tauri::command]
pub fn preloader_set_max_cache_size(bytes: usize) {
    preloader::PRELOADER.set_max_cache_size(bytes);
}

#[tauri::command]
pub fn preloader_get_max_cache_size() -> usize {
    preloader::PRELOADER.get_max_cache_size()
}

#[tauri::command]
pub async fn cache_clear() {
    cache::clear_all_caches().await;
}

#[tauri::command]
pub async fn cache_stats() -> cache::CacheStats {
    cache::get_cache_stats().await
}

#[tauri::command]
pub async fn test_source_speed(source_id: String, custom_url: Option<String>) -> SpeedTestResult {
    use std::time::Instant;

    let cache_key = if source_id == "custom" {
        format!("speed_custom_{}", custom_url.as_deref().unwrap_or(""))
    } else {
        format!("speed_{}", source_id)
    };

    if let Some(cached) = cache::get_speed_cached(&cache_key) {
        return cached;
    }

    let (api_base_url, source_name) = if source_id == "custom" {
        let url = custom_url.unwrap_or_default();
        (url, "自定义源".to_string())
    } else if let Some(info) = config::get_api_source(&source_id) {
        (info.api_base_url.clone(), info.name.clone())
    } else {
        return SpeedTestResult {
            source_id,
            source_name: "未知".to_string(),
            latency_ms: 0,
            download_speed_kbps: 0.0,
            status: "error".to_string(),
            error: Some("Unknown source".to_string()),
            network_id: "default".to_string(),
        };
    };

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
            let body_size_kb = (resp.body.len() as f64) / 1024.0;
            let elapsed_secs = elapsed.as_secs_f64();
            let result = SpeedTestResult {
                source_id: source_id.clone(),
                source_name,
                latency_ms: elapsed.as_millis() as u64,
                download_speed_kbps: if elapsed_secs > 0.0 { body_size_kb / elapsed_secs } else { 0.0 },
                status: "success".to_string(),
                error: None,
                network_id: "default".to_string(),
            };
            cache::set_speed_cached(&cache_key, result.clone());
            result
        }
        Err(e) => {
            let result = SpeedTestResult {
                source_id: source_id.clone(),
                source_name,
                latency_ms: start.elapsed().as_millis() as u64,
                download_speed_kbps: 0.0,
                status: "error".to_string(),
                error: Some(e.error),
                network_id: "default".to_string(),
            };
            cache::set_speed_cached(&cache_key, result.clone());
            result
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

// History commands
#[tauri::command]
pub fn history_get_all(state: tauri::State<'_, storage::Storage>) -> Vec<HistoryItem> {
    state.history_get_all()
}

#[tauri::command]
pub fn history_add(item: HistoryItem, state: tauri::State<'_, storage::Storage>) {
    state.history_add(item);
}

#[tauri::command]
pub fn history_remove(id: String, source: String, episode: Option<String>, state: tauri::State<'_, storage::Storage>) {
    state.history_remove(&id, &source, episode.as_deref());
}

#[tauri::command]
pub fn history_clear(state: tauri::State<'_, storage::Storage>) {
    state.history_clear();
}

// Favourites commands
#[tauri::command]
pub fn favourites_get_all(state: tauri::State<'_, storage::Storage>) -> Vec<FavouriteItem> {
    state.favourites_get_all()
}

#[tauri::command]
pub fn favourites_add(item: FavouriteItem, state: tauri::State<'_, storage::Storage>) {
    state.favourites_add(item);
}

#[tauri::command]
pub fn favourites_remove(id: String, source: String, episode: Option<String>, state: tauri::State<'_, storage::Storage>) {
    state.favourites_remove(&id, &source, episode.as_deref());
}

#[tauri::command]
pub fn favourites_has(id: String, source: String, episode: Option<String>, state: tauri::State<'_, storage::Storage>) -> bool {
    state.favourites_has(&id, &source, episode.as_deref())
}

#[tauri::command]
pub fn favourites_clear(state: tauri::State<'_, storage::Storage>) {
    state.favourites_clear();
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn get_gst_libav_status() -> crate::gst_check::GstLibavInfo {
    crate::gst_check::check_gst_libav_status()
}

#[tauri::command]
pub fn speed_cache_load(network_id: String) -> Vec<SpeedTestResult> {
    if let Some(storage) = SPEED_CACHE_STORAGE.get() {
        let results = storage.load(&network_id);
        for result in &results {
            cache::set_speed_cached(&result.source_id, result.clone());
        }
        results
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn speed_cache_save(network_id: String) {
    if let Some(storage) = SPEED_CACHE_STORAGE.get() {
        let results = cache::get_all_speed_cached();
        storage.save(&network_id, &results);
    }
}

#[tauri::command]
pub fn speed_cache_get(source_id: String) -> Option<SpeedTestResult> {
    cache::get_speed_cached(&source_id)
}

#[tauri::command]
pub fn speed_cache_clear_all() {
    if let Some(storage) = SPEED_CACHE_STORAGE.get() {
        storage.clear_all();
    }
    cache::clear_speed_cache();
}

#[tauri::command]
pub fn get_network_id() -> String {
    #[cfg(target_os = "android")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("getprop")
            .args(["wifi.interface"])
            .output()
        {
            let iface = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !iface.is_empty() {
                if let Ok(output) = Command::new("getprop")
                    .args([&format!("net.{}.ssid", iface)])
                    .output()
                {
                    let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !ssid.is_empty() && ssid != "<unknown ssid>" {
                        return format!("android_{}", ssid);
                    }
                }
            }
        }

        if let Ok(output) = Command::new("dumpsys")
            .args(["wifi"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("SSID") && !line.contains("null") && !line.contains("<unknown") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(ssid) = parts.last() {
                        let ssid = ssid.trim();
                        if !ssid.is_empty() {
                            return format!("android_{}", ssid);
                        }
                    }
                }
            }
        }

        "android_unknown".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let output = Command::new("iw")
            .args(["dev", "-M", "link"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("SSID:") {
                    let ssid = line.strip_prefix("SSID:").unwrap_or("").trim();
                    if !ssid.is_empty() {
                        return ssid.to_string();
                    }
                }
            }
        }
        let nm_output = Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
            .output();
        if let Ok(output) = nm_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && parts[0] == "yes" {
                    let ssid = parts[1].trim();
                    if !ssid.is_empty() {
                        return ssid.to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        "default".to_string()
    }
}
