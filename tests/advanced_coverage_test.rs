//! Advanced coverage tests for hard-to-reach code paths
//! This covers cache, metrics, error handling, and edge case scenarios

use gouqi::core::{ClientCore, Credentials, JiraDeploymentType, RequestContext, SearchApiVersion};
use gouqi::{Error, Jira};
use mockito::Server;

#[test]
fn test_request_context_creation_and_span() {
    let ctx = RequestContext::new("GET", "/test");

    // Test span creation - we can't test the name but can verify it creates a span
    let _span = ctx.create_span();

    // Test finish with success and failure
    ctx.finish(true);

    let ctx2 = RequestContext::new("POST", "/another");
    ctx2.finish(false);
}

#[test]
fn test_post_and_put_methods() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test POST with empty body
    let mock_post = server
        .mock("POST", "/rest/api/latest/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"created": true}"#)
        .create();

    #[derive(serde::Serialize, Default)]
    struct TestBody {
        name: String,
    }

    let body = TestBody {
        name: "test".to_string(),
    };
    let result: Result<serde_json::Value, _> = jira.post("api", "/test", body);

    mock_post.assert();
    assert!(result.is_ok());

    // Test PUT method
    let mock_put = server
        .mock("PUT", "/rest/api/latest/update")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"updated": true}"#)
        .create();

    let body = TestBody {
        name: "updated".to_string(),
    };
    let result: Result<serde_json::Value, _> = jira.put("api", "/update", body);

    mock_put.assert();
    assert!(result.is_ok());

    // Test DELETE method
    let mock_delete = server
        .mock("DELETE", "/rest/api/latest/item/123")
        .with_status(204)
        .with_header("content-type", "application/json")
        .with_body("")
        .create();

    let result: Result<gouqi::core::EmptyResponse, _> = jira.delete("api", "/item/123");
    mock_delete.assert();
    assert!(result.is_ok());
}

#[test]
fn test_interface_creation_methods() {
    let jira = Jira::new("https://test.example.com", Credentials::Anonymous).unwrap();

    // Test all interface creation methods for coverage
    let _transitions = jira.transitions("TEST-1");
    let _attachments = jira.attachments();
    let _components = jira.components();
    let _versions = jira.versions();
    let _resolution = jira.resolution();
    let _sprints = jira.sprints();

    // Test method that uses tracing instruments
    let _search = jira.search();
    let _issues = jira.issues();
    let _projects = jira.projects();
    let _boards = jira.boards();
}

#[test]
fn test_client_creation_with_custom_client() {
    let client = reqwest::blocking::Client::new();
    let result = Jira::from_client("https://test.example.com", Credentials::Anonymous, client);
    assert!(result.is_ok());
}

#[test]
fn test_client_with_core_creation() {
    let core = ClientCore::new("https://test.example.com", Credentials::Anonymous).unwrap();
    let result = Jira::with_core(core);
    assert!(result.is_ok());
}

#[cfg(feature = "cache")]
#[test]
fn test_cache_functionality() {
    use gouqi::cache::{MemoryCache, RuntimeCacheConfig};
    use std::sync::Arc;
    use std::time::Duration;

    let custom_cache = Arc::new(MemoryCache::new(Duration::from_secs(60)));
    let cache_config = RuntimeCacheConfig::default();

    let core = ClientCore::with_cache(
        "https://test.example.com",
        Credentials::Anonymous,
        custom_cache,
        cache_config,
    )
    .unwrap();

    let jira = Jira::with_core(core).unwrap();

    // Test cache stats and clear
    let _stats = jira.cache_stats();
    jira.clear_cache();
}

#[cfg(any(feature = "metrics", feature = "cache"))]
#[test]
fn test_observability_methods() {
    let jira = Jira::new("https://test.example.com", Credentials::Anonymous).unwrap();

    // Test observability methods
    let _report = jira.observability_report();
    let _health = jira.health_status();
}

#[test]
fn test_error_response_handling() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test 400 client error (which should be treated as an error)
    let mock_400 = server
        .mock("GET", "/rest/api/latest/badrequest")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Bad request"]}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get("api", "/badrequest");
    mock_400.assert();
    assert!(result.is_err(), "Expected error for 400 status, got: {:?}", result);

    // Test 401 unauthorized
    let mock_401 = server
        .mock("GET", "/rest/api/latest/unauthorized")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Unauthorized"]}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get("api", "/unauthorized");
    mock_401.assert();
    assert!(result.is_err(), "Expected error for 401 status, got: {:?}", result);

    match result {
        Err(Error::Unauthorized) => {
            // Expected
        }
        _ => panic!("Expected Unauthorized error, got: {:?}", result),
    }

    // Test 500 server error (which currently gets parsed as JSON, not treated as error)
    let mock_500 = server
        .mock("GET", "/rest/api/latest/servererror")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Internal server error"]}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get("api", "/servererror");
    mock_500.assert();
    // 500 is currently treated as successful response and parsed as JSON
    assert!(result.is_ok(), "500 status should be parsed as JSON, got: {:?}", result);
}

#[test]
fn test_deployment_detection_comprehensive() {
    // Test more deployment detection scenarios
    let test_cases = vec![
        (
            "https://company.atlassian.net/secure",
            JiraDeploymentType::Cloud,
        ),
        ("http://jira.internal.com", JiraDeploymentType::Unknown),
        ("https://192.168.1.100:8080", JiraDeploymentType::Unknown),
    ];

    for (url, expected) in test_cases {
        let core = ClientCore::new(url, Credentials::Anonymous).unwrap();
        let detected = core.detect_deployment_type();
        assert_eq!(detected, expected, "Failed for URL: {}", url);
    }
}

