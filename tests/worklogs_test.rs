//! Tests for worklog operations

use gouqi::issues::{AdjustEstimate, WorklogOptions};
use gouqi::{Credentials, Jira, WorklogInput};
use serde_json::json;

#[test]
fn test_get_worklogs() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "startAt": 0,
        "maxResults": 20,
        "total": 2,
        "worklogs": [
            {
                "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
                "id": "10001",
                "author": {
                    "self": format!("{}/rest/api/latest/user?username=user1", server.url()),
                    "name": "user1",
                    "displayName": "User One",
                    "active": true
                },
                "updateAuthor": {
                    "self": format!("{}/rest/api/latest/user?username=user1", server.url()),
                    "name": "user1",
                    "displayName": "User One",
                    "active": true
                },
                "comment": "Fixed the bug",
                "created": "2024-01-01T10:00:00.000+0000",
                "updated": "2024-01-01T10:00:00.000+0000",
                "started": "2024-01-01T09:00:00.000+0000",
                "timeSpent": "2h",
                "timeSpentSeconds": 7200,
                "issueId": "10000"
            },
            {
                "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10002", server.url()),
                "id": "10002",
                "author": {
                    "self": format!("{}/rest/api/latest/user?username=user2", server.url()),
                    "name": "user2",
                    "displayName": "User Two",
                    "active": true
                },
                "timeSpent": "1h 30m",
                "timeSpentSeconds": 5400,
                "issueId": "10000"
            }
        ]
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-1/worklog")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get_worklogs("TEST-1");

    assert!(result.is_ok());
    let worklogs = result.unwrap();
    assert_eq!(worklogs.total, 2);
    assert_eq!(worklogs.worklogs.len(), 2);
    assert_eq!(worklogs.worklogs[0].id, "10001");
    assert_eq!(worklogs.worklogs[0].time_spent_seconds, Some(7200));
}

#[test]
fn test_get_worklog_by_id() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
        "id": "10001",
        "author": {
            "self": format!("{}/rest/api/latest/user?username=user1", server.url()),
            "name": "user1",
            "displayName": "User One",
            "active": true
        },
        "comment": "Worked on feature",
        "timeSpent": "3h 20m",
        "timeSpentSeconds": 12000,
        "issueId": "10000"
    });

    server
        .mock("GET", "/rest/api/latest/issue/TEST-1/worklog/10001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().get_worklog("TEST-1", "10001");

    assert!(result.is_ok());
    let worklog = result.unwrap();
    assert_eq!(worklog.id, "10001");
    assert_eq!(worklog.time_spent_seconds, Some(12000));
    assert_eq!(worklog.comment, Some("Worked on feature".to_string()));
}

#[test]
fn test_add_worklog() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10003", server.url()),
        "id": "10003",
        "author": {
            "self": format!("{}/rest/api/latest/user?username=currentuser", server.url()),
            "name": "currentuser",
            "displayName": "Current User",
            "active": true
        },
        "comment": "Fixed critical bug",
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(7200).with_comment("Fixed critical bug");

    let result = jira.issues().add_worklog("TEST-1", worklog);

    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.id, "10003");
    assert_eq!(created.time_spent_seconds, Some(7200));
}

#[test]
fn test_add_worklog_with_time_string() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10004", server.url()),
        "id": "10004",
        "timeSpent": "1h 30m",
        "timeSpentSeconds": 5400,
        "issueId": "10000"
    });

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(5400).with_time_spent("1h 30m");

    let result = jira.issues().add_worklog("TEST-1", worklog);

    assert!(result.is_ok());
}

#[test]
fn test_update_worklog() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
        "id": "10001",
        "comment": "Updated comment",
        "timeSpent": "3h",
        "timeSpentSeconds": 10800,
        "issueId": "10000"
    });

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1/worklog/10001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(10800).with_comment("Updated comment");

    let result = jira.issues().update_worklog("TEST-1", "10001", worklog);

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.time_spent_seconds, Some(10800));
}

