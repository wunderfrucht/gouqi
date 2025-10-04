//! Additional tests to improve coverage for issues functionality

use gouqi::{Board, Credentials, Jira, SearchOptions};
use serde_json::json;
use std::collections::BTreeMap;

fn get_test_board() -> Board {
    Board {
        id: 1,
        name: "Test Board".to_string(),
        type_name: "scrum".to_string(),
        self_link: "http://localhost/rest/agile/latest/board/1".to_string(),
        location: None,
    }
}

#[test]
fn test_issues_get_success() {
    let mut server = mockito::Server::new();

    let mock_issue = json!({
        "id": "10001",
        "key": "TEST-1",
        "self": format!("{}/rest/api/latest/issue/10001", server.url()),
        "fields": {
            "summary": "Test Issue",
            "status": {
                "name": "Open"
            }
        }
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_issue.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get("TEST-1");

    match &result {
        Ok(issue) => assert_eq!(issue.key, "TEST-1"),
        Err(e) => panic!("Expected Ok, got Err: {:?}", e),
    }
}

#[test]
fn test_issues_list_for_board() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 1,
        "issues": [
            {
                "id": "10001",
                "key": "TEST-1",
                "self": format!("{}/rest/agile/latest/issue/10001", server.url()),
                "fields": {}
            }
        ]
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/agile/latest/board/1/issue".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = get_test_board();
    let result = jira.issues().list(&board, &SearchOptions::default());

    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.total, 1);
    assert_eq!(results.issues.len(), 1);
    assert_eq!(results.issues[0].key, "TEST-1");
}

#[test]
fn test_issues_delete() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/issue/TEST-1")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().delete("TEST-1");

    assert!(result.is_ok());
}

#[test]
fn test_issues_archive() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/archive")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().archive("TEST-1");

    assert!(result.is_ok());
}

#[test]
fn test_issues_assign() {
    let mut server = mockito::Server::new();

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1/assignee")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().assign("TEST-1", Some("johndoe".to_string()));

    assert!(result.is_ok());
}

#[test]
fn test_issues_unassign() {
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
fn test_issues_get_watchers() {
    let mut server = mockito::Server::new();

    let mock_watchers = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/watchers", server.url()),
        "watchers": [
            {
                "active": true,
                "displayName": "John Doe",
                "name": "jdoe",
                "self": format!("{}/rest/api/latest/user?username=jdoe", server.url())
            }
        ],
        "watchCount": 1,
        "isWatching": false
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
    assert_eq!(watchers.watch_count, 1);
    assert!(!watchers.is_watching);
    assert_eq!(watchers.watchers.len(), 1);
}

#[test]
fn test_issues_add_watcher() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/watchers")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().add_watcher("TEST-1", "johndoe".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_issues_remove_watcher() {
    let mut server = mockito::Server::new();

    server
        .mock(
            "DELETE",
            "/rest/api/latest/issue/TEST-1/watchers?username=johndoe",
        )
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira
        .issues()
        .remove_watcher("TEST-1", "johndoe".to_string());

    assert!(result.is_ok());
}

#[test]
fn test_issues_vote() {
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
fn test_issues_unvote() {
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
fn test_issues_edit() {
    let mut server = mockito::Server::new();

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let mut fields = BTreeMap::new();
    fields.insert(
        "summary".to_string(),
        serde_json::Value::String("Updated summary".to_string()),
    );
    let edit_issue = gouqi::issues::EditIssue { fields };

    let result = jira.issues().update("TEST-1", edit_issue);

    assert!(result.is_ok());
}

#[test]
fn test_issues_iterator_single_page() {
    let mut server = mockito::Server::new();

    let mock_results = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 2,
        "issues": [
            {
                "id": "10001",
                "key": "TEST-1",
                "self": format!("{}/rest/agile/latest/issue/10001", server.url()),
                "fields": {}
            },
            {
                "id": "10002",
                "key": "TEST-2",
                "self": format!("{}/rest/agile/latest/issue/10002", server.url()),
                "fields": {}
            }
        ]
    });

    server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/rest/agile/latest/board/1/issue".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_results.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let board = get_test_board();
    let options = SearchOptions::default();
    let mut iter = jira.issues().iter(&board, &options).unwrap();

    // Iterator should return issues in reverse order (pop from end)
    let issue1 = iter.next();
    assert!(issue1.is_some());
    assert_eq!(issue1.unwrap().key, "TEST-2");

    let issue2 = iter.next();
    assert!(issue2.is_some());
    assert_eq!(issue2.unwrap().key, "TEST-1");

    // No more items
    assert!(iter.next().is_none());
}

#[test]
fn test_component_new() {
    use gouqi::issues::Component;

    let component = Component::new("10000", "Backend API");

    assert_eq!(component.id, "10000");
    assert_eq!(component.name, "Backend API");
    assert!(component.description.is_none());
    assert!(component.project_id.is_none());
}

#[test]
fn test_add_comment_new() {
    use gouqi::issues::AddComment;

    let comment = AddComment::new("Test comment");

    assert_eq!(comment.body, "Test comment");
    assert!(comment.visibility.is_none());
}

#[test]
fn test_add_comment_with_visibility() {
    use gouqi::Visibility;
    use gouqi::issues::AddComment;

    let comment = AddComment::new("Test comment").with_visibility(Visibility {
        visibility_type: "role".to_string(),
        value: "Administrators".to_string(),
    });

    assert_eq!(comment.body, "Test comment");
    assert!(comment.visibility.is_some());
}

#[cfg(feature = "async")]
mod async_issues_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_issues_get() {
        let mut server = mockito::Server::new_async().await;

        let mock_issue = json!({
            "id": "10001",
            "key": "ASYNC-1",
            "self": format!("{}/rest/api/latest/issue/10001", server.url()),
            "fields": {
                "summary": "Async Test Issue"
            }
        });

        server
            .mock("GET", "/rest/api/latest/issue/ASYNC-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_issue.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().get("ASYNC-1").await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.key, "ASYNC-1");
    }

    #[tokio::test]
    async fn test_async_issues_delete() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/issue/ASYNC-1")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().delete("ASYNC-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_issues_archive() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/ASYNC-1/archive")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().archive("ASYNC-1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_issues_assign() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("PUT", "/rest/api/latest/issue/ASYNC-1/assignee")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .issues()
            .assign("ASYNC-1", Some("johndoe".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_issues_vote() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/issue/ASYNC-1/votes")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().vote("ASYNC-1").await;

        assert!(result.is_ok());
    }
}
