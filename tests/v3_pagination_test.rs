//! Comprehensive tests for V3 API pagination with nextPageToken
//! Tests both sync and async implementations to ensure parity

use gouqi::core::SearchApiVersion;
use gouqi::{Credentials, Jira, SearchOptions};
use serde_json::json;

#[cfg(feature = "async")]
use futures::stream::StreamExt;
#[cfg(feature = "async")]
use gouqi::r#async::Jira as AsyncJira;

// Test V3 sync iterator with multiple pages using nextPageToken
#[test]
fn test_v3_sync_pagination_with_next_page_token() {
    let mut server = mockito::Server::new();
    let url = server.url();

    // First page with nextPageToken
    let first_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=TEST".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "10001",
                        "key": "TEST-1",
                        "self": "https://jira.example.com/rest/api/3/issue/10001",
                        "fields": {
                            "summary": "First issue"
                        }
                    },
                    {
                        "id": "10002",
                        "key": "TEST-2",
                        "self": "https://jira.example.com/rest/api/3/issue/10002",
                        "fields": {
                            "summary": "Second issue"
                        }
                    }
                ],
                "isLast": false,
                "nextPageToken": "token_page_2"
            })
            .to_string(),
        )
        .create();

    // Second page with nextPageToken
    let second_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=TEST".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "token_page_2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "10003",
                        "key": "TEST-3",
                        "self": "https://jira.example.com/rest/api/3/issue/10003",
                        "fields": {
                            "summary": "Third issue"
                        }
                    },
                    {
                        "id": "10004",
                        "key": "TEST-4",
                        "self": "https://jira.example.com/rest/api/3/issue/10004",
                        "fields": {
                            "summary": "Fourth issue"
                        }
                    }
                ],
                "isLast": false,
                "nextPageToken": "token_page_3"
            })
            .to_string(),
        )
        .create();

    // Third and final page
    let third_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=TEST".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "token_page_3".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "10005",
                        "key": "TEST-5",
                        "self": "https://jira.example.com/rest/api/3/issue/10005",
                        "fields": {
                            "summary": "Fifth issue"
                        }
                    }
                ],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .create();

    // Create V3 Jira client
    let jira =
        Jira::with_search_api_version(url, Credentials::Anonymous, SearchApiVersion::V3).unwrap();
    let search_options = SearchOptions::builder().max_results(2).build();

    // Test iterator
    let iter = jira.search().iter("project=TEST", &search_options).unwrap();

    // Collect all issues
    let issues: Vec<_> = iter.collect();

    // Verify we got all 5 issues (in reverse order due to pop())
    assert_eq!(issues.len(), 5);
    assert_eq!(issues[0].key, "TEST-5");
    assert_eq!(issues[1].key, "TEST-4");
    assert_eq!(issues[2].key, "TEST-3");
    assert_eq!(issues[3].key, "TEST-2");
    assert_eq!(issues[4].key, "TEST-1");

    // Verify all mocks were called
    first_page_mock.assert();
    second_page_mock.assert();
    third_page_mock.assert();
}

// Test V3 async stream with multiple pages using nextPageToken
#[cfg(feature = "async")]
#[tokio::test]
async fn test_v3_async_pagination_with_next_page_token() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // First page with nextPageToken
    let first_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=ASYNC".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "20001",
                        "key": "ASYNC-1",
                        "self": "https://jira.example.com/rest/api/3/issue/20001",
                        "fields": {
                            "summary": "First async issue"
                        }
                    },
                    {
                        "id": "20002",
                        "key": "ASYNC-2",
                        "self": "https://jira.example.com/rest/api/3/issue/20002",
                        "fields": {
                            "summary": "Second async issue"
                        }
                    }
                ],
                "isLast": false,
                "nextPageToken": "async_token_2"
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Second page with nextPageToken
    let second_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=ASYNC".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "async_token_2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "20003",
                        "key": "ASYNC-3",
                        "self": "https://jira.example.com/rest/api/3/issue/20003",
                        "fields": {
                            "summary": "Third async issue"
                        }
                    },
                    {
                        "id": "20004",
                        "key": "ASYNC-4",
                        "self": "https://jira.example.com/rest/api/3/issue/20004",
                        "fields": {
                            "summary": "Fourth async issue"
                        }
                    }
                ],
                "isLast": false,
                "nextPageToken": "async_token_3"
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Third and final page
    let third_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=ASYNC".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "async_token_3".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "20005",
                        "key": "ASYNC-5",
                        "self": "https://jira.example.com/rest/api/3/issue/20005",
                        "fields": {
                            "summary": "Fifth async issue"
                        }
                    }
                ],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Create V3 async Jira client
    let jira =
        AsyncJira::with_search_api_version(url, Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();
    let search_options = SearchOptions::builder().max_results(2).build();

    // Test stream
    let search = jira.search();
    let mut stream = search
        .stream("project=ASYNC", &search_options)
        .await
        .unwrap();

    // Collect all issues
    let mut issues = Vec::new();
    while let Some(issue) = stream.next().await {
        issues.push(issue);
    }

    // Verify we got all 5 issues (async stream returns in forward order)
    assert_eq!(issues.len(), 5);
    assert_eq!(issues[0].key, "ASYNC-1");
    assert_eq!(issues[1].key, "ASYNC-2");
    assert_eq!(issues[2].key, "ASYNC-3");
    assert_eq!(issues[3].key, "ASYNC-4");
    assert_eq!(issues[4].key, "ASYNC-5");

    // Verify all mocks were called
    first_page_mock.assert_async().await;
    second_page_mock.assert_async().await;
    third_page_mock.assert_async().await;
}

