//! Additional tests for core functionality to improve coverage
//! This covers edge cases and different scenarios in the core module

use gouqi::core::{ClientCore, Credentials, JiraDeploymentType, SearchApiVersion};

#[test]
fn test_deployment_detection_edge_cases() {
    // Test various domain patterns
    let test_cases = vec![
        ("https://company.atlassian.net", JiraDeploymentType::Cloud),
        (
            "https://subdomain.atlassian.net/path",
            JiraDeploymentType::Cloud,
        ),
        ("https://test.atlassian.net:8080", JiraDeploymentType::Cloud),
        ("https://jira.company.com", JiraDeploymentType::Unknown),
        ("https://localhost:2990", JiraDeploymentType::Unknown),
        ("http://internal-jira.corp", JiraDeploymentType::Unknown),
        (
            "https://jira-server.example.org",
            JiraDeploymentType::Unknown,
        ),
    ];

    for (host, expected_type) in test_cases {
        let core = ClientCore::new(host, Credentials::Anonymous)
            .unwrap_or_else(|_| panic!("Failed to create ClientCore for {}", host));

        let detected_type = core.detect_deployment_type();
        assert_eq!(
            detected_type, expected_type,
            "Failed deployment detection for {}",
            host
        );
    }
}

#[test]
fn test_search_api_version_resolution_all_combinations() {
    // Test all combinations of deployment types and version settings
    let combinations = vec![
        // Cloud + Auto should give V3
        (
            "https://test.atlassian.net",
            SearchApiVersion::Auto,
            SearchApiVersion::V3,
        ),
        // Cloud + explicit V2 should stay V2
        (
            "https://test.atlassian.net",
            SearchApiVersion::V2,
            SearchApiVersion::V2,
        ),
        // Cloud + explicit V3 should stay V3
        (
            "https://test.atlassian.net",
            SearchApiVersion::V3,
            SearchApiVersion::V3,
        ),
        // Unknown + Auto should give V2
        (
            "https://jira.company.com",
            SearchApiVersion::Auto,
            SearchApiVersion::V2,
        ),
        // Unknown + explicit V2 should stay V2
        (
            "https://jira.company.com",
            SearchApiVersion::V2,
            SearchApiVersion::V2,
        ),
        // Unknown + explicit V3 should stay V3
        (
            "https://jira.company.com",
            SearchApiVersion::V3,
            SearchApiVersion::V3,
        ),
    ];

    for (host, input_version, expected_version) in combinations {
        let core = ClientCore::with_search_api_version(
            host,
            Credentials::Anonymous,
            input_version.clone(),
        )
        .unwrap_or_else(|_| panic!("Failed to create ClientCore for {}", host));

        let resolved_version = core.get_search_api_version();
        assert_eq!(
            resolved_version, expected_version,
            "Failed version resolution for {} with {:?}",
            host, input_version
        );
    }
}

#[test]
fn test_build_versioned_url_edge_cases() {
    let core = ClientCore::new("https://test.atlassian.net", Credentials::Anonymous).unwrap();

    // Test with empty endpoint
    let url = core.build_versioned_url("api", Some("3"), "").unwrap();
    assert_eq!(url.as_str(), "https://test.atlassian.net/rest/api/3");

    // Test with endpoint starting with /
    let url = core
        .build_versioned_url("api", Some("3"), "/endpoint")
        .unwrap();
    assert_eq!(
        url.as_str(),
        "https://test.atlassian.net/rest/api/3/endpoint"
    );

    // Test with endpoint not starting with /
    let url = core
        .build_versioned_url("api", Some("3"), "endpoint")
        .unwrap();
    assert_eq!(
        url.as_str(),
        "https://test.atlassian.net/rest/api/3endpoint"
    );

    // Test with None version (should default to "latest")
    let url = core.build_versioned_url("api", None, "/endpoint").unwrap();
    assert_eq!(
        url.as_str(),
        "https://test.atlassian.net/rest/api/latest/endpoint"
    );

    // Test different API types
    let url = core
        .build_versioned_url("agile", Some("1.0"), "/board")
        .unwrap();
    assert_eq!(
        url.as_str(),
        "https://test.atlassian.net/rest/agile/1.0/board"
    );

    let url = core
        .build_versioned_url("auth", Some("1"), "/session")
        .unwrap();
    assert_eq!(
        url.as_str(),
        "https://test.atlassian.net/rest/auth/1/session"
    );
}