#[test]
fn test_credentials_display_and_equality() {
    let creds1 = Credentials::Basic("user".to_string(), "pass".to_string());
    let _creds2 = Credentials::Basic("user".to_string(), "pass".to_string());
    let creds3 = Credentials::Bearer("token".to_string());

    // Test Debug formatting (already covered in core_coverage_test.rs but ensures coverage)
    let debug_str = format!("{:?}", creds1);
    assert!(debug_str.contains("Basic"));

    // Note: Credentials doesn't implement PartialEq, so we can't test equality
    // but we can test that they can be created and formatted
    let _debug_bearer = format!("{:?}", creds3);
}

#[test]
fn test_search_api_version_cases() {
    // Test the SearchApiVersion enum exhaustively
    let auto = SearchApiVersion::Auto;
    let v2 = SearchApiVersion::V2;
    let v3 = SearchApiVersion::V3;

    assert_eq!(SearchApiVersion::default(), SearchApiVersion::Auto);
    assert_ne!(auto, v2);
    assert_ne!(v2, v3);

    // Test cloning
    let auto_clone = auto.clone();
    assert_eq!(auto, auto_clone);
}

#[test]
fn test_more_url_construction_edge_cases() {
    let core = ClientCore::new("https://test.com", Credentials::Anonymous).unwrap();

    // Test with query parameters in endpoint
    let url = core
        .build_versioned_url("api", Some("3"), "/search?existing=param&another=value")
        .unwrap();
    assert!(url.as_str().contains("existing=param"));
    assert!(url.as_str().contains("another=value"));

    // Test with fragment
    let url = core.build_url("api", "/endpoint#fragment").unwrap();
    assert!(url.as_str().contains("endpoint#fragment"));

    // Test with special characters that need encoding
    let url = core.build_url("api", "/endpoint with spaces").unwrap();
    assert!(url.as_str().contains("endpoint%20with%20spaces"));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_additional_coverage() {
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = mockito::Server::new_async().await;
    let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

    // Test async POST
    let mock_post = server
        .mock("POST", "/rest/api/latest/async-post")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"created": "async"}"#)
        .create_async()
        .await;

    #[derive(serde::Serialize)]
    struct AsyncBody {
        data: String,
    }

    let body = AsyncBody {
        data: "test".to_string(),
    };
    let result: Result<serde_json::Value, _> = jira.post("api", "/async-post", body).await;

    mock_post.assert_async().await;
    assert!(result.is_ok());

    // Test async PUT
    let mock_put = server
        .mock("PUT", "/rest/api/latest/async-put")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"updated": "async"}"#)
        .create_async()
        .await;

    let body = AsyncBody {
        data: "updated".to_string(),
    };
    let result: Result<serde_json::Value, _> = jira.put("api", "/async-put", body).await;

    mock_put.assert_async().await;
    assert!(result.is_ok());

    // Test async DELETE
    let mock_delete = server
        .mock("DELETE", "/rest/api/latest/async-delete")
        .with_status(204)
        .with_body("")
        .create_async()
        .await;

    let result: Result<gouqi::core::EmptyResponse, _> = jira.delete("api", "/async-delete").await;
    mock_delete.assert_async().await;
    assert!(result.is_ok());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_client_creation_methods() {
    use gouqi::r#async::Jira as AsyncJira;

    // Test from_client
    let client = reqwest::Client::new();
    let jira = AsyncJira::from_client("https://test.com", Credentials::Anonymous, client).unwrap();

    // Test with_core
    let core = ClientCore::new("https://test.com", Credentials::Anonymous).unwrap();
    let jira2 = AsyncJira::with_core(core).unwrap();

    // Test interface creation methods
    let _search = jira.search();
    let _issues = jira.issues();
    let _projects = jira.projects();
    let _boards = jira.boards();

    // Test debug method
    #[cfg(debug_assertions)]
    {
        let _version = jira2.get_search_api_version();
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_cache_and_observability() {
    use gouqi::r#async::Jira as AsyncJira;

    let jira = AsyncJira::new("https://test.com", Credentials::Anonymous).unwrap();

    #[cfg(feature = "cache")]
    {
        let _stats = jira.cache_stats();
        jira.clear_cache();
    }

    #[cfg(any(feature = "metrics", feature = "cache"))]
    {
        let _report = jira.observability_report();
        let _health = jira.health_status();
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_sync_conversion() {
    use gouqi::r#async::Jira as AsyncJira;

    let async_jira = AsyncJira::new("https://test.com", Credentials::Anonymous).unwrap();

    // Test conversion from async to sync - do this without dropping inside async context
    let core = async_jira.core.clone();
    let sync_jira = gouqi::sync::Jira::with_core(core).unwrap();

    // Verify the sync client works by checking its debug output
    let debug_str = format!("{:?}", sync_jira);
    assert!(debug_str.contains("Jira"));
}

#[test]
fn test_error_branches_and_edge_cases() {
    // Test with invalid host to trigger connection error
    let jira = Jira::new("http://invalid-host-does-not-exist.example", Credentials::Anonymous).unwrap();

    // This should fail with a network error
    let result: Result<serde_json::Value, _> = jira.get("api", "/will-fail");
    assert!(result.is_err(), "Expected network error, got: {:?}", result);
}