#[test]
fn test_delete_worklog() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/issue/TEST-1/worklog/10001")
        .with_status(204)
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().delete_worklog("TEST-1", "10001");

    assert!(result.is_ok());
}

#[test]
fn test_delete_worklog_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("DELETE", "/rest/api/latest/issue/TEST-1/worklog/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Worklog not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.issues().delete_worklog("TEST-1", "99999");

    assert!(result.is_err());
}

#[test]
fn test_worklog_input_builder() {
    let worklog = WorklogInput::new(3600)
        .with_comment("Test comment")
        .with_time_spent("1h");

    assert_eq!(worklog.time_spent_seconds, Some(3600));
    assert_eq!(worklog.comment, Some("Test comment".to_string()));
    assert_eq!(worklog.time_spent, Some("1h".to_string()));
}

#[test]
fn test_add_worklog_with_options_new_estimate() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10005", server.url()),
        "id": "10005",
        "timeSpent": "2h",
        "timeSpentSeconds": 7200,
        "issueId": "10000"
    });

    // Verify the query parameters are sent correctly
    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "new".to_string()),
            mockito::Matcher::UrlEncoded("newEstimate".to_string(), "1d".to_string()),
            mockito::Matcher::UrlEncoded("notifyUsers".to_string(), "false".to_string()),
        ]))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(7200).with_comment("Fixed bug");
    let options = WorklogOptions::builder()
        .adjust_estimate(AdjustEstimate::New("1d".to_string()))
        .notify_users(false)
        .build();

    let result = jira
        .issues()
        .add_worklog_with_options("TEST-1", worklog, &options);

    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.id, "10005");
    assert_eq!(created.time_spent_seconds, Some(7200));
}

#[test]
fn test_add_worklog_with_options_manual_reduce() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10006", server.url()),
        "id": "10006",
        "timeSpent": "1h",
        "timeSpentSeconds": 3600,
        "issueId": "10000"
    });

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "manual".to_string()),
            mockito::Matcher::UrlEncoded("reduceBy".to_string(), "30m".to_string()),
        ]))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(3600);
    let options = WorklogOptions::builder()
        .adjust_estimate(AdjustEstimate::Manual("30m".to_string()))
        .build();

    let result = jira
        .issues()
        .add_worklog_with_options("TEST-1", worklog, &options);

    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.id, "10006");
}

#[test]
fn test_add_worklog_with_options_leave_estimate() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10007", server.url()),
        "id": "10007",
        "timeSpent": "3h",
        "timeSpentSeconds": 10800,
        "issueId": "10000"
    });

    server
        .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
        .match_query(mockito::Matcher::UrlEncoded(
            "adjustEstimate".to_string(),
            "leave".to_string(),
        ))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(10800);
    let options = WorklogOptions::builder()
        .adjust_estimate(AdjustEstimate::Leave)
        .build();

    let result = jira
        .issues()
        .add_worklog_with_options("TEST-1", worklog, &options);

    assert!(result.is_ok());
}

