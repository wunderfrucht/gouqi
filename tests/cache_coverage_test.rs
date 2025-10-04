//! Additional tests to improve coverage for cache functionality

#[cfg(feature = "cache")]
mod cache_tests {
    use gouqi::cache::{
        Cache, CacheStats, MemoryCache, RuntimeCacheConfig, RuntimeCacheStrategy,
        generate_cache_key, jira_cache_key,
    };
    use std::collections::HashMap;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_cache_new() {
        let cache = MemoryCache::new(Duration::from_secs(60));
        let stats = cache.stats();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(cache.default_ttl(), Duration::from_secs(60));
    }

    #[test]
    fn test_memory_cache_with_capacity() {
        let cache = MemoryCache::with_capacity(10, Duration::from_secs(30));
        let stats = cache.stats();

        assert_eq!(stats.max_capacity, 10);
        assert_eq!(cache.default_ttl(), Duration::from_secs(30));
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        let key = "test_key";
        let value = b"test_value".to_vec();

        cache.set(key, value.clone(), Duration::from_secs(60));

        let retrieved = cache.get(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        let key = "expiring_key";
        let value = b"expiring_value".to_vec();

        // Set with very short TTL
        cache.set(key, value, Duration::from_millis(10));

        // Should be available immediately
        assert!(cache.get(key).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Should be expired
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_delete() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        let key = "delete_key";
        let value = b"delete_value".to_vec();

        cache.set(key, value, Duration::from_secs(60));
        assert!(cache.get(key).is_some());

        cache.delete(key);
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        cache.set("key1", b"value1".to_vec(), Duration::from_secs(60));
        cache.set("key2", b"value2".to_vec(), Duration::from_secs(60));
        cache.set("key3", b"value3".to_vec(), Duration::from_secs(60));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 3);

        cache.clear();

        let stats_after = cache.stats();
        assert_eq!(stats_after.total_entries, 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        cache.set("key1", b"short".to_vec(), Duration::from_secs(60));
        cache.set("key2", b"longer_value".to_vec(), Duration::from_millis(10));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert!(stats.total_size_bytes > 0);

        // Wait for one to expire
        thread::sleep(Duration::from_millis(20));

        let stats_after = cache.stats();
        assert_eq!(stats_after.total_entries, 2); // Still 2 total
        assert_eq!(stats_after.active_entries, 1); // Only 1 active
        assert_eq!(stats_after.expired_entries, 1); // 1 expired
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        cache.set("active", b"value1".to_vec(), Duration::from_secs(60));
        cache.set("expiring", b"value2".to_vec(), Duration::from_millis(10));

        thread::sleep(Duration::from_millis(20));

        // Before cleanup, both entries exist
        let stats_before = cache.stats();
        assert_eq!(stats_before.total_entries, 2);

        // Cleanup expired entries
        cache.cleanup_expired();

        // After cleanup, only active entry remains
        let stats_after = cache.stats();
        assert_eq!(stats_after.total_entries, 1);
        assert_eq!(stats_after.active_entries, 1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = MemoryCache::with_capacity(3, Duration::from_secs(60));

        // Fill cache to capacity
        cache.set("key1", b"value1".to_vec(), Duration::from_secs(60));
        thread::sleep(Duration::from_millis(10));
        cache.set("key2", b"value2".to_vec(), Duration::from_secs(60));
        thread::sleep(Duration::from_millis(10));
        cache.set("key3", b"value3".to_vec(), Duration::from_secs(60));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 3);

        // Adding one more should evict the oldest (key1)
        cache.set("key4", b"value4".to_vec(), Duration::from_secs(60));

        // key1 should be evicted
        assert!(cache.get("key1").is_none());
        // Others should still be there
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
        assert!(cache.get("key4").is_some());

        let stats_after = cache.stats();
        assert_eq!(stats_after.total_entries, 3);
    }

    #[test]
    fn test_cache_stats_utilization() {
        let stats = CacheStats {
            total_entries: 5,
            active_entries: 8,
            expired_entries: 2,
            total_size_bytes: 1000,
            max_capacity: 10,
        };

        assert_eq!(stats.utilization_percent(), 80.0);
    }

    #[test]
    fn test_cache_stats_avg_entry_size() {
        let stats = CacheStats {
            total_entries: 5,
            active_entries: 4,
            expired_entries: 1,
            total_size_bytes: 400,
            max_capacity: 10,
        };

        assert_eq!(stats.avg_entry_size_bytes(), 100.0);
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::empty();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.max_capacity, 0);
        assert_eq!(stats.utilization_percent(), 0.0);
        assert_eq!(stats.avg_entry_size_bytes(), 0.0);
    }

    #[test]
    fn test_generate_cache_key() {
        let key1 = generate_cache_key("/rest/api/2/issue/TEST-1", "");
        let key2 = generate_cache_key("/rest/api/2/issue/TEST-1", "expand=changelog");
        let key3 = generate_cache_key("/rest/api/2/issue/TEST-2", "");

        // Same endpoint and params should generate same key
        let key1_again = generate_cache_key("/rest/api/2/issue/TEST-1", "");
        assert_eq!(key1, key1_again);

        // Different params should generate different key
        assert_ne!(key1, key2);

        // Different endpoint should generate different key
        assert_ne!(key1, key3);

        // Keys should be readable (contain endpoint)
        assert!(key1.contains("_rest_api_2_issue_TEST-1"));
    }

    #[test]
    fn test_jira_cache_key() {
        let key1 = jira_cache_key("issues", "TEST-1", "");
        let key2 = jira_cache_key("issues", "TEST-1", "expand=changelog");
        let key3 = jira_cache_key("issues", "TEST-2", "");
        let key4 = jira_cache_key("issues", "", "jql=project=TEST");

        // Same operation and resource should generate same key
        let key1_again = jira_cache_key("issues", "TEST-1", "");
        assert_eq!(key1, key1_again);

        // Different params should generate different key
        assert_ne!(key1, key2);

        // Different resource should generate different key
        assert_ne!(key1, key3);

        // Empty resource should work
        assert!(key4.contains("issues"));
    }

    #[test]
    fn test_runtime_cache_config_default() {
        let config = RuntimeCacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert_eq!(config.max_entries, 1000);
        assert!(config.strategies.contains_key("issues"));
        assert!(config.strategies.contains_key("projects"));
        assert!(config.strategies.contains_key("users"));
        assert!(config.strategies.contains_key("search"));
    }

    #[test]
    fn test_runtime_cache_config_strategy_for_endpoint() {
        let config = RuntimeCacheConfig::default();

        // Should find matching strategy for issues
        let issues_strategy = config.strategy_for_endpoint("/rest/api/2/issues/TEST-1");
        assert_eq!(issues_strategy.ttl, Duration::from_secs(300));

        // Should find matching strategy for projects
        let projects_strategy = config.strategy_for_endpoint("/rest/api/2/projects");
        assert_eq!(projects_strategy.ttl, Duration::from_secs(3600));

        // Should find matching strategy for users
        let users_strategy = config.strategy_for_endpoint("/rest/api/2/users/john");
        assert_eq!(users_strategy.ttl, Duration::from_secs(1800));

        // Should return default for unknown endpoint
        let unknown_strategy = config.strategy_for_endpoint("/rest/api/2/unknown");
        assert_eq!(unknown_strategy.ttl, config.default_ttl);
    }

    #[test]
    fn test_runtime_cache_config_should_cache_endpoint() {
        let config = RuntimeCacheConfig::default();

        // Should cache regular endpoints
        assert!(config.should_cache_endpoint("/rest/api/2/issues/TEST-1"));
        assert!(config.should_cache_endpoint("/rest/api/2/projects"));

        // Should not cache search endpoints by default
        assert!(!config.should_cache_endpoint("/rest/api/2/search"));
    }

    #[test]
    fn test_runtime_cache_config_disabled() {
        let config = RuntimeCacheConfig {
            enabled: false,
            ..Default::default()
        };

        // Should not cache any endpoint when disabled
        assert!(!config.should_cache_endpoint("/rest/api/2/issues/TEST-1"));
        assert!(!config.should_cache_endpoint("/rest/api/2/projects"));
    }

    #[test]
    fn test_runtime_cache_strategy() {
        let strategy = RuntimeCacheStrategy {
            ttl: Duration::from_secs(600),
            cache_errors: true,
            use_etag: false,
        };

        assert_eq!(strategy.ttl, Duration::from_secs(600));
        assert!(strategy.cache_errors);
        assert!(!strategy.use_etag);
    }

    #[test]
    fn test_runtime_cache_config_custom_strategies() {
        let mut strategies = HashMap::new();
        strategies.insert(
            "custom".to_string(),
            RuntimeCacheStrategy {
                ttl: Duration::from_secs(120),
                cache_errors: false,
                use_etag: true,
            },
        );

        let config = RuntimeCacheConfig {
            enabled: true,
            default_ttl: Duration::from_secs(60),
            max_entries: 500,
            strategies,
        };

        let custom_strategy = config.strategy_for_endpoint("/rest/api/2/custom");
        assert_eq!(custom_strategy.ttl, Duration::from_secs(120));
    }
}

#[cfg(not(feature = "cache"))]
mod cache_disabled_tests {
    use gouqi::cache::{Cache, CacheStats, MemoryCache};
    use std::time::Duration;

    #[test]
    fn test_no_op_cache_new() {
        let cache = MemoryCache::new(Duration::from_secs(60));
        assert!(cache.get("any_key").is_none());
    }

    #[test]
    fn test_no_op_cache_with_capacity() {
        let cache = MemoryCache::with_capacity(10, Duration::from_secs(30));
        assert!(cache.get("any_key").is_none());
    }

    #[test]
    fn test_no_op_cache_operations() {
        let cache = MemoryCache::new(Duration::from_secs(60));

        // All operations should be no-ops
        cache.set("key", b"value".to_vec(), Duration::from_secs(60));
        assert!(cache.get("key").is_none());

        cache.delete("key");
        cache.clear();
        cache.cleanup_expired();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 0);
    }
}
