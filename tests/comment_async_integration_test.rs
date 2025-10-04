// Async integration tests for comment API

#[cfg(feature = "async")]
mod async_tests {
    use gouqi::{AddComment, Credentials, r#async::Jira};
    use mockito::Server;

    #[tokio::test]
    async fn test_async_comment_v2_server_routing() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock V2 comment endpoint response
        let mock = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/comment")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "body": "Async test comment"
            })))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "12345",
                "self": "https://example.com/rest/api/2/issue/TEST-123/comment/12345",
                "author": {
                    "active": true,
                    "name": "testuser",
                    "displayName": "Test User",
                    "self": "https://example.com/rest/api/2/user?username=testuser"
                },
                "body": "Async test comment",
                "created": "2024-01-01T10:00:00.000+0000",
                "updated": "2024-01-01T10:00:00.000+0000"
            }"#,
            )
            .create();

        // Call async comment method
        let comment_data = AddComment::new("Async test comment");
        let result = jira.issues().comment("TEST-123", comment_data).await;

        mock.assert_async().await;
        assert!(
            result.is_ok(),
            "Async comment should succeed: {:?}",
            result.err()
        );

        let comment = result.unwrap();
        assert_eq!(comment.id, Some("12345".to_string()));
    }

    #[tokio::test]
    async fn test_async_comment_with_visibility() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        use gouqi::Visibility;
        let visibility = Visibility {
            visibility_type: "role".to_string(),
            value: "Administrators".to_string(),
        };

        // Mock V2 comment endpoint with visibility
        let mock = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/comment")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "body": "Private async comment",
                "visibility": {
                    "type": "role",
                    "value": "Administrators"
                }
            })))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "12346",
                "self": "https://example.com/rest/api/2/issue/TEST-123/comment/12346",
                "author": {
                    "active": true,
                    "name": "testuser",
                    "displayName": "Test User",
                    "self": "https://example.com/rest/api/2/user?username=testuser"
                },
                "body": "Private async comment",
                "created": "2024-01-01T10:00:00.000+0000",
                "updated": "2024-01-01T10:00:00.000+0000",
                "visibility": {
                    "type": "role",
                    "value": "Administrators"
                }
            }"#,
            )
            .create();

        let comment_data = AddComment::new("Private async comment").with_visibility(visibility);
        let result = jira.issues().comment("TEST-123", comment_data).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_post_versioned_method() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Test the async post_versioned method directly
        let mock = server
            .mock("POST", "/rest/api/3/test/endpoint")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "test": "async data"
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"result": "async success"}"#)
            .create();

        use serde::{Deserialize, Serialize};

        #[derive(Serialize)]
        struct TestRequest {
            test: String,
        }

        #[derive(Deserialize, Debug)]
        struct TestResponse {
            result: String,
        }

        let result: Result<TestResponse, _> = jira
            .post_versioned(
                "api",
                Some("3"),
                "/test/endpoint",
                TestRequest {
                    test: "async data".to_string(),
                },
            )
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().result, "async success");
    }

    #[tokio::test]
    async fn test_async_comment_error_handling() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock error response
        let mock = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/comment")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "errorMessages": ["Comment body is required"],
                "errors": {}
            }"#,
            )
            .create();

        let comment_data = AddComment::new("Test comment");
        let result = jira.issues().comment("TEST-123", comment_data).await;

        mock.assert_async().await;
        assert!(result.is_err(), "Should return error for 400 response");
    }
}