#[test]
fn test_update_worklog_with_options() {
    let mut server = mockito::Server::new();

    let response_data = json!({
        "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
        "id": "10001",
        "timeSpent": "4h",
        "timeSpentSeconds": 14400,
        "issueId": "10000"
    });

    server
        .mock("PUT", "/rest/api/latest/issue/TEST-1/worklog/10001")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "new".to_string()),
            mockito::Matcher::UrlEncoded("newEstimate".to_string(), "2h".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_data.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();

    let worklog = WorklogInput::new(14400);
    let options = WorklogOptions::builder()
        .adjust_estimate(AdjustEstimate::New("2h".to_string()))
        .build();

    let result = jira
        .issues()
        .update_worklog_with_options("TEST-1", "10001", worklog, &options);

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.time_spent_seconds, Some(14400));
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_get_worklogs() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "startAt": 0,
            "maxResults": 20,
            "total": 1,
            "worklogs": [
                {
                    "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
                    "id": "10001",
                    "timeSpent": "2h",
                    "timeSpentSeconds": 7200,
                    "issueId": "10000"
                }
            ]
        });

        server
            .mock("GET", "/rest/api/latest/issue/TEST-1/worklog")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().get_worklogs("TEST-1").await;

        assert!(result.is_ok());
        let worklogs = result.unwrap();
        assert_eq!(worklogs.total, 1);
    }

    #[tokio::test]
    async fn test_async_get_worklog_by_id() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
            "id": "10001",
            "timeSpent": "2h",
            "timeSpentSeconds": 7200,
            "issueId": "10000"
        });

        server
            .mock("GET", "/rest/api/latest/issue/TEST-1/worklog/10001")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().get_worklog("TEST-1", "10001").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_add_worklog() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10002", server.url()),
            "id": "10002",
            "timeSpent": "1h",
            "timeSpentSeconds": 3600,
            "issueId": "10000"
        });

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let worklog = WorklogInput::new(3600);
        let result = jira.issues().add_worklog("TEST-1", worklog).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_update_worklog() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
            "id": "10001",
            "timeSpent": "2h",
            "timeSpentSeconds": 7200,
            "issueId": "10000"
        });

        server
            .mock("PUT", "/rest/api/latest/issue/TEST-1/worklog/10001")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let worklog = WorklogInput::new(7200);
        let result = jira
            .issues()
            .update_worklog("TEST-1", "10001", worklog)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_delete_worklog() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/issue/TEST-1/worklog/10001")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.issues().delete_worklog("TEST-1", "10001").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_add_worklog_with_options_new_estimate() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10008", server.url()),
            "id": "10008",
            "timeSpent": "2h",
            "timeSpentSeconds": 7200,
            "issueId": "10000"
        });

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "new".to_string()),
                mockito::Matcher::UrlEncoded("newEstimate".to_string(), "3h".to_string()),
            ]))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let worklog = WorklogInput::new(7200);
        let options = WorklogOptions::builder()
            .adjust_estimate(AdjustEstimate::New("3h".to_string()))
            .build();

        let result = jira
            .issues()
            .add_worklog_with_options("TEST-1", worklog, &options)
            .await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id, "10008");
    }

    #[tokio::test]
    async fn test_async_add_worklog_with_options_manual_reduce() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10009", server.url()),
            "id": "10009",
            "timeSpent": "1h",
            "timeSpentSeconds": 3600,
            "issueId": "10000"
        });

        server
            .mock("POST", "/rest/api/latest/issue/TEST-1/worklog")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "manual".to_string()),
                mockito::Matcher::UrlEncoded("reduceBy".to_string(), "45m".to_string()),
            ]))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let worklog = WorklogInput::new(3600);
        let options = WorklogOptions::builder()
            .adjust_estimate(AdjustEstimate::Manual("45m".to_string()))
            .build();

        let result = jira
            .issues()
            .add_worklog_with_options("TEST-1", worklog, &options)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_update_worklog_with_options() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/issue/TEST-1/worklog/10001", server.url()),
            "id": "10001",
            "timeSpent": "5h",
            "timeSpentSeconds": 18000,
            "issueId": "10000"
        });

        server
            .mock("PUT", "/rest/api/latest/issue/TEST-1/worklog/10001")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("adjustEstimate".to_string(), "leave".to_string()),
                mockito::Matcher::UrlEncoded("notifyUsers".to_string(), "false".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let worklog = WorklogInput::new(18000);
        let options = WorklogOptions::builder()
            .adjust_estimate(AdjustEstimate::Leave)
            .notify_users(false)
            .build();

        let result = jira
            .issues()
            .update_worklog_with_options("TEST-1", "10001", worklog, &options)
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.time_spent_seconds, Some(18000));
    }
}
