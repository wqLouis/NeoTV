use std::collections::HashMap;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, REFERER, ACCEPT};
use serde::{Deserialize, Serialize};
use url;

const MAX_RECURSION: u8 = 5;

static USER_AGENTS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15".to_string(),
        "Mozilla/5.0 (Linux; Android 10; SM-G975F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36".to_string(),
    ]
});

fn get_random_user_agent() -> String {
    USER_AGENTS.get(0).cloned().unwrap_or_else(|| "LibreTV/1.0".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub content_type: String,
    pub is_m3u8: bool,
    pub processed_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpError {
    pub error: String,
    pub details: Option<String>,
}

pub fn is_m3u8_content(content: &str, content_type: &str) -> bool {
    if content_type.contains("application/vnd.apple.mpegurl")
        || content_type.contains("application/x-mpegurl")
        || content_type.contains("audio/mpegurl")
    {
        return true;
    }
    content.trim_start().starts_with("#EXTM3U")
}

fn get_base_url(url_str: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;
    
    let path_segments: Vec<&str> = parsed.path_segments().map_or(Vec::new(), |s| s.collect());
    
    if path_segments.is_empty() || (path_segments.len() == 1 && parsed.path() == "/") {
        return Ok(format!("{}://{}/", parsed.scheme(), parsed.host_str().unwrap_or("")));
    }
    
    let mut new_url = parsed.clone();
    let path_parts: Vec<&str> = new_url.path().trim_end_matches('/').split('/').collect();
    
    if path_parts.len() > 1 {
        let new_path = path_parts[..path_parts.len() - 1].join("/");
        new_url.set_path(&format!("{}/", new_path));
    } else {
        new_url.set_path("/");
    }
    
    Ok(new_url.to_string())
}

fn resolve_url(base_url: &str, relative_url: &str) -> Result<String, String> {
    if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
        return Ok(relative_url.to_string());
    }
    
    let base = url::Url::parse(base_url).map_err(|e| format!("Invalid base URL: {}", e))?;
    base.join(relative_url).map(|u: url::Url| u.to_string()).map_err(|e| format!("URL join error: {:?}", e))
}

fn rewrite_url_to_proxy(target_url: &str) -> String {
    format!("app-media://{}", urlencoding::encode(target_url))
}

pub async fn process_m3u8_content(
    target_url: &str,
    content: String,
    recursion_depth: u8,
) -> Result<String, String> {
    if recursion_depth > MAX_RECURSION {
        return Err(format!("Max recursion depth exceeded for M3U8: {}", target_url));
    }

    if content.contains("#EXT-X-STREAM-INF") || content.contains("#EXT-X-MEDIA:") {
        process_master_playlist(target_url, &content, recursion_depth).await
    } else {
        Ok(process_media_playlist(target_url, &content))
    }
}

async fn process_master_playlist(
    url_str: &str,
    content: &str,
    recursion_depth: u8,
) -> Result<String, String> {
    let base_url = get_base_url(url_str)?;
    let mut highest_bandwidth = -1;
    let mut best_variant_url: Option<String> = None;

    let lines: Vec<&str> = content.lines().collect();
    for i in 0..lines.len() {
        if lines[i].starts_with("#EXT-X-STREAM-INF") {
            let bandwidth_match = lines[i]
                .split(',')
                .find(|s| s.starts_with("BANDWIDTH="))
                .and_then(|s| s.split('=').nth(1))
                .and_then(|s| s.parse::<i64>().ok());

            let current_bandwidth = bandwidth_match.unwrap_or(0);

            if i + 1 < lines.len() && !lines[i + 1].trim().is_empty() && !lines[i + 1].starts_with('#') {
                let variant_uri = lines[i + 1].trim();
                if current_bandwidth >= highest_bandwidth {
                    highest_bandwidth = current_bandwidth;
                    best_variant_url = Some(resolve_url(&base_url, variant_uri)?);
                }
            }
        }
    }

    if best_variant_url.is_none() {
        for line in lines.iter() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') && trimmed_line.contains(".m3u8") {
                best_variant_url = Some(resolve_url(&base_url, trimmed_line)?);
                break;
            }
        }
    }

    if let Some(variant_url) = best_variant_url {
        let sub_playlist_data = fetch_content(&variant_url).await?;
        if is_m3u8_content(&sub_playlist_data.content, &sub_playlist_data.content_type) {
            Box::pin(process_m3u8_content(&variant_url, sub_playlist_data.content, recursion_depth + 1)).await
        } else {
            Ok(process_media_playlist(&variant_url, &sub_playlist_data.content))
        }
    } else {
        Ok(process_media_playlist(url_str, content))
    }
}

