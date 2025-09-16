//! Tests to ensure sync and async search implementations behave identically
//! This prevents regressions like issue #112 where async and sync diverged

use gouqi::core::SearchApiVersion;
use gouqi::{Credentials, Issue, Jira, SearchOptions};
use serde_json::json;

#[cfg(feature = "async")]
use futures::stream::StreamExt;
#[cfg(feature = "async")]
use gouqi::r#async::Jira as AsyncJira;

// Helper function to create test issue JSON
fn create_test_issue(id: &str, key: &str, summary: &str) -> serde_json::Value {
    json!({
        "id": id,
        "key": key,
        "self": format!("https://jira.example.com/rest/api/latest/issue/{}", id),
        "fields": {
            "summary": summary,
            "status": {
                "name": "Open",
                "id": "1",
                "description": "Open status",
                "iconUrl": "https://jira.example.com/icons/status_open.png",
                "self": "https://jira.example.com/rest/api/latest/status/1"
            }
        }
    })
}

// Test V2 sync vs async pagination with startAt
#[cfg(feature = "async")]
#[tokio::test]
async fn test_v2_sync_async_parity() {
    // Set up sync server
    let mut sync_server = mockito::Server::new();
    let sync_url = sync_server.url();

    // Set up async server
    let mut async_server = mockito::Server::new_async().await;
    let async_url = async_server.url();

    // First page for sync
    let sync_first_page = sync_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 4,
                "issues": [
                    create_test_issue("10001", "PARITY-1", "First issue"),
                    create_test_issue("10002", "PARITY-2", "Second issue")
                ]
            })
            .to_string(),
        )
        .create();

    // Second page for sync
    let sync_second_page = sync_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 2,
                "maxResults": 2,
                "total": 4,
                "issues": [
                    create_test_issue("10003", "PARITY-3", "Third issue"),
                    create_test_issue("10004", "PARITY-4", "Fourth issue")
                ]
            })
            .to_string(),
        )
        .create();

    // First page for async
    let async_first_page = async_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 4,
                "issues": [
                    create_test_issue("10001", "PARITY-1", "First issue"),
                    create_test_issue("10002", "PARITY-2", "Second issue")
                ]
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Second page for async
    let async_second_page = async_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("startAt".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 2,
                "maxResults": 2,
                "total": 4,
                "issues": [
                    create_test_issue("10003", "PARITY-3", "Third issue"),
                    create_test_issue("10004", "PARITY-4", "Fourth issue")
                ]
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Test sync
    let sync_jira = Jira::new(sync_url, Credentials::Anonymous).unwrap();
    let sync_options = SearchOptions::builder().max_results(2).start_at(0).build();
    let sync_iter = sync_jira
        .search()
        .iter("project=PARITY", &sync_options)
        .unwrap();
    let sync_issues: Vec<Issue> = sync_iter.collect();

    // Test async
    let async_jira = AsyncJira::new(async_url, Credentials::Anonymous).unwrap();
    let async_options = SearchOptions::builder().max_results(2).start_at(0).build();
    let search = async_jira.search();
    let mut async_stream = search
        .stream("project=PARITY", &async_options)
        .await
        .unwrap();

    let mut async_issues = Vec::new();
    while let Some(issue) = async_stream.next().await {
        async_issues.push(issue);
    }

    // Verify both got the same results
    assert_eq!(sync_issues.len(), async_issues.len());
    assert_eq!(sync_issues.len(), 4);

    for (sync_issue, async_issue) in sync_issues.iter().zip(async_issues.iter()) {
        assert_eq!(sync_issue.key, async_issue.key);
        assert_eq!(sync_issue.summary(), async_issue.summary());
    }

    // Verify mock calls
    sync_first_page.assert();
    sync_second_page.assert();
    async_first_page.assert_async().await;
    async_second_page.assert_async().await;
}

