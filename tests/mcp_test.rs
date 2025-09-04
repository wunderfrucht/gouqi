use gouqi::mcp::*;
use gouqi::mcp::{error, schema, uri, validation};
use gouqi::{Board, Error, Issue, Location, Project, ProjectComponent, Sprint, User, Version};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn test_issue_uri_generation() {
    assert_eq!(uri::issue_uri("DEMO-123"), "jira://issue/DEMO-123");
    assert_eq!(uri::issue_uri("PROJ-456"), "jira://issue/PROJ-456");
}

#[test]
fn test_project_uri_generation() {
    assert_eq!(uri::project_uri("DEMO"), "jira://project/DEMO");
    assert_eq!(uri::project_uri("MYPROJECT"), "jira://project/MYPROJECT");
}

#[test]
fn test_user_uri_generation() {
    assert_eq!(uri::user_uri("123456789"), "jira://user/123456789");
    assert_eq!(uri::user_uri("user-id"), "jira://user/user-id");
}

#[test]
fn test_component_uri_generation() {
    assert_eq!(uri::component_uri("10001"), "jira://component/10001");
    assert_eq!(uri::component_uri("comp-123"), "jira://component/comp-123");
}

#[test]
fn test_version_uri_generation() {
    assert_eq!(uri::version_uri("10002"), "jira://version/10002");
    assert_eq!(uri::version_uri("v1.0.0"), "jira://version/v1.0.0");
}

#[test]
fn test_parse_jira_uri_valid() {
    let result = uri::parse_jira_uri("jira://issue/DEMO-123").unwrap();
    assert_eq!(result, ("issue".to_string(), "DEMO-123".to_string()));

    let result = uri::parse_jira_uri("jira://project/DEMO").unwrap();
    assert_eq!(result, ("project".to_string(), "DEMO".to_string()));

    let result = uri::parse_jira_uri("jira://user/123456789").unwrap();
    assert_eq!(result, ("user".to_string(), "123456789".to_string()));
}

#[test]
fn test_parse_jira_uri_invalid() {
    assert!(uri::parse_jira_uri("http://example.com").is_err());
    assert!(uri::parse_jira_uri("invalid-uri").is_err());
    assert!(uri::parse_jira_uri("").is_err());
}

#[test]
fn test_validate_jira_uri_valid() {
    assert!(uri::validate_jira_uri("jira://issue/DEMO-123").is_ok());
    assert!(uri::validate_jira_uri("jira://project/DEMO").is_ok());
    assert!(uri::validate_jira_uri("jira://user/123456789").is_ok());
    assert!(uri::validate_jira_uri("jira://component/10001").is_ok());
    assert!(uri::validate_jira_uri("jira://version/10002").is_ok());
}

#[test]
fn test_validate_jira_uri_invalid() {
    assert!(uri::validate_jira_uri("http://example.com").is_err());
    assert!(uri::validate_jira_uri("jira://invalid/resource").is_err());
    assert!(uri::validate_jira_uri("invalid-scheme://issue/DEMO-123").is_err());
}

#[test]
fn test_issue_search_tool_schema() {
    let tool = schema::issue_search_tool();
    assert_eq!(tool.name, "jira_search_issues");
    assert!(tool.description.contains("Search for Jira issues"));

    // Verify the schema has required properties
    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("jql"));
    assert!(properties.contains_key("start_at"));
    assert!(properties.contains_key("max_results"));
    assert!(properties.contains_key("fields"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("jql")));
}

