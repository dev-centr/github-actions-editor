//! Cache storage implementation
//!
//! File-based cache with TTL support for offline access and reduced API calls.

use chrono::{DateTime, Duration, Utc};
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache miss: {0}")]
    Miss(String),

    #[error("Cache expired: {0}")]
    Expired(String),
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,

    /// When this entry was cached
    pub cached_at: DateTime<Utc>,

    /// Time-to-live in seconds
    pub ttl_seconds: i64,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl_seconds: ttl.num_seconds(),
        }
    }

    /// Check if this entry is still valid
    pub fn is_valid(&self) -> bool {
        let expiry = self.cached_at + Duration::seconds(self.ttl_seconds);
        Utc::now() < expiry
    }

    /// Get time until expiry
    pub fn time_until_expiry(&self) -> Duration {
        let expiry = self.cached_at + Duration::seconds(self.ttl_seconds);
        expiry - Utc::now()
    }
}

/// Cache storage manager
pub struct CacheStore {
    cache_dir: PathBuf,
    memory_cache: HashMap<String, Vec<u8>>,
}

impl CacheStore {
    /// Create a new cache store
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            memory_cache: HashMap::new(),
        })
    }

    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf, CacheError> {
        ProjectDirs::from("com", "amdphreak", "github-actions-editor")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .ok_or_else(|| {
                CacheError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine cache directory",
                ))
            })
    }

    /// Store a value in the cache
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError> {
        let entry = CacheEntry::new(value, ttl);
        let json = serde_json::to_vec(&entry)?;

        // Store in memory cache
        self.memory_cache.insert(key.to_string(), json.clone());

        // Store on disk
        let path = self.key_to_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;

        tracing::debug!("Cached: {} (TTL: {} seconds)", key, ttl.num_seconds());
        Ok(())
    }

    /// Get a value from the cache
    pub fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<T, CacheError> {
        // Try memory cache first
        if let Some(json) = self.memory_cache.get(key) {
            let entry: CacheEntry<T> = serde_json::from_slice(json)?;
            if entry.is_valid() {
                tracing::debug!("Cache hit (memory): {}", key);
                return Ok(entry.data);
            }
        }

        // Try disk cache
        let path = self.key_to_path(key);
        if path.exists() {
            let json = fs::read(&path)?;
            let entry: CacheEntry<T> = serde_json::from_slice(&json)?;

            if entry.is_valid() {
                // Populate memory cache
                self.memory_cache.insert(key.to_string(), json);
                tracing::debug!("Cache hit (disk): {}", key);
                return Ok(entry.data);
            } else {
                // Remove expired entry
                let _ = fs::remove_file(&path);
                tracing::debug!("Cache expired: {}", key);
                return Err(CacheError::Expired(key.to_string()));
            }
        }

        tracing::debug!("Cache miss: {}", key);
        Err(CacheError::Miss(key.to_string()))
    }

    /// Get a value, allowing stale data if still within grace period
    pub fn get_with_stale<T: DeserializeOwned>(
        &mut self,
        key: &str,
        grace_period: Duration,
    ) -> Result<(T, bool), CacheError> {
        // Try memory cache first
        if let Some(json) = self.memory_cache.get(key) {
            let entry: CacheEntry<T> = serde_json::from_slice(json)?;
            let is_fresh = entry.is_valid();
            let within_grace = entry.cached_at + Duration::seconds(entry.ttl_seconds) + grace_period > Utc::now();

            if is_fresh || within_grace {
                return Ok((entry.data, is_fresh));
            }
        }

        // Try disk cache
        let path = self.key_to_path(key);
        if path.exists() {
            let json = fs::read(&path)?;
            let entry: CacheEntry<T> = serde_json::from_slice(&json)?;
            let is_fresh = entry.is_valid();
            let within_grace = entry.cached_at + Duration::seconds(entry.ttl_seconds) + grace_period > Utc::now();

            if is_fresh || within_grace {
                self.memory_cache.insert(key.to_string(), json);
                return Ok((entry.data, is_fresh));
            }
        }

        Err(CacheError::Miss(key.to_string()))
    }

    /// Check if a key exists and is valid
    pub fn has(&mut self, key: &str) -> bool {
        self.get::<serde_json::Value>(key).is_ok()
    }

    /// Remove a key from the cache
    pub fn remove(&mut self, key: &str) -> Result<(), CacheError> {
        self.memory_cache.remove(key);
        let path = self.key_to_path(key);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Clear all cached data
    pub fn clear(&mut self) -> Result<(), CacheError> {
        self.memory_cache.clear();
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        tracing::info!("Cache cleared");
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let disk_entries = fs::read_dir(&self.cache_dir)
            .map(|entries| entries.count())
            .unwrap_or(0);

        let disk_size = fs::read_dir(&self.cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0);

        CacheStats {
            memory_entries: self.memory_cache.len(),
            disk_entries,
            disk_size_bytes: disk_size,
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Convert a cache key to a file path
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Sanitize key for filesystem
        let safe_key = key
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_");
        self.cache_dir.join(format!("{}.json", safe_key))
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new().expect("Failed to create cache store")
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub disk_entries: usize,
    pub disk_size_bytes: u64,
    pub cache_dir: PathBuf,
}

impl CacheStats {
    /// Format disk size as human-readable string
    pub fn disk_size_human(&self) -> String {
        let bytes = self.disk_size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

