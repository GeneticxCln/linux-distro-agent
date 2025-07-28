use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub last_cleanup: Option<SystemTime>,
}

pub struct PackageCache {
    cache: HashMap<String, CacheEntry>,
    expiration_duration: Duration,
}

struct CacheEntry {
    data: Vec<u8>,
    last_accessed: SystemTime,
}

impl PackageCache {
    pub fn new(expiration_duration: Duration) -> Self {
        PackageCache {
            cache: HashMap::new(),
            expiration_duration,
        }
    }

    pub fn store(&mut self, package_name: String, data: Vec<u8>) {
        let entry = CacheEntry {
            data,
            last_accessed: SystemTime::now(),
        };
        self.cache.insert(package_name, entry);
    }

    pub fn retrieve(&mut self, package_name: &str) -> Option<&Vec<u8>> {
        if let Some(entry) = self.cache.get_mut(package_name) {
            entry.last_accessed = SystemTime::now();
            return Some(&entry.data);
        }
        None
    }

    pub fn cleanup(&mut self) {
        let now = SystemTime::now();
        self.cache.retain(|_, entry| {
            now.duration_since(entry.last_accessed).unwrap() < self.expiration_duration
        });
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let total_size_bytes = self.cache.values()
            .map(|entry| entry.data.len())
            .sum();
        
        CacheStats {
            total_entries,
            total_size_bytes,
            hit_count: 0,  // Placeholder - would need tracking for real implementation
            miss_count: 0, // Placeholder - would need tracking for real implementation
            last_cleanup: None, // Placeholder - would need tracking for real implementation
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn list_entries(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}
