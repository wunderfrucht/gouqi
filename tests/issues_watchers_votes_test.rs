//! Tests for issue watchers and votes functionality

use gouqi::{Credentials, Jira};
use serde_json::json;

#[test]
fn test_assign_issue() {
    let mut server = mockito::Server::new();

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1/assignee")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().assign("TEST-1", Some("john.doe".to_string()));

    assert!(result.is_ok());
}

#[test]
fn test_assign_issue_to_none() {
    let mut server = mockito::Server::new();

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1/assignee")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().assign("TEST-1", None);

    assert!(result.is_ok());
}

#[test]
fn test_get_watchers_success() {
    let mut server = mockito::Server::new();

    let mock_watchers = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/watchers", server.url()),
        "isWatching": true,
        "watchCount": 2,
        "watchers": [
            {
                "self": format!("{}/rest/api/latest/user?username=user1", server.url()),
                "accountId": "user1",
                "displayName": "User One",
                "active": true
            },
            {
                "self": format!("{}/rest/api/latest/user?username=user2", server.url()),
                "accountId": "user2",
                "displayName": "User Two",
                "active": true
            }
        ]
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-1/watchers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_watchers.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get_watchers("TEST-1");

    assert!(result.is_ok());
    let watchers = result.unwrap();
    assert_eq!(watchers.watch_count, 2);
    assert!(watchers.is_watching);
}

#[test]
fn test_get_watchers_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/issue/INVALID-1/watchers")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Issue not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get_watchers("INVALID-1");

    assert!(result.is_err());
}

#[test]
fn test_add_watcher_success() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/watchers")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().add_watcher("TEST-1", "john.doe".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_add_watcher_unauthorized() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/watchers")
        .with_status(401)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().add_watcher("TEST-1", "john.doe".to_string());

    assert!(result.is_err());
}

#[test]
fn test_remove_watcher_success() {
    let mut server = mockito::Server::new();

    server
        .mock(
            "DELETE",
            "/rest/api/latest/issue/TEST-1/watchers?username=john.doe",
        )
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira
        .issues()
        .remove_watcher("TEST-1", "john.doe".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_remove_watcher_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock(
            "DELETE",
            "/rest/api/latest/issue/TEST-1/watchers?username=unknown",
        )
        .with_status(404)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira
        .issues()
        .remove_watcher("TEST-1", "unknown".to_string());

    assert!(result.is_err());
}

#[test]
fn test_vote_success() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/votes")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().vote("TEST-1");

    assert!(result.is_ok());
}

#[test]
fn test_vote_already_voted() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/votes")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["User has already voted"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().vote("TEST-1");

    assert!(result.is_err());
}

#[test]
fn test_unvote_success() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/issue/TEST-1/votes")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().unvote("TEST-1");

    assert!(result.is_ok());
}

#[test]
fn test_unvote_not_voted() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/issue/TEST-1/votes")
        .with_status(404)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().unvote("TEST-1");

    assert!(result.is_err());
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_assign_issue() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("PUT", "/rest/api/latest/issue/TEST-1/assignee")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .issues()
            .assign("TEST-1", Some("john.doe".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_get_watchers() {
        let mut server = mockito::Server::new_async().await;

        let mock_watchers = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/watchers", server.url()),
            "isWatching": false,
            "watchCount": 1,
            "watchers": []
        });

        server
            .mock("GET", "/rest/api/latest/issue/TEST-1/watchers")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_watchers.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().get_watchers("TEST-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_add_watcher() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/watchers")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .issues()
            .add_watcher("TEST-1", "user".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_remove_watcher() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock(
                "DELETE",
                "/rest/api/latest/issue/TEST-1/watchers?username=user",
            )
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .issues()
            .remove_watcher("TEST-1", "user".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_vote() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/votes")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().vote("TEST-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_unvote() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/issue/TEST-1/votes")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().unvote("TEST-1").await;

        assert!(result.is_ok());
    }
}
