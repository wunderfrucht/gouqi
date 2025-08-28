//! Tests for search iterator functionality and edge cases
//! This covers the remaining uncovered paths in search.rs

use gouqi::{Credentials, Jira, SearchOptions};
use mockito::Server;

#[test]
fn test_search_iterator_functionality() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // First page response
    let mock_page1 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=0",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                    "key": "TEST-1",
                    "id": "10001",
                    "fields": {"summary": "First issue"}
                },
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-2", 
                    "key": "TEST-2",
                    "id": "10002",
                    "fields": {"summary": "Second issue"}
                }
            ],
            "total": 4,
            "startAt": 0,
            "maxResults": 2
        }"#,
        )
        .create();

    // Second page response
    let mock_page2 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=2",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-3",
                    "key": "TEST-3", 
                    "id": "10003",
                    "fields": {"summary": "Third issue"}
                },
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-4",
                    "key": "TEST-4",
                    "id": "10004", 
                    "fields": {"summary": "Fourth issue"}
                }
            ],
            "total": 4,
            "startAt": 2,
            "maxResults": 2
        }"#,
        )
        .create();

    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    // Create iterator and consume all issues
    let mut iter = jira.search().iter("project=TEST", &search_options).unwrap();

    // Iterate through all issues to test pagination
    let mut issue_keys = Vec::new();
    for issue in &mut iter {
        issue_keys.push(issue.key.clone());
    }

    mock_page1.assert();
    mock_page2.assert();

    // Should have collected all 4 issues (though order might be reversed due to pop())
    assert_eq!(issue_keys.len(), 4);
    assert!(issue_keys.contains(&"TEST-1".to_string()));
    assert!(issue_keys.contains(&"TEST-2".to_string()));
    assert!(issue_keys.contains(&"TEST-3".to_string()));
    assert!(issue_keys.contains(&"TEST-4".to_string()));
}

#[test]
fn test_search_iterator_with_errors() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // First page succeeds
    let mock_page1 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=0",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                    "key": "TEST-1",
                    "id": "10001",
                    "fields": {"summary": "First issue"}
                }
            ],
            "total": 3,
            "startAt": 0,
            "maxResults": 2
        }"#,
        )
        .create();

    // Second page fails
    let mock_page2_error = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=2",
        )
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Server error"]}"#)
        .create();

    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    // Create iterator
    let mut iter = jira.search().iter("project=TEST", &search_options).unwrap();

    // First issue should work
    let first_issue = iter.next();
    assert!(first_issue.is_some());
    assert_eq!(first_issue.unwrap().key, "TEST-1");

    // Second page request fails, so iteration should stop
    let second_issue = iter.next();
    assert!(second_issue.is_none());

    mock_page1.assert();
    mock_page2_error.assert();
}

#[test]
fn test_search_iterator_empty_results() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Empty results
    let mock_empty = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DNONEXISTENT&maxResults=50&startAt=0",
        )
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

    let search_options = SearchOptions::builder().build();
    let mut iter = jira
        .search()
        .iter("project=NONEXISTENT", &search_options)
        .unwrap();

    // Should return None immediately
    let issue = iter.next();
    assert!(issue.is_none());

    mock_empty.assert();
}

#[test]
fn test_search_options_serialization_edge_cases() {
    // Test SearchOptions with various configurations
    let minimal_options = SearchOptions::builder().build();
    let serialized = minimal_options.serialize();
    assert!(serialized.is_some());

    let full_options = SearchOptions::builder()
        .max_results(100)
        .start_at(50)
        .fields(vec!["key", "summary", "status"])
        .expand(vec!["renderedFields", "names"])
        .build();

    let serialized = full_options.serialize();
    assert!(serialized.is_some());
    let query = serialized.unwrap();
    assert!(query.contains("maxResults=100"));
    assert!(query.contains("startAt=50"));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_stream_functionality() {
    use futures::stream::StreamExt;
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = mockito::Server::new_async().await;
    let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

    // First page
    let mock_page1 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=0",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                    "key": "TEST-1",
                    "id": "10001",
                    "fields": {"summary": "First issue"}
                },
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-2",
                    "key": "TEST-2", 
                    "id": "10002",
                    "fields": {"summary": "Second issue"}
                }
            ],
            "total": 3,
            "startAt": 0,
            "maxResults": 2
        }"#,
        )
        .create_async()
        .await;

    // Second page
    let mock_page2 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=2",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-3",
                    "key": "TEST-3",
                    "id": "10003",
                    "fields": {"summary": "Third issue"}
                }
            ],
            "total": 3,
            "startAt": 2,
            "maxResults": 2
        }"#,
        )
        .create_async()
        .await;

    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    // Test stream functionality
    let search = jira.search();
    let stream = search
        .stream("project=TEST", &search_options)
        .await
        .unwrap();
    let issues: Vec<_> = stream.collect().await;

    mock_page1.assert_async().await;
    mock_page2.assert_async().await;

    assert_eq!(issues.len(), 3);
    assert_eq!(issues[0].key, "TEST-1");
    assert_eq!(issues[1].key, "TEST-2");
    assert_eq!(issues[2].key, "TEST-3");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_stream_empty_page() {
    use futures::stream::StreamExt;
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = mockito::Server::new_async().await;
    let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

    // Page with issues
    let mock_page1 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=0",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                    "key": "TEST-1",
                    "id": "10001",
                    "fields": {"summary": "First issue"}
                }
            ],
            "total": 2,
            "startAt": 0,
            "maxResults": 2
        }"#,
        )
        .create_async()
        .await;

    // Empty second page
    let mock_page2 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=2",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [],
            "total": 2,
            "startAt": 2,
            "maxResults": 2
        }"#,
        )
        .create_async()
        .await;

    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    // Test stream with empty second page
    let search = jira.search();
    let stream = search
        .stream("project=TEST", &search_options)
        .await
        .unwrap();
    let issues: Vec<_> = stream.collect().await;

    mock_page1.assert_async().await;
    mock_page2.assert_async().await;

    // Should only get the one issue from first page
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key, "TEST-1");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_search_stream_error_handling() {
    use futures::stream::StreamExt;
    use gouqi::r#async::Jira as AsyncJira;

    let mut server = mockito::Server::new_async().await;
    let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

    // First page works
    let mock_page1 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=0",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "issues": [
                {
                    "self": "https://test.atlassian.net/rest/api/2/issue/TEST-1",
                    "key": "TEST-1",
                    "id": "10001",
                    "fields": {"summary": "First issue"}
                }
            ],
            "total": 3,
            "startAt": 0,
            "maxResults": 2
        }"#,
        )
        .create_async()
        .await;

    // Second page fails
    let mock_page2 = server
        .mock(
            "GET",
            "/rest/api/latest/search?jql=project%3DTEST&maxResults=2&startAt=2",
        )
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Server error"]}"#)
        .create_async()
        .await;

    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    // Test stream with error on second page
    let search = jira.search();
    let stream = search
        .stream("project=TEST", &search_options)
        .await
        .unwrap();
    let issues: Vec<_> = stream.collect().await;

    mock_page1.assert_async().await;
    mock_page2.assert_async().await;

    // Should only get the issue from first page since second page fails
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key, "TEST-1");
}
