//! Tests for complete Sprints CRUD operations including new update and delete methods

use gouqi::{Credentials, Jira, UpdateSprint};
use serde_json::json;

#[test]
fn test_sprint_update() {
    let mut server = mockito::Server::new();

    let mock_sprint = json!({
        "id": 1,
        "name": "Updated Sprint",
        "state": "active",
        "self": format!("{}/rest/agile/latest/sprint/1", server.url())
    });

    server
        .mock("POST", "/rest/agile/latest/sprint/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprint.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let update = UpdateSprint {
        name: Some("Updated Sprint".to_string()),
        start_date: None,
        end_date: None,
        state: Some("active".to_string()),
    };

    let result = jira.sprints().update(1u64, update);

    assert!(result.is_ok());
    let sprint = result.unwrap();
    assert_eq!(sprint.name, "Updated Sprint");
}

#[test]
fn test_sprint_delete() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/agile/latest/sprint/1")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.sprints().delete(1u64);

    assert!(result.is_ok());
}

#[test]
fn test_sprint_delete_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/agile/latest/sprint/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Sprint not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.sprints().delete(99999u64);

    assert!(result.is_err());
}

#[test]
fn test_sprint_update_state() {
    let mut server = mockito::Server::new();

    let mock_sprint = json!({
        "id": 1,
        "name": "Sprint 1",
        "state": "closed",
        "self": format!("{}/rest/agile/latest/sprint/1", server.url())
    });

    server
        .mock("POST", "/rest/agile/latest/sprint/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_sprint.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let update = UpdateSprint {
        name: None,
        start_date: None,
        end_date: None,
        state: Some("closed".to_string()),
    };

    let result = jira.sprints().update(1u64, update);

    assert!(result.is_ok());
}

#[cfg(feature = "async")]
mod async_sprints_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_sprint_update() {
        let mut server = mockito::Server::new_async().await;

        let mock_sprint = json!({
            "id": 1,
            "name": "Updated Sprint",
            "state": "active",
            "self": format!("{}/rest/agile/latest/sprint/1", server.url())
        });

        server
            .mock("POST", "/rest/agile/latest/sprint/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_sprint.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let update = UpdateSprint {
            name: Some("Updated Sprint".to_string()),
            start_date: None,
            end_date: None,
            state: Some("active".to_string()),
        };

        let result = jira.sprints().update(1u64, update).await;

        assert!(result.is_ok());
        let sprint = result.unwrap();
        assert_eq!(sprint.name, "Updated Sprint");
    }

    #[tokio::test]
    async fn test_async_sprint_delete() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/agile/latest/sprint/1")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.sprints().delete(1u64).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_sprint_get() {
        let mut server = mockito::Server::new_async().await;

        let mock_sprint = json!({
            "id": 1,
            "name": "Sprint 1",
            "state": "active",
            "self": format!("{}/rest/agile/latest/sprint/1", server.url())
        });

        server
            .mock("GET", "/rest/agile/latest/sprint/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_sprint.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.sprints().get("1").await;

        assert!(result.is_ok());
        let sprint = result.unwrap();
        assert_eq!(sprint.name, "Sprint 1");
    }
}
