//! Additional tests for search endpoint selection to improve coverage
//! This covers edge cases and different scenarios for the search functionality

use gouqi::core::{ClientCore, SearchApiVersion};
use gouqi::{Credentials, Jira};
use mockito::Server;

#[test]
fn test_search_endpoint_selection_with_auto_fallback() {
    // Test the Auto case that should never happen in practice
    // but needs coverage for completeness
    let _core = ClientCore::with_search_api_version(
        "https://unknown-host.example.com",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .unwrap();

    let jira = Jira::with_search_api_version(
        "https://unknown-host.example.com",
        Credentials::Anonymous,
        SearchApiVersion::Auto,
    )
    .unwrap();

    let search = jira.search();
    let (api_name, endpoint, version) = search.get_search_endpoint();

    // For unknown hosts, should default to V2
    assert_eq!(api_name, "api");
    assert_eq!(endpoint, "/search");
    assert_eq!(version, Some("latest"));
}

#[test]
fn test_search_endpoint_selection_all_versions() {
    // Test explicit V2 selection
    let jira_v2 = Jira::with_search_api_version(
        "https://test.example.com",
        Credentials::Anonymous,
        SearchApiVersion::V2,
    )
    .unwrap();

    let search_v2 = jira_v2.search();
    let (api_name, endpoint, version) = search_v2.get_search_endpoint();
    assert_eq!(api_name, "api");
    assert_eq!(endpoint, "/search");
    assert_eq!(version, Some("latest"));

    // Test explicit V3 selection
    let jira_v3 = Jira::with_search_api_version(
        "https://test.example.com",
        Credentials::Anonymous,
        SearchApiVersion::V3,
    )
    .unwrap();

    let search_v3 = jira_v3.search();
    let (api_name, endpoint, version) = search_v3.get_search_endpoint();
    assert_eq!(api_name, "api");
    assert_eq!(endpoint, "/search/jql");
    assert_eq!(version, Some("3"));
}

#[test]
fn test_search_with_versioned_request_v2() {
    let mut server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V2)
            .unwrap();

    // Mock V2 search response
    let mock = server
        .mock("GET", "/rest/api/latest/search?jql=project%3DTEST")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [],
            "total": 0,
            "startAt": 0,
            "maxResults": 50
        }"#,
        )
        .create();

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list("project=TEST", &search_options);

    mock.assert();
    assert!(result.is_ok());
    let search_results = result.unwrap();
    assert_eq!(search_results.total, 0);
}

#[test]
fn test_search_with_versioned_request_v3() {
    let mut server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();

    // Mock V3 search response (now includes auto-injected essential fields and maxResults)
    let mock = server
        .mock(
            "GET",
            "/rest/api/3/search/jql?fields=id%2Cself%2Ckey%2Csummary%2Cstatus&maxResults=50&jql=project%3DTEST",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [],
            "isLast": true
        }"#,
        )
        .create();

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list("project=TEST", &search_options);

    mock.assert();
    assert!(result.is_ok());
    let search_results = result.unwrap();
    assert_eq!(search_results.total, 0); // V3 conversion: 0 + 0 issues = 0 total
}

#[test]
fn test_search_with_complex_jql_query() {
    let mut server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();

    // Mock V3 search with complex JQL (now includes auto-injected essential fields and maxResults)
    let mock = server
        .mock(
            "GET",
            "/rest/api/3/search/jql?fields=id%2Cself%2Ckey%2Csummary%2Cstatus&maxResults=50&jql=project%3DTEST+AND+status%3DOpen+ORDER+BY+created+DESC",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [{
                "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                "key": "TEST-1",
                "id": "10001",
                "fields": {"summary": "Test issue"}
            }],
            "isLast": true
        }"#,
        )
        .create();

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list(
        "project=TEST AND status=Open ORDER BY created DESC",
        &search_options,
    );

    mock.assert();
    match result {
        Ok(search_results) => {
            assert_eq!(search_results.total, 1); // V3 conversion: 0 + 1 issue = 1 total
            assert_eq!(search_results.issues.len(), 1);
            assert_eq!(search_results.issues[0].key, "TEST-1");
        }
        Err(e) => {
            panic!("Search failed with error: {:?}", e);
        }
    }
}

#[test]
fn test_search_error_handling_v3_format() {
    let mut server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();

    // Mock V3 error response format (now includes auto-injected essential fields and maxResults)
    let mock = server
        .mock(
            "GET",
            "/rest/api/3/search/jql?fields=id%2Cself%2Ckey%2Csummary%2Cstatus&maxResults=50&jql=invalid+jql",
        )
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Invalid JQL query"}"#)
        .create();

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list("invalid jql", &search_options);

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(gouqi::Error::Fault { code, errors }) => {
            assert_eq!(code, reqwest::StatusCode::BAD_REQUEST);
            assert_eq!(errors.error, Some("Invalid JQL query".to_string()));
        }
        _ => panic!("Expected Fault error with V3 format"),
    }
}

#[test]
fn test_search_error_handling_v2_format() {
    let mut server = Server::new();
    let jira =
        Jira::with_search_api_version(server.url(), Credentials::Anonymous, SearchApiVersion::V2)
            .unwrap();

    // Mock V2 error response format (spaces become + in URL encoding)
    let mock = server
        .mock("GET", "/rest/api/latest/search?jql=invalid+jql")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["JQL syntax error"], "errors": {}}"#)
        .create();

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list("invalid jql", &search_options);

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(gouqi::Error::Fault { code, errors }) => {
            assert_eq!(code, reqwest::StatusCode::BAD_REQUEST);
            assert_eq!(errors.error_messages, vec!["JQL syntax error"]);
        }
        _ => panic!("Expected Fault error with V2 format"),
    }
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_endpoint_selection() {
    use gouqi::r#async::Jira as AsyncJira;

    let jira = AsyncJira::with_search_api_version(
        "https://test.atlassian.net",
        Credentials::Anonymous,
        SearchApiVersion::V3,
    )
    .unwrap();

    let search = jira.search();
    let (api_name, endpoint, version) = search.get_search_endpoint();

    assert_eq!(api_name, "api");
    assert_eq!(endpoint, "/search/jql");
    assert_eq!(version, Some("3"));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_with_versioned_request() {
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = Server::new_async().await;
    let jira = AsyncJira::with_search_api_version(
        server.url(),
        Credentials::Anonymous,
        SearchApiVersion::V3,
    )
    .unwrap();

    // Mock async V3 search response (now includes auto-injected essential fields and maxResults)
    let mock = server
        .mock(
            "GET",
            "/rest/api/3/search/jql?fields=id%2Cself%2Ckey%2Csummary%2Cstatus&maxResults=50&jql=project%3DTEST",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [],
            "isLast": true
        }"#,
        )
        .create_async()
        .await;

    let search_options = gouqi::SearchOptions::builder().build();
    let result = jira.search().list("project=TEST", &search_options).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let search_results = result.unwrap();
    assert_eq!(search_results.total, 0); // V3 conversion: 0 + 0 issues = 0 total
}
