use crate::http;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

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

impl CacheEntry {}

#[derive(Debug)]
pub struct LruCache {
    entries: BTreeMap<String, Arc<CacheEntry>>,
    access_order: Vec<String>,
    total_size: usize,
}

impl LruCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            access_order: Vec::new(),
            total_size: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<Arc<CacheEntry>> {
        if let Some(entry) = self.entries.get(key).cloned() {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.swap_remove(pos);
                self.access_order.push(key.to_string());
            }
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, entry: Arc<CacheEntry>) {
        let entry_size = entry.data.len();

        if let Some(old) = self.entries.remove(&key) {
            self.total_size -= old.data.len();
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.swap_remove(pos);
            }
        }

        while self.entries.len() >= MAX_CACHE_ENTRIES || self.total_size + entry_size > MAX_CACHE_SIZE_BYTES {
            if let Some(oldest) = self.access_order.first().cloned() {
                if let Some(removed) = self.entries.remove(&oldest) {
                    self.total_size -= removed.data.len();
                    self.access_order.remove(0);
                }
            } else {
                break;
            }
        }

        self.total_size += entry_size;
        self.entries.insert(key.clone(), entry);
        self.access_order.push(key);
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
static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

fn get_mem_cache() -> &'static Mutex<LruCache> {
    MEM_CACHE.get_or_init(|| Mutex::new(LruCache::new()))
}

pub fn init_cache_dir(dir: PathBuf) {
    let _ = CACHE_DIR.set(dir);
}

fn get_cache_dir() -> PathBuf {
    if let Some(dir) = CACHE_DIR.get() {
        return dir.clone();
    }
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("libretv_cache");
    dir
}

fn encode_key(url: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(url.as_bytes())
}

fn decode_key(encoded: &str) -> Option<String> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

fn get_cache_path(key: &str) -> PathBuf {
    get_cache_dir().join(format!("cache_{}", key))
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

pub async fn load_cache_from_disk() {
    let cache_dir = get_cache_dir();
    if !cache_dir.exists() {
        eprintln!("[Cache] Cache dir does not exist: {:?}", cache_dir);
        return;
    }

    let entries = match std::fs::read_dir(&cache_dir) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("[Cache] Failed to read cache dir: {}", e);
            return;
        }
    };

    let mut loaded = 0;
    let mut failed = 0;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[Cache] Failed to read dir entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let encoded_key = match filename.strip_prefix("cache_") {
            Some(key) => key,
            None => continue,
        };

        let url = match decode_key(encoded_key) {
            Some(decoded) => decoded,
            None => {
                eprintln!("[Cache] Failed to decode key: {}", encoded_key);
                failed += 1;
                continue;
            }
        };

        let data = match tokio::fs::read(&path).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("[Cache] Failed to read file {:?}: {}", path, e);
                failed += 1;
                continue;
            }
        };

        let cache_entry: CacheEntry = match serde_json::from_slice(&data) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("[Cache] Failed to parse cache file {:?}: {}", path, e);
                failed += 1;
                continue;
            }
        };

        if let Ok(mut cache) = get_mem_cache().lock() {
            cache.insert(url, Arc::new(cache_entry));
            loaded += 1;
        }
    }

    eprintln!("[Cache] Loaded {} entries from disk, {} failed", loaded, failed);
}

