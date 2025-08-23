//! Tests for Project management operations

#[cfg(test)]
mod projects_tests {
    use gouqi::{CreateProject, Credentials, Jira, ProjectSearchOptions, UpdateProject};
    use mockito::{Matcher, Server};
    use serde_json::json;

    fn get_test_jira(server: &Server) -> Jira {
        Jira::new(server.url(), Credentials::Anonymous).expect("Failed to create test Jira client")
    }

    #[test]
    fn test_list_projects() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "self": "http://localhost/rest/api/2/project/DEMO",
                    "id": "10000",
                    "key": "DEMO",
                    "name": "Demo Project",
                    "description": "A demonstration project",
                    "projectTypeKey": "software"
                }
            ]"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().list();

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].key, "DEMO");
        assert_eq!(projects[0].name, "Demo Project");
    }

    #[test]
    fn test_get_project() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/DEMO",
                "id": "10000",
                "key": "DEMO",
                "name": "Demo Project",
                "description": "A demonstration project",
                "projectTypeKey": "software"
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get("DEMO");

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.key, "DEMO");
        assert_eq!(project.name, "Demo Project");
        assert_eq!(project.id, "10000");
    }

    #[test]
    fn test_create_project() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/rest/api/latest/project")
            .with_status(201)
            .with_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "key": "NEW",
                "name": "New Project",
                "projectTypeKey": "software",
                "description": "A new project for testing"
            })))
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/NEW",
                "id": "10001",
                "key": "NEW",
                "name": "New Project",
                "description": "A new project for testing",
                "projectTypeKey": "software"
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let create_project = CreateProject {
            key: "NEW".to_string(),
            name: "New Project".to_string(),
            project_type_key: "software".to_string(),
            description: Some("A new project for testing".to_string()),
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            issue_security_scheme: None,
            permission_scheme: None,
            notification_scheme: None,
            category_id: None,
        };

        let result = jira.projects().create(create_project);

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.key, "NEW");
        assert_eq!(project.name, "New Project");
    }

    #[test]
    fn test_update_project() {
        let mut server = Server::new();
        let _m = server
            .mock("PUT", "/rest/api/latest/project/DEMO")
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "name": "Updated Demo Project",
                "description": "Updated description"
            })))
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/DEMO",
                "id": "10000",
                "key": "DEMO",
                "name": "Updated Demo Project",
                "description": "Updated description",
                "projectTypeKey": "software"
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let update_project = UpdateProject {
            key: None,
            name: Some("Updated Demo Project".to_string()),
            description: Some("Updated description".to_string()),
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            category_id: None,
        };

        let result = jira.projects().update("DEMO", update_project);

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name, "Updated Demo Project");
    }

    #[test]
    fn test_delete_project() {
        let mut server = Server::new();
        let _m = server
            .mock("DELETE", "/rest/api/latest/project/DEMO")
            .with_status(204)
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().delete("DEMO");

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_project_versions() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO/versions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "self": "http://localhost/rest/api/2/version/1",
                    "id": "1",
                    "name": "Version 1.0",
                    "projectId": 10000,
                    "archived": false,
                    "released": true
                }
            ]"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get_versions("DEMO");

        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].name, "Version 1.0");
    }

    #[test]
    fn test_get_project_components() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO/components")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "id": "1",
                    "name": "Frontend",
                    "description": "Frontend component"
                }
            ]"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get_components("DEMO");

        assert!(result.is_ok());
        let components = result.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name, "Frontend");
    }

    #[test]
    fn test_search_projects() {
        let mut server = Server::new();
        let _m = server
            .mock(
                "GET",
                "/rest/api/latest/project/search?query=demo&maxResults=10",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "startAt": 0,
                "maxResults": 10,
                "total": 1,
                "values": [
                    {
                        "self": "http://localhost/rest/api/2/project/DEMO",
                        "id": "10000",
                        "key": "DEMO",
                        "name": "Demo Project",
                        "projectTypeKey": "software"
                    }
                ]
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let options = ProjectSearchOptions {
            query: Some("demo".to_string()),
            start_at: None,
            max_results: Some(10),
            order_by: None,
            category_id: None,
            project_type_key: None,
        };

        let result = jira.projects().search(&options);

        assert!(result.is_ok());
        let search_results = result.unwrap();
        assert_eq!(search_results.total, 1);
        assert_eq!(search_results.values.len(), 1);
        assert_eq!(search_results.values[0].key, "DEMO");
    }

    #[test]
    fn test_get_project_roles() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO/role")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "Administrators": "http://localhost/rest/api/2/project/DEMO/role/10002",
                "Developers": "http://localhost/rest/api/2/project/DEMO/role/10001"
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get_roles("DEMO");

        assert!(result.is_ok());
        let roles = result.unwrap();
        assert!(roles.contains_key("Administrators"));
        assert!(roles.contains_key("Developers"));
    }

    #[test]
    fn test_get_role_users() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO/role/10001")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/DEMO/role/10001",
                "name": "Developers",
                "id": 10001,
                "description": "A project role that represents developers",
                "actors": [
                    {
                        "id": 1,
                        "displayName": "John Doe",
                        "type": "atlassian-user-role-actor",
                        "name": "johndoe"
                    }
                ]
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get_role_users("DEMO", 10001);

        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.name, "Developers");
        assert_eq!(role.id, 10001);
        assert!(role.actors.is_some());
        let actors = role.actors.unwrap();
        assert_eq!(actors.len(), 1);
        assert_eq!(actors[0].name, "johndoe");
    }

    #[test]
    fn test_project_not_found() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/rest/api/latest/project/NOTFOUND")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "errorMessages": ["No project could be found with key 'NOTFOUND'."],
                "errors": {}
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let result = jira.projects().get("NOTFOUND");

        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_validation_error() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/rest/api/latest/project")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "errorMessages": [],
                "errors": {
                    "key": "A project with that name already exists."
                }
            }"#,
            )
            .create();

        let jira = get_test_jira(&server);
        let create_project = CreateProject {
            key: "EXISTING".to_string(),
            name: "Existing Project".to_string(),
            project_type_key: "software".to_string(),
            description: None,
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            issue_security_scheme: None,
            permission_scheme: None,
            notification_scheme: None,
            category_id: None,
        };

        let result = jira.projects().create(create_project);

        assert!(result.is_err());
    }
}

