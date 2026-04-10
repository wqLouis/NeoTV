use crate::api::{self, HttpRequestOptions, HttpError};
use crate::cache;
use crate::config;
use crate::m3u8;
use crate::transcoder::TRANSCODER;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::api::HttpResponse;

#[tauri::command]
pub async fn make_http_request(options: HttpRequestOptions) -> Result<HttpResponse, HttpError> {
    api::http_request(options).await
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
    cache::clear_cache();
}

#[derive(Serialize, Deserialize)]
pub struct CacheStats {
    mem_count: usize,
    mem_size: usize,
    disk_count: usize,
    disk_size: usize,
}

#[tauri::command]
pub fn cache_stats() -> CacheStats {
    let (mem_count, mem_size, disk_count, disk_size) = cache::get_cache_stats();
    CacheStats {
        mem_count,
        mem_size,
        disk_count,
        disk_size,
    }
}

#[tauri::command]
pub async fn test_source_speed(source_id: String, custom_url: Option<String>) -> SpeedTestResult {
    use std::time::Instant;

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
            SpeedTestResult {
                source_id,
                source_name,
                latency_ms: elapsed.as_millis() as u64,
                download_speed_kbps: if elapsed_secs > 0.0 { body_size_kb / elapsed_secs } else { 0.0 },
                status: "success".to_string(),
                error: None,
            }
        }
        Err(e) => SpeedTestResult {
            source_id,
            source_name,
            latency_ms: start.elapsed().as_millis() as u64,
            download_speed_kbps: 0.0,
            status: "error".to_string(),
            error: Some(e.error),
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpeedTestResult {
    source_id: String,
    source_name: String,
    latency_ms: u64,
    download_speed_kbps: f64,
    status: String,
    error: Option<String>,
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

#[derive(Serialize, Deserialize)]
pub struct TranscoderInfo {
    pub url: String,
    pub port: u16,
    pub duration: Option<f64>,
    pub vaapi_available: bool,
    pub ffmpeg_available: bool,
}

#[tauri::command]
pub fn check_transcoder() -> TranscoderInfo {
    let manager = TRANSCODER.lock().unwrap();
    let (vaapi, ffmpeg, _) = manager.check_system();
    TranscoderInfo {
        url: String::new(),
        port: 0,
        duration: None,
        vaapi_available: vaapi,
        ffmpeg_available: ffmpeg,
    }
}

#[tauri::command]
pub fn start_transcoded_stream(
    id: String,
    m3u8_url: String,
    referer: Option<String>,
) -> Result<TranscoderInfo, String> {
    let (vaapi_available, ffmpeg_available) = {
        let manager = TRANSCODER.lock().unwrap();
        let (vaapi, ffmpeg, _) = manager.check_system();
        (vaapi, ffmpeg)
    };

    if !ffmpeg_available {
        return Err("ffmpeg not available".to_string());
    }

    let stream_info = crate::transcoder::run_streaming_transcoder(
        id,
        m3u8_url,
        referer,
    )?;

    Ok(TranscoderInfo {
        url: stream_info.url,
        port: stream_info.port,
        duration: stream_info.duration,
        vaapi_available,
        ffmpeg_available,
    })
}

#[tauri::command]
pub fn stop_transcoded_stream(id: String) {
    let mut manager = TRANSCODER.lock().unwrap();
    manager.stop_stream(&id);
}
