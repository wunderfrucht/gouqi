//! Tests for feature-conditional code paths and edge cases
//! This covers cache, metrics, and other conditional compilation paths

use gouqi::core::{ClientCore, SearchApiVersion};
use gouqi::{Credentials, Jira};

#[test]
fn test_conditional_features() {
    let _jira = Jira::new("https://test.example.com", Credentials::Anonymous).unwrap();

    // These methods exist regardless of features, testing the conditional compilation paths
    #[cfg(feature = "cache")]
    {
        let _stats = _jira.cache_stats();
        _jira.clear_cache();
    }

    #[cfg(any(feature = "metrics", feature = "cache"))]
    {
        let _report = _jira.observability_report();
        let _health = _jira.health_status();
    }
}

#[cfg(feature = "cache")]
#[test]
fn test_cache_edge_cases() {
    use gouqi::cache::{MemoryCache, RuntimeCacheConfig};
    use std::sync::Arc;
    use std::time::Duration;

    // Test with very short cache duration
    let short_cache = Arc::new(MemoryCache::new(Duration::from_millis(1)));
    let cache_config = RuntimeCacheConfig::default();

    let core = ClientCore::with_cache(
        "https://test.example.com",
        Credentials::Bearer("test-token".to_string()),
        short_cache,
        cache_config,
    )
    .unwrap();

    let jira = Jira::with_core(core).unwrap();

    // Test cache operations
    let stats_before = jira.cache_stats();
    assert_eq!(stats_before.total_entries, 0);

    jira.clear_cache();

    let stats_after = jira.cache_stats();
    assert_eq!(stats_after.total_entries, 0);
}

#[test]
fn test_debug_assertions() {
    let jira = Jira::new("https://test.example.com", Credentials::Anonymous).unwrap();

    // Test debug-only method
    #[cfg(debug_assertions)]
    {
        let version = jira.get_search_api_version();
        // Should resolve to a concrete version (V3 for Cloud)
        assert!(version == SearchApiVersion::V3 || version == SearchApiVersion::V2);
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_debug_assertions() {
    use gouqi::r#async::Jira as AsyncJira;

    let jira = AsyncJira::new("https://test.example.com", Credentials::Anonymous).unwrap();

    #[cfg(debug_assertions)]
    {
        let version = jira.get_search_api_version();
        assert!(version == SearchApiVersion::V3 || version == SearchApiVersion::V2);
    }
}

#[test]
fn test_credentials_variants() {
    // Test all credential variants for coverage
    let anon = Credentials::Anonymous;
    let basic = Credentials::Basic("user".to_string(), "pass".to_string());
    let bearer = Credentials::Bearer("token123".to_string());
    let cookie = Credentials::Cookie("JSESSIONID=abc123".to_string());

    // Test that all can be used to create clients
    let _jira1 = Jira::new("https://test1.com", anon).unwrap();
    let _jira2 = Jira::new("https://test2.com", basic).unwrap();
    let _jira3 = Jira::new("https://test3.com", bearer).unwrap();
    let _jira4 = Jira::new("https://test4.com", cookie).unwrap();
}

#[test]
fn test_empty_response_struct() {
    use gouqi::core::EmptyResponse;

    // Test that EmptyResponse can be serialized/deserialized
    let empty = EmptyResponse;
    let json = serde_json::to_string(&empty).unwrap();
    // EmptyResponse serializes to null, not {}
    assert_eq!(json, "null");

    let deserialized: EmptyResponse = serde_json::from_str("null").unwrap();
    let debug_str = format!("{:?}", deserialized);
    assert!(debug_str.contains("EmptyResponse"));
}

#[test]
fn test_url_edge_cases() {
    let core = ClientCore::new("https://jira.example.com:8080", Credentials::Anonymous).unwrap();

    // Test URL building with port
    let url = core.build_url("api", "/test").unwrap();
    assert!(url.as_str().contains(":8080"));

    // Test versioned URL with port
    let versioned_url = core.build_versioned_url("api", Some("3"), "/test").unwrap();
    assert!(versioned_url.as_str().contains(":8080"));
    assert!(versioned_url.as_str().contains("/rest/api/3/test"));
}

#[test]
fn test_search_api_version_resolution_edge_cases() {
    // Test Auto resolution for different deployment types
    let cloud_core = ClientCore::with_search_api_version(
        "https://test.atlassian.net",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .unwrap();

    let resolved = cloud_core.get_search_api_version();
    assert_eq!(resolved, SearchApiVersion::V3); // Cloud should resolve to V3

    let unknown_core = ClientCore::with_search_api_version(
        "https://unknown.example.com",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .unwrap();

    let resolved = unknown_core.get_search_api_version();
    assert_eq!(resolved, SearchApiVersion::V2); // Unknown should resolve to V2
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_conversion_edge_cases() {
    use gouqi::r#async::Jira as AsyncJira;

    let async_jira = AsyncJira::new(
        "https://convert-test.com",
        Credentials::Bearer("token".to_string()),
    )
    .unwrap();

    // Test that the async client works by checking debug output
    let async_debug = format!("{:?}", async_jira);
    assert!(async_debug.contains("Jira"));

    // Test async interface creation methods
    let _search = async_jira.search();
    let _issues = async_jira.issues();
    let _projects = async_jira.projects();
    let _boards = async_jira.boards();
}

#[test]
fn test_sync_conversion_edge_cases() {
    let sync_jira = gouqi::sync::Jira::new(
        "https://convert-test.com",
        Credentials::Bearer("token".to_string()),
    )
    .unwrap();

    // Test that the sync client works by checking debug output
    let sync_debug = format!("{:?}", sync_jira);
    assert!(sync_debug.contains("Jira"));

    // Test sync interface creation methods
    let _search = sync_jira.search();
    let _issues = sync_jira.issues();
    let _projects = sync_jira.projects();
    let _boards = sync_jira.boards();
}

#[test]
fn test_all_interface_methods() {
    let jira = Jira::new("https://interface-test.com", Credentials::Anonymous).unwrap();

    // Test that all interface creation methods work
    let _transitions = jira.transitions("ISSUE-1");
    let _attachments = jira.attachments();
    let _components = jira.components();
    let _versions = jira.versions();
    let _resolution = jira.resolution();
}

#[test]
fn test_core_clone_and_debug() {
    let core = ClientCore::new(
        "https://debug-test.com",
        Credentials::Bearer("test".to_string()),
    )
    .unwrap();

    // Test cloning
    let cloned_core = core.clone();
    assert_eq!(core.host, cloned_core.host);

    // Test debug formatting
    let debug_str = format!("{:?}", core);
    assert!(debug_str.contains("ClientCore"));
    assert!(debug_str.contains("host"));
    assert!(debug_str.contains("credentials"));
    assert!(debug_str.contains("search_api_version"));
    assert!(debug_str.contains("cache_enabled"));
}
