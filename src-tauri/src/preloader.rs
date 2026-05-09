use crate::error::HttpError;
use crate::http;
use crate::m3u8;
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinSet;
use tokio::time::sleep;

const DEFAULT_WORKER_COUNT: usize = 6;
const DEFAULT_MAX_CACHE_BYTES: usize = 512 * 1024 * 1024;
const MAX_RETRIES: usize = 3;
pub static PRELOADER: Lazy<Preloader> = Lazy::new(Preloader::new);

#[derive(Debug, Clone)]
struct CachedSegment {
    data: Vec<u8>,
}

/// All cache state behind a single mutex — no lock-ordering problems.
struct CacheData {
    entries: HashMap<String, CachedSegment>,
    /// LRU order: front = oldest (eligible for eviction), back = newest
    lru: VecDeque<String>,
    /// Maps URL -> segment index, so we can compare against `played_up_to`
    indices: HashMap<String, usize>,
    total_bytes: usize,
    max_bytes: usize,
}

impl CacheData {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru: VecDeque::new(),
            indices: HashMap::new(),
            total_bytes: 0,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        }
    }

    /// Insert a segment. Returns `true` if there was room (inserted),
    /// `false` if insertion was skipped because we can't evict un-played data.
    fn try_insert(
        &mut self,
        url: String,
        data: Vec<u8>,
        index: usize,
        played_up_to: usize,
    ) -> bool {
        let data_len = data.len();

        // If we already have this URL, skip
        if self.entries.contains_key(&url) {
            return true;
        }

        // Make room: evict only played segments (index < played_up_to)
        self.evict_played(played_up_to);

        // Check if there's room now
        if self.total_bytes + data_len > self.max_bytes {
            // Still no room — caller should pause preloading
            return false;
        }

        self.entries.insert(
            url.clone(),
            CachedSegment { data },
        );
        self.lru.push_back(url.clone());
        self.indices.insert(url, index);
        self.total_bytes += data_len;
        true
    }

    /// Evict segments whose index < played_up_to, until we free at least
    /// `needed` bytes, or run out of evictable segments.
    fn evict_played(&mut self, played_up_to: usize) {
        let mut evicted = 0usize;
        while let Some(front_url) = self.lru.front() {
            let idx = match self.indices.get(front_url) {
                Some(&i) => i,
                None => break, // no index info — keep it
            };
            if idx >= played_up_to {
                break; // this segment hasn't been played yet — stop evicting
            }
            // Evict this played segment
            let url = self.lru.pop_front().unwrap();
            if let Some(seg) = self.entries.remove(&url) {
                self.total_bytes -= seg.data.len();
                self.indices.remove(&url);
                evicted += 1;
            }
        }
        if evicted > 0 {
            eprintln!(
                "[Preloader] Evicted {} played segments, total_bytes={}",
                evicted, self.total_bytes
            );
        }
    }

    fn contains(&self, url: &str) -> bool {
        self.entries.contains_key(url)
    }

    fn get(&self, url: &str) -> Option<Vec<u8>> {
        self.entries.get(url).map(|s| s.data.clone())
    }

    fn get_index(&self, url: &str) -> Option<usize> {
        self.indices.get(url).copied()
    }

    fn stats(&self) -> (usize, usize) {
        (self.entries.len(), self.total_bytes)
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.lru.clear();
        self.indices.clear();
        self.total_bytes = 0;
    }
}

pub struct Preloader {
    cache: Arc<TokioMutex<CacheData>>,
    /// Signalled to tell workers to stop.
    running: Arc<AtomicBool>,
    /// The highest segment index that has been requested for playback.
    played_up_to: Arc<AtomicUsize>,
    /// Remaining segment URLs to prefetch.
    segment_urls: Arc<TokioMutex<Vec<String>>>,
    /// Next segment index to claim.
    next_index: Arc<AtomicUsize>,
    /// Total number of segments in the playlist.
    total_segments: Arc<AtomicUsize>,
    /// Worker join-set for graceful shutdown.
    workers: Arc<TokioMutex<JoinSet<()>>>,
    worker_count: Arc<AtomicUsize>,
    /// Persisted max cache size (not reset on start/stop).
    max_cache_bytes: Arc<AtomicUsize>,
}