// Async tests
#[cfg(feature = "async")]
#[cfg(test)]
mod async_projects_tests {
    use gouqi::{CreateProject, Credentials, ProjectSearchOptions, r#async::Jira as AsyncJira};
    use mockito::{Matcher, Server};
    use serde_json::json;

    fn get_test_jira(server: &Server) -> AsyncJira {
        AsyncJira::new(server.url(), Credentials::Anonymous)
            .expect("Failed to create test async Jira client")
    }

    #[tokio::test]
    async fn test_async_list_projects() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/rest/api/latest/project")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "self": "http://localhost/rest/api/2/project/DEMO",
                    "id": "10000",
                    "key": "DEMO",
                    "name": "Demo Project",
                    "description": "A demonstration project",
                    "projectTypeKey": "software"
                }
            ]"#,
            )
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira.projects().list().await;

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].key, "DEMO");
    }

    #[tokio::test]
    async fn test_async_get_project() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/rest/api/latest/project/DEMO")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/DEMO",
                "id": "10000",
                "key": "DEMO",
                "name": "Demo Project",
                "description": "A demonstration project",
                "projectTypeKey": "software"
            }"#,
            )
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let result = jira.projects().get("DEMO").await;

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.key, "DEMO");
        assert_eq!(project.name, "Demo Project");
    }

    #[tokio::test]
    async fn test_async_create_project() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/rest/api/latest/project")
            .with_status(201)
            .with_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "key": "NEW",
                "name": "New Project",
                "projectTypeKey": "software"
            })))
            .with_body(
                r#"{
                "self": "http://localhost/rest/api/2/project/NEW",
                "id": "10001",
                "key": "NEW",
                "name": "New Project",
                "projectTypeKey": "software"
            }"#,
            )
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let create_project = CreateProject {
            key: "NEW".to_string(),
            name: "New Project".to_string(),
            project_type_key: "software".to_string(),
            description: None,
            lead: None,
            url: None,
            assignee_type: None,
            avatar_id: None,
            issue_security_scheme: None,
            permission_scheme: None,
            notification_scheme: None,
            category_id: None,
        };

        let result = jira.projects().create(create_project).await;

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.key, "NEW");
    }

    #[tokio::test]
    async fn test_async_search_projects() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/rest/api/latest/project/search?query=demo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "startAt": 0,
                "maxResults": 50,
                "total": 1,
                "values": [
                    {
                        "self": "http://localhost/rest/api/2/project/DEMO",
                        "id": "10000",
                        "key": "DEMO",
                        "name": "Demo Project",
                        "projectTypeKey": "software"
                    }
                ]
            }"#,
            )
            .create_async()
            .await;

        let jira = get_test_jira(&server);
        let options = ProjectSearchOptions {
            query: Some("demo".to_string()),
            start_at: None,
            max_results: None,
            order_by: None,
            category_id: None,
            project_type_key: None,
        };

        let result = jira.projects().search(&options).await;

        assert!(result.is_ok());
        let search_results = result.unwrap();
        assert_eq!(search_results.total, 1);
        assert_eq!(search_results.values[0].key, "DEMO");
    }
}
