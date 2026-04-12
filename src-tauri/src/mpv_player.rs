use libc::{c_char, setlocale};
use libmpv2::Mpv;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct MpvPlaybackState {
    pub playing: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f64,
}

pub struct MpvPlayer {
    mpv: Mpv,
    #[allow(dead_code)]
    id: String,
}

impl MpvPlayer {
    pub fn new(url: &str, window_id: u64) -> Result<Self, String> {
        unsafe { setlocale(libc::LC_NUMERIC, b"C\0".as_ptr() as *const c_char) };
        
        let mpv = Mpv::new().map_err(|e| e.to_string())?;
        
        mpv.set_property("vo", "x11").map_err(|e| e.to_string())?;
        mpv.set_property("wid", window_id.to_string()).map_err(|e| e.to_string())?;
        mpv.set_property("border", "no").map_err(|e| e.to_string())?;
        mpv.set_property("autofit", "100%x100%").map_err(|e| e.to_string())?;
        mpv.command("loadfile", &[url, "replace"]).map_err(|e| e.to_string())?;
        
        Ok(Self {
            mpv,
            id: String::new(),
        })
    }
    
    pub fn play(&mut self) -> Result<(), String> {
        self.mpv.set_property("pause", false).map_err(|e| e.to_string())
    }
    
    pub fn pause(&mut self) -> Result<(), String> {
        self.mpv.set_property("pause", true).map_err(|e| e.to_string())
    }
    
    pub fn seek(&mut self, position_secs: f64) -> Result<(), String> {
        self.mpv.command("seek", &[&format!("{}", position_secs), "absolute"])
            .map_err(|e| e.to_string())
    }
    
    pub fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.mpv.set_property("volume", volume).map_err(|e| e.to_string())
    }
    
    pub fn get_state(&self) -> Result<MpvPlaybackState, String> {
        let paused: bool = self.mpv.get_property("pause").map_err(|e| e.to_string())?;
        let position_secs: f64 = self.mpv.get_property("time-pos").map_err(|e| e.to_string())?;
        let duration_secs: f64 = self.mpv.get_property("duration").map_err(|e| e.to_string())?;
        let volume: f64 = self.mpv.get_property("volume").map_err(|e| e.to_string())?;
        
        Ok(MpvPlaybackState {
            playing: !paused,
            position_secs,
            duration_secs,
            volume,
        })
    }
    
    pub fn stop(&mut self) -> Result<(), String> {
        self.mpv.command("quit", &[]).map_err(|e| e.to_string())
    }
}

pub struct MpvManager {
    players: Arc<Mutex<HashMap<String, MpvPlayer>>>,
}

impl MpvManager {
    pub fn new() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn create_player(&self, id: String, url: String, window_id: u64) -> Result<(), String> {
        let player = MpvPlayer::new(&url, window_id)?;
        let mut players = self.players.lock().await;
        players.insert(id, player);
        Ok(())
    }
    
    pub async fn destroy_player(&self, id: &str) -> Result<(), String> {
        let mut players = self.players.lock().await;
        if let Some(mut player) = players.remove(id) {
            player.stop()?;
        }
        Ok(())
    }
    
    pub async fn play(&self, id: &str) -> Result<(), String> {
        let mut players = self.players.lock().await;
        let player = players.get_mut(id).ok_or("Player not found")?;
        player.play()
    }
    
    pub async fn pause(&self, id: &str) -> Result<(), String> {
        let mut players = self.players.lock().await;
        let player = players.get_mut(id).ok_or("Player not found")?;
        player.pause()
    }
    
    pub async fn seek(&self, id: &str, position_secs: f64) -> Result<(), String> {
        let mut players = self.players.lock().await;
        let player = players.get_mut(id).ok_or("Player not found")?;
        player.seek(position_secs)
    }
    
    pub async fn set_volume(&self, id: &str, volume: f64) -> Result<(), String> {
        let mut players = self.players.lock().await;
        let player = players.get_mut(id).ok_or("Player not found")?;
        player.set_volume(volume)
    }
    
    pub async fn get_state(&self, id: &str) -> Result<MpvPlaybackState, String> {
        let players = self.players.lock().await;
        let player = players.get(id).ok_or("Player not found")?;
        player.get_state()
    }
}

impl Default for MpvManager {
    fn default() -> Self {
        Self::new()
    }
}
