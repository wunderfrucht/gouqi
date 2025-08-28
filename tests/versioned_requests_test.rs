//! Unit tests for versioned request functionality
//! This ensures coverage of the new get_versioned and request_versioned methods

use gouqi::{Credentials, Jira};
use mockito::Server;

#[test]
fn test_get_versioned_request_construction() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Mock a V3 API response
    let mock = server.mock("GET", "/rest/api/3/test-endpoint?param=value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result": "success"}"#)
        .create();

    // Test versioned request construction
    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("3"), 
        "/test-endpoint?param=value"
    );

    mock.assert();
    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json["result"], "success");
}

#[test]
fn test_get_versioned_with_default_version() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Mock a latest API response  
    let mock = server.mock("GET", "/rest/api/latest/test-endpoint")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version": "latest"}"#)
        .create();

    // Test with None version (should default to "latest")
    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        None,
        "/test-endpoint"
    );

    mock.assert();
    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json["version"], "latest");
}

#[test] 
fn test_get_versioned_error_handling() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Mock a 404 error response
    let mock = server.mock("GET", "/rest/api/3/nonexistent")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Not Found"}"#)
        .create();

    // Test error handling
    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("3"),
        "/nonexistent"
    );

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(gouqi::Error::NotFound) => {
            // Expected error type
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_get_versioned_with_bearer_auth() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Bearer("test-token".to_string())).unwrap();

    // Mock endpoint that expects Bearer auth
    let mock = server.mock("GET", "/rest/api/3/secure-endpoint")
        .match_header("Authorization", "Bearer test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authenticated": true}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("3"),
        "/secure-endpoint"
    );

    mock.assert();
    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json["authenticated"], true);
}

#[test]
fn test_get_versioned_with_basic_auth() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Basic("user".to_string(), "pass".to_string())).unwrap();

    // Mock endpoint that expects Basic auth (base64 of "user:pass" is "dXNlcjpwYXNz")
    let mock = server.mock("GET", "/rest/api/2/basic-endpoint")
        .match_header("Authorization", "Basic dXNlcjpwYXNz")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"auth_type": "basic"}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("2"),
        "/basic-endpoint"
    );

    mock.assert();
    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json["auth_type"], "basic");
}

#[test]
fn test_get_versioned_different_apis() {
    let mut server = Server::new();
    let jira = Jira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Test agile API
    let agile_mock = server.mock("GET", "/rest/agile/latest/board")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"api": "agile"}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get_versioned("agile", None, "/board");
    agile_mock.assert();
    assert!(result.is_ok());

    // Test auth API
    let auth_mock = server.mock("GET", "/rest/auth/latest/session")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"api": "auth"}"#)
        .create();

    let result: Result<serde_json::Value, _> = jira.get_versioned("auth", None, "/session");
    auth_mock.assert();
    assert!(result.is_ok());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_get_versioned_request_construction() {
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = Server::new_async().await;
    let jira = AsyncJira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Mock a V3 API response
    let mock = server.mock("GET", "/rest/api/3/async-endpoint")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"async": true}"#)
        .create_async()
        .await;

    // Test async versioned request
    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("3"),
        "/async-endpoint"
    ).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json["async"], true);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_get_versioned_error_handling() {
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = Server::new_async().await;
    let jira = AsyncJira::new(&server.url(), Credentials::Anonymous).unwrap();

    // Mock a 403 error response
    let mock = server.mock("GET", "/rest/api/3/forbidden")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Forbidden"}"#)
        .create_async()
        .await;

    // Test async error handling
    let result: Result<serde_json::Value, _> = jira.get_versioned(
        "api",
        Some("3"),
        "/forbidden"
    ).await;

    mock.assert_async().await;
    assert!(result.is_err());
    // Should be handled as a fault with status code 403
    match result {
        Err(gouqi::Error::Fault { code, .. }) => {
            assert_eq!(code, reqwest::StatusCode::FORBIDDEN);
        }
        _ => panic!("Expected Fault error"),
    }
}