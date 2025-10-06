//! Tests for async group operations

#[cfg(feature = "async")]
mod async_groups {
    use gouqi::Credentials;
    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::groups::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_async_groups_list_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "header": "Showing 2 of 2 matching groups",
            "total": 2,
            "groups": [
                {
                    "name": "jira-administrators",
                    "groupId": "276f955c-63d7-42c8-9520-92d01dca0625",
                    "html": "jira-administrators"
                },
                {
                    "name": "jira-software-users",
                    "groupId": "6e87dc72-4f1f-421f-9382-2fee8b652487",
                    "html": "jira-software-users"
                }
            ]
        });

        server
            .mock("GET", "/rest/api/latest/groups/picker")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = GroupSearchOptions::default();
        let result = jira.groups().list(&options).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total, 2);
        assert_eq!(response.groups.len(), 2);
        assert_eq!(response.groups[0].name, "jira-administrators");
    }

    #[tokio::test]
    async fn test_async_groups_list_with_query() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "header": "Showing 1 of 1 matching groups",
            "total": 1,
            "groups": [
                {
                    "name": "developers",
                    "groupId": "276f955c-63d7-42c8-9520-92d01dca0625",
                    "html": "developers"
                }
            ]
        });

        server
            .mock(
                "GET",
                "/rest/api/latest/groups/picker?query=dev&maxResults=10",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = GroupSearchOptions::builder()
            .query("dev")
            .max_results(10)
            .build();

        let result = jira.groups().list(&options).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total, 1);
        assert_eq!(response.groups[0].name, "developers");
    }

    #[tokio::test]
    async fn test_async_get_group_members_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "self": format!("{}/rest/api/latest/group/member?groupId=276f955c-63d7-42c8-9520-92d01dca0625", server.url()),
            "maxResults": 50,
            "startAt": 0,
            "total": 2,
            "isLast": true,
            "values": [
                {
                    "accountId": "5b10a2844c20165700ede21g",
                    "displayName": "John Doe",
                    "active": true,
                    "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede21g", server.url())
                },
                {
                    "accountId": "5b10a2844c20165700ede22h",
                    "displayName": "Jane Doe",
                    "active": true,
                    "self": format!("{}/rest/api/latest/user?accountId=5b10a2844c20165700ede22h", server.url())
                }
            ]
        });

        server
            .mock(
                "GET",
                "/rest/api/latest/group/member?groupId=276f955c-63d7-42c8-9520-92d01dca0625",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = GroupMemberOptions::default();
        let result = jira
            .groups()
            .get_members("276f955c-63d7-42c8-9520-92d01dca0625", &options)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total, 2);
        assert_eq!(response.values.len(), 2);
        assert_eq!(response.values[0].display_name, "John Doe");
        assert!(response.is_last);
    }

    #[tokio::test]
    async fn test_async_create_group_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "name": "new-developers",
            "groupId": "abc123-def456",
            "self": format!("{}/rest/api/latest/group?groupId=abc123-def456", server.url())
        });

        server
            .mock("POST", "/rest/api/latest/group")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.groups().create("new-developers").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["name"], "new-developers");
        assert_eq!(response["groupId"], "abc123-def456");
    }

    #[tokio::test]
    async fn test_async_create_group_already_exists() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/rest/api/latest/group")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Group already exists"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.groups().create("existing-group").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_delete_group_success() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/group?groupId=abc123")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.groups().delete("abc123", None).await;

        if let Err(ref e) = result {
            eprintln!("Delete failed: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_delete_group_with_swap() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock(
                "DELETE",
                "/rest/api/latest/group?groupId=abc123&swapGroupId=xyz789",
            )
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .groups()
            .delete("abc123", Some("xyz789".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_add_user_to_group_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = json!({
            "name": "developers",
            "groupId": "abc123",
            "self": format!("{}/rest/api/latest/group?groupId=abc123", server.url())
        });

        server
            .mock("POST", "/rest/api/latest/group/user?groupId=abc123")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .groups()
            .add_user("abc123", "5b10a2844c20165700ede21g")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_remove_user_from_group_success() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock(
                "DELETE",
                "/rest/api/latest/group/user?groupId=abc123&accountId=5b10a2844c20165700ede21g",
            )
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira
            .groups()
            .remove_user("abc123", "5b10a2844c20165700ede21g")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_groups_unauthorized() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/api/latest/groups/picker")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Unauthorized"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let options = GroupSearchOptions::default();
        let result = jira.groups().list(&options).await;

        assert!(result.is_err());
    }
}
