use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub content_type: String,
    pub cached_at: u64,
}

#[derive(Debug, Default)]
pub struct MemCache {
    entries: HashMap<String, CacheEntry>,
}

impl MemCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: String, entry: CacheEntry) {
        if self.entries.len() > 1000 {
            if let Some(oldest) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(key, entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn size(&self) -> usize {
        self.entries.values().map(|e| e.data.len()).sum()
    }
}

static MEM_CACHE: OnceLock<Mutex<MemCache>> = OnceLock::new();

fn get_mem_cache() -> &'static Mutex<MemCache> {
    MEM_CACHE.get_or_init(|| Mutex::new(MemCache::new()))
}

fn get_cache_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("libretv_cache");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

fn get_cache_path(key: &str) -> PathBuf {
    get_cache_dir().join(format!("cache_{}", key))
}

fn hash_key(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    format!("{:x}", s.finish())
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_expired(entry: &CacheEntry, ttl_secs: u64) -> bool {
    current_timestamp() - entry.cached_at > ttl_secs
}

pub fn get_cached(url: &str, ttl_secs: u64) -> Option<CacheEntry> {
    let key = hash_key(url);

    if let Ok(cache) = get_mem_cache().lock() {
        if let Some(entry) = cache.get(&key) {
            if !is_expired(entry, ttl_secs) {
                return Some(entry.clone());
            }
        }
    }

    let path = get_cache_path(&key);
    if path.exists() {
        if let Ok(data) = fs::read(&path) {
            if let Ok(entry) = serde_json::from_slice::<CacheEntry>(&data) {
                if !is_expired(&entry, ttl_secs) {
                    if let Ok(mut cache) = get_mem_cache().lock() {
                        cache.insert(key, entry.clone());
                    }
                    return Some(entry);
                }
            }
        }
    }

    None
}

pub fn set_cached(url: &str, data: Vec<u8>, content_type: String) {
    let key = hash_key(url);
    let entry = CacheEntry {
        data,
        content_type,
        cached_at: current_timestamp(),
    };

    if let Ok(mut cache) = get_mem_cache().lock() {
        cache.insert(key.clone(), entry.clone());
    }

    if let Ok(json) = serde_json::to_vec(&entry) {
        let path = get_cache_path(&key);
        let _ = fs::write(path, json);
    }
}

pub fn clear_cache() {
    if let Ok(mut cache) = get_mem_cache().lock() {
        cache.clear();
    }

    let cache_dir = get_cache_dir();
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
}

pub fn get_cache_stats() -> (usize, usize, usize, usize) {
    let (mem_count, mem_size) = if let Ok(cache) = get_mem_cache().lock() {
        (cache.len(), cache.size())
    } else {
        (0, 0)
    };

    let cache_dir = get_cache_dir();
    let mut disk_count = 0;
    let mut disk_size = 0;

    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                disk_count += 1;
                if let Ok(meta) = fs::metadata(entry.path()) {
                    disk_size += meta.len() as usize;
                }
            }
        }
    }

    (mem_count, mem_size, disk_count, disk_size)
}

pub async fn fetch_url(url: &str, referer: Option<&str>) -> Result<(Vec<u8>, String), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static("LibreTV/1.0"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));

    if let Some(referer_url) = referer {
        if let Ok(val) = HeaderValue::from_str(referer_url) {
            headers.insert("Referer", val);
        }
    }

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

    let bytes = response.bytes().await
        .map_err(|e| format!("Read error: {}", e))?;

    Ok((bytes.to_vec(), content_type))
}
