//! Tests for async user operations

#[cfg(feature = "async")]
mod async_users {
    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::{Credentials, User, users::*};
    use serde_json::json;

    #[tokio::test]
    async fn test_async_user_get_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_user = json!({
            "accountId": "5b10a2844c20165700ede21g",
            "accountType": "atlassian",
            "displayName": "John Doe",
            "emailAddress": "john@example.com",
            "active": true,
            "avatarUrls": {
                "48x48": "https://avatar-management.services.atlassian.com/initials/JD-0.png"
            },
            "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
        });

        server
            .mock(
                "GET",
                "/rest/api/latest/user?accountId=5b10a2844c20165700ede21g",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_user.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result: Result<User, _> = jira.users().get("5b10a2844c20165700ede21g").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.account_id.unwrap(), "5b10a2844c20165700ede21g");
        assert_eq!(user.display_name, "John Doe");
    }

    #[tokio::test]
    async fn test_async_user_get_not_found() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/api/latest/user?accountId=invalid")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["User does not exist"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result: Result<User, _> = jira.users().get("invalid").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_user_search_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!([
            {
                "accountId": "5b10a2844c20165700ede21g",
                "accountType": "atlassian",
                "displayName": "John Doe",
                "emailAddress": "john@example.com",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            },
            {
                "accountId": "5b10a2844c20165700ede22h",
                "accountType": "atlassian",
                "displayName": "Jane Doe",
                "emailAddress": "jane@example.com",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede22h", server.url())
            }
        ]);

        server
            .mock(
                "GET",
                "/rest/api/latest/user/search?query=john&maxResults=10",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = UserSearchOptions::builder()
            .query("john")
            .max_results(10)
            .build();

        let result: Result<Vec<User>, _> = jira.users().search(&options).await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].display_name, "John Doe");
        assert_eq!(users[1].display_name, "Jane Doe");
    }

    #[tokio::test]
    async fn test_async_get_assignable_users_for_project() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!([
            {
                "accountId": "5b10a2844c20165700ede21g",
                "accountType": "atlassian",
                "displayName": "Assignable User 1",
                "active": true,
                "avatarUrls": {
                    "48x48": "https://avatar-management.services.atlassian.com/initials/AU1-0.png"
                },
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            },
            {
                "accountId": "5b10a2844c20165700ede22h",
                "accountType": "atlassian",
                "displayName": "Assignable User 2",
                "active": true,
                "avatarUrls": {
                    "48x48": "https://avatar-management.services.atlassian.com/initials/AU2-0.png"
                },
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede22h", server.url())
            }
        ]);

        let _m = server
            .mock(
                "GET",
                "/rest/api/latest/user/assignable/search?maxResults=50&project=TEST",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = AssignableUserOptions::builder().max_results(50).build();

        let result: Result<Vec<User>, _> =
            jira.users().get_assignable_users("TEST", &options).await;

        if let Err(ref e) = result {
            eprintln!("Get assignable users failed: {:?}", e);
        }
        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].display_name, "Assignable User 1");
    }

    #[tokio::test]
    async fn test_async_get_assignable_users_for_issue() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!([
            {
                "accountId": "5b10a2844c20165700ede21g",
                "displayName": "Assignable User 1",
                "active": true,
                "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
            }
        ]);

        server
            .mock(
                "GET",
                "/rest/api/latest/user/assignable/search?issueKey=TEST-123",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = AssignableUserOptions::default();
        let result: Result<Vec<User>, _> = jira
            .users()
            .get_assignable_users_for_issue("TEST-123", &options)
            .await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 1);
    }

    #[tokio::test]
    async fn test_async_user_search_unauthorized() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/api/latest/user/search?query=test")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Unauthorized"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = UserSearchOptions::builder().query("test").build();

        let result: Result<Vec<User>, _> = jira.users().search(&options).await;

        assert!(result.is_err());
    }
}
