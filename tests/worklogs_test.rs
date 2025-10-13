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
    assert_eq!(
        worklog.comment.as_ref().map(|c| c.as_ref()),
        Some("Worked on feature")
    );
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
    assert_eq!(worklog.comment(), Some("Test comment"));
    assert_eq!(worklog.time_spent, Some("1h".to_string()));
}

#[test]
fn test_worklog_input_started_serialization() {
    use time::OffsetDateTime;

    // Test UTC timezone - should produce format: "2024-01-01T09:00:00.000+0000"
    let started = OffsetDateTime::from_unix_timestamp(1704096000).unwrap(); // 2024-01-01 08:00:00 UTC
    let worklog = WorklogInput::new(7200).with_started(started);

    let serialized = serde_json::to_string(&worklog).unwrap();
    println!("Serialized: {}", serialized);

    // Verify the format matches JIRA expectations
    assert!(serialized.contains("\"started\":\"2024-01-01T08:00:00.000+0000\""));
    assert!(!serialized.contains("+002024")); // Should not have year prefix
    assert!(!serialized.contains("Z\"")); // Should not use Z notation

    // Verify it includes the other fields
    assert!(serialized.contains("\"timeSpentSeconds\":7200"));
}

#[test]
fn test_worklog_input_started_with_different_timezone() {
    use time::{OffsetDateTime, UtcOffset};

    // Test with EST timezone (-05:00)
    let utc_time = OffsetDateTime::from_unix_timestamp(1704096000).unwrap();
    let est_offset = UtcOffset::from_hms(-5, 0, 0).unwrap();
    let est_time = utc_time.to_offset(est_offset);

    let worklog = WorklogInput::new(3600).with_started(est_time);
    let serialized = serde_json::to_string(&worklog).unwrap();
    println!("EST Serialized: {}", serialized);

    // Should include -0500 offset
    assert!(serialized.contains("-0500"));
    assert!(serialized.contains("2024-01-01T03:00:00.000-0500"));
}

#[test]
fn test_worklog_input_started_with_positive_timezone() {
    use time::{OffsetDateTime, UtcOffset};

    // Test with +09:30 timezone (Australian Central)
    let utc_time = OffsetDateTime::from_unix_timestamp(1704096000).unwrap();
    let act_offset = UtcOffset::from_hms(9, 30, 0).unwrap();
    let act_time = utc_time.to_offset(act_offset);

    let worklog = WorklogInput::new(3600).with_started(act_time);
    let serialized = serde_json::to_string(&worklog).unwrap();
    println!("ACT Serialized: {}", serialized);

    // Should include +0930 offset
    assert!(serialized.contains("+0930"));
    assert!(serialized.contains("2024-01-01T17:30:00.000+0930"));
}

#[test]
fn test_worklog_input_started_with_milliseconds() {
    use time::OffsetDateTime;

    // Test that milliseconds are properly formatted (3 digits, not 9)
    let started = OffsetDateTime::from_unix_timestamp_nanos(1_704_096_000_123_456_789).unwrap();
    let worklog = WorklogInput::new(7200).with_started(started);

    let serialized = serde_json::to_string(&worklog).unwrap();
    println!("With milliseconds: {}", serialized);

    // Should have exactly 3 digits for milliseconds (.123)
    assert!(serialized.contains(".123+0000"));
    // Should NOT have 9 digits
    assert!(!serialized.contains(".123456789"));
}

#[test]
fn test_worklog_input_without_started() {
    // Test that omitting started field works correctly
    let worklog = WorklogInput::new(3600).with_comment("No started time");
    let serialized = serde_json::to_string(&worklog).unwrap();
    println!("Without started: {}", serialized);

    // Should not include started field at all
    assert!(!serialized.contains("\"started\""));
    assert!(serialized.contains("\"timeSpentSeconds\":3600"));
    // Comment should be in plain string format (v2 API - default serialization)
    assert!(serialized.contains("\"comment\":\"No started time\""));
}

#[test]
fn test_worklog_from_minutes() {
    let worklog = WorklogInput::from_minutes(30);
    assert_eq!(worklog.time_spent_seconds, Some(1800)); // 30 * 60
}

