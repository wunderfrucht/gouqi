//! Tests for search functionality edge cases
//! This covers the remaining uncovered paths in search.rs

use gouqi::{Credentials, Jira, SearchOptions};

#[test]
fn test_search_get_endpoint_method() {
    // Test the get_search_endpoint method directly
    let jira = Jira::new("https://test.example.com", Credentials::Anonymous).unwrap();
    let search = jira.search();
    
    let (api_name, endpoint, version) = search.get_search_endpoint();
    assert_eq!(api_name, "api");
    assert!(endpoint == "/search" || endpoint == "/search/jql");
    assert!(version.is_some());
}

#[test]
fn test_search_options_serialization() {
    // Test SearchOptions with various configurations
    let minimal_options = SearchOptions::builder().build();
    let serialized = minimal_options.serialize();
    // Empty SearchOptions returns None
    assert!(serialized.is_none());

    let options_with_params = SearchOptions::builder()
        .max_results(100)
        .start_at(50)
        .build();

    let serialized = options_with_params.serialize();
    assert!(serialized.is_some());
    let query = serialized.unwrap();
    assert!(query.contains("maxResults=100"));
    assert!(query.contains("startAt=50"));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_get_endpoint() {
    use gouqi::r#async::Jira as AsyncJira;
    
    let jira = AsyncJira::new("https://test.example.com", Credentials::Anonymous).unwrap();
    let search = jira.search();
    
    let (api_name, endpoint, version) = search.get_search_endpoint();
    assert_eq!(api_name, "api");
    assert!(endpoint == "/search" || endpoint == "/search/jql");
    assert!(version.is_some());
}