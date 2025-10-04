//! Additional tests to improve coverage for search functionality

use gouqi::{Credentials, Jira, SearchOptions};
use serde_json::json;

#[test]
fn test_search_with_empty_results() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 0,
        "issues": []
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.search().list("", &SearchOptions::default());

    assert!(result.is_ok());
    let search_results = result.unwrap();
    assert_eq!(search_results.total, 0);
    assert!(search_results.issues.is_empty());
}

#[test]
fn test_search_with_pagination_options() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 10,
        "maxResults": 5,
        "total": 100,
        "issues": []
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*startAt=10.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = SearchOptions::default()
        .as_builder()
        .start_at(10)
        .max_results(5)
        .build();

    let result = jira.search().list("project = TEST", &options);

    assert!(result.is_ok());
    let search_results = result.unwrap();
    assert_eq!(search_results.start_at, 10);
    assert_eq!(search_results.max_results, 5);
    assert_eq!(search_results.total, 100);
}

#[test]
fn test_search_error_invalid_jql() {
    let mut server = mockito::Server::new();

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*".to_string()),
        )
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Invalid JQL query"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira
        .search()
        .list("INVALID SYNTAX", &SearchOptions::default());

    assert!(result.is_err());
}

#[cfg(feature = "async")]
mod async_search_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_search_list() {
        let mut server = mockito::Server::new_async().await;

        let mock_results = json!({
            "startAt": 0,
            "maxResults": 50,
            "total": 1,
            "issues": [
                {
                    "id": "10001",
                    "key": "ASYNC-1",
                    "self": format!("{}/rest/api/latest/issue/10001", server.url()),
                    "fields": {}
                }
            ]
        });

        server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/rest/api/latest/search\?.*".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_results.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .search()
            .list("project = ASYNC", &SearchOptions::default())
            .await;

        assert!(result.is_ok());
        let search_results = result.unwrap();
        assert_eq!(search_results.total, 1);
        assert_eq!(search_results.issues[0].key, "ASYNC-1");
    }
}
