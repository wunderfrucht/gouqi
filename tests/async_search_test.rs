#[cfg(feature = "async")]
mod async_search_tests {
    // No extern crate needed in Rust 2024 edition

    // Import StreamExt for tests that use stream.next()
    #[allow(unused_imports)]
    use futures::stream::StreamExt;
    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::*;
    use serde_json::json;

    // Note: These tests focus on unit testing the API structures and functions
    // without requiring a mock server or real server connection.
    //
    // Mock server integration tests would need to be implemented as part of a
    // separate integration test suite that addresses the runtime conflicts between
    // mockito and tokio.

    #[test]
    fn test_async_jira_new() {
        let credentials = Credentials::Basic("user".to_string(), "pwd".to_string());
        let jira = AsyncJira::new("http://jira.com", credentials);
        assert!(jira.is_ok());
    }

    #[test]
    fn test_search_parameters() {
        // Test the search parameters
        let params = SearchOptions::default()
            .as_builder()
            .jql("project = TEST")
            .validate(true)
            .max_results(50)
            .start_at(10)
            .build();

        let serialized = params.serialize().unwrap();
        assert!(serialized.contains("jql=project+%3D+TEST"));
        assert!(serialized.contains("validateQuery=true"));
        assert!(serialized.contains("maxResults=50"));
        assert!(serialized.contains("startAt=10"));
    }