pub async fn get_cached(url: &str, ttl_secs: u64) -> Option<Arc<CacheEntry>> {
    if let Ok(mut cache) = get_mem_cache().lock() {
        if let Some(entry) = cache.get(url) {
            if !is_expired(entry.as_ref(), ttl_secs) {
                record_hit();
                return Some(entry);
            }
        }
    }

    let encoded_key = encode_key(url);
    let path = get_cache_path(&encoded_key);
    if path.exists() {
        match tokio::fs::read(&path).await {
            Ok(data) => {
                match serde_json::from_slice::<CacheEntry>(&data) {
                    Ok(entry) => {
                        if !is_expired(&entry, ttl_secs) {
                            let entry = Arc::new(entry);
                            if let Ok(mut cache) = get_mem_cache().lock() {
                                cache.insert(url.to_string(), entry.clone());
                            }
                            record_hit();
                            return Some(entry);
                        }
                    }
                    Err(e) => {
                        eprintln!("[Cache] Failed to parse cache file {:?}: {}", path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[Cache] Failed to read cache file {:?}: {}", path, e);
            }
        }
    }

    record_miss();
    None
}

pub async fn set_cached(url: &str, data: Vec<u8>, content_type: String) {
    let entry = CacheEntry {
        data,
        content_type,
        cached_at: current_timestamp(),
    };

    let entry = Arc::new(entry);

    if let Ok(mut cache) = get_mem_cache().lock() {
        cache.insert(url.to_string(), entry.clone());
    }

    let encoded_key = encode_key(url);
    let path = get_cache_path(&encoded_key);

    let cache_dir = get_cache_dir();
    if !cache_dir.exists() {
        if let Err(e) = tokio::fs::create_dir_all(&cache_dir).await {
            eprintln!("[Cache] Failed to create cache dir: {}", e);
            return;
        }
    }

    match serde_json::to_vec(entry.as_ref()) {
        Ok(json) => {
            if let Err(e) = tokio::fs::write(&path, &json).await {
                eprintln!("[Cache] Failed to write cache file {:?}: {}", path, e);
                return;
            }
            eprintln!("[Cache] Wrote cache: {} -> {:?}", url, path);
        }
        Err(e) => {
            eprintln!("[Cache] Failed to serialize cache entry: {}", e);
        }
    }
}

pub async fn clear_cache() {
    if let Ok(mut cache) = get_mem_cache().lock() {
        cache.clear();
    }

    let cache_dir = get_cache_dir();
    if !cache_dir.exists() {
        return;
    }

    match std::fs::read_dir(&cache_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let _ = tokio::fs::remove_file(entry.path()).await;
                }
            }
        }
        Err(e) => {
            eprintln!("[Cache] Failed to read cache dir: {}", e);
        }
    }
    reset_stats();
}

pub async fn get_cache_stats() -> CacheStats {
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

    if cache_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_file() {
                        disk_count += 1;
                        if let Ok(meta) = entry.metadata() {
                            disk_size += meta.len() as usize;
                        }
                    }
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
    pub network_id: String,
}

#[derive(Debug)]
pub struct SpeedCache {
    entries: BTreeMap<String, SpeedTestCacheEntry>,
    access_order: Vec<String>,
}

impl SpeedCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            access_order: Vec::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&SpeedTestCacheEntry> {
        let entry_opt = self.entries.get(key).cloned();
        
        match entry_opt {
            Some(_entry) => {
                self.access_order.retain(|k| k != key);
                self.access_order.push(key.to_string());
                self.entries.get(key).map(|e| e)
            }
            None => None,
        }
    }

    pub fn insert(&mut self, key: String, result: SpeedTestResult) {
        if let Some(_) = self.entries.remove(&key) {
            self.access_order.retain(|k| k != &key);
        }

        let entry = SpeedTestCacheEntry {
            result,
            cached_at: current_timestamp(),
        };
        self.entries.insert(key.clone(), entry);
        self.access_order.push(key);
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

pub fn get_speed_cache() -> &'static Mutex<SpeedCache> {
    SPEED_CACHE.get_or_init(|| Mutex::new(SpeedCache::new()))
}

pub fn get_all_speed_cached() -> Vec<SpeedTestResult> {
    if let Ok(cache) = get_speed_cache().lock() {
        cache.entries.values().map(|e| e.result.clone()).collect()
    } else {
        Vec::new()
    }
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

pub async fn clear_all_caches() {
    clear_cache().await;
    clear_speed_cache();
}

pub async fn fetch_url(url: &str, referer: Option<&str>) -> Result<(Vec<u8>, String), String> {
    http::fetch_bytes_with_content_type(url, referer).await
}