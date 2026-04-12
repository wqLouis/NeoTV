use crate::error::HttpError;
use crate::http;
use crate::m3u8;
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinSet;
use tokio::time::sleep;

const DEFAULT_WORKER_COUNT: usize = 6;
const MAX_CACHE_BYTES: usize = 2 * 1024 * 1024 * 1024;
const MAX_RETRIES: usize = 3;

pub static PRELOADER: Lazy<Preloader> = Lazy::new(Preloader::new);

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedSegment {
    data: Vec<u8>,
    loaded_at: Instant,
    index: usize,
}

#[derive(Debug)]
struct PreloaderState {
    current_index: usize,
    is_running: bool,
    worker_count: usize,
}

impl PreloaderState {
    fn new(worker_count: usize) -> Self {
        Self {
            current_index: 0,
            is_running: false,
            worker_count,
        }
    }
}

pub struct Preloader {
    cache: Arc<TokioMutex<HashMap<String, CachedSegment>>>,
    lru_queue: Arc<TokioMutex<VecDeque<String>>>,
    state: Arc<TokioMutex<PreloaderState>>,
    segment_urls: Arc<TokioMutex<Vec<String>>>,
    segment_indices: Arc<TokioMutex<HashMap<String, usize>>>,
    workers: Arc<TokioMutex<JoinSet<()>>>,
    total_bytes: Arc<TokioMutex<usize>>,
    next_index: Arc<AtomicUsize>,
    total_segments: Arc<AtomicUsize>,
}

impl Preloader {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(TokioMutex::new(HashMap::new())),
            lru_queue: Arc::new(TokioMutex::new(VecDeque::new())),
            state: Arc::new(TokioMutex::new(PreloaderState::new(DEFAULT_WORKER_COUNT))),
            segment_urls: Arc::new(TokioMutex::new(Vec::new())),
            segment_indices: Arc::new(TokioMutex::new(HashMap::new())),
            workers: Arc::new(TokioMutex::new(JoinSet::new())),
            total_bytes: Arc::new(TokioMutex::new(0)),
            next_index: Arc::new(AtomicUsize::new(0)),
            total_segments: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn set_worker_count(&self, count: usize) {
        let mut state = self.state.blocking_lock();
        state.worker_count = count;
    }

    pub async fn start(&self, content: &str, base_url: &str) {
        self.stop().await;

        let urls = Self::extract_segment_urls(content, base_url);
        if urls.is_empty() {
            return;
        }

        let total = urls.len();

        {
            let mut segment_urls = self.segment_urls.lock().await;
            *segment_urls = urls.clone();
        }

        {
            let mut segment_indices = self.segment_indices.lock().await;
            segment_indices.clear();
            for (i, url) in urls.iter().enumerate() {
                segment_indices.insert(url.clone(), i);
            }
        }

        let worker_count = {
            let state = self.state.lock().await;
            state.worker_count
        };

        {
            let mut state = self.state.lock().await;
            state.is_running = true;
            state.current_index = 0;
        }

        self.next_index.store(0, Ordering::Relaxed);
        self.total_segments.store(total, Ordering::Relaxed);

        let urls_clone = self.segment_urls.clone();
        let cache_clone = self.cache.clone();
        let lru_clone = self.lru_queue.clone();
        let state_clone = self.state.clone();
        let total_bytes_clone = self.total_bytes.clone();
        let next_index_clone = self.next_index.clone();
        let total_segments_clone = self.total_segments.clone();

        for worker_id in 0..worker_count {
            let urls = urls_clone.clone();
            let cache = cache_clone.clone();
            let lru = lru_clone.clone();
            let state = state_clone.clone();
            let total_bytes = total_bytes_clone.clone();
            let next_index = next_index_clone.clone();
            let total_segments = total_segments_clone.clone();

            self.workers.lock().await.spawn(async move {
                worker_loop(worker_id, urls, cache, lru, state, total_bytes, next_index, total_segments).await;
            });
        }

        eprintln!("[Preloader] Started with {} workers, {} segments", worker_count, total);
    }

