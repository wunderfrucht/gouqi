//! Tests for async Jira utility methods

#[cfg(feature = "async")]
mod async_jira_utilities {
    use gouqi::Credentials;
    use gouqi::r#async::Jira as AsyncJira;
    use serde_json::json;

    #[tokio::test]
    async fn test_async_session() {
        let mut server = mockito::Server::new_async().await;

        let mock_session = json!({
            "self": format!("{}/rest/auth/1/session", server.url()),
            "name": "JSESSIONID",
            "loginInfo": {
                "failedLoginCount": 0,
                "loginCount": 10,
                "lastFailedLoginTime": null,
                "previousLoginTime": "2024-01-01T10:00:00.000+0000"
            }
        });

        server
            .mock("GET", "/rest/auth/latest/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_session.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.session().await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.name, "JSESSIONID");
    }

    #[tokio::test]
    async fn test_async_session_unauthorized() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/auth/latest/session")
            .with_status(401)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.session().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_get_search_api_version() {
        let jira = AsyncJira::new("http://test.atlassian.net", Credentials::Anonymous).unwrap();
        let version = jira.get_search_api_version();

        // Cloud URLs default to V3
        assert_eq!(version, gouqi::core::SearchApiVersion::V3);
    }

    #[tokio::test]
    async fn test_async_jira_from_client() {
        let client = reqwest::Client::new();
        let result = AsyncJira::from_client("http://example.com", Credentials::Anonymous, client);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_jira_with_search_api_version() {
        let result = AsyncJira::with_search_api_version(
            "http://example.com",
            Credentials::Anonymous,
            gouqi::core::SearchApiVersion::V2,
        );

        assert!(result.is_ok());
        let jira = result.unwrap();
        assert_eq!(
            jira.get_search_api_version(),
            gouqi::core::SearchApiVersion::V2
        );
    }

    #[cfg(feature = "cache")]
    #[tokio::test]
    async fn test_async_clear_cache() {
        let jira = AsyncJira::new("http://example.com", Credentials::Anonymous).unwrap();
        jira.clear_cache();
        // Just verify it doesn't panic
    }

    #[cfg(feature = "cache")]
    #[tokio::test]
    async fn test_async_cache_stats() {
        let jira = AsyncJira::new("http://example.com", Credentials::Anonymous).unwrap();
        let stats = jira.cache_stats();

        // Stats should be accessible
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn test_async_observability_report() {
        let jira = AsyncJira::new("http://example.com", Credentials::Anonymous).unwrap();
        let report = jira.observability_report();

        // Report should be accessible
        assert!(report.timestamp > 0);
        // request_count is u64, always >= 0
    }

    #[cfg(feature = "metrics")]
    #[tokio::test]
    async fn test_async_health_status() {
        let jira = AsyncJira::new("http://example.com", Credentials::Anonymous).unwrap();
        let health = jira.health_status();

        // Health status should be accessible
        // uptime is u64, always >= 0
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_async_delete_method() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/test/123")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result: Result<gouqi::EmptyResponse, _> = jira.delete("api", "/test/123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_get_method() {
        let mut server = mockito::Server::new_async().await;

        let mock_data = json!({"id": "123", "name": "Test"});

        server
            .mock("GET", "/rest/api/latest/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result: Result<serde_json::Value, _> = jira.get("api", "/test").await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["id"], "123");
    }

    #[tokio::test]
    async fn test_async_post_method() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({"id": "456", "status": "created"});

        server
            .mock("POST", "/rest/api/latest/test")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let body = json!({"name": "New Item"});
        let result: Result<serde_json::Value, _> = jira.post("api", "/test", body).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["id"], "456");
    }

    #[tokio::test]
    async fn test_async_put_method() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({"id": "789", "status": "updated"});

        server
            .mock("PUT", "/rest/api/latest/test/789")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let body = json!({"name": "Updated Item"});
        let result: Result<serde_json::Value, _> = jira.put("api", "/test/789", body).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["status"], "updated");
    }

    #[tokio::test]
    async fn test_async_get_bytes() {
        let mut server = mockito::Server::new_async().await;

        let binary_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header

        server
            .mock("GET", "/rest/api/latest/content/download")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(binary_data.clone())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.get_bytes("api", "/content/download").await;

        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(bytes, binary_data);
    }

    #[tokio::test]
    async fn test_async_get_versioned_v2() {
        let mut server = mockito::Server::new_async().await;

        let mock_data = json!({"version": "v2", "data": "test"});

        server
            .mock("GET", "/rest/api/2/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result: Result<serde_json::Value, _> =
            jira.get_versioned("api", Some("2"), "/test").await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["version"], "v2");
    }

    #[tokio::test]
    async fn test_async_post_versioned_v3() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({"version": "v3", "created": true});

        server
            .mock("POST", "/rest/api/3/test")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let body = json!({"data": "test"});
        let result: Result<serde_json::Value, _> =
            jira.post_versioned("api", Some("3"), "/test", body).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["version"], "v3");
    }
}