#[test]
fn test_worklog_from_hours() {
    let worklog = WorklogInput::from_hours(2);
    assert_eq!(worklog.time_spent_seconds, Some(7200)); // 2 * 3600
}

#[test]
fn test_worklog_from_days() {
    let worklog = WorklogInput::from_days(1);
    assert_eq!(worklog.time_spent_seconds, Some(28800)); // 1 * 8 * 3600
}

#[test]
fn test_worklog_from_weeks() {
    let worklog = WorklogInput::from_weeks(1);
    assert_eq!(worklog.time_spent_seconds, Some(144000)); // 1 * 5 * 8 * 3600
}

#[test]
fn test_worklog_started_hours_ago() {
    use time::OffsetDateTime;

    let before = OffsetDateTime::now_utc();
    let worklog = WorklogInput::from_hours(2).started_hours_ago(3);
    let after = OffsetDateTime::now_utc();

    assert!(worklog.started.is_some());
    let started = worklog.started.unwrap();

    // Started time should be approximately 3 hours ago
    let three_hours_before = before - time::Duration::hours(3);
    let three_hours_after = after - time::Duration::hours(3);

    // Allow 1 second tolerance for test execution time
    assert!(started >= three_hours_before - time::Duration::seconds(1));
    assert!(started <= three_hours_after + time::Duration::seconds(1));
}

#[test]
fn test_worklog_started_days_ago() {
    use time::OffsetDateTime;

    let before = OffsetDateTime::now_utc();
    let worklog = WorklogInput::from_hours(4).started_days_ago(2);
    let after = OffsetDateTime::now_utc();

    assert!(worklog.started.is_some());
    let started = worklog.started.unwrap();

    // Started time should be approximately 2 days ago
    let two_days_before = before - time::Duration::days(2);
    let two_days_after = after - time::Duration::days(2);

    // Allow 1 second tolerance
    assert!(started >= two_days_before - time::Duration::seconds(1));
    assert!(started <= two_days_after + time::Duration::seconds(1));
}

#[test]
fn test_worklog_started_at() {
    use time::macros::datetime;

    let specific_time = datetime!(2024-01-15 14:30:00 UTC);
    let worklog = WorklogInput::from_hours(3).started_at(specific_time);

    assert_eq!(worklog.started, Some(specific_time));
}

#[test]
fn test_worklog_builder_chain() {
    use time::macros::datetime;

    // Test chaining multiple methods
    let worklog = WorklogInput::from_hours(2)
        .with_comment("Fixed critical bug")
        .started_at(datetime!(2024-01-15 09:00:00 UTC));

    assert_eq!(worklog.time_spent_seconds, Some(7200));
    assert_eq!(worklog.comment(), Some("Fixed critical bug"));
    assert_eq!(worklog.started, Some(datetime!(2024-01-15 09:00:00 UTC)));
}

#[test]
fn test_worklog_started_minutes_ago() {
    use time::OffsetDateTime;

    let before = OffsetDateTime::now_utc();
    let worklog = WorklogInput::from_minutes(45).started_minutes_ago(30);
    let after = OffsetDateTime::now_utc();

    assert!(worklog.started.is_some());
    let started = worklog.started.unwrap();

    // Started time should be approximately 30 minutes ago
    let thirty_min_before = before - time::Duration::minutes(30);
    let thirty_min_after = after - time::Duration::minutes(30);

    // Allow 1 second tolerance
    assert!(started >= thirty_min_before - time::Duration::seconds(1));
    assert!(started <= thirty_min_after + time::Duration::seconds(1));
}

#[test]
fn test_worklog_started_weeks_ago() {
    use time::OffsetDateTime;

    let before = OffsetDateTime::now_utc();
    let worklog = WorklogInput::from_days(3).started_weeks_ago(1);
    let after = OffsetDateTime::now_utc();

    assert!(worklog.started.is_some());
    let started = worklog.started.unwrap();

    // Started time should be approximately 1 week ago
    let one_week_before = before - time::Duration::weeks(1);
    let one_week_after = after - time::Duration::weeks(1);

    // Allow 1 second tolerance
    assert!(started >= one_week_before - time::Duration::seconds(1));
    assert!(started <= one_week_after + time::Duration::seconds(1));
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
