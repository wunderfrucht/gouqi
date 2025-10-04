//! Tests for complete Components CRUD operations including new delete method

use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_component_delete() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/component/10000")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().delete("10000");

    assert!(result.is_ok());
}

#[test]
fn test_component_delete_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/component/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Component not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().delete("99999");

    assert!(result.is_err());
}

#[test]
fn test_component_delete_unauthorized() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/component/10000")
        .with_status(401)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().delete("10000");

    assert!(result.is_err());
}

#[cfg(feature = "async")]
mod async_components_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_component_get() {
        let mut server = mockito::Server::new_async().await;

        let mock_component = json!({
            "id": "10000",
            "name": "Backend",
            "description": "Backend component",
            "self": format!("{}/rest/api/latest/component/10000", server.url())
        });

        server
            .mock("GET", "/rest/api/latest/component/10000")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_component.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.components().get("10000").await;

        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.name, "Backend");
    }

    #[tokio::test]
    async fn test_async_component_delete() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/component/10000")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.components().delete("10000").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_component_list() {
        let mut server = mockito::Server::new_async().await;

        let mock_components = json!([
            {
                "id": "10000",
                "name": "Backend",
                "self": format!("{}/rest/api/latest/component/10000", server.url())
            },
            {
                "id": "10001",
                "name": "Frontend",
                "self": format!("{}/rest/api/latest/component/10001", server.url())
            }
        ]);

        server
            .mock("GET", "/rest/api/latest/project/TEST/components")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_components.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.components().list("TEST").await;

        assert!(result.is_ok());
        let components = result.unwrap();
        assert_eq!(components.len(), 2);
    }
}
