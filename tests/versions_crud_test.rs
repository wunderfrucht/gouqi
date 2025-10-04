//! Tests for complete Versions CRUD operations including new delete method

use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_version_delete() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/version/10000")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().delete("10000");

    assert!(result.is_ok());
}

#[test]
fn test_version_delete_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/version/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Version not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.versions().delete("99999");

    assert!(result.is_err());
}

#[cfg(feature = "async")]
mod async_versions_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_version_delete() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/version/10000")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.versions().delete("10000").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_project_versions() {
        let mut server = mockito::Server::new_async().await;

        let mock_versions = json!([
            {
                "id": "10000",
                "name": "1.0.0",
                "archived": false,
                "projectId": 12345,
                "released": true,
                "self": format!("{}/rest/api/latest/version/10000", server.url())
            }
        ]);

        server
            .mock("GET", "/rest/api/latest/project/TEST/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_versions.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.versions().project_versions("TEST").await;

        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 1);
    }

    #[tokio::test]
    async fn test_async_version_create() {
        let mut server = mockito::Server::new_async().await;

        let mock_version = json!({
            "id": "10000",
            "name": "2.0.0",
            "archived": false,
            "projectId": 12345,
            "released": false,
            "self": format!("{}/rest/api/latest/version/10000", server.url())
        });

        server
            .mock("POST", "/rest/api/latest/version")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(mock_version.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.versions().create(12345, "2.0.0").await;

        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.name, "2.0.0");
    }
}