#[test]
fn test_get_issue_tool_schema() {
    let tool = schema::get_issue_tool();
    assert_eq!(tool.name, "jira_get_issue");
    assert!(tool.description.contains("Get a specific Jira issue"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("issue_key"));
    assert!(properties.contains_key("fields"));
    assert!(properties.contains_key("expand"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("issue_key")));
}

#[test]
fn test_create_issue_tool_schema() {
    let tool = schema::create_issue_tool();
    assert_eq!(tool.name, "jira_create_issue");
    assert!(tool.description.contains("Create a new Jira issue"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("project"));
    assert!(properties.contains_key("issue_type"));
    assert!(properties.contains_key("summary"));
    assert!(properties.contains_key("description"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("project")));
    assert!(required.contains(&json!("issue_type")));
    assert!(required.contains(&json!("summary")));
}

#[test]
fn test_list_projects_tool_schema() {
    let tool = schema::list_projects_tool();
    assert_eq!(tool.name, "jira_list_projects");
    assert!(
        tool.description
            .contains("List all available Jira projects")
    );

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("recent"));
    assert!(properties.contains_key("expand"));
}

#[test]
fn test_list_issue_transitions_tool_schema() {
    let tool = schema::list_issue_transitions_tool();
    assert_eq!(tool.name, "jira_list_issue_transitions");
    assert!(tool.description.contains("List available transitions"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("issue_key"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("issue_key")));
}

#[test]
fn test_trigger_issue_transition_tool_schema() {
    let tool = schema::trigger_issue_transition_tool();
    assert_eq!(tool.name, "jira_trigger_issue_transition");
    assert!(tool.description.contains("Trigger a transition"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("issue_key"));
    assert!(properties.contains_key("transition_id"));
    assert!(properties.contains_key("comment"));
    assert!(properties.contains_key("resolution"));
    assert!(properties.contains_key("fields"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("issue_key")));
    assert!(required.contains(&json!("transition_id")));
}

#[test]
fn test_list_issue_attachments_tool_schema() {
    let tool = schema::list_issue_attachments_tool();
    assert_eq!(tool.name, "jira_list_issue_attachments");
    assert!(tool.description.contains("List all attachments"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("issue_key"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("issue_key")));
}

#[test]
fn test_upload_issue_attachment_tool_schema() {
    let tool = schema::upload_issue_attachment_tool();
    assert_eq!(tool.name, "jira_upload_issue_attachment");
    assert!(tool.description.contains("Upload an attachment"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("issue_key"));
    assert!(properties.contains_key("filename"));
    assert!(properties.contains_key("content"));
    assert!(properties.contains_key("content_type"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("issue_key")));
    assert!(required.contains(&json!("filename")));
    assert!(required.contains(&json!("content")));
}

#[test]
fn test_create_project_component_tool_schema() {
    let tool = schema::create_project_component_tool();
    assert_eq!(tool.name, "jira_create_project_component");
    assert!(tool.description.contains("Create a new component"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("project"));
    assert!(properties.contains_key("name"));
    assert!(properties.contains_key("description"));
    assert!(properties.contains_key("lead"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("project")));
    assert!(required.contains(&json!("name")));
}

#[test]
fn test_update_project_component_tool_schema() {
    let tool = schema::update_project_component_tool();
    assert_eq!(tool.name, "jira_update_project_component");
    assert!(tool.description.contains("Update an existing"));

    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("component_id"));
    assert!(properties.contains_key("name"));
    assert!(properties.contains_key("description"));
    assert!(properties.contains_key("lead"));

    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("component_id")));
}

#[test]
fn test_all_tools() {
    let tools = schema::all_tools();
    assert_eq!(tools.len(), 10);

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    // Core CRUD operations
    assert!(tool_names.contains(&"jira_search_issues".to_string()));
    assert!(tool_names.contains(&"jira_get_issue".to_string()));
    assert!(tool_names.contains(&"jira_create_issue".to_string()));
    assert!(tool_names.contains(&"jira_list_projects".to_string()));
    // Transition operations
    assert!(tool_names.contains(&"jira_list_issue_transitions".to_string()));
    assert!(tool_names.contains(&"jira_trigger_issue_transition".to_string()));
    // Attachment operations
    assert!(tool_names.contains(&"jira_list_issue_attachments".to_string()));
    assert!(tool_names.contains(&"jira_upload_issue_attachment".to_string()));
    // Component operations
    assert!(tool_names.contains(&"jira_create_project_component".to_string()));
    assert!(tool_names.contains(&"jira_update_project_component".to_string()));
}

#[test]
fn test_error_mapping_unauthorized() {
    let jira_error = Error::Unauthorized;
    let mcp_error = error::to_mcp_error(&jira_error);

    assert_eq!(mcp_error.code, 401);
    assert!(mcp_error.message.contains("Unauthorized"));
    assert!(mcp_error.data.is_some());

    let data = mcp_error.data.unwrap();
    assert_eq!(data["type"], "authentication_error");
}

#[test]
fn test_error_mapping_not_found() {
    let jira_error = Error::NotFound;
    let mcp_error = error::to_mcp_error(&jira_error);

    assert_eq!(mcp_error.code, 404);
    assert!(mcp_error.message.contains("not found"));
    assert!(mcp_error.data.is_some());

    let data = mcp_error.data.unwrap();
    assert_eq!(data["type"], "not_found_error");
}

#[test]
fn test_error_mapping_method_not_allowed() {
    let jira_error = Error::MethodNotAllowed;
    let mcp_error = error::to_mcp_error(&jira_error);

    assert_eq!(mcp_error.code, 405);
    assert!(mcp_error.message.contains("not allowed"));
    assert!(mcp_error.data.is_some());

    let data = mcp_error.data.unwrap();
    assert_eq!(data["type"], "method_not_allowed_error");
}

#[test]
fn test_error_mapping_invalid_query() {
    let jira_error = Error::InvalidQuery {
        message: "V3 API requires bounded queries. Please set maxResults parameter (max 5000)."
            .to_string(),
    };
    let mcp_error = error::to_mcp_error(&jira_error);

    assert_eq!(mcp_error.code, 400);
    assert!(mcp_error.message.contains("Invalid query"));
    assert!(
        mcp_error
            .message
            .contains("V3 API requires bounded queries")
    );
    assert!(mcp_error.data.is_some());

    let data = mcp_error.data.unwrap();
    assert_eq!(data["type"], "invalid_query_error");
}

#[test]
fn test_validate_issue_key_valid() {
    assert!(validation::validate_issue_key("DEMO-123").is_ok());
    assert!(validation::validate_issue_key("PROJECT-456").is_ok());
    assert!(validation::validate_issue_key("ABC-1").is_ok());
    assert!(validation::validate_issue_key("LONGPROJECT-999999").is_ok());
}

#[test]
fn test_validate_issue_key_invalid() {
    assert!(validation::validate_issue_key("").is_err());
    assert!(validation::validate_issue_key("DEMO").is_err());
    assert!(validation::validate_issue_key("demo-123").is_err());
    assert!(validation::validate_issue_key("DEMO-").is_err());
    assert!(validation::validate_issue_key("-123").is_err());
    assert!(validation::validate_issue_key("DEMO-abc").is_err());
    assert!(validation::validate_issue_key("DEMO_123").is_err());
}

#[test]
fn test_validate_project_key_valid() {
    assert!(validation::validate_project_key("DEMO").is_ok());
    assert!(validation::validate_project_key("PROJECT").is_ok());
    assert!(validation::validate_project_key("ABC").is_ok());
    assert!(validation::validate_project_key("PROJ123").is_ok());
    assert!(validation::validate_project_key("A").is_ok());
}

#[test]
fn test_validate_project_key_invalid() {
    assert!(validation::validate_project_key("").is_err());
    assert!(validation::validate_project_key("demo").is_err());
    assert!(validation::validate_project_key("Demo").is_err());
    assert!(validation::validate_project_key("DEMO-PROJECT").is_err());
    assert!(validation::validate_project_key("DEMO_PROJECT").is_err());
    assert!(validation::validate_project_key("TOOLONGPROJECT").is_err()); // > 10 chars
}

#[test]
fn test_validate_jql_valid() {
    assert!(validation::validate_jql("project = DEMO").is_ok());
    assert!(validation::validate_jql("assignee = currentUser()").is_ok());
    assert!(validation::validate_jql("status = 'In Progress' AND project = DEMO").is_ok());
    assert!(validation::validate_jql("created >= -7d").is_ok());
}

#[test]
fn test_validate_jql_invalid() {
    assert!(validation::validate_jql("").is_err());
    assert!(validation::validate_jql("DROP TABLE users").is_err());
    assert!(validation::validate_jql("DELETE FROM issues").is_err());
    assert!(validation::validate_jql("INSERT INTO projects").is_err());
    assert!(validation::validate_jql("UPDATE issues SET").is_err());
    assert!(validation::validate_jql("SELECT * FROM issues UNION").is_err());
    assert!(validation::validate_jql("project = DEMO -- comment").is_err());
    assert!(validation::validate_jql("project = DEMO /* comment */").is_err());
}

#[test]
fn test_validate_pagination_valid() {
    assert!(validation::validate_pagination(Some(0), Some(50)).is_ok());
    assert!(validation::validate_pagination(Some(100), Some(25)).is_ok());
    assert!(validation::validate_pagination(None, None).is_ok());
    assert!(validation::validate_pagination(Some(0), None).is_ok());
    assert!(validation::validate_pagination(None, Some(100)).is_ok());
}

#[test]
fn test_validate_pagination_invalid() {
    assert!(validation::validate_pagination(Some(-1), Some(50)).is_err());
    assert!(validation::validate_pagination(Some(0), Some(0)).is_err());
    assert!(validation::validate_pagination(Some(0), Some(1001)).is_err());
}

#[test]
fn test_issue_to_mcp_resource() {
    let issue = create_sample_issue();
    let resource = issue.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://issue/DEMO-123");
    assert_eq!(resource.name, "DEMO-123: Test issue summary");
    assert_eq!(resource.description, Some("Test description".to_string()));
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("project").unwrap(), "DEMO");
    assert_eq!(annotations.get("status").unwrap(), "Open");
    assert_eq!(annotations.get("issue_type").unwrap(), "Bug");
    assert_eq!(annotations.get("assignee").unwrap(), "Test User");
}

#[test]
fn test_project_to_mcp_resource() {
    let project = create_sample_project();
    let resource = project.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://project/DEMO");
    assert_eq!(resource.name, "DEMO: Demo Project");
    assert_eq!(
        resource.description,
        Some("A demo project for testing".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("project_type").unwrap(), "software");
    assert_eq!(annotations.get("lead").unwrap(), "Project Lead");
}

#[test]
fn test_user_to_mcp_resource() {
    let user = create_sample_user();
    let resource = user.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://user/Test User");
    assert_eq!(resource.name, "User: Test User");
    assert_eq!(
        resource.description,
        Some("Jira user Test User".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("active").unwrap(), true);
    assert_eq!(annotations.get("email").unwrap(), "test@example.com");
    assert_eq!(annotations.get("username").unwrap(), "testuser");
}

#[test]
fn test_version_to_mcp_resource() {
    let version = create_sample_version();
    let resource = version.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://version/10001");
    assert_eq!(resource.name, "Version: 1.0.0");
    assert_eq!(
        resource.description,
        Some("Project version 1.0.0 (Released)".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("project_id").unwrap(), 10000);
    assert_eq!(annotations.get("released").unwrap(), true);
    assert_eq!(annotations.get("archived").unwrap(), false);
    assert_eq!(annotations.get("status").unwrap(), "Released");
}

#[test]
fn test_board_to_mcp_resource() {
    let board = create_sample_board();
    let resource = board.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://board/1");
    assert_eq!(resource.name, "Board: Demo Board");
    assert_eq!(
        resource.description,
        Some("Jira scrum board: Demo Board".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("board_type").unwrap(), "scrum");
    assert_eq!(annotations.get("board_id").unwrap(), 1);
    assert_eq!(annotations.get("project_id").unwrap(), 10000);
}

#[test]
fn test_sprint_to_mcp_resource() {
    let sprint = create_sample_sprint();
    let resource = sprint.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://sprint/100");
    assert_eq!(resource.name, "Sprint: Sprint 1");
    assert_eq!(
        resource.description,
        Some("Sprint: Sprint 1 (ACTIVE)".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("sprint_id").unwrap(), 100);
    assert_eq!(annotations.get("state").unwrap(), "ACTIVE");
    assert_eq!(annotations.get("origin_board_id").unwrap(), 1);
    // Date fields are present (just check they exist since format may vary)
    assert!(annotations.contains_key("start_date"));
    assert!(annotations.contains_key("end_date"));
}

#[test]
fn test_component_to_mcp_resource() {
    let component = create_sample_component();
    let resource = component.to_mcp_resource("https://jira.example.com");

    assert_eq!(resource.uri, "jira://component/10050");
    assert_eq!(resource.name, "Component: Authentication");
    assert_eq!(
        resource.description,
        Some("Handles user login and registration".to_string())
    );
    assert_eq!(resource.mime_type, "application/json");

    let annotations = resource.annotations.unwrap();
    assert_eq!(annotations.get("component_id").unwrap(), "10050");
}

// Helper function to create a sample issue for testing
fn create_sample_issue() -> Issue {
    let mut fields = BTreeMap::new();

    // Add project field
    fields.insert(
        "project".to_string(),
        json!({
            "self": "https://jira.example.com/rest/api/2/project/10000",
            "id": "10000",
            "key": "DEMO",
            "name": "Demo Project",
            "projectTypeKey": "software"
        }),
    );

    // Add status field
    fields.insert(
        "status".to_string(),
        json!({
            "self": "https://jira.example.com/rest/api/2/status/1",
            "description": "The issue is open and ready for the assignee to start work on it.",
            "iconUrl": "https://jira.example.com/images/icons/status_open.gif",
            "name": "Open",
            "id": "1"
        }),
    );

    // Add issue type field
    fields.insert(
        "issuetype".to_string(),
        json!({
            "self": "https://jira.example.com/rest/api/2/issuetype/1",
            "id": "1",
            "description": "A problem which impairs or prevents the functions of the product.",
            "iconUrl": "https://jira.example.com/images/icons/bug.gif",
            "name": "Bug",
            "subtask": false
        }),
    );

    // Add summary field
    fields.insert("summary".to_string(), json!("Test issue summary"));

    // Add description field
    fields.insert("description".to_string(), json!("Test description"));

    // Add assignee field
    fields.insert(
        "assignee".to_string(),
        json!({
            "self": "https://jira.example.com/rest/api/2/user?username=test",
            "name": "test",
            "displayName": "Test User",
            "accountId": "123456789",
            "active": true
        }),
    );

    Issue {
        self_link: "https://jira.example.com/rest/api/2/issue/10001".to_string(),
        id: "10001".to_string(),
        key: "DEMO-123".to_string(),
        fields,
    }
}

// Helper function to create a sample project for testing
fn create_sample_project() -> Project {
    Project {
        self_link: "https://jira.example.com/rest/api/2/project/10000".to_string(),
        id: "10000".to_string(),
        key: "DEMO".to_string(),
        name: "Demo Project".to_string(),
        project_type_key: "software".to_string(),
        avatar_urls: None,
        project_category: None,
        description: Some("A demo project for testing".to_string()),
        lead: Some(User {
            active: true,
            avatar_urls: None,
            display_name: "Project Lead".to_string(),
            email_address: None,
            key: None,
            name: Some("lead".to_string()),
            self_link: "https://jira.example.com/rest/api/2/user?username=lead".to_string(),
            timezone: None,
        }),
        components: None,
        issue_types: None,
        versions: None,
        roles: None,
    }
}

// Helper function to create a sample user for testing
fn create_sample_user() -> User {
    User {
        active: true,
        avatar_urls: None,
        display_name: "Test User".to_string(),
        email_address: Some("test@example.com".to_string()),
        key: Some("testuser".to_string()),
        name: Some("testuser".to_string()),
        self_link: "https://jira.example.com/rest/api/2/user?username=testuser".to_string(),
        timezone: Some("America/New_York".to_string()),
    }
}

// Helper function to create a sample version for testing
fn create_sample_version() -> Version {
    Version {
        archived: false,
        id: "10001".to_string(),
        name: "1.0.0".to_string(),
        project_id: 10000,
        released: true,
        self_link: "https://jira.example.com/rest/api/2/version/10001".to_string(),
    }
}

// Helper function to create a sample board for testing
fn create_sample_board() -> Board {
    Board {
        self_link: "https://jira.example.com/rest/agile/1.0/board/1".to_string(),
        id: 1,
        name: "Demo Board".to_string(),
        type_name: "scrum".to_string(),
        location: Some(Location {
            project_id: Some(10000),
            user_id: None,
            user_account_id: None,
            display_name: Some("Demo Project".to_string()),
            project_name: Some("Demo Project".to_string()),
            project_key: Some("DEMO".to_string()),
            project_type_key: Some("software".to_string()),
            name: None,
        }),
    }
}

// Helper function to create a sample sprint for testing
fn create_sample_sprint() -> Sprint {
    use time::OffsetDateTime;

    Sprint {
        id: 100,
        self_link: "https://jira.example.com/rest/agile/1.0/sprint/100".to_string(),
        name: "Sprint 1".to_string(),
        state: Some("ACTIVE".to_string()),
        start_date: Some(OffsetDateTime::now_utc()),
        end_date: Some(OffsetDateTime::now_utc()),
        complete_date: None,
        origin_board_id: Some(1),
    }
}

// Helper function to create a sample component for testing
fn create_sample_component() -> ProjectComponent {
    ProjectComponent {
        id: "10050".to_string(),
        name: "Authentication".to_string(),
        description: Some("Handles user login and registration".to_string()),
        self_link: Some("https://jira.example.com/rest/api/2/component/10050".to_string()),
    }
}
