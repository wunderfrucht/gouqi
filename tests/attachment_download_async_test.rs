// Async integration tests for attachment download API

#[cfg(feature = "async")]
mod async_tests {
    use gouqi::{Credentials, r#async::Jira};
    use mockito::Server;

    #[tokio::test]
    async fn test_async_attachment_download_success() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock attachment content endpoint
        let attachment_content = b"This is a test PDF file content";
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/12345")
            .with_status(200)
            .with_header("content-type", "application/pdf")
            .with_body(attachment_content)
            .create();

        let result = jira.attachments().download("12345").await;

        mock.assert_async().await;
        assert!(
            result.is_ok(),
            "Download should succeed: {:?}",
            result.err()
        );

        let content = result.unwrap();
        assert_eq!(content, attachment_content.to_vec());
    }

    #[tokio::test]
    async fn test_async_attachment_download_image() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock image attachment
        let image_content = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/67890")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(&image_content)
            .create();

        let result = jira.attachments().download("67890").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), image_content);
    }

    #[tokio::test]
    async fn test_async_attachment_download_not_found() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock 404 response
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/99999")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errorMessages": ["Attachment not found"], "errors": {}}"#)
            .create();

        let result = jira.attachments().download("99999").await;

        mock.assert_async().await;
        assert!(result.is_err(), "Should return error for 404 response");
    }

    #[tokio::test]
    async fn test_async_attachment_download_unauthorized() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock 401 response
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/12345")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errorMessages": ["Unauthorized"], "errors": {}}"#)
            .create();

        let result = jira.attachments().download("12345").await;

        mock.assert_async().await;
        assert!(result.is_err(), "Should return error for 401 response");
    }

    #[tokio::test]
    async fn test_async_attachment_download_with_basic_auth() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(
            server.url(),
            Credentials::Basic("user".to_string(), "pass".to_string()),
        )
        .unwrap();

        let attachment_content = b"Authenticated content";
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/12345")
            .match_header("authorization", "Basic dXNlcjpwYXNz") // base64(user:pass)
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(attachment_content)
            .create();

        let result = jira.attachments().download("12345").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), attachment_content.to_vec());
    }

    #[tokio::test]
    async fn test_async_attachment_download_large_file() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Create a larger mock file (1MB)
        let large_content = vec![0x42; 1024 * 1024];
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/large")
            .with_status(200)
            .with_header("content-type", "application/zip")
            .with_body(&large_content)
            .create();

        let result = jira.attachments().download("large").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1024 * 1024);
    }

    #[tokio::test]
    async fn test_async_attachment_download_empty_file() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Mock empty attachment
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/content/empty")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("")
            .create();

        let result = jira.attachments().download("empty").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_async_attachment_get_metadata() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Test the existing get() method for completeness
        let mock = server
            .mock("GET", "/rest/api/latest/attachment/12345")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "self": "https://example.com/rest/api/2/attachment/12345",
                "filename": "test.pdf",
                "author": {
                    "active": true,
                    "avatarUrls": {},
                    "displayName": "Test User",
                    "name": "testuser",
                    "self": "https://example.com/rest/api/2/user?username=testuser"
                },
                "created": "2024-01-01T10:00:00.000+0000",
                "size": 1024,
                "mimeType": "application/pdf",
                "content": "https://example.com/secure/attachment/12345/test.pdf",
                "thumbnail": "https://example.com/secure/thumbnail/12345"
            }"#,
            )
            .create();

        let result = jira.attachments().get("12345").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.filename, "test.pdf");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.mime_type, "application/pdf");
    }

    #[tokio::test]
    async fn test_async_attachment_delete() {
        let mut server = Server::new_async().await;
        let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

        // Test the existing delete() method for completeness
        let mock = server
            .mock("DELETE", "/rest/api/latest/attachment/12345")
            .with_status(204)
            .create();

        let result = jira.attachments().delete("12345").await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }
}