#[test]
fn test_build_url_vs_build_versioned_url_consistency() {
    let core = ClientCore::new("https://test.example.com", Credentials::Anonymous).unwrap();

    // build_url should be equivalent to build_versioned_url with version "latest"
    let url1 = core.build_url("api", "/endpoint").unwrap();
    let url2 = core
        .build_versioned_url("api", Some("latest"), "/endpoint")
        .unwrap();
    assert_eq!(url1, url2);

    // Test with None version (defaults to "latest")
    let url3 = core.build_versioned_url("api", None, "/endpoint").unwrap();
    assert_eq!(url1, url3);
}

#[test]
fn test_credentials_debug_representation() {
    // Test that credentials are properly debuggable (important for logging)
    let creds_anonymous = Credentials::Anonymous;
    let debug_str = format!("{:?}", creds_anonymous);
    assert!(debug_str.contains("Anonymous"));

    let creds_basic = Credentials::Basic("user".to_string(), "pass".to_string());
    let debug_str = format!("{:?}", creds_basic);
    assert!(debug_str.contains("Basic"));

    let creds_bearer = Credentials::Bearer("token".to_string());
    let debug_str = format!("{:?}", creds_bearer);
    assert!(debug_str.contains("Bearer"));

    let creds_cookie = Credentials::Cookie("session123".to_string());
    let debug_str = format!("{:?}", creds_cookie);
    assert!(debug_str.contains("Cookie"));
}

#[test]
fn test_client_core_debug_representation() {
    let core = ClientCore::with_search_api_version(
        "https://test.atlassian.net",
        Credentials::Bearer("token".to_string()),
        SearchApiVersion::V3,
    )
    .unwrap();

    let debug_str = format!("{:?}", core);
    assert!(debug_str.contains("ClientCore"));
    assert!(debug_str.contains("host"));
    assert!(debug_str.contains("credentials"));
    assert!(debug_str.contains("search_api_version"));
    assert!(debug_str.contains("cache_enabled"));
}

#[test]
fn test_enum_equality_and_cloning() {
    // Test SearchApiVersion equality and cloning
    let v1 = SearchApiVersion::V3;
    let v2 = v1.clone();
    assert_eq!(v1, v2);

    let auto1 = SearchApiVersion::Auto;
    let auto2 = SearchApiVersion::Auto;
    assert_eq!(auto1, auto2);

    assert_ne!(SearchApiVersion::V2, SearchApiVersion::V3);
    assert_ne!(SearchApiVersion::Auto, SearchApiVersion::V2);

    // Test JiraDeploymentType equality and cloning
    let cloud1 = JiraDeploymentType::Cloud;
    let cloud2 = cloud1.clone();
    assert_eq!(cloud1, cloud2);

    assert_ne!(JiraDeploymentType::Cloud, JiraDeploymentType::Server);
    assert_ne!(JiraDeploymentType::DataCenter, JiraDeploymentType::Unknown);
}

#[test]
fn test_search_api_version_default() {
    let default_version = SearchApiVersion::default();
    assert_eq!(default_version, SearchApiVersion::Auto);
}

#[test]
fn test_invalid_url_handling() {
    // Test that invalid URLs are properly rejected
    let invalid_urls = vec!["not-a-url", "", "https://"];

    for invalid_url in invalid_urls {
        let result = ClientCore::new(invalid_url, Credentials::Anonymous);
        assert!(
            result.is_err(),
            "Should reject invalid URL: {}",
            invalid_url
        );
    }

    // FTP URLs are actually valid URLs, just not typical for Jira
    // But they should still create a ClientCore successfully
    let result = ClientCore::new("ftp://invalid-protocol.com", Credentials::Anonymous);
    assert!(result.is_ok(), "FTP URLs should be accepted by URL parser");
}

#[test]
fn test_host_with_various_schemes() {
    // Test different valid URL schemes
    let valid_urls = vec![
        "http://jira.company.com",
        "https://jira.company.com",
        "http://localhost:8080",
        "https://company.atlassian.net",
    ];

    for url in valid_urls {
        let result = ClientCore::new(url, Credentials::Anonymous);
        assert!(result.is_ok(), "Should accept valid URL: {}", url);
    }
}

#[cfg(feature = "cache")]
#[test]
fn test_client_core_with_cache_configuration() {
    use gouqi::cache::{MemoryCache, RuntimeCacheConfig};
    use std::sync::Arc;
    use std::time::Duration;

    let custom_cache = Arc::new(MemoryCache::new(Duration::from_secs(600)));
    let cache_config = RuntimeCacheConfig::default();

    let core = ClientCore::with_cache(
        "https://test.atlassian.net",
        Credentials::Anonymous,
        custom_cache,
        cache_config,
    )
    .unwrap();

    // Should default to Auto version when using cache constructor
    assert_eq!(core.get_search_api_version(), SearchApiVersion::V3); // Cloud should resolve to V3
}
