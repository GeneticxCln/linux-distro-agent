use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub operation: String,
    pub package: Option<String>,
    pub success: bool,
    pub output: Option<String>,
    pub distro: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    pub fn load() -> Result<Self> {
        let history_path = Self::history_path()?;
        
        if history_path.exists() {
            let content = fs::read_to_string(&history_path)
                .with_context(|| format!("Failed to read history file: {:?}", history_path))?;
            
            let history: History = serde_json::from_str(&content)
                .with_context(|| "Failed to parse history file")?;
            
            Ok(history)
        } else {
            Ok(History::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let history_path = Self::history_path()?;
        
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create history directory: {:?}", parent))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize history")?;
        
        fs::write(&history_path, content)
            .with_context(|| format!("Failed to write history file: {:?}", history_path))?;
        
        Ok(())
    }
    
    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
        
        // Keep only last 1000 entries to prevent file from growing too large
        if self.entries.len() > 1000 {
            self.entries.drain(0..self.entries.len() - 1000);
        }
    }
    
    pub fn get_recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(limit)
            .collect()
    }
    
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.command.contains(query) 
                    || entry.operation.contains(query)
                    || entry.package.as_ref().map_or(false, |p| p.contains(query))
            })
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    fn history_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("linux-distro-agent").join("history.json"))
    }
}

pub struct HistoryManager {
    history: History,
}

impl HistoryManager {
    pub fn new() -> Result<Self> {
        let history = History::load()?;
        Ok(Self { history })
    }
    
    pub fn add_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        self.history.add_entry(entry);
        self.history.save()
    }
    
    pub fn get_recent(&self, limit: usize) -> Result<Vec<&HistoryEntry>> {
        Ok(self.history.get_recent(limit))
    }
    
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<&HistoryEntry>> {
        let mut results = self.history.search(query);
        results.reverse(); // Show most recent first
        results.truncate(limit);
        Ok(results)
    }
    
    pub fn clear(&mut self) -> Result<()> {
        self.history.clear();
        self.history.save()
    }
}
