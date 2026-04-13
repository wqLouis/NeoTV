use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::cache::SpeedTestResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub title: String,
    pub source: String,
    pub cover: Option<String>,
    pub episode: Option<String>,
    pub episode_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavouriteItem {
    pub id: String,
    pub title: String,
    pub source: String,
    pub cover: Option<String>,
    pub episode: Option<String>,
    pub episode_index: Option<i32>,
    pub added_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct HistoryData {
    items: Vec<HistoryItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct FavouritesData {
    items: Vec<FavouriteItem>,
}

pub struct Storage {
    history: Mutex<HistoryData>,
    favourites: Mutex<FavouritesData>,
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!(
                "[Storage] Failed to create data directory {:?}: {}",
                data_dir, e
            );
        }

        let storage = Self {
            history: Mutex::new(HistoryData::default()),
            favourites: Mutex::new(FavouritesData::default()),
            data_dir: data_dir.clone(),
        };

        storage.load_history();
        storage.load_favourites();

        eprintln!("[Storage] Storage initialized at {:?}", data_dir);

        storage
    }

    fn history_path(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }

    fn favourites_path(&self) -> PathBuf {
        self.data_dir.join("favourites.json")
    }

    fn load_history(&self) {
        let path = self.history_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str::<HistoryData>(&data) {
                    Ok(history) => {
                        *self.history.lock().unwrap() = history;
                        eprintln!(
                            "[Storage] Loaded {} history items",
                            self.history.lock().unwrap().items.len()
                        );
                        return;
                    }
                    Err(e) => eprintln!("[Storage] Failed to parse history: {}", e),
                },
                Err(e) => eprintln!("[Storage] Failed to read history file: {}", e),
            }
            eprintln!("[Storage] Failed to load history, starting fresh");
        }
        *self.history.lock().unwrap() = HistoryData::default();
    }

    fn load_favourites(&self) {
        let path = self.favourites_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str::<FavouritesData>(&data) {
                    Ok(favourites) => {
                        *self.favourites.lock().unwrap() = favourites;
                        eprintln!(
                            "[Storage] Loaded {} favourites",
                            self.favourites.lock().unwrap().items.len()
                        );
                        return;
                    }
                    Err(e) => eprintln!("[Storage] Failed to parse favourites: {}", e),
                },
                Err(e) => eprintln!("[Storage] Failed to read favourites file: {}", e),
            }
            eprintln!("[Storage] Failed to load favourites, starting fresh");
        }
        *self.favourites.lock().unwrap() = FavouritesData::default();
    }

    fn save_history(&self) {
        let history = self.history.lock().unwrap();
        let items_count = history.items.len();
        match serde_json::to_string_pretty(&*history) {
            Ok(data) => {
                let path = self.history_path();
                match fs::write(&path, data) {
                    Ok(_) => eprintln!("[Storage] Saved {} history items", items_count),
                    Err(e) => eprintln!("[Storage] Failed to write history file {:?}: {}", path, e),
                }
            }
            Err(e) => eprintln!("[Storage] Failed to serialize history: {}", e),
        }
    }

    fn save_favourites(&self) {
        let favourites = self.favourites.lock().unwrap();
        let items_count = favourites.items.len();
        match serde_json::to_string_pretty(&*favourites) {
            Ok(data) => {
                let path = self.favourites_path();
                match fs::write(&path, data) {
                    Ok(_) => eprintln!("[Storage] Saved {} favourites", items_count),
                    Err(e) => eprintln!(
                        "[Storage] Failed to write favourites file {:?}: {}",
                        path, e
                    ),
                }
            }
            Err(e) => eprintln!("[Storage] Failed to serialize favourites: {}", e),
        }
    }

    // History operations
    pub fn history_get_all(&self) -> Vec<HistoryItem> {
        self.history.lock().unwrap().items.clone()
    }

    pub fn history_add(&self, item: HistoryItem) {
        let mut history = self.history.lock().unwrap();

        // Check if exists and update, otherwise add
        if let Some(existing) = history
            .items
            .iter_mut()
            .find(|h| h.id == item.id && h.source == item.source && h.episode == item.episode)
        {
            *existing = item;
        } else {
            history.items.insert(0, item);
            // Keep only last 100 items
            history.items.truncate(100);
        }

        drop(history);
        self.save_history();
    }

    pub fn history_remove(&self, id: &str, source: &str, episode: Option<&str>) {
        let mut history = self.history.lock().unwrap();
        history
            .items
            .retain(|h| !(h.id == id && h.source == source && h.episode.as_deref() == episode));
        drop(history);
        self.save_history();
    }

    pub fn history_clear(&self) {
        *self.history.lock().unwrap() = HistoryData::default();
        self.save_history();
    }

    // Favourites operations
    pub fn favourites_get_all(&self) -> Vec<FavouriteItem> {
        self.favourites.lock().unwrap().items.clone()
    }

    pub fn favourites_add(&self, item: FavouriteItem) {
        let mut favourites = self.favourites.lock().unwrap();

        // Check if already exists
        if favourites
            .items
            .iter()
            .any(|f| f.id == item.id && f.source == item.source && f.episode == item.episode)
        {
            return; // Already exists
        }

        favourites.items.insert(0, item);
        drop(favourites);
        self.save_favourites();
    }

    pub fn favourites_remove(&self, id: &str, source: &str, episode: Option<&str>) {
        let mut favourites = self.favourites.lock().unwrap();
        favourites
            .items
            .retain(|f| !(f.id == id && f.source == source && f.episode.as_deref() == episode));
        drop(favourites);
        self.save_favourites();
    }

    pub fn favourites_has(&self, id: &str, source: &str, episode: Option<&str>) -> bool {
        let favourites = self.favourites.lock().unwrap();
        favourites
            .items
            .iter()
            .any(|f| f.id == id && f.source == source && f.episode.as_deref() == episode)
    }

    pub fn favourites_clear(&self) {
        *self.favourites.lock().unwrap() = FavouritesData::default();
        self.save_favourites();
    }
}

