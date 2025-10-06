//! Tests to cover missing lines in users and groups modules

use gouqi::{Credentials, Jira, User, groups::*, users::*};
use serde_json::json;

#[test]
fn test_assignable_user_options_with_all_params() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "displayName": "Test User",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/assignable/search?query=test&startAt=5&maxResults=20&project=TEST",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = AssignableUserOptions::builder()
        .query("test")
        .start_at(5)
        .max_results(20)
        .build();

    let result: Result<Vec<User>, _> = jira.users().get_assignable_users("TEST", &options);

    assert!(result.is_ok());
}

#[test]
fn test_user_search_all_options() {
    let mut server = mockito::Server::new();

    let mock_response = json!([
        {
            "accountId": "5b10a2844c20165700ede21g",
            "displayName": "Test User",
            "active": true,
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        }
    ]);

    server
        .mock(
            "GET",
            "/rest/api/latest/user/search?query=test&startAt=10&maxResults=25",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = UserSearchOptions::builder()
        .query("test")
        .start_at(10)
        .max_results(25)
        .build();

    let result: Result<Vec<User>, _> = jira.users().search(&options);

    assert!(result.is_ok());
}

#[test]
fn test_group_members_with_all_options() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "self": format!("{}/rest/api/latest/group/member?groupId=abc123", server.url()),
        "maxResults": 50,
        "startAt": 5,
        "total": 10,
        "isLast": false,
        "values": [
            {
                "accountId": "5b10a2844c20165700ede21g",
                "displayName": "Test User",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            }
        ]
    });

    server
        .mock(
            "GET",
            "/rest/api/latest/group/member?includeInactiveUsers=true&startAt=5&maxResults=50&groupId=abc123",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let options = GroupMemberOptions::builder()
        .include_inactive_users(true)
        .start_at(5)
        .max_results(50)
        .build();

    let result = jira.groups().get_members("abc123", &options);

    assert!(result.is_ok());
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use gouqi::r#async::Jira as AsyncJira;

    #[tokio::test]
    async fn test_async_user_search_all_options() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!([
            {
                "accountId": "5b10a2844c20165700ede21g",
                "displayName": "Test User",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            }
        ]);

        server
            .mock(
                "GET",
                "/rest/api/latest/user/search?query=test&startAt=10&maxResults=25",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = UserSearchOptions::builder()
            .query("test")
            .start_at(10)
            .max_results(25)
            .build();

        let result: Result<Vec<User>, _> = jira.users().search(&options).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_assignable_user_options_with_all_params() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!([
            {
                "accountId": "5b10a2844c20165700ede21g",
                "displayName": "Test User",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            }
        ]);

        server
            .mock(
                "GET",
                "/rest/api/latest/user/assignable/search?query=test&startAt=5&maxResults=20&project=TEST",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = AssignableUserOptions::builder()
            .query("test")
            .start_at(5)
            .max_results(20)
            .build();

        let result: Result<Vec<User>, _> =
            jira.users().get_assignable_users("TEST", &options).await;

        assert!(result.is_ok());
    }
}
