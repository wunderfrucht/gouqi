#[cfg(feature = "async")]
mod async_methods_tests {
    use gouqi::r#async::Jira;
    use gouqi::{Credentials, Result};

    #[tokio::test]
    async fn test_search_method() -> Result<()> {
        let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
        let _search = jira.search();

        // Just verify we can create an AsyncSearch instance without panic
        Ok(())
    }

    #[tokio::test]
    async fn test_issues_method() -> Result<()> {
        let jira = Jira::new("http://example.com", Credentials::Anonymous)?;
        let _issues = jira.issues();

        // Just verify we can create an AsyncIssues instance without panic
        Ok(())
    }

    // We can't easily test session without mocking, so we'll skip a real test here

    #[tokio::test]
    async fn test_from_client_method() -> Result<()> {
        let client = reqwest::Client::new();
        let _jira = Jira::from_client("http://example.com", Credentials::Anonymous, client)?;

        // Just verify we can create a Jira instance with a custom client
        Ok(())
    }

    #[tokio::test]
    async fn test_http_methods() -> Result<()> {
        // Use a local non-existent URL that will fail quickly instead of potentially timing out
        let jira = Jira::new("http://localhost:99999", Credentials::Anonymous)?;

        // Test method presence - these will error quickly due to connection refused,
        // but we're just checking that the methods exist and are accessible
        let get_result = jira.get::<serde_json::Value>("api", "/endpoint").await;
        assert!(get_result.is_err());

        let post_result = jira
            .post::<serde_json::Value, serde_json::Value>("api", "/endpoint", serde_json::json!({}))
            .await;
        assert!(post_result.is_err());

        let put_result = jira
            .put::<serde_json::Value, serde_json::Value>("api", "/endpoint", serde_json::json!({}))
            .await;
        assert!(put_result.is_err());

        let delete_result = jira.delete::<serde_json::Value>("api", "/endpoint").await;
        assert!(delete_result.is_err());

        Ok(())
    }
}
