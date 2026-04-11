use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, REFERER, USER_AGENT};
use std::time::Duration;

pub fn create_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Client build error: {}", e))
}

pub async fn fetch_text(url: &str, referer: Option<&str>) -> Result<(String, String), String> {
    let client = create_client(20)?;

    let referer_url = referer
        .map(String::from)
        .or_else(|| get_base_url(url).ok())
        .unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer_url).unwrap_or_else(|_| HeaderValue::from_static("https://www.google.com")));

    let response = client.get(url).headers(headers).send().await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status.as_u16()));
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let content = response.text().await
        .map_err(|e| format!("Read error: {}", e))?;

    Ok((content, content_type))
}

pub async fn fetch_bytes_with_content_type(url: &str, referer: Option<&str>) -> Result<(Vec<u8>, String), String> {
    let client = create_client(30)?;

    let referer_url = referer
        .map(String::from)
        .or_else(|| get_base_url(url).ok())
        .unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("LibreTV/1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(&referer_url).unwrap_or_else(|_| HeaderValue::from_static("*")));

    let response = client.get(url).headers(headers).send().await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status().as_u16()));
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response.bytes().await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Read error: {}", e))?;

    Ok((bytes, content_type))
}

pub async fn fetch_bytes(url: &str, referer: Option<&str>) -> Result<Vec<u8>, String> {
    fetch_bytes_with_content_type(url, referer).await.map(|(b, _)| b)
}

pub fn get_base_url(url_str: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;
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

pub fn resolve_url(base_url: &str, relative_url: &str) -> Result<String, String> {
    if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
        return Ok(relative_url.to_string());
    }
    url::Url::parse(base_url)
        .and_then(|u| u.join(relative_url))
        .map(|u| u.to_string())
        .map_err(|e| format!("URL join error: {}", e))
}
