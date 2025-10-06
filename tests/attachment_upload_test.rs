// Sync integration tests for attachment upload API

use gouqi::{Credentials, Jira};
use mockito::Server;
use serde_json::json;

#[test]
fn test_sync_attachment_upload_single_file() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let response_data = json!([{
        "self": format!("{}/rest/api/latest/attachment/12345", server.url()),
        "filename": "document.pdf",
        "author": {
            "active": true,
            "avatarUrls": {},
            "displayName": "Test User",
            "name": "testuser",
            "self": format!("{}/rest/api/latest/user?username=testuser", server.url())
        },
        "created": "2024-01-01T00:00:00.000+0000",
        "size": 1024,
        "mimeType": "application/pdf",
        "content": format!("{}/secure/attachment/12345/document.pdf", server.url())
    }]);

    let mock = server
        .mock("POST", "/rest/api/latest/issue/PROJ-123/attachments")
        .match_header("x-atlassian-token", "no-check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let file_content = vec![0x25, 0x50, 0x44, 0x46]; // PDF header
    let result = jira
        .issues()
        .upload_attachment("PROJ-123", vec![("document.pdf", file_content)]);

    mock.assert();
    assert!(result.is_ok(), "Upload should succeed: {:?}", result.err());

    let attachments = result.unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename, "document.pdf");
}

#[test]
fn test_sync_attachment_upload_multiple_files() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let response_data = json!([
        {
            "self": format!("{}/rest/api/latest/attachment/12345", server.url()),
            "filename": "doc1.pdf",
            "author": {
                "active": true,
                "avatarUrls": {},
                "displayName": "Test User",
                "name": "testuser",
                "self": format!("{}/rest/api/latest/user?username=testuser", server.url())
            },
            "created": "2024-01-01T00:00:00.000+0000",
            "size": 1024,
            "mimeType": "application/pdf",
            "content": format!("{}/secure/attachment/12345/doc1.pdf", server.url())
        },
        {
            "self": format!("{}/rest/api/latest/attachment/12346", server.url()),
            "filename": "image.png",
            "author": {
                "active": true,
                "avatarUrls": {},
                "displayName": "Test User",
                "name": "testuser",
                "self": format!("{}/rest/api/latest/user?username=testuser", server.url())
            },
            "created": "2024-01-01T00:00:01.000+0000",
            "size": 2048,
            "mimeType": "image/png",
            "content": format!("{}/secure/attachment/12346/image.png", server.url())
        }
    ]);

    let mock = server
        .mock("POST", "/rest/api/latest/issue/PROJ-456/attachments")
        .match_header("x-atlassian-token", "no-check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let pdf_content = vec![0x25, 0x50, 0x44, 0x46];
    let png_content = vec![0x89, 0x50, 0x4E, 0x47];

    let result = jira.issues().upload_attachment(
        "PROJ-456",
        vec![("doc1.pdf", pdf_content), ("image.png", png_content)],
    );

    mock.assert();
    assert!(result.is_ok());

    let attachments = result.unwrap();
    assert_eq!(attachments.len(), 2);
    assert_eq!(attachments[0].filename, "doc1.pdf");
    assert_eq!(attachments[1].filename, "image.png");
}

#[test]
fn test_sync_attachment_upload_unauthorized() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let mock = server
        .mock("POST", "/rest/api/latest/issue/PROJ-123/attachments")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Unauthorized"], "errors": {}}"#)
        .create();

    let result = jira
        .issues()
        .upload_attachment("PROJ-123", vec![("file.txt", vec![1, 2, 3])]);

    mock.assert();
    assert!(result.is_err(), "Should return error for 401 response");
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_attachment_upload_single_file() {
        let mut server = mockito::Server::new_async().await;
        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let response_data = json!([{
            "self": format!("{}/rest/api/latest/attachment/12345", server.url()),
            "filename": "document.pdf",
            "author": {
                "active": true,
                "avatarUrls": {},
                "displayName": "Test User",
                "name": "testuser",
                "self": format!("{}/rest/api/latest/user?username=testuser", server.url())
            },
            "created": "2024-01-01T00:00:00.000+0000",
            "size": 1024,
            "mimeType": "application/pdf",
            "content": format!("{}/secure/attachment/12345/document.pdf", server.url())
        }]);

        let mock = server
            .mock("POST", "/rest/api/latest/issue/PROJ-123/attachments")
            .match_header("x-atlassian-token", "no-check")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let file_content = vec![0x25, 0x50, 0x44, 0x46];
        let result = jira
            .issues()
            .upload_attachment("PROJ-123", vec![("document.pdf", file_content)])
            .await;

        mock.assert_async().await;
        assert!(result.is_ok(), "Upload should succeed: {:?}", result.err());

        let attachments = result.unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "document.pdf");
    }

    #[tokio::test]
    async fn test_async_attachment_upload_with_basic_auth() {
        let mut server = mockito::Server::new_async().await;
        let jira = AsyncJira::new(
            server.url(),
            Credentials::Basic("user".to_string(), "pass".to_string()),
        )
        .unwrap();

        let response_data = json!([{
            "self": format!("{}/rest/api/latest/attachment/99999", server.url()),
            "filename": "secure.txt",
            "author": {
                "active": true,
                "avatarUrls": {},
                "displayName": "Authorized User",
                "name": "authuser",
                "self": format!("{}/rest/api/latest/user?username=authuser", server.url())
            },
            "created": "2024-01-01T00:00:00.000+0000",
            "size": 512,
            "mimeType": "text/plain",
            "content": format!("{}/secure/attachment/99999/secure.txt", server.url())
        }]);

        let mock = server
            .mock("POST", "/rest/api/latest/issue/SEC-1/attachments")
            .match_header("x-atlassian-token", "no-check")
            .match_header("authorization", "Basic dXNlcjpwYXNz") // base64(user:pass)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let content = b"Secret content".to_vec();
        let result = jira
            .issues()
            .upload_attachment("SEC-1", vec![("secure.txt", content)])
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].filename, "secure.txt");
    }
}
