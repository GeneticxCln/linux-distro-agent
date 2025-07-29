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
    last_cleanup: Option<SystemTime>,
    hit_count: u64,
    miss_count: u64,
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
            last_cleanup: None,
            hit_count: 0,
            miss_count: 0,
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
            self.hit_count += 1;
            return Some(&entry.data);
        }
        self.miss_count += 1;
        None
    }

    pub fn cleanup(&mut self) {
        let now = SystemTime::now();
        self.cache.retain(|_, entry| {
            now.duration_since(entry.last_accessed).unwrap() < self.expiration_duration
        });
        self.last_cleanup = Some(now);
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let total_size_bytes = self.cache.values()
            .map(|entry| entry.data.len())
            .sum();
        
        CacheStats {
            total_entries,
            total_size_bytes,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            last_cleanup: self.last_cleanup,
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn list_entries(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}