    pub async fn stop(&self) {
        {
            let mut state = self.state.lock().await;
            state.is_running = false;
        }

        let mut workers = self.workers.lock().await;
        while workers.try_join_next().is_some() {}
        drop(workers);

        {
            let mut cache = self.cache.lock().await;
            let mut total_bytes = self.total_bytes.lock().await;
            cache.clear();
            *total_bytes = 0;
        }

        {
            let mut lru = self.lru_queue.lock().await;
            lru.clear();
        }

        {
            let mut segment_urls = self.segment_urls.lock().await;
            segment_urls.clear();
        }

        {
            let mut segment_indices = self.segment_indices.lock().await;
            segment_indices.clear();
        }

        self.next_index.store(0, Ordering::Relaxed);
        self.total_segments.store(0, Ordering::Relaxed);

        eprintln!("[Preloader] Stopped");
    }

    pub async fn get_segment(&self, url: &str) -> Option<Vec<u8>> {
        if let Some(idx) = self.segment_indices.lock().await.get(url).copied() {
            let mut state = self.state.lock().await;
            state.current_index = idx;
        }

        let cache = self.cache.lock().await;
        cache.get(url).map(|seg| seg.data.clone())
    }

    pub async fn get_segment_or_fetch(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        if let Some(data) = self.get_segment(url).await {
            return Ok(data);
        }
        m3u8::fetch_media_segment(url).await
    }

    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total_bytes = self.total_bytes.lock().await;
        (cache.len(), *total_bytes)
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

async fn worker_loop(
    worker_id: usize,
    segment_urls: Arc<TokioMutex<Vec<String>>>,
    cache: Arc<TokioMutex<HashMap<String, CachedSegment>>>,
    lru_queue: Arc<TokioMutex<VecDeque<String>>>,
    state: Arc<TokioMutex<PreloaderState>>,
    total_bytes: Arc<TokioMutex<usize>>,
    next_index: Arc<AtomicUsize>,
    total_segments: Arc<AtomicUsize>,
) {
    eprintln!("[Preloader] Worker {} started", worker_id);

    loop {
        let (url, index) = {
            let is_running = {
                let state_guard = state.lock().await;
                state_guard.is_running
            };
            if !is_running {
                break;
            }

            let index = next_index.fetch_add(1, Ordering::Relaxed);
            let total = total_segments.load(Ordering::Relaxed);

            if index >= total {
                break;
            }

            let urls = segment_urls.lock().await;
            match urls.get(index).map(|s| s.clone()) {
                Some(url) => (url, index),
                None => {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
            }
        };

        {
            let cache_guard = cache.lock().await;
            if cache_guard.contains_key(&url) {
                continue;
            }
        }

        let mut retries = 0;
        loop {
            match m3u8::fetch_media_segment(&url).await {
                Ok(data) => {
                    let data_len = data.len();

                    evict_if_needed(&cache, &lru_queue, &total_bytes, data_len).await;

                    let mut cache_guard = cache.lock().await;
                    let mut lru_guard = lru_queue.lock().await;
                    let mut total_bytes_guard = total_bytes.lock().await;

                    cache_guard.insert(url.clone(), CachedSegment {
                        data,
                        loaded_at: Instant::now(),
                        index,
                    });
                    lru_guard.push_back(url.clone());
                    *total_bytes_guard += data_len;
                    break;
                }
                Err(e) => {
                    retries += 1;
                    if retries < MAX_RETRIES {
                        eprintln!("[Preloader] Worker {} retry {}/{} for segment {}: {:?}",
                            worker_id, retries, MAX_RETRIES, index, e);
                        sleep(Duration::from_millis(100 * retries as u64)).await;
                    } else {
                        eprintln!("[Preloader] Worker {} failed after {} retries for segment {}: {:?}",
                            worker_id, MAX_RETRIES, index, e);
                        break;
                    }
                }
            }
        }
    }

    eprintln!("[Preloader] Worker {} stopped", worker_id);
}

async fn evict_if_needed(
    cache: &Arc<TokioMutex<HashMap<String, CachedSegment>>>,
    lru_queue: &Arc<TokioMutex<VecDeque<String>>>,
    total_bytes: &Arc<TokioMutex<usize>>,
    incoming_size: usize,
) {
    let mut total = total_bytes.lock().await;

    while *total + incoming_size > MAX_CACHE_BYTES {
        let mut lru = lru_queue.lock().await;
        if let Some(oldest_url) = lru.pop_front() {
            if let Some(old_seg) = cache.lock().await.remove(&oldest_url) {
                *total -= old_seg.data.len();
                eprintln!("[Preloader] Evicted segment {}, freed {} bytes", oldest_url, old_seg.data.len());
            }
        } else {
            break;
        }
    }
}