impl Preloader {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(TokioMutex::new(CacheData::new())),
            running: Arc::new(AtomicBool::new(false)),
            played_up_to: Arc::new(AtomicUsize::new(0)),
            segment_urls: Arc::new(TokioMutex::new(Vec::new())),
            next_index: Arc::new(AtomicUsize::new(0)),
            total_segments: Arc::new(AtomicUsize::new(0)),
            workers: Arc::new(TokioMutex::new(JoinSet::new())),
            worker_count: Arc::new(AtomicUsize::new(DEFAULT_WORKER_COUNT)),
            max_cache_bytes: Arc::new(AtomicUsize::new(DEFAULT_MAX_CACHE_BYTES)),
        }
    }

    pub fn set_worker_count(&self, count: usize) {
        self.worker_count.store(count, Ordering::Relaxed);
    }

    pub fn set_max_cache_size(&self, bytes: usize) {
        self.max_cache_bytes.store(bytes, Ordering::Relaxed);
        let mut cache = self.cache.blocking_lock();
        cache.max_bytes = bytes;
        eprintln!(
            "[Preloader] Max cache size set to {} bytes ({} MB)",
            bytes,
            bytes / 1024 / 1024
        );
    }

    /// Start prefetching segments from a parsed M3U8 manifest.
    pub async fn start(&self, content: &str, base_url: &str) {
        self.stop().await;

        let urls = Self::extract_segment_urls(content, base_url);
        if urls.is_empty() {
            return;
        }
        let total = urls.len();
        let count = self.worker_count.load(Ordering::Relaxed);

        // Store segment URLs and seed the cache index map
        {
            let mut cache = self.cache.lock().await;
            cache.clear();
            cache.max_bytes = self.max_cache_bytes.load(Ordering::Relaxed);
            self.segment_urls.lock().await.clone_from(&urls);
            // Pre-populate indices so eviction can compare against played_up_to
            for (i, url) in urls.iter().enumerate() {
                cache.indices.insert(url.clone(), i);
            }
        }

        self.next_index.store(0, Ordering::Relaxed);
        self.total_segments.store(total, Ordering::Relaxed);
        self.played_up_to.store(0, Ordering::Relaxed);
        self.running.store(true, Ordering::Relaxed);

        let running = self.running.clone();
        let cache = self.cache.clone();
        let segment_urls = self.segment_urls.clone();
        let next_index = self.next_index.clone();
        let total_segments = self.total_segments.clone();
        let played_up_to = self.played_up_to.clone();

        for worker_id in 0..count {
            let running = running.clone();
            let cache = cache.clone();
            let segment_urls = segment_urls.clone();
            let next_index = next_index.clone();
            let total_segments = total_segments.clone();
            let played_up_to = played_up_to.clone();

            self.workers.lock().await.spawn(async move {
                worker_loop(
                    worker_id, running, cache, segment_urls, next_index,
                    total_segments, played_up_to,
                )
                .await;
            });
        }

        eprintln!(
            "[Preloader] Started with {} workers, {} segments",
            count, total
        );
    }

    /// Stop all workers and optionally preserve the cache.
    pub async fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);

        let mut workers = self.workers.lock().await;
        // Workers check `running` between each segment, so they'll stop quickly.
        // Give them a short window, then abort remaining.
        for _ in 0..50 {
            // 50 * 10ms = 500ms max wait
            if workers.try_join_next().is_none() {
                break;
            }
        }
        // Any remaining workers are abandoned (they hold no locks).
        workers.abort_all();
        drop(workers);
    }

    /// Look up a segment. Returns cached data, and marks it as "played" so
    /// the eviction policy knows it's safe to discard later.
    pub async fn get_segment(&self, url: &str) -> Option<Vec<u8>> {
        // Track playback position: the highest index the player has requested
        if let Some(idx) = self.cache.lock().await.get_index(url) {
            let prev = self.played_up_to.fetch_max(idx + 1, Ordering::Relaxed);
            if idx + 1 > prev {
                eprintln!("[Preloader] Playhead advanced to segment {}", idx);
            }
        }

        self.cache.lock().await.get(url)
    }

    /// Get segment from cache, or fetch it on cache miss.
    pub async fn get_segment_or_fetch(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        if let Some(data) = self.get_segment(url).await {
            return Ok(data);
        }
        m3u8::fetch_media_segment(url).await
    }

    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        cache.stats()
    }

    fn extract_segment_urls(content: &str, base_url: &str) -> Vec<String> {
        let base = http::get_base_url(base_url).unwrap_or_else(|_| base_url.to_string());
        let mut urls = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Ok(url) = http::resolve_url(&base, trimmed) {
                urls.push(url);
            }
        }
        urls
    }
}

