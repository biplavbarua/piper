use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: String, // ISO 8601 or readable
    pub original_size: u64,
    pub compressed_size: u64,
    pub savings: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnalyticsHistory {
    pub entries: Vec<HistoryEntry>,
}

impl AnalyticsHistory {
    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    return history;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::get_path();
        // Ensure dir exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn add_entry(&mut self, original: u64, compressed: u64) {
        let savings = original.saturating_sub(compressed);
        let entry = HistoryEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            original_size: original,
            compressed_size: compressed,
            savings,
        };
        self.entries.push(entry);
        self.save();
    }

    fn get_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".piper");
        path.push("history.json");
        path
    }
}
