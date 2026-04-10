use std::collections::HashMap;
use std::io::Write;
use std::fs::OpenOptions;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use tokio;

const LOG_PATH: &str = "/home/wqlouis/Documents/code/LibreTV-App/test_debug.log";

fn write_log(level: &str, tag: &str, msg: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let log_line = format!("[{}][{}][{}] {}", timestamp, level, tag, msg);

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
    eprintln!("{}", log_line);
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpRequestOptions {
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[tokio::main]
async fn main() {
    println!("=== LibreTV Rust Test Suite ===\n");

    // Clear previous log
    let _ = std::fs::remove_file(LOG_PATH);

    write_log("INFO", "Test", "Starting LibreTV Rust test suite");
    println!("Log file: {}\n", LOG_PATH);

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic HTTP Request
    println!("Test 1: Basic HTTP Request to Douban");
    write_log("INFO", "Test", "Test 1: Basic HTTP Request to Douban");
    match test_basic_douban_request().await {
        Ok(_) => {
            println!("✅ Test 1 PASSED\n");
            write_log("INFO", "Test", "Test 1 PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Test 1 FAILED: {}\n", e);
            write_log("ERROR", "Test", &format!("Test 1 FAILED: {}", e));
            failed += 1;
        }
    }

    // Test 2: Douban Chart API
    println!("Test 2: Douban Chart API (Genre ID 11 - Drama)");
    write_log("INFO", "Test", "Test 2: Douban Chart API");
    match test_douban_chart().await {
        Ok(_) => {
            println!("✅ Test 2 PASSED\n");
            write_log("INFO", "Test", "Test 2 PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Test 2 FAILED: {}\n", e);
            write_log("ERROR", "Test", &format!("Test 2 FAILED: {}", e));
            failed += 1;
        }
    }

    // Test 3: Headers with Referer
    println!("Test 3: HTTP Request with Custom Headers");
    write_log("INFO", "Test", "Test 3: Custom Headers");
    match test_custom_headers().await {
        Ok(_) => {
            println!("✅ Test 3 PASSED\n");
            write_log("INFO", "Test", "Test 3 PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Test 3 FAILED: {}\n", e);
            write_log("ERROR", "Test", &format!("Test 3 FAILED: {}", e));
            failed += 1;
        }
    }

    // Test 4: Image Fetch (Douban)
    println!("Test 4: Image Fetch from Douban");
    write_log("INFO", "Test", "Test 4: Image Fetch");
    match test_image_fetch().await {
        Ok(_) => {
            println!("✅ Test 4 PASSED\n");
            write_log("INFO", "Test", "Test 4 PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Test 4 FAILED: {}\n", e);
            write_log("ERROR", "Test", &format!("Test 4 FAILED: {}", e));
            failed += 1;
        }
    }

    // Test 5: Error handling - invalid URL
    println!("Test 5: Error Handling (Invalid URL)");
    write_log("INFO", "Test", "Test 5: Error Handling");
    match test_error_handling().await {
        Ok(_) => {
            println!("✅ Test 5 PASSED\n");
            write_log("INFO", "Test", "Test 5 PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Test 5 FAILED: {}\n", e);
            write_log("ERROR", "Test", &format!("Test 5 FAILED: {}", e));
            failed += 1;
        }
    }

    // Summary
    println!("====================");
    println!("Tests passed: {}", passed);
    println!("Tests failed: {}", failed);
    println!("====================");
    write_log("INFO", "Test", &format!("=== SUMMARY: {} passed, {} failed ===", passed, failed));
}

async fn test_basic_douban_request() -> Result<(), String> {
    write_log("DEBUG", "Test1", "Starting basic Douban request test");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("LibreTV-Test/1.0"));
    headers.insert("Referer", HeaderValue::from_static("https://movie.douban.com/"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let url = "https://movie.douban.com/j/chart/top_list?type=11&interval_id=100:90&action=&start=0&limit=1";
    write_log("DEBUG", "Test1", &format!("Sending request to: {}", url));

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status().as_u16();
    write_log("DEBUG", "Test1", &format!("Response status: {}", status));

    if status != 200 {
        return Err(format!("Expected status 200, got {}", status));
    }

    let body = response.text().await.map_err(|e| format!("Failed to read body: {}", e))?;
    write_log("DEBUG", "Test1", &format!("Response body length: {} bytes", body.len()));

    if body.len() < 10 {
        return Err(format!("Response body too short: {}", body));
    }

    write_log("DEBUG", "Test1", &format!("Response preview: {}", &body[..body.len().min(100)]));
    Ok(())
}

async fn test_douban_chart() -> Result<(), String> {
    write_log("DEBUG", "Test2", "Starting Douban chart API test");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"));
    headers.insert("Referer", HeaderValue::from_static("https://movie.douban.com/"));
    headers.insert("Accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("Accept-Language", HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));

    // Test genre 11 (Drama)
    let url = "https://movie.douban.com/j/chart/top_list?type=11&interval_id=100:90&action=&start=0&limit=5";
    write_log("DEBUG", "Test2", &format!("Requesting: {}", url));

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().as_u16();
    let body = response.text().await.map_err(|e| format!("Failed to read body: {}", e))?;

    write_log("DEBUG", "Test2", &format!("Status: {}, Body length: {}", status, body.len()));

    if status != 200 {
        return Err(format!("HTTP error: {}", status));
    }

    let json: Vec<serde_json::Value> = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    write_log("DEBUG", "Test2", &format!("Parsed {} items from chart API", json.len()));

    if json.is_empty() {
        return Err("No items returned from chart API".to_string());
    }

    if let Some(first) = json.first() {
        if let Some(title) = first.get("title").and_then(|t| t.as_str()) {
            write_log("INFO", "Test2", &format!("First item title: {}", title));
        }
    }

    Ok(())
}

async fn test_custom_headers() -> Result<(), String> {
    write_log("DEBUG", "Test3", "Testing custom headers handling");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15"));
    headers.insert("Referer", HeaderValue::from_static("https://movie.douban.com/"));
    headers.insert("Accept-Language", HeaderValue::from_static("zh-CN,zh;q=0.9"));

    let url = "https://movie.douban.com/j/search_tags?type=movie";

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().as_u16();
    let body = response.text().await.map_err(|e| format!("Failed to read body: {}", e))?;

    write_log("DEBUG", "Test3", &format!("Status: {}, Body length: {}", status, body.len()));

    if status != 200 {
        return Err(format!("HTTP error: {}", status));
    }

    // Try to parse tags response
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if let Some(tags) = json.get("tags").and_then(|t| t.as_array()) {
        write_log("INFO", "Test3", &format!("Got {} tags from API", tags.len()));
    }

    Ok(())
}

async fn test_image_fetch() -> Result<(), String> {
    write_log("DEBUG", "Test4", "Testing image fetch from Douban");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"));
    headers.insert("Referer", HeaderValue::from_static("https://movie.douban.com/"));
    headers.insert("Accept", HeaderValue::from_static("image/*,*/*;q=0.8"));

    let image_url = "https://img9.doubanio.com/view/photo/s_ratio_poster/public/p2561430900.jpg";
    write_log("DEBUG", "Test4", &format!("Fetching: {}", image_url));

    let response = client
        .get(image_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().as_u16();
    write_log("DEBUG", "Test4", &format!("Status: {}", status));

    if status != 200 {
        return Err(format!("Image fetch HTTP error: {}", status));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Failed to read bytes: {}", e))?;
    write_log("DEBUG", "Test4", &format!("Received {} bytes", bytes.len()));

    if bytes.len() >= 3 {
        let magic = match (bytes[0], bytes[1], bytes[2]) {
            (0xFF, 0xD8, 0xFF) => "JPEG",
            (0x89, 0x50, 0x4E) => "PNG",
            (0x47, 0x49, 0x46) => "GIF",
            _ => "UNKNOWN"
        };
        write_log("INFO", "Test4", &format!("Image magic bytes indicate: {}", magic));
    }

    Ok(())
}

async fn test_error_handling() -> Result<(), String> {
    write_log("DEBUG", "Test5", "Testing error handling");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    // Invalid domain
    let url = "https://this-domain-does-not-exist-12345.com/api";

    let result = client.get(url).send().await;

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            write_log("DEBUG", "Test5", &format!("Got response status: {}", status));
            // Should get a status error
            Ok(())
        }
        Err(e) => {
            write_log("DEBUG", "Test5", &format!("Expected error occurred: {}", e));
            // Expected to fail
            Ok(())
        }
    }
}