use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::cache_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::distro::DistroInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub timestamp: DateTime<Utc>,
    pub data: CacheData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheData {
    DistroInfo(DistroInfo),
    PackageList(Vec<String>),
    PackageInfo { package: String, info: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    pub entries: std::collections::HashMap<String, CacheEntry>,
}

impl Cache {
    pub fn load() -> Result<Self> {
        let cache_path = Self::cache_path()?;
        
        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)
                .with_context(|| format!("Failed to read cache file: {:?}", cache_path))?;
            
            let cache: Cache = serde_json::from_str(&content)
                .with_context(|| "Failed to parse cache file")?;
            
            Ok(cache)
        } else {
            Ok(Cache::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let cache_path = Self::cache_path()?;
        
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache directory: {:?}", parent))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize cache")?;
        
        fs::write(&cache_path, content)
            .with_context(|| format!("Failed to write cache file: {:?}", cache_path))?;
        
        Ok(())
    }
    
    pub fn get(&self, key: &str, max_age: Duration) -> Option<&CacheData> {
        if let Some(entry) = self.entries.get(key) {
            let now = SystemTime::now();
            let entry_time = SystemTime::UNIX_EPOCH + Duration::from_secs(entry.timestamp.timestamp() as u64);
            
            if now.duration_since(entry_time).unwrap_or(Duration::MAX) <= max_age {
                return Some(&entry.data);
            }
        }
        None
    }
    
    pub fn set(&mut self, key: String, data: CacheData) {
        let entry = CacheEntry {
            timestamp: Utc::now(),
            data,
        };
        self.entries.insert(key, entry);
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn cleanup_expired(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        self.entries.retain(|_, entry| {
            let entry_time = SystemTime::UNIX_EPOCH + Duration::from_secs(entry.timestamp.timestamp() as u64);
            now.duration_since(entry_time).unwrap_or(Duration::MAX) <= max_age
        });
    }
    
    fn cache_path() -> Result<PathBuf> {
        let cache_dir = cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;
        
        Ok(cache_dir.join("linux-distro-agent").join("cache.json"))
    }
}

#[derive(Debug)]
pub struct CacheStatus {
    pub entry_count: usize,
    pub total_size: u64,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct CacheEntryInfo {
    pub key: String,
    pub created_at: DateTime<Utc>,
}

pub struct CacheManager {
    cache: Cache,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache = Cache::load()?;
        Ok(Self { cache })
    }
    
    pub fn status(&self) -> Result<CacheStatus> {
        let entry_count = self.cache.entries.len();
        let last_updated = self.cache.entries.values()
            .map(|entry| entry.timestamp)
            .max();
            
        // Calculate total size (rough estimate based on serialized JSON)
        let total_size = serde_json::to_string(&self.cache)
            .map(|s| s.len() as u64)
            .unwrap_or(0);
            
        Ok(CacheStatus {
            entry_count,
            total_size,
            last_updated,
        })
    }
    
    pub fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        self.cache.save()
    }
    
    pub fn list(&self) -> Result<Vec<CacheEntryInfo>> {
        let mut entries: Vec<CacheEntryInfo> = self.cache.entries
            .iter()
            .map(|(key, entry)| CacheEntryInfo {
                key: key.clone(),
                created_at: entry.timestamp,
            })
            .collect();
            
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(entries)
    }
    
    pub fn get(&self, key: &str, max_age: Duration) -> Option<&CacheData> {
        self.cache.get(key, max_age)
    }
    
    pub fn set(&mut self, key: String, data: CacheData) -> Result<()> {
        self.cache.set(key, data);
        self.cache.save()
    }
}
