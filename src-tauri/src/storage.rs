use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

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
        fs::create_dir_all(&data_dir).ok();

        let storage = Self {
            history: Mutex::new(HistoryData::default()),
            favourites: Mutex::new(FavouritesData::default()),
            data_dir: data_dir.clone(),
        };

        storage.load_history();
        storage.load_favourites();

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
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str::<HistoryData>(&data) {
                    *self.history.lock().unwrap() = history;
                    return;
                }
            }
            eprintln!("[Storage] Failed to load history, starting fresh");
        }
        *self.history.lock().unwrap() = HistoryData::default();
    }

    fn load_favourites(&self) {
        let path = self.favourites_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(favourites) = serde_json::from_str::<FavouritesData>(&data) {
                    *self.favourites.lock().unwrap() = favourites;
                    return;
                }
            }
            eprintln!("[Storage] Failed to load favourites, starting fresh");
        }
        *self.favourites.lock().unwrap() = FavouritesData::default();
    }

    fn save_history(&self) {
        let history = self.history.lock().unwrap();
        if let Ok(data) = serde_json::to_string_pretty(&*history) {
            if let Err(e) = fs::write(self.history_path(), data) {
                eprintln!("[Storage] Failed to save history: {}", e);
            }
        }
    }

    fn save_favourites(&self) {
        let favourites = self.favourites.lock().unwrap();
        if let Ok(data) = serde_json::to_string_pretty(&*favourites) {
            if let Err(e) = fs::write(self.favourites_path(), data) {
                eprintln!("[Storage] Failed to save favourites: {}", e);
            }
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
