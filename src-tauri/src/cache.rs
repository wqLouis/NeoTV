use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

const MAX_CACHE_SIZE_BYTES: usize = 50 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub content_type: String,
    pub cached_at: u64,
}

#[derive(Debug)]
pub struct LruCache {
    entries: HashMap<String, CacheEntry>,
    access_order: VecDeque<String>,
    total_size: usize,
}

impl LruCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            total_size: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.to_string());
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, entry: CacheEntry) {
        let entry_size = entry.data.len();

        if let Some(old) = self.entries.remove(&key) {
            self.total_size -= old.data.len();
            self.access_order.retain(|k| k != &key);
        }

        while self.entries.len() >= MAX_CACHE_ENTRIES || self.total_size + entry_size > MAX_CACHE_SIZE_BYTES {
            if let Some(oldest) = self.access_order.pop_front() {
                if let Some(removed) = self.entries.remove(&oldest) {
                    self.total_size -= removed.data.len();
                }
            } else {
                break;
            }
        }

        self.total_size += entry_size;
        self.entries.insert(key.clone(), entry);
        self.access_order.push_back(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.total_size = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn size(&self) -> usize {
        self.total_size
    }
}

impl Default for LruCache {
    fn default() -> Self {
        Self::new()
    }
}

static MEM_CACHE: OnceLock<Mutex<LruCache>> = OnceLock::new();

fn get_mem_cache() -> &'static Mutex<LruCache> {
    MEM_CACHE.get_or_init(|| Mutex::new(LruCache::new()))
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

pub fn record_hit() {
    CACHE_HITS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_miss() {
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
}

pub fn reset_stats() {
    CACHE_HITS.store(0, Ordering::Relaxed);
    CACHE_MISSES.store(0, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total: u64,
    pub hit_rate: f64,
    pub mem_count: usize,
    pub mem_size: usize,
    pub disk_count: usize,
    pub disk_size: usize,
}

pub fn get_cached(url: &str, ttl_secs: u64) -> Option<CacheEntry> {
    let key = hash_key(url);

    if let Ok(mut cache) = get_mem_cache().lock() {
        if let Some(entry) = cache.get(&key) {
            if !is_expired(entry, ttl_secs) {
                record_hit();
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
                    record_hit();
                    return Some(entry);
                }
            }
        }
    }

    record_miss();
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
    reset_stats();
}

pub fn get_cache_stats() -> CacheStats {
    let hits = CACHE_HITS.load(Ordering::Relaxed);
    let misses = CACHE_MISSES.load(Ordering::Relaxed);
    let total = hits + misses;
    let hit_rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };

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

    CacheStats {
        hits,
        misses,
        total,
        hit_rate,
        mem_count,
        mem_size,
        disk_count,
        disk_size,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestCacheEntry {
    pub result: SpeedTestResult,
    pub cached_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub source_id: String,
    pub source_name: String,
    pub latency_ms: u64,
    pub download_speed_kbps: f64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct SpeedCache {
    entries: HashMap<String, SpeedTestCacheEntry>,
    access_order: VecDeque<String>,
}

impl SpeedCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&SpeedTestCacheEntry> {
        let entry_opt = self.entries.get(key).cloned();
        
        match entry_opt {
            Some(entry) if current_timestamp() - entry.cached_at < 30 * 60 => {
                self.access_order.retain(|k| k != key);
                self.access_order.push_back(key.to_string());
                self.entries.get(key).map(|e| e)
            }
            Some(_) => {
                self.entries.remove(key);
                self.access_order.retain(|k| k != key);
                None
            }
            None => None,
        }
    }

    pub fn insert(&mut self, key: String, result: SpeedTestResult) {
        if let Some(_old) = self.entries.remove(&key) {
            self.access_order.retain(|k| k != &key);
        }

        while self.entries.len() >= 100 {
            if let Some(oldest) = self.access_order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        let entry = SpeedTestCacheEntry {
            result,
            cached_at: current_timestamp(),
        };
        self.entries.insert(key.clone(), entry);
        self.access_order.push_back(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
}

impl Default for SpeedCache {
    fn default() -> Self {
        Self::new()
    }
}

static SPEED_CACHE: OnceLock<Mutex<SpeedCache>> = OnceLock::new();

fn get_speed_cache() -> &'static Mutex<SpeedCache> {
    SPEED_CACHE.get_or_init(|| Mutex::new(SpeedCache::new()))
}

pub fn get_speed_cached(key: &str) -> Option<SpeedTestResult> {
    if let Ok(mut cache) = get_speed_cache().lock() {
        if let Some(entry) = cache.get(key) {
            return Some(entry.result.clone());
        }
    }
    None
}

pub fn set_speed_cached(key: &str, result: SpeedTestResult) {
    if let Ok(mut cache) = get_speed_cache().lock() {
        cache.insert(key.to_string(), result);
    }
}

pub fn clear_speed_cache() {
    if let Ok(mut cache) = get_speed_cache().lock() {
        cache.clear();
    }
}

pub fn clear_all_caches() {
    clear_cache();
    clear_speed_cache();
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