pub struct SpeedCacheStorage {
    data_dir: PathBuf,
}

impl SpeedCacheStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        let speed_cache_dir = data_dir.join("speed_cache");
        if let Err(e) = fs::create_dir_all(&speed_cache_dir) {
            eprintln!(
                "[SpeedCacheStorage] Failed to create speed cache directory {:?}: {}",
                speed_cache_dir, e
            );
        }
        eprintln!("[SpeedCacheStorage] Initialized at {:?}", speed_cache_dir);
        Self { data_dir }
    }

    fn speed_cache_dir(&self) -> PathBuf {
        self.data_dir.join("speed_cache")
    }

    fn cache_path(&self, network_id: &str) -> PathBuf {
        let safe_name = network_id.replace(['/', '\\', ':', ' '], "_");
        self.speed_cache_dir().join(format!("{}.json", safe_name))
    }

    pub fn load(&self, network_id: &str) -> Vec<SpeedTestResult> {
        let path = self.cache_path(network_id);
        if !path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<Vec<SpeedTestResult>>(&data) {
                Ok(results) => {
                    eprintln!(
                        "[SpeedCacheStorage] Loaded {} speed results for network '{}'",
                        results.len(),
                        network_id
                    );
                    results
                }
                Err(e) => {
                    eprintln!(
                        "[SpeedCacheStorage] Failed to parse speed cache for '{}': {}",
                        network_id, e
                    );
                    Vec::new()
                }
            },
            Err(e) => {
                eprintln!(
                    "[SpeedCacheStorage] Failed to read speed cache for '{}': {}",
                    network_id, e
                );
                Vec::new()
            }
        }
    }

    pub fn save(&self, network_id: &str, results: &[SpeedTestResult]) {
        let path = self.cache_path(network_id);
        let dir = self.speed_cache_dir();
        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(&dir) {
                eprintln!(
                    "[SpeedCacheStorage] Failed to create speed cache dir: {}",
                    e
                );
                return;
            }
        }

        match serde_json::to_string_pretty(results) {
            Ok(data) => match fs::write(&path, data) {
                Ok(_) => eprintln!(
                    "[SpeedCacheStorage] Saved {} speed results for network '{}'",
                    results.len(),
                    network_id
                ),
                Err(e) => eprintln!(
                    "[SpeedCacheStorage] Failed to write speed cache for '{}': {}",
                    network_id, e
                ),
            },
            Err(e) => eprintln!(
                "[SpeedCacheStorage] Failed to serialize speed results: {}",
                e
            ),
        }
    }

    pub fn clear_all(&self) {
        let dir = self.speed_cache_dir();
        if !dir.exists() {
            return;
        }

        match fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let _ = fs::remove_file(entry.path());
                    }
                }
                eprintln!("[SpeedCacheStorage] Cleared all speed caches");
            }
            Err(e) => {
                eprintln!("[SpeedCacheStorage] Failed to read speed cache dir: {}", e);
            }
        }
    }
}
