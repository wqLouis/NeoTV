use crate::api::{self, HttpRequestOptions, HttpError};
use crate::cache::{self, SpeedTestResult};
use crate::config;
use crate::m3u8;
use crate::storage::{self, HistoryItem, FavouriteItem};
use std::collections::HashMap;

pub use crate::api::HttpResponse;

#[tauri::command]
pub async fn make_http_request(options: HttpRequestOptions) -> Result<HttpResponse, HttpError> {
    const CACHE_TTL_SECS: u64 = 600; // 10 minutes
    const MAX_CACHE_SIZE: usize = 5 * 1024 * 1024; // 5MB

    let url = options.url.clone();

    // 1. Check cache first for non-media content
    if let Some(cached) = cache::get_cached(&url, CACHE_TTL_SECS) {
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

    // 2. Make the request
    let response = api::http_request(options).await?;

    // 3. Cache if it's not a media type and size is acceptable
    let content_type = response.headers.get("content-type")
        .map(|h| h.as_str())
        .unwrap_or("");

    if !content_type.starts_with("video/")
        && !content_type.starts_with("audio/")
        && response.body.len() < MAX_CACHE_SIZE
    {
        cache::set_cached(&url, response.body.clone().into_bytes(), content_type.to_string());
    }

    Ok(response)
}

#[tauri::command]
pub async fn fetch_url(url: String, referer: Option<String>) -> Result<String, String> {
    if let Some(cached) = cache::get_cached(&url, 3600) {
        return Ok(format!("data:{};base64,{}", cached.content_type, base64_encode(&cached.data)));
    }

    let referer_str = referer.as_deref();
    let (data, content_type) = cache::fetch_url(&url, referer_str).await?;

    cache::set_cached(&url, data.clone(), content_type.clone());

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
        let url = custom_api_url.ok_or_else(|| HttpError {
            error: "Custom source selected but no API URL provided".to_string(),
            details: None,
        })?;
        config::ApiSourceInfo {
            api_base_url: url.clone(),
            name: "Custom".to_string(),
            detail_base_url: None,
            api_type: config::ApiType::Json,
            search_path: None,
            detail_path: None,
        }
    } else {
        config::get_api_source(&source_id).ok_or_else(|| HttpError {
            error: format!("Unknown source_id: {}", source_id),
            details: None,
        })?
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
        Err(HttpError {
            error: format!("API request failed: {}", resp.status),
            details: Some(resp.body),
        })
    }
}

#[tauri::command]
pub async fn get_video_detail(video_id: String, source_id: String) -> Result<String, HttpError> {
    let source_info = config::get_api_source(&source_id).ok_or_else(|| HttpError {
        error: format!("Unknown source_id: {}", source_id),
        details: None,
    })?;

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
        Err(HttpError {
            error: format!("HTTP {}", resp.status),
            details: Some(resp.body),
        })
    }
}

#[tauri::command]
pub async fn fetch_media_url(url: String, ad_filtering: Option<bool>) -> Result<m3u8::MediaInfo, m3u8::HttpError> {
    m3u8::fetch_and_process_m3u8(&url, ad_filtering.unwrap_or(true)).await
}

#[tauri::command]
pub async fn fetch_media_segment(url: String) -> Result<Vec<u8>, m3u8::HttpError> {
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
    m3u8::fetch_m3u8_content(&url, ad_filtering.unwrap_or(true))
        .await
        .map_err(|e| e.error)
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
    m3u8::fetch_media_segment(&actual_url)
        .await
        .map_err(|e| e.error)
}

#[tauri::command]
pub fn cache_clear() {
    cache::clear_all_caches();
}

#[tauri::command]
pub fn cache_stats() -> cache::CacheStats {
    cache::get_cache_stats()
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
            };
            cache::set_speed_cached(&cache_key, result.clone());
            result
        }
    }
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

