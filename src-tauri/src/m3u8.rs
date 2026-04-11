use reqwest::header::{HeaderMap, HeaderValue, REFERER, ACCEPT};
use serde::{Deserialize, Serialize};
use url::Url;

const MAX_RECURSION: u8 = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub content_type: String,
    pub is_m3u8: bool,
    pub processed_content: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpError {
    pub error: String,
    pub details: Option<String>,
}

pub fn is_m3u8_content(content: &str, content_type: &str) -> bool {
    content_type.contains("application/vnd.apple.mpegurl")
        || content_type.contains("application/x-mpegurl")
        || content_type.contains("audio/mpegurl")
        || content.trim_start().starts_with("#EXTM3U")
}

fn get_base_url(url_str: &str) -> Result<String, String> {
    let parsed = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;
    let path_segments: Vec<&str> = parsed.path_segments().map_or(Vec::new(), |s| s.collect());
    
    if path_segments.is_empty() || (path_segments.len() == 1 && parsed.path() == "/") {
        return Ok(format!("{}://{}/", parsed.scheme(), parsed.host_str().unwrap_or("")));
    }
    
    let path_parts: Vec<&str> = parsed.path().trim_end_matches('/').split('/').collect();
    let new_path = if path_parts.len() > 1 {
        path_parts[..path_parts.len() - 1].join("/")
    } else {
        String::new()
    };
    
    Ok(format!("{}://{}/{}/", parsed.scheme(), parsed.host_str().unwrap_or(""), new_path))
}

fn resolve_url(base_url: &str, relative_url: &str) -> Result<String, String> {
    if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
        return Ok(relative_url.to_string());
    }
    Url::parse(base_url)
        .and_then(|u| u.join(relative_url))
        .map(|u| u.to_string())
        .map_err(|e| format!("URL join error: {}", e))
}

fn filter_ad_segments(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_ad_section = false;
    let mut ad_cue_count = 0;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed.starts_with("#EXT-X-CUE-OUT") {
            in_ad_section = true;
            ad_cue_count += 1;
            continue;
        }
        
        if trimmed.starts_with("#EXT-X-CUE-IN") {
            in_ad_section = false;
            continue;
        }
        
        if trimmed.starts_with("#EXT-X-DISCONTINUITY") && ad_cue_count > 0 {
            continue;
        }
        
        if in_ad_section {
            continue;
        }
        
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let lower_url = trimmed.to_lowercase();
            if lower_url.contains("/ads/") 
                || lower_url.contains("/ad/") 
                || lower_url.contains("/ads.")
                || lower_url.contains("advertisement")
                || lower_url.contains("/commercial/")
                || lower_url.contains("/sponsor/")
                || lower_url.contains("_ad_")
                || lower_url.contains("-ad-")
                || lower_url.contains("pre-roll")
                || lower_url.contains("post-roll")
            {
                continue;
            }
        }
        
        result.push(line);
    }
    
    result.join("\n")
}

fn parse_m3u8_duration(content: &str) -> Option<f64> {
    let mut total_duration = 0.0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("#EXTINF:") {
            let after_extinf = trimmed.trim_start_matches("#EXTINF:");
            if let Some(duration_str) = after_extinf.split(',').next() {
                if let Ok(d) = duration_str.parse::<f64>() {
                    total_duration += d;
                }
            }
        }
    }

    if total_duration > 0.0 {
        Some(total_duration)
    } else {
        None
    }
}

pub async fn fetch_and_process_m3u8(url: &str, ad_filtering: bool) -> Result<MediaInfo, HttpError> {
    let (content, content_type) = fetch_url(url).await.map_err(|e| HttpError {
        error: e.clone(),
        details: Some(e),
    })?;

    if !is_m3u8_content(&content, &content_type) {
        return Ok(MediaInfo {
            url: url.to_string(),
            content_type,
            is_m3u8: false,
            processed_content: None,
            duration: None,
        });
    }

    let processed = process_m3u8_recursive(url, &content, ad_filtering, 0).await
        .map_err(|e| HttpError {
            error: "M3U8 processing error".to_string(),
            details: Some(e),
        })?;

    let duration = parse_m3u8_duration(&processed);

    Ok(MediaInfo {
        url: url.to_string(),
        content_type: "application/vnd.apple.mpegurl;charset=utf-8".to_string(),
        is_m3u8: true,
        processed_content: Some(processed),
        duration,
    })
}

