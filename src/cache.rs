//! Response caching system
//!
//! This module provides configurable response caching with TTL support
//! to improve performance and reduce API call frequency.

#[cfg(feature = "cache")]
use std::time::{Duration, Instant};
#[cfg(feature = "cache")]
use std::collections::HashMap;

#[cfg(feature = "cache")]
use parking_lot::RwLock;
#[cfg(feature = "cache")]
use serde::{Serialize, Deserialize};
use tracing::{debug, info};

#[cfg(feature = "cache")]
use crate::metrics::{METRICS, MetricsCollector};

/// Cache interface for storing and retrieving API responses
pub trait Cache: Send + Sync {
    /// Get a cached value by key
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    
    /// Store a value in cache with TTL
    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration);
    
    /// Delete a specific cache entry
    fn delete(&self, key: &str);
    
    /// Clear all cache entries
    fn clear(&self);
    
    /// Get cache statistics
    fn stats(&self) -> CacheStats;
    
    /// Cleanup expired entries (for maintenance)
    fn cleanup_expired(&self);
}

/// In-memory cache implementation with TTL support
#[cfg(feature = "cache")]
#[derive(Debug)]
pub struct MemoryCache {
    store: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
    max_entries: usize,
}

#[cfg(feature = "cache")]
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Instant,
    created_at: Instant,
    access_count: u32,
}

#[cfg(feature = "cache")]
impl MemoryCache {
    /// Create a new memory cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            default_ttl,
            max_entries: 1000, // Default limit
        }
    }
    
    /// Create a new memory cache with capacity and TTL limits
    pub fn with_capacity(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            store: RwLock::new(HashMap::with_capacity(capacity)),
            default_ttl,
            max_entries: capacity,
        }
    }
    
    /// Get the default TTL
    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }
    
    /// Check if cache has reached capacity
    fn is_at_capacity(&self) -> bool {
        let store = self.store.read();
        store.len() >= self.max_entries
    }
    
    /// Evict least recently used entry to make room
    fn evict_lru(&self) {
        let mut store = self.store.write();
        if store.is_empty() {
            return;
        }
        
        // Find the entry with the oldest access time
        let oldest_key = store.iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone());
            
        if let Some(key) = oldest_key {
            store.remove(&key);
            debug!(cache_key = %key, "Evicted LRU cache entry");
        }
    }
}

#[cfg(feature = "cache")]
impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut store = self.store.write();
        
        if let Some(entry) = store.get_mut(key) {
            if entry.expires_at > Instant::now() {
                entry.access_count += 1;
                debug!(cache_key = key, "Cache hit");
                METRICS.record_cache_hit(key);
                Some(entry.data.clone())
            } else {
                // Entry expired, remove it
                store.remove(key);
                debug!(cache_key = key, "Cache entry expired and removed");
                METRICS.record_cache_miss(key);
                None
            }
        } else {
            debug!(cache_key = key, "Cache miss");
            METRICS.record_cache_miss(key);
            None
        }
    }
    
    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) {
        let now = Instant::now();
        let expires_at = now + ttl;
        
        // If at capacity, evict LRU entry first
        if self.is_at_capacity() {
            self.evict_lru();
        }
        
        let value_size = value.len();
        let entry = CacheEntry {
            data: value,
            expires_at,
            created_at: now,
            access_count: 0,
        };
        
        let mut store = self.store.write();
        store.insert(key.to_string(), entry);
        
        debug!(
            cache_key = key,
            ttl_secs = ttl.as_secs(),
            value_size = value_size,
            "Cache entry stored"
        );
    }
    
    fn delete(&self, key: &str) {
        let mut store = self.store.write();
        if store.remove(key).is_some() {
            debug!(cache_key = key, "Cache entry deleted");
        }
    }
    
    fn clear(&self) {
        let mut store = self.store.write();
        let count = store.len();
        store.clear();
        info!(entries_cleared = count, "Cache cleared");
    }
    
    fn stats(&self) -> CacheStats {
        let store = self.store.read();
        let now = Instant::now();
        let total_entries = store.len();
        let expired_entries = store.values()
            .filter(|entry| entry.expires_at <= now)
            .count();
        let total_size_bytes = store.values()
            .map(|entry| entry.data.len())
            .sum();
        
        CacheStats {
            total_entries,
            active_entries: total_entries - expired_entries,
            expired_entries,
            total_size_bytes,
            max_capacity: self.max_entries,
        }
    }
    
    fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut store = self.store.write();
        let original_count = store.len();
        
        store.retain(|key, entry| {
            let is_valid = entry.expires_at > now;
            if !is_valid {
                debug!(cache_key = key, "Removing expired cache entry");
            }
            is_valid
        });
        
        let removed_count = original_count - store.len();
        if removed_count > 0 {
            info!(removed_entries = removed_count, "Cleaned up expired cache entries");
        }
    }
}