// Test V3 with single page (isLast=true on first page)
#[test]
fn test_v3_sync_single_page() {
    let mut server = mockito::Server::new();
    let url = server.url();

    let single_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=SINGLE".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "30001",
                        "key": "SINGLE-1",
                        "self": "https://jira.example.com/rest/api/3/issue/30001",
                        "fields": {
                            "summary": "Only issue"
                        }
                    }
                ],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .expect(1) // Should only be called once
        .create();

    let jira =
        Jira::with_search_api_version(url, Credentials::Anonymous, SearchApiVersion::V3).unwrap();
    let search_options = SearchOptions::builder().max_results(10).build();

    let mut iter = jira
        .search()
        .iter("project=SINGLE", &search_options)
        .unwrap();

    // Should get exactly one issue
    assert_eq!(iter.next().unwrap().key, "SINGLE-1");
    assert!(iter.next().is_none());

    single_page_mock.assert();
}

// Test V3 with empty results
#[cfg(feature = "async")]
#[tokio::test]
async fn test_v3_async_empty_results() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let empty_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=EMPTY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;

    let jira =
        AsyncJira::with_search_api_version(url, Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();
    let search_options = SearchOptions::builder().max_results(10).build();

    let search = jira.search();
    let mut stream = search
        .stream("project=EMPTY", &search_options)
        .await
        .unwrap();

    // Should get no issues
    assert!(stream.next().await.is_none());

    empty_mock.assert_async().await;
}

// Test that V2 still uses startAt (regression test)
#[test]
fn test_v2_still_uses_start_at() {
    let mut server = mockito::Server::new();
    let url = server.url();

    // First page with V2 format (no nextPageToken)
    let first_page_mock = server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V2TEST".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 3,
                "issues": [
                    {
                        "id": "40001",
                        "key": "V2TEST-1",
                        "self": "https://jira.example.com/rest/api/latest/issue/40001",
                        "fields": {
                            "summary": "V2 First"
                        }
                    },
                    {
                        "id": "40002",
                        "key": "V2TEST-2",
                        "self": "https://jira.example.com/rest/api/latest/issue/40002",
                        "fields": {
                            "summary": "V2 Second"
                        }
                    }
                ]
            })
            .to_string(),
        )
        .create();

    // Second page using startAt
    let second_page_mock = server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V2TEST".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 2,
                "maxResults": 2,
                "total": 3,
                "issues": [
                    {
                        "id": "40003",
                        "key": "V2TEST-3",
                        "self": "https://jira.example.com/rest/api/latest/issue/40003",
                        "fields": {
                            "summary": "V2 Third"
                        }
                    }
                ]
            })
            .to_string(),
        )
        .create();

    // Use default (V2) Jira client
    let jira = Jira::new(url, Credentials::Anonymous).unwrap();
    let search_options = SearchOptions::builder().max_results(2).start_at(0).build();

    let iter = jira
        .search()
        .iter("project=V2TEST", &search_options)
        .unwrap();

    // Collect all issues
    let issues: Vec<_> = iter.collect();

    assert_eq!(issues.len(), 3);
    // Note: Iterator uses .pop() across pages, resulting in this specific order
    assert_eq!(issues[0].key, "V2TEST-2"); // Last from first page
    assert_eq!(issues[1].key, "V2TEST-1"); // First from first page
    assert_eq!(issues[2].key, "V2TEST-3"); // Last from second page

    first_page_mock.assert();
    second_page_mock.assert();
}

// Test error handling when nextPageToken is invalid
#[cfg(feature = "async")]
#[tokio::test]
async fn test_v3_async_invalid_token_stops_iteration() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    // First page returns a token
    let first_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=BADTOKEN".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    {
                        "id": "50001",
                        "key": "BADTOKEN-1",
                        "self": "https://jira.example.com/rest/api/3/issue/50001",
                        "fields": {
                            "summary": "First issue"
                        }
                    }
                ],
                "isLast": false,
                "nextPageToken": "bad_token"
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Second page returns an error
    let second_page_mock = server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=BADTOKEN".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "1".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "bad_token".into()),
        ]))
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorMessages": ["Invalid pagination token"],
                "errors": {}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let jira =
        AsyncJira::with_search_api_version(url, Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();
    let search_options = SearchOptions::builder().max_results(1).build();

    let search = jira.search();
    let mut stream = search
        .stream("project=BADTOKEN", &search_options)
        .await
        .unwrap();

    // Should get first issue
    assert_eq!(stream.next().await.unwrap().key, "BADTOKEN-1");

    // Should stop iteration after error
    assert!(stream.next().await.is_none());

    first_page_mock.assert_async().await;
    second_page_mock.assert_async().await;
}
