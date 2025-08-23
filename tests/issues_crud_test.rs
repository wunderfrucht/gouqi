//! Tests for new Issue CRUD operations

#[cfg(test)]
mod issue_crud_tests {
    use gouqi::issues::{BulkIssueUpdate, BulkUpdateRequest};
    use gouqi::{Credentials, Jira};
    use mockito::{Matcher, Server};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn get_test_jira(server: &Server) -> Jira {
        Jira::new(server.url(), Credentials::Anonymous).expect("Failed to create test Jira client")
    }

    #[test]
    fn test_delete_issue() {
        let mut server = Server::new();
        let _m = server
            .mock("DELETE", "/rest/api/latest/issue/TEST-123")
            .with_status(204)
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().delete("TEST-123");

        assert!(result.is_ok());
    }

    #[test]
    fn test_archive_issue() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/archive")
            .with_status(204)
            .with_body("{}")
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().archive("TEST-123");

        assert!(result.is_ok());
    }

    #[test]
    fn test_assign_issue() {
        let mut server = Server::new();
        let _m = server
            .mock("PUT", "/rest/api/latest/issue/TEST-123/assignee")
            .with_status(204)
            .match_body(Matcher::Json(json!({
                "assignee": "johndoe"
            })))
            .create();

        let jira = get_test_jira(&server);
        let result = jira
            .issues()
            .assign("TEST-123", Some("johndoe".to_string()));

        assert!(result.is_ok());
    }

    #[test]
    fn test_unassign_issue() {
        let mut server = Server::new();
        let _m = server
            .mock("PUT", "/rest/api/latest/issue/TEST-123/assignee")
            .with_status(204)
            .match_body(Matcher::Json(json!({
                "assignee": null
            })))
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().assign("TEST-123", None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_watchers() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/issue/TEST-123/watchers")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/issue/TEST-123/watchers",
                "watchers": [
                    {
                        "self": "http://localhost/rest/api/2/user?username=johndoe",
                        "name": "johndoe",
                        "key": "johndoe",
                        "emailAddress": "john.doe@example.com",
                        "displayName": "John Doe",
                        "active": true,
                        "timeZone": "America/New_York",
                        "avatarUrls": {}
                    }
                ],
                "watchCount": 1,
                "isWatching": false
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().get_watchers("TEST-123");

        assert!(result.is_ok());
        let watchers = result.unwrap();
        assert_eq!(watchers.watch_count, 1);
        assert!(!watchers.is_watching);
        assert_eq!(watchers.watchers.len(), 1);
        assert_eq!(watchers.watchers[0].name, Some("johndoe".to_string()));
    }

    #[test]
    fn test_add_watcher() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/watchers")
            .with_status(204)
            .match_body("\"johndoe\"")
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().add_watcher("TEST-123", "johndoe".to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_watcher() {
        let mut server = Server::new();
        let _m = server
            .mock(
                "DELETE",
                "/rest/api/latest/issue/TEST-123/watchers?username=johndoe",
            )
            .with_status(204)
            .create();

        let jira = get_test_jira(&server);
        let result = jira
            .issues()
            .remove_watcher("TEST-123", "johndoe".to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn test_vote() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/votes")
            .with_status(204)
            .with_body("{}")
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().vote("TEST-123");

        assert!(result.is_ok());
    }

    #[test]
    fn test_unvote() {
        let mut server = Server::new();
        let _m = server
            .mock("DELETE", "/rest/api/latest/issue/TEST-123/votes")
            .with_status(204)
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().unvote("TEST-123");

        assert!(result.is_ok());
    }

    #[test]
    fn test_bulk_update() {
        let mut server = Server::new();
        let _m = server
            .mock("PUT", "/rest/api/latest/issue/bulk")
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body(Matcher::JsonString(
                r#"{
                "issueUpdates": [
                    {
                        "key": "TEST-123",
                        "fields": {
                            "summary": "Updated summary"
                        }
                    }
                ]
            }"#
                .to_string(),
            ))
            .with_body(
                r#"{
                "issues": [
                    {
                        "self": "http://localhost/rest/api/2/issue/10001",
                        "key": "TEST-123",
                        "id": "10001",
                        "fields": {}
                    }
                ],
                "errors": []
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);

        let mut fields = BTreeMap::new();
        fields.insert(
            "summary".to_string(),
            serde_json::Value::String("Updated summary".to_string()),
        );

        let update = BulkIssueUpdate {
            key: "TEST-123".to_string(),
            fields,
        };

        let request = BulkUpdateRequest {
            issue_updates: vec![update],
        };

        let result = jira.issues().bulk_update(request);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.issues.len(), 1);
        assert_eq!(response.issues[0].key, "TEST-123");
        assert_eq!(response.errors.len(), 0);
    }

    #[test]
    fn test_delete_issue_error() {
        let mut server = Server::new();
        let _m = server
            .mock("DELETE", "/rest/api/latest/issue/TEST-123")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "errorMessages": ["You do not have permission to delete this issue."],
                "errors": {}
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.issues().delete("TEST-123");

        assert!(result.is_err());
    }

    #[test]
    fn test_assign_invalid_user_error() {
        let mut server = Server::new();
        let _m = server
            .mock("PUT", "/rest/api/latest/issue/TEST-123/assignee")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "errorMessages": ["User 'invaliduser' cannot be assigned to this issue."],
                "errors": {}
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira
            .issues()
            .assign("TEST-123", Some("invaliduser".to_string()));

        assert!(result.is_err());
    }
}

// Async tests
#[cfg(feature = "async")]
#[cfg(test)]
mod async_issue_crud_tests {
    use gouqi::{Credentials, r#async::Jira as AsyncJira};
    use mockito::{Matcher, Server};
    use serde_json::json;

    fn get_test_jira(server: &Server) -> AsyncJira {
        AsyncJira::new(server.url(), Credentials::Anonymous)
            .expect("Failed to create test async Jira client")
    }

    #[tokio::test]
    async fn test_async_delete_issue() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("DELETE", "/rest/api/latest/issue/TEST-123")
            .with_status(204)
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira.issues().delete("TEST-123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_archive_issue() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/archive")
            .with_status(204)
            .with_body("{}")
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira.issues().archive("TEST-123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_assign_issue() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("PUT", "/rest/api/latest/issue/TEST-123/assignee")
            .with_status(204)
            .match_body(Matcher::Json(json!({
                "assignee": "johndoe"
            })))
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira
            .issues()
            .assign("TEST-123", Some("johndoe".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_vote() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/rest/api/latest/issue/TEST-123/votes")
            .with_status(204)
            .with_body("{}")
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira.issues().vote("TEST-123").await;

        assert!(result.is_ok());
    }
}