/// Cache statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Number of active (non-expired) entries
    pub active_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Total size of cached data in bytes
    pub total_size_bytes: usize,
    /// Maximum cache capacity
    pub max_capacity: usize,
}

impl CacheStats {
    /// Create empty cache stats
    pub fn empty() -> Self {
        Self {
            total_entries: 0,
            active_entries: 0,
            expired_entries: 0,
            total_size_bytes: 0,
            max_capacity: 0,
        }
    }
    
    /// Calculate cache utilization as a percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.max_capacity == 0 {
            0.0
        } else {
            (self.active_entries as f64 / self.max_capacity as f64) * 100.0
        }
    }
    
    /// Calculate average entry size in bytes
    pub fn avg_entry_size_bytes(&self) -> f64 {
        if self.active_entries == 0 {
            0.0
        } else {
            self.total_size_bytes as f64 / self.active_entries as f64
        }
    }
}

/// Cache key generation utilities
pub fn generate_cache_key(endpoint: &str, params: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let full_key = if params.is_empty() {
        endpoint.to_string()
    } else {
        format!("{}?{}", endpoint, params)
    };
    
    let mut hasher = DefaultHasher::new();
    full_key.hash(&mut hasher);
    
    // Create a readable cache key
    let endpoint_safe = endpoint.replace('/', "_").replace('?', "_").replace('&', "_");
    format!("gouqi:{}:{:x}", endpoint_safe, hasher.finish())
}

/// Generate cache key for specific Jira operations
pub fn jira_cache_key(operation: &str, resource_id: &str, params: &str) -> String {
    let base_key = if resource_id.is_empty() {
        operation.to_string()
    } else {
        format!("{}/{}", operation, resource_id)
    };
    
    generate_cache_key(&base_key, params)
}

/// Runtime cache configuration for different endpoint types  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCacheConfig {
    /// Enable caching globally
    pub enabled: bool,
    /// Default TTL for cached entries
    #[serde(with = "humantime_serde")]
    pub default_ttl: Duration,
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Per-endpoint cache strategies
    pub strategies: HashMap<String, RuntimeCacheStrategy>,
}

/// Runtime cache strategy for specific endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCacheStrategy {
    /// TTL for this endpoint
    #[serde(with = "humantime_serde")]
    pub ttl: Duration,
    /// Whether to cache error responses
    pub cache_errors: bool,
    /// Whether to use ETags for cache validation
    pub use_etag: bool,
}

impl Default for RuntimeCacheConfig {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        
        // Configure common Jira endpoints with appropriate cache strategies
        strategies.insert("issues".to_string(), RuntimeCacheStrategy {
            ttl: Duration::from_secs(300), // 5 minutes for issues
            cache_errors: false,
            use_etag: true,
        });
        
        strategies.insert("projects".to_string(), RuntimeCacheStrategy {
            ttl: Duration::from_secs(3600), // 1 hour for projects (less frequent changes)
            cache_errors: false,
            use_etag: true,
        });
        
        strategies.insert("users".to_string(), RuntimeCacheStrategy {
            ttl: Duration::from_secs(1800), // 30 minutes for users
            cache_errors: false,
            use_etag: false,
        });
        
        strategies.insert("search".to_string(), RuntimeCacheStrategy {
            ttl: Duration::from_secs(60), // 1 minute for search (results change frequently)
            cache_errors: false,
            use_etag: false,
        });
        
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(300), // 5 minutes default
            max_entries: 1000,
            strategies,
        }
    }
}

impl RuntimeCacheConfig {
    /// Get cache strategy for an endpoint
    pub fn strategy_for_endpoint(&self, endpoint: &str) -> RuntimeCacheStrategy {
        // Try to find a matching strategy
        for (pattern, strategy) in &self.strategies {
            if endpoint.contains(pattern) {
                return strategy.clone();
            }
        }
        
        // Return default strategy
        RuntimeCacheStrategy {
            ttl: self.default_ttl,
            cache_errors: false,
            use_etag: true,
        }
    }
    
    /// Check if an endpoint should be cached
    pub fn should_cache_endpoint(&self, endpoint: &str) -> bool {
        self.enabled && !endpoint.contains("search") // Don't cache search by default
    }
}

/// No-op cache implementation when caching is disabled
#[cfg(not(feature = "cache"))]
pub struct MemoryCache;

#[cfg(not(feature = "cache"))]
impl MemoryCache {
    pub fn new(_default_ttl: Duration) -> Self {
        Self
    }
    
    pub fn with_capacity(_capacity: usize, _default_ttl: Duration) -> Self {
        Self
    }
}

#[cfg(not(feature = "cache"))]
impl Cache for MemoryCache {
    fn get(&self, _key: &str) -> Option<Vec<u8>> { None }
    fn set(&self, _key: &str, _value: Vec<u8>, _ttl: Duration) {}
    fn delete(&self, _key: &str) {}
    fn clear(&self) {}
    fn stats(&self) -> CacheStats { CacheStats::empty() }
    fn cleanup_expired(&self) {}
}