async fn process_m3u8_recursive(url_str: &str, content: &str, ad_filtering: bool, depth: u8) -> Result<String, String> {
    if depth > MAX_RECURSION {
        return Err(format!("Max recursion depth exceeded for M3U8: {}", url_str));
    }

    if content.contains("#EXT-X-STREAM-INF") || content.contains("#EXT-X-MEDIA:") {
        let base_url = get_base_url(url_str)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut best_url: Option<String> = None;
        let mut highest_bandwidth = -1;

        for i in 0..lines.len() {
            if lines[i].starts_with("#EXT-X-STREAM-INF") {
                let bandwidth = lines[i]
                    .split(',')
                    .find(|s| s.starts_with("BANDWIDTH="))
                    .and_then(|s| s.split('=').nth(1))
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);

                if i + 1 < lines.len() && !lines[i + 1].trim().is_empty() && !lines[i + 1].starts_with('#') {
                    let uri = lines[i + 1].trim();
                    if bandwidth >= highest_bandwidth {
                        highest_bandwidth = bandwidth;
                        best_url = resolve_url(&base_url, uri).ok();
                    }
                }
            }
        }

        if best_url.is_none() {
            for line in &lines {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains(".m3u8") {
                    best_url = resolve_url(&base_url, trimmed).ok();
                    break;
                }
            }
        }

        let variant_url = best_url.unwrap_or_else(|| url_str.to_string());
        let (sub_content, sub_type) = fetch_url(&variant_url).await?;

        if is_m3u8_content(&sub_content, &sub_type) {
            Box::pin(process_m3u8_recursive(&variant_url, &sub_content, ad_filtering, depth + 1)).await
        } else {
            Ok(process_media_playlist(&variant_url, &sub_content, ad_filtering))
        }
    } else {
        Ok(process_media_playlist(url_str, content, ad_filtering))
    }
}

fn process_media_playlist(url_str: &str, content: &str, ad_filtering: bool) -> String {
    let base_url = get_base_url(url_str).unwrap_or_else(|_| url_str.to_string());
    let processed = if ad_filtering { filter_ad_segments(content) } else { content.to_string() };

    processed.lines().map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return trimmed.to_string();
        }

        if trimmed.starts_with("#EXT-X-KEY") {
            process_key_line(&base_url, trimmed)
        } else if trimmed.starts_with("#EXT-X-MAP") {
            process_map_line(&base_url, trimmed)
        } else if trimmed.starts_with('#') {
            trimmed.to_string()
        } else {
            resolve_url(&base_url, trimmed).unwrap_or_else(|_| trimmed.to_string())
        }
    }).collect::<Vec<_>>().join("\n")
}

fn process_key_line(base_url: &str, line: &str) -> String {
    let parts: Vec<&str> = line.split(',').collect();
    let mut new_parts = Vec::new();
    for part in parts {
        if part.starts_with("URI=\"") {
            let uri_val = part.trim_start_matches("URI=\"").trim_end_matches('"');
            if let Ok(abs) = resolve_url(base_url, uri_val) {
                new_parts.push(format!("URI=\"{}\"", abs));
            } else {
                new_parts.push(part.to_string());
            }
        } else {
            new_parts.push(part.to_string());
        }
    }
    new_parts.join(",")
}

fn process_map_line(base_url: &str, line: &str) -> String {
    if let Some(start) = line.find("URI=\"") {
        let rest = &line[start + 5..];
        if let Some(end) = rest.find('"') {
            let uri = &rest[..end];
            if let Ok(abs) = resolve_url(base_url, uri) {
                return format!("{}URI=\"{}\"{}", &line[..start], abs, &rest[end + 1..]);
            }
        }
    }
    line.to_string()
}

pub async fn fetch_media_segment(url: &str) -> Result<Vec<u8>, HttpError> {
    fetch_bytes(url).await.map_err(|e| HttpError {
        error: "Request failed".to_string(),
        details: Some(e.to_string()),
    })
}

pub async fn fetch_m3u8_content(url: &str, ad_filtering: bool) -> Result<String, HttpError> {
    let (content, content_type) = fetch_url(url).await.map_err(|e| HttpError {
        error: e.clone(),
        details: Some(e),
    })?;

    if !is_m3u8_content(&content, &content_type) {
        return Err(HttpError {
            error: "Not an m3u8 content".to_string(),
            details: Some(content_type),
        });
    }

    let processed = process_m3u8_recursive(url, &content, ad_filtering, 0).await
        .map_err(|e| HttpError {
            error: "M3U8 processing error".to_string(),
            details: Some(e),
        })?;

    Ok(processed)
}

async fn fetch_url(url: &str) -> Result<(String, String), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let referer = get_base_url(url).unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer).unwrap_or_else(|_| HeaderValue::from_static("https://www.google.com")));

    let response = client.get(url).headers(headers).send().await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status.as_u16()));
    }

    let content_type = response.headers().get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let content = response.text().await
        .map_err(|e| format!("Read error: {}", e))?;

    Ok((content, content_type))
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let referer = get_base_url(url).unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("LibreTV/1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer).unwrap_or_else(|_| HeaderValue::from_static("*")));

    let response = client.get(url).headers(headers).send().await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status().as_u16()));
    }

    response.bytes().await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Read error: {}", e))
}