// Test V3 sync vs async pagination with nextPageToken
#[cfg(feature = "async")]
#[tokio::test]
async fn test_v3_sync_async_parity() {
    // Set up sync server
    let mut sync_server = mockito::Server::new();
    let sync_url = sync_server.url();

    // Set up async server
    let mut async_server = mockito::Server::new_async().await;
    let async_url = async_server.url();

    // Create V3 issue format
    fn create_v3_issue(id: &str, key: &str, summary: &str) -> serde_json::Value {
        json!({
            "id": id,
            "key": key,
            "self": format!("https://jira.example.com/rest/api/3/issue/{}", id),
            "fields": {
                "summary": summary
            }
        })
    }

    // First page for sync
    let sync_first_page = sync_server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V3PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    create_v3_issue("20001", "V3PARITY-1", "V3 First issue"),
                    create_v3_issue("20002", "V3PARITY-2", "V3 Second issue")
                ],
                "isLast": false,
                "nextPageToken": "sync_token_2"
            })
            .to_string(),
        )
        .create();

    // Second page for sync
    let sync_second_page = sync_server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V3PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "sync_token_2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    create_v3_issue("20003", "V3PARITY-3", "V3 Third issue")
                ],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .create();

    // First page for async
    let async_first_page = async_server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V3PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    create_v3_issue("20001", "V3PARITY-1", "V3 First issue"),
                    create_v3_issue("20002", "V3PARITY-2", "V3 Second issue")
                ],
                "isLast": false,
                "nextPageToken": "async_token_2"
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Second page for async
    let async_second_page = async_server
        .mock("GET", "/rest/api/3/search/jql")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".into(), "project=V3PARITY".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            mockito::Matcher::UrlEncoded("nextPageToken".into(), "async_token_2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "issues": [
                    create_v3_issue("20003", "V3PARITY-3", "V3 Third issue")
                ],
                "isLast": true,
                "nextPageToken": null
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Test sync V3
    let sync_jira =
        Jira::with_search_api_version(sync_url, Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();
    let sync_options = SearchOptions::builder().max_results(2).build();
    let sync_iter = sync_jira
        .search()
        .iter("project=V3PARITY", &sync_options)
        .unwrap();
    let sync_issues: Vec<Issue> = sync_iter.collect();

    // Test async V3
    let async_jira =
        AsyncJira::with_search_api_version(async_url, Credentials::Anonymous, SearchApiVersion::V3)
            .unwrap();
    let async_options = SearchOptions::builder().max_results(2).build();
    let search = async_jira.search();
    let mut async_stream = search
        .stream("project=V3PARITY", &async_options)
        .await
        .unwrap();

    let mut async_issues = Vec::new();
    while let Some(issue) = async_stream.next().await {
        async_issues.push(issue);
    }

    // Verify both got the same results
    assert_eq!(sync_issues.len(), async_issues.len());
    assert_eq!(sync_issues.len(), 3);

    for (sync_issue, async_issue) in sync_issues.iter().zip(async_issues.iter()) {
        assert_eq!(sync_issue.key, async_issue.key);
        assert_eq!(sync_issue.summary(), async_issue.summary());
    }

    // Verify results are in correct order
    assert_eq!(sync_issues[0].key, "V3PARITY-1");
    assert_eq!(sync_issues[1].key, "V3PARITY-2");
    assert_eq!(sync_issues[2].key, "V3PARITY-3");

    assert_eq!(async_issues[0].key, "V3PARITY-1");
    assert_eq!(async_issues[1].key, "V3PARITY-2");
    assert_eq!(async_issues[2].key, "V3PARITY-3");

    // Verify mock calls
    sync_first_page.assert();
    sync_second_page.assert();
    async_first_page.assert_async().await;
    async_second_page.assert_async().await;
}

// Test empty results parity
#[cfg(feature = "async")]
#[tokio::test]
async fn test_empty_results_parity() {
    // Set up sync server
    let mut sync_server = mockito::Server::new();
    let sync_url = sync_server.url();

    // Set up async server
    let mut async_server = mockito::Server::new_async().await;
    let async_url = async_server.url();

    // Empty sync response
    let sync_empty = sync_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "jql".into(),
            "project=EMPTY".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 50,
                "total": 0,
                "issues": []
            })
            .to_string(),
        )
        .create();

    // Empty async response
    let async_empty = async_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "jql".into(),
            "project=EMPTY".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 50,
                "total": 0,
                "issues": []
            })
            .to_string(),
        )
        .create_async()
        .await;

    // Test sync
    let sync_jira = Jira::new(sync_url, Credentials::Anonymous).unwrap();
    let default_options = SearchOptions::default();
    let sync_iter = sync_jira
        .search()
        .iter("project=EMPTY", &default_options)
        .unwrap();
    let sync_issues: Vec<Issue> = sync_iter.collect();

    // Test async
    let async_jira = AsyncJira::new(async_url, Credentials::Anonymous).unwrap();
    let default_options = SearchOptions::default();
    let search = async_jira.search();
    let mut async_stream = search
        .stream("project=EMPTY", &default_options)
        .await
        .unwrap();

    let mut async_issues = Vec::new();
    while let Some(issue) = async_stream.next().await {
        async_issues.push(issue);
    }

    // Both should be empty
    assert_eq!(sync_issues.len(), 0);
    assert_eq!(async_issues.len(), 0);

    sync_empty.assert();
    async_empty.assert_async().await;
}

// Test single page parity
#[cfg(feature = "async")]
#[tokio::test]
async fn test_single_page_parity() {
    // Set up servers
    let mut sync_server = mockito::Server::new();
    let sync_url = sync_server.url();

    let mut async_server = mockito::Server::new_async().await;
    let async_url = async_server.url();

    // Single page response
    let sync_single = sync_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "jql".into(),
            "project=SINGLE".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 50,
                "total": 1,
                "issues": [
                    create_test_issue("30001", "SINGLE-1", "Only issue")
                ]
            })
            .to_string(),
        )
        .expect(1) // Should only be called once
        .create();

    let async_single = async_server
        .mock("GET", "/rest/api/latest/search")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "jql".into(),
            "project=SINGLE".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "startAt": 0,
                "maxResults": 50,
                "total": 1,
                "issues": [
                    create_test_issue("30001", "SINGLE-1", "Only issue")
                ]
            })
            .to_string(),
        )
        .expect(1) // Should only be called once
        .create_async()
        .await;

    // Test sync
    let sync_jira = Jira::new(sync_url, Credentials::Anonymous).unwrap();
    let default_options = SearchOptions::default();
    let sync_iter = sync_jira
        .search()
        .iter("project=SINGLE", &default_options)
        .unwrap();
    let sync_issues: Vec<Issue> = sync_iter.collect();

    // Test async
    let async_jira = AsyncJira::new(async_url, Credentials::Anonymous).unwrap();
    let default_options = SearchOptions::default();
    let search = async_jira.search();
    let mut async_stream = search
        .stream("project=SINGLE", &default_options)
        .await
        .unwrap();

    let mut async_issues = Vec::new();
    while let Some(issue) = async_stream.next().await {
        async_issues.push(issue);
    }

    // Both should have exactly one issue
    assert_eq!(sync_issues.len(), 1);
    assert_eq!(async_issues.len(), 1);
    assert_eq!(sync_issues[0].key, async_issues[0].key);
    assert_eq!(sync_issues[0].key, "SINGLE-1");

    sync_single.assert();
    async_single.assert_async().await;
}