fn process_media_playlist(url_str: &str, content: &str) -> String {
    let base_url = get_base_url(url_str).unwrap_or_else(|_| url_str.to_string());
    let mut output_lines = Vec::new();

    for line in content.lines() {
        let trimmed_line = line.trim();
        
        if trimmed_line.is_empty() {
            output_lines.push(trimmed_line.to_string());
            continue;
        }

        if trimmed_line.starts_with("#EXT-X-KEY") {
            let parts: Vec<&str> = trimmed_line.split(',').collect();
            let mut new_parts = Vec::new();
            for part in parts {
                if part.starts_with("URI=\"") {
                    let uri_val = part.trim_start_matches("URI=\"").trim_end_matches('"');
                    if let Ok(absolute_uri) = resolve_url(&base_url, uri_val) {
                        let proxied_uri = rewrite_url_to_proxy(&absolute_uri);
                        new_parts.push(format!("URI=\"{}\"", proxied_uri));
                    } else {
                        new_parts.push(part.to_string());
                    }
                } else {
                    new_parts.push(part.to_string());
                }
            }
            output_lines.push(new_parts.join(","));
        } else if trimmed_line.starts_with("#EXT-X-MAP") {
            if let Some(uri_start) = trimmed_line.find("URI=\"") {
                let uri_and_rest = &trimmed_line[uri_start + 5..];
                if let Some(uri_end) = uri_and_rest.find('"') {
                    let uri_val = &uri_and_rest[..uri_end];
                    if let Ok(absolute_uri) = resolve_url(&base_url, uri_val) {
                        let proxied_uri = rewrite_url_to_proxy(&absolute_uri);
                        output_lines.push(format!("{}URI=\"{}\"{}", &trimmed_line[..uri_start], proxied_uri, &uri_and_rest[uri_end + 1..]));
                    } else {
                        output_lines.push(trimmed_line.to_string());
                    }
                } else {
                    output_lines.push(trimmed_line.to_string());
                }
            } else {
                output_lines.push(trimmed_line.to_string());
            }
        } else if !trimmed_line.starts_with('#') {
            if let Ok(absolute_url) = resolve_url(&base_url, trimmed_line) {
                output_lines.push(rewrite_url_to_proxy(&absolute_url));
            } else {
                output_lines.push(trimmed_line.to_string());
            }
        } else {
            output_lines.push(trimmed_line.to_string());
        }
    }
    
    output_lines.join("\n")
}

struct FetchResult {
    content: String,
    content_type: String,
}

async fn fetch_content(url: &str) -> Result<FetchResult, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let referer_base = get_base_url(url).unwrap_or_else(|_| url.to_string());

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("LibreTV/1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer_base).unwrap_or_else(|_| HeaderValue::from_static("*")));

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status.as_u16()));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let content = response.text().await.map_err(|e| format!("Read error: {}", e))?;

    Ok(FetchResult { content, content_type })
}

pub async fn fetch_and_process_m3u8(url: &str) -> Result<MediaInfo, HttpError> {
    let fetch_result = fetch_content(url).await.map_err(|e| HttpError {
        error: e.clone(),
        details: Some(e),
    })?;

    let is_m3u8 = is_m3u8_content(&fetch_result.content, &fetch_result.content_type);
    
    if is_m3u8 {
        let processed = process_m3u8_content(url, fetch_result.content, 0).await.map_err(|e| HttpError {
            error: "M3U8 processing error".to_string(),
            details: Some(e),
        })?;
        
        Ok(MediaInfo {
            url: rewrite_url_to_proxy(url),
            content_type: "application/vnd.apple.mpegurl;charset=utf-8".to_string(),
            is_m3u8: true,
            processed_content: Some(processed),
        })
    } else {
        Ok(MediaInfo {
            url: url.to_string(),
            content_type: fetch_result.content_type,
            is_m3u8: false,
            processed_content: None,
        })
    }
}

pub async fn fetch_media_segment(url: &str) -> Result<Vec<u8>, HttpError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HttpError {
            error: "Client build error".to_string(),
            details: Some(e.to_string()),
        })?;

    let referer_base = get_base_url(url).unwrap_or_else(|_| url.to_string());

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("LibreTV/1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer_base).unwrap_or_else(|_| HeaderValue::from_static("*")));

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| HttpError {
            error: "Request failed".to_string(),
            details: Some(e.to_string()),
        })?;

    if !response.status().is_success() {
        return Err(HttpError {
            error: format!("HTTP error: {}", response.status().as_u16()),
            details: None,
        });
    }

    let bytes = response.bytes().await.map_err(|e| HttpError {
        error: "Read error".to_string(),
        details: Some(e.to_string()),
    })?;

    Ok(bytes.to_vec())
}
