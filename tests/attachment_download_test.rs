// Sync integration tests for attachment download API

use gouqi::{Credentials, Jira};
use mockito::Server;

#[test]
fn test_sync_attachment_download_success() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Mock attachment content endpoint
    let attachment_content = b"This is a test PDF file content";
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/12345")
        .with_status(200)
        .with_header("content-type", "application/pdf")
        .with_body(attachment_content)
        .create();

    let result = jira.attachments().download("12345");

    mock.assert();
    assert!(
        result.is_ok(),
        "Download should succeed: {:?}",
        result.err()
    );

    let content = result.unwrap();
    assert_eq!(content, attachment_content.to_vec());
}

#[test]
fn test_sync_attachment_download_image() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Mock image attachment
    let image_content = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/67890")
        .with_status(200)
        .with_header("content-type", "image/jpeg")
        .with_body(&image_content)
        .create();

    let result = jira.attachments().download("67890");

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), image_content);
}

#[test]
fn test_sync_attachment_download_not_found() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Mock 404 response
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Attachment not found"], "errors": {}}"#)
        .create();

    let result = jira.attachments().download("99999");

    mock.assert();
    assert!(result.is_err(), "Should return error for 404 response");
}

#[test]
fn test_sync_attachment_download_unauthorized() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Mock 401 response
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/12345")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages": ["Unauthorized"], "errors": {}}"#)
        .create();

    let result = jira.attachments().download("12345");

    mock.assert();
    assert!(result.is_err(), "Should return error for 401 response");
}

#[test]
fn test_sync_attachment_download_with_basic_auth() {
    let mut server = Server::new();
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

    let result = jira.attachments().download("12345");

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), attachment_content.to_vec());
}

#[test]
fn test_sync_attachment_download_large_file() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Create a larger mock file (1MB)
    let large_content = vec![0x42; 1024 * 1024];
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/large")
        .with_status(200)
        .with_header("content-type", "application/zip")
        .with_body(&large_content)
        .create();

    let result = jira.attachments().download("large");

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1024 * 1024);
}

#[test]
fn test_sync_attachment_download_empty_file() {
    let mut server = Server::new();
    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    // Mock empty attachment
    let mock = server
        .mock("GET", "/rest/api/latest/attachment/content/empty")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("")
        .create();

    let result = jira.attachments().download("empty");

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}
