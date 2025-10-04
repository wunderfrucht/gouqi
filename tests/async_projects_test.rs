//! Tests for async project operations

#[cfg(feature = "async")]
mod async_projects {
    use gouqi::r#async::Jira as AsyncJira;
    use gouqi::{Credentials, UpdateProject};
    use serde_json::json;

    #[tokio::test]
    async fn test_async_update_project() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "self": format!("{}/rest/api/latest/project/10000", server.url()),
            "id": "10000",
            "key": "TEST",
            "name": "Updated Test Project",
            "projectTypeKey": "software",
            "lead": {
                "self": format!("{}/rest/api/latest/user?username=admin", server.url()),
                "name": "admin",
                "displayName": "Admin User",
                "active": true
            }
        });

        server
            .mock("PUT", "/rest/api/latest/project/10000")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let update = UpdateProject {
            key: None,
            name: Some("Updated Test Project".to_string()),
            description: Some("Updated description".to_string()),
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            category_id: None,
        };

        let result = jira.projects().update("10000", update).await;

        if let Err(e) = &result {
            eprintln!("Update failed: {:?}", e);
        }
        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name, "Updated Test Project");
    }

    #[tokio::test]
    async fn test_async_update_project_not_found() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("PUT", "/rest/api/latest/project/999")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Project not found"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();

        let update = UpdateProject {
            key: None,
            name: Some("Test".to_string()),
            description: None,
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            category_id: None,
        };

        let result = jira.projects().update("999", update).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_delete_project() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/project/10000")
            .with_status(204)
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.projects().delete("10000").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_delete_project_unauthorized() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("DELETE", "/rest/api/latest/project/10000")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(json!({"errorMessages": ["Forbidden"]}).to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.projects().delete("10000").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_get_versions() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!([
            {
                "self": format!("{}/rest/api/latest/version/10000", server.url()),
                "id": "10000",
                "name": "Version 1.0",
                "archived": false,
                "released": true,
                "projectId": 10001
            },
            {
                "self": format!("{}/rest/api/latest/version/10001", server.url()),
                "id": "10001",
                "name": "Version 2.0",
                "archived": false,
                "released": false,
                "projectId": 10001
            }
        ]);

        server
            .mock("GET", "/rest/api/latest/project/TEST/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.projects().get_versions("TEST").await;

        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].name, "Version 1.0");
        assert_eq!(versions[1].name, "Version 2.0");
    }

    #[tokio::test]
    async fn test_async_get_components() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!([
            {
                "self": format!("{}/rest/api/latest/component/10000", server.url()),
                "id": "10000",
                "name": "Component A"
            },
            {
                "self": format!("{}/rest/api/latest/component/10001", server.url()),
                "id": "10001",
                "name": "Component B"
            }
        ]);

        server
            .mock("GET", "/rest/api/latest/project/TEST/components")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_data.to_string())
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.projects().get_components("TEST").await;

        assert!(result.is_ok());
        let components = result.unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name, "Component A");
        assert_eq!(components[1].name, "Component B");
    }

    #[tokio::test]
    async fn test_async_get_versions_empty() {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/rest/api/latest/project/EMPTY/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let jira = AsyncJira::new(server.url(), Credentials::Anonymous).unwrap();
        let result = jira.projects().get_versions("EMPTY").await;

        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 0);
    }
}