impl Default for Preloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker: claims segments via `next_index`, fetches them, and inserts into
/// the shared cache.  If the cache is full and no played segments can be
/// evicted, the worker pauses and retries later (playback may advance).
async fn worker_loop(
    worker_id: usize,
    running: Arc<AtomicBool>,
    cache: Arc<TokioMutex<CacheData>>,
    segment_urls: Arc<TokioMutex<Vec<String>>>,
    next_index: Arc<AtomicUsize>,
    total_segments: Arc<AtomicUsize>,
    played_up_to: Arc<AtomicUsize>,
) {
    eprintln!("[Preloader] Worker {} started", worker_id);

    while running.load(Ordering::Relaxed) {
        let index = next_index.fetch_add(1, Ordering::Relaxed);
        let total = total_segments.load(Ordering::Relaxed);

        if index >= total {
            break;
        }

        let url = {
            let urls = segment_urls.lock().await;
            match urls.get(index).cloned() {
                Some(u) => u,
                None => {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
            }
        };

        // Skip if already cached (another worker got there first)
        {
            let c = cache.lock().await;
            if c.contains(&url) {
                continue;
            }
        }

        // Fetch the segment with retries
        let data = match fetch_with_retry(&url, &running).await {
            Some(d) => d,
            None => continue, // all retries exhausted or stop requested
        };

        // Try to insert — if cache is full and no played segments to evict,
        // we pause preloading until playback advances.
        let inserted = {
            let mut c = cache.lock().await;
            let playhead = played_up_to.load(Ordering::Relaxed);
            c.try_insert(url, data, index, playhead)
        };

        if !inserted {
            eprintln!(
                "[Preloader] Worker {}: cache full, pausing (segment {})",
                worker_id, index
            );
            // Playback hasn't caught up — wait a bit and retry this index
            next_index.fetch_sub(1, Ordering::Relaxed);
            sleep(Duration::from_millis(200)).await;
            continue;
        }
    }

    eprintln!("[Preloader] Worker {} stopped", worker_id);
}

/// Fetch a single segment with up to MAX_RETRIES attempts.
/// Returns `None` if all retries fail or `running` becomes false.
async fn fetch_with_retry(url: &str, running: &AtomicBool) -> Option<Vec<u8>> {
    for retry in 0..=MAX_RETRIES {
        if !running.load(Ordering::Relaxed) {
            return None;
        }

        match m3u8::fetch_media_segment(url).await {
            Ok(data) => return Some(data),
            Err(e) if retry < MAX_RETRIES => {
                eprintln!(
                    "[Preloader] Retry {}/{} for {}: {:?}",
                    retry + 1,
                    MAX_RETRIES,
                    url,
                    e
                );
                sleep(Duration::from_millis(100 * (retry as u64 + 1))).await;
            }
            Err(e) => {
                eprintln!(
                    "[Preloader] Failed after {} retries for {}: {:?}",
                    MAX_RETRIES, url, e
                );
                return None;
            }
        }
    }
    None
}