    #[tokio::test]
    async fn async_search_list() {
        // Use the async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create the mock using the async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/search?jql=project%3DTEST")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "startAt": 0,
                    "maxResults": 50,
                    "total": 1,
                    "expand": "names,schema",
                    "issues": [
                        {
                            "id": "10000",
                            "key": "TEST-1",
                            "self": "https://jira.example.com/rest/api/latest/issue/10000",
                            "fields": {
                                "summary": "Test issue",
                                "status": {
                                    "name": "Open",
                                    "id": "1",
                                    "description": "Open status",
                                    "iconUrl": "https://jira.example.com/icons/status_open.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/1"
                                }
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let search = jira.search();
        let result = search
            .list("project=TEST", &SearchOptions::default())
            .await
            .unwrap();

        // Verify results
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].key, "TEST-1");
        assert_eq!(result.issues[0].summary().unwrap(), "Test issue");

        // Use the async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_search_stream_one_page() {
        // Need to add the StreamExt trait for .next() on stream
        use futures::stream::StreamExt;

        // Use the async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create the mock using the async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/search?jql=project%3DTEST")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "startAt": 0,
                    "maxResults": 50,
                    "total": 1,
                    "expand": "names,schema",
                    "issues": [
                        {
                            "id": "10000",
                            "key": "TEST-1",
                            "self": "https://jira.example.com/rest/api/latest/issue/10000",
                            "fields": {
                                "summary": "Test issue",
                                "status": {
                                    "name": "Open",
                                    "id": "1",
                                    "description": "Open status",
                                    "iconUrl": "https://jira.example.com/icons/status_open.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/1"
                                }
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let search = jira.search();

        let search_options = SearchOptions::default();
        let mut stream = search
            .stream("project=TEST", &search_options)
            .await
            .unwrap();

        // Should be exactly one issue
        let issue = stream.next().await.unwrap();
        assert_eq!(issue.key, "TEST-1");

        // No more issues
        assert!(stream.next().await.is_none());

        // Use the async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_search_stream_multiple_pages() {
        // Need to add the StreamExt trait for .next() on stream
        use futures::stream::StreamExt;

        // Test actual multiple page streaming with V2 API
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // First page mock
        let first_page = server
            .mock("GET", "/rest/api/latest/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("jql".into(), "project=TEST".into()),
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
                    "expand": "names,schema",
                    "issues": [
                        {
                            "id": "10000",
                            "key": "TEST-1",
                            "self": "https://jira.example.com/rest/api/latest/issue/10000",
                            "fields": {
                                "summary": "Test issue 1",
                                "status": {
                                    "name": "Open",
                                    "id": "1",
                                    "description": "Open status",
                                    "iconUrl": "https://jira.example.com/icons/status_open.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/1"
                                }
                            }
                        },
                        {
                            "id": "10001",
                            "key": "TEST-2",
                            "self": "https://jira.example.com/rest/api/latest/issue/10001",
                            "fields": {
                                "summary": "Test issue 2",
                                "status": {
                                    "name": "In Progress",
                                    "id": "2",
                                    "description": "In Progress status",
                                    "iconUrl": "https://jira.example.com/icons/status_in_progress.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/2"
                                }
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Second page mock
        let second_page = server
            .mock("GET", "/rest/api/latest/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("jql".into(), "project=TEST".into()),
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
                    "expand": "names,schema",
                    "issues": [
                        {
                            "id": "10002",
                            "key": "TEST-3",
                            "self": "https://jira.example.com/rest/api/latest/issue/10002",
                            "fields": {
                                "summary": "Test issue 3",
                                "status": {
                                    "name": "Done",
                                    "id": "3",
                                    "description": "Done status",
                                    "iconUrl": "https://jira.example.com/icons/status_done.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/3"
                                }
                            }
                        },
                        {
                            "id": "10003",
                            "key": "TEST-4",
                            "self": "https://jira.example.com/rest/api/latest/issue/10003",
                            "fields": {
                                "summary": "Test issue 4",
                                "status": {
                                    "name": "Closed",
                                    "id": "4",
                                    "description": "Closed status",
                                    "iconUrl": "https://jira.example.com/icons/status_closed.png",
                                    "self": "https://jira.example.com/rest/api/latest/status/4"
                                }
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let search = jira.search();

        let options = SearchOptions::builder().max_results(2).start_at(0).build();
        let mut stream = search.stream("project=TEST", &options).await.unwrap();

        // Collect all issues in order
        let mut issues = Vec::new();
        while let Some(issue) = stream.next().await {
            issues.push(issue);
        }

        // Verify we got all 4 issues in correct order
        assert_eq!(issues.len(), 4);
        assert_eq!(issues[0].key, "TEST-1");
        assert_eq!(issues[1].key, "TEST-2");
        assert_eq!(issues[2].key, "TEST-3");
        assert_eq!(issues[3].key, "TEST-4");

        // Assert the mocks
        first_page.assert_async().await;
        second_page.assert_async().await;
    }

    #[tokio::test]
    async fn async_search_stream_empty() {
        // Need to add the StreamExt trait for .next() on stream
        use futures::stream::StreamExt;

        // Use the async version of Server::new
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create the mock using the async version of create
        let mock = server
            .mock("GET", "/rest/api/latest/search?jql=project%3DEMPTY")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "startAt": 0,
                    "maxResults": 50,
                    "total": 0,
                    "issues": [],
                    "expand": "names,schema"
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Run the test
        let jira = AsyncJira::new(url, Credentials::Anonymous).unwrap();
        let search = jira.search();

        let search_options = SearchOptions::default();
        let mut stream = search
            .stream("project=EMPTY", &search_options)
            .await
            .unwrap();

        // No issues
        assert!(stream.next().await.is_none());

        // Use the async version of assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn async_search_stream_v3_with_next_page_token() {
        // Test V3 async streaming with nextPageToken
        use futures::stream::StreamExt;

        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // First page with nextPageToken
        let first_page = server
            .mock("GET", "/rest/api/3/search/jql")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("jql".into(), "project=V3TEST".into()),
                mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "issues": [
                        {
                            "id": "30001",
                            "key": "V3TEST-1",
                            "self": "https://jira.example.com/rest/api/3/issue/30001",
                            "fields": {
                                "summary": "V3 First issue"
                            }
                        },
                        {
                            "id": "30002",
                            "key": "V3TEST-2",
                            "self": "https://jira.example.com/rest/api/3/issue/30002",
                            "fields": {
                                "summary": "V3 Second issue"
                            }
                        }
                    ],
                    "isLast": false,
                    "nextPageToken": "v3_token_2"
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Second page with nextPageToken
        let second_page = server
            .mock("GET", "/rest/api/3/search/jql")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("jql".into(), "project=V3TEST".into()),
                mockito::Matcher::UrlEncoded("maxResults".into(), "2".into()),
                mockito::Matcher::UrlEncoded("nextPageToken".into(), "v3_token_2".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "issues": [
                        {
                            "id": "30003",
                            "key": "V3TEST-3",
                            "self": "https://jira.example.com/rest/api/3/issue/30003",
                            "fields": {
                                "summary": "V3 Third issue"
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

        // Create V3 Jira client
        let jira = AsyncJira::with_search_api_version(
            url,
            Credentials::Anonymous,
            gouqi::core::SearchApiVersion::V3,
        )
        .unwrap();
        let search = jira.search();

        let options = SearchOptions::builder().max_results(2).build();
        let mut stream = search.stream("project=V3TEST", &options).await.unwrap();

        // Collect all issues
        let mut issues = Vec::new();
        while let Some(issue) = stream.next().await {
            issues.push(issue);
        }

        // Verify we got all 3 issues
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].key, "V3TEST-1");
        assert_eq!(issues[1].key, "V3TEST-2");
        assert_eq!(issues[2].key, "V3TEST-3");

        // Verify all mocks were called
        first_page.assert_async().await;
        second_page.assert_async().await;
    }
}
