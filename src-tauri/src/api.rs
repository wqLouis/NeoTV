use crate::error::HttpError;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequestOptions {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(default)]
    pub cached: bool,
}

pub async fn http_request(options: HttpRequestOptions) -> Result<HttpResponse, HttpError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    eprintln!("[{}][DEBUG][API] http_request called: {} {:?}", timestamp, options.url, options.method);

    let client_builder = reqwest::Client::builder();

    let timeout_duration = std::time::Duration::from_secs(options.timeout_secs.unwrap_or(20));
    let client = match client_builder.timeout(timeout_duration).build() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to build HTTP client: {}", e);
            eprintln!("[API] {}", err_msg);
            return Err(HttpError {
                error: "Failed to build HTTP client".to_string(),
                details: Some(e.to_string()),
            });
        }
    };

    let method_str = options.method.unwrap_or_else(|| "GET".to_string());
    let method_for_reqwest = match method_str.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        _ => reqwest::Method::GET,
    };

    let mut request_builder = client.request(method_for_reqwest.clone(), &options.url);

    let mut req_headers = HeaderMap::new();
    req_headers.insert(USER_AGENT, HeaderValue::from_static("LibreTV-TauriApp/1.0"));

    if let Some(h) = options.headers {
        for (key, value) in h {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(&value) {
                    req_headers.insert(header_name, header_value);
                }
            }
        }
    }
    request_builder = request_builder.headers(req_headers.clone());

    if let Some(body_value) = options.body {
        let content_type_is_json = req_headers.get(CONTENT_TYPE).map_or(false, |ct| {
            ct.to_str().unwrap_or("").contains("application/json")
        });

        if method_str.eq_ignore_ascii_case("POST")
            || method_str.eq_ignore_ascii_case("PUT")
            || method_str.eq_ignore_ascii_case("PATCH")
        {
            if content_type_is_json {
                request_builder = request_builder.json(&body_value);
            } else if let Some(body_str) = body_value.as_str() {
                request_builder = request_builder.body(body_str.to_string());
            } else {
                request_builder = request_builder.body(body_value.to_string());
            }
        }
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let mut resp_headers = HashMap::new();
            for (key, value) in response.headers().iter() {
                if let Ok(val_str) = value.to_str() {
                    resp_headers.insert(key.as_str().to_string(), val_str.to_string());
                }
            }

            match response.text().await {
                Ok(text_body) => {
                    let success_msg = format!("http_request successful! status={}, body_len={}", status, text_body.len());
                    eprintln!("[API] {}", success_msg);
                    Ok(HttpResponse {
                        status,
                        headers: resp_headers,
                        body: text_body,
                        cached: false,
                    })
                }
                Err(e) => {
                    let err_msg = format!("Failed to read response body: {}", e);
                    eprintln!("[API] {}", err_msg);
                    Err(HttpError {
                        error: "Failed to read response body".to_string(),
                        details: Some(e.to_string()),
                    })
                }
            }
        }
        Err(e) => {
            let err_msg = format!("http_request failed: {} - {}", options.url, e);
            eprintln!("[API] {}", err_msg);
            Err(HttpError {
                error: "Request failed".to_string(),
                details: Some(e.to_string()),
            })
        }
    }
}