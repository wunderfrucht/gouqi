use gouqi::{Credentials, Jira, components::*};
use serde_json::json;

#[test]
fn test_components_creation() {
    let jira = Jira::new("http://localhost", Credentials::Anonymous).unwrap();
    let components = jira.components();

    // Just testing that we can create the components interface
    assert!(std::mem::size_of_val(&components) > 0);
}

#[test]
fn test_component_get_success() {
    let mut server = mockito::Server::new();

    let mock_component = json!({
        "id": "10000",
        "name": "Backend API",
        "description": "Backend API component",
        "lead": {
            "name": "admin",
            "displayName": "Admin User"
        },
        "assigneeType": "PROJECT_LEAD",
        "assignee": {
            "name": "admin",
            "displayName": "Admin User"
        },
        "realAssigneeType": "PROJECT_LEAD",
        "realAssignee": {
            "name": "admin",
            "displayName": "Admin User"
        },
        "isAssigneeTypeValid": true,
        "project": "TEST",
        "projectId": 10001,
        "self": format!("{}/rest/api/latest/component/10000", server.url())
    });

    server
        .mock("GET", "/rest/api/latest/component/10000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_component.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().get("10000");

    assert!(result.is_ok());
    let component = result.unwrap();
    assert_eq!(component.id, "10000");
    assert_eq!(component.name, "Backend API");
}

#[test]
fn test_component_get_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/component/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Component does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().get("99999");

    assert!(result.is_err());
}

#[test]
fn test_component_create_success() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "id": "10001",
        "name": "New Component",
        "description": "A newly created component",
        "project": "TEST",
        "self": format!("{}/rest/api/latest/component/10001", server.url())
    });

    server
        .mock("POST", "/rest/api/latest/component")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let create_data = CreateComponent {
        name: "New Component".to_string(),
        description: Some("A newly created component".to_string()),
        project: "TEST".to_string(),
    };

    let result = jira.components().create(create_data);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, "10001");
    assert_eq!(response.name, "New Component");
    assert_eq!(
        response.description,
        Some("A newly created component".to_string())
    );
    assert_eq!(response.project, "TEST");
    assert!(response.url.contains("/component/10001"));
}

#[test]
fn test_component_create_without_description() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "id": "10002",
        "name": "Simple Component",
        "description": null,
        "project": "TEST",
        "self": format!("{}/rest/api/latest/component/10002", server.url())
    });

    server
        .mock("POST", "/rest/api/latest/component")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let create_data = CreateComponent {
        name: "Simple Component".to_string(),
        description: None,
        project: "TEST".to_string(),
    };

    let result = jira.components().create(create_data);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, "10002");
    assert_eq!(response.name, "Simple Component");
    assert_eq!(response.description, None);
    assert_eq!(response.project, "TEST");
}

#[test]
fn test_component_create_invalid_project() {
    let mut server = mockito::Server::new();

    server
        .mock("POST", "/rest/api/latest/component")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Invalid project key"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let create_data = CreateComponent {
        name: "Invalid Component".to_string(),
        description: None,
        project: "INVALID".to_string(),
    };

    let result = jira.components().create(create_data);

    assert!(result.is_err());
}

#[test]
fn test_component_edit_success() {
    let mut server = mockito::Server::new();

    let mock_response = json!({
        "id": "10000",
        "name": "Updated Component",
        "description": "Updated description",
        "project": "TEST",
        "self": format!("{}/rest/api/latest/component/10000", server.url())
    });

    server
        .mock("PUT", "/rest/api/latest/component/10000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let edit_data = CreateComponent {
        name: "Updated Component".to_string(),
        description: Some("Updated description".to_string()),
        project: "TEST".to_string(),
    };

    let result = jira.components().edit("10000", edit_data);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, "10000");
    assert_eq!(response.name, "Updated Component");
    assert_eq!(
        response.description,
        Some("Updated description".to_string())
    );
    assert_eq!(response.project, "TEST");
}

#[test]
fn test_component_edit_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("PUT", "/rest/api/latest/component/99999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Component does not exist"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let edit_data = CreateComponent {
        name: "Nonexistent Component".to_string(),
        description: None,
        project: "TEST".to_string(),
    };

    let result = jira.components().edit("99999", edit_data);

    assert!(result.is_err());
}

#[test]
fn test_component_list_success() {
    let mut server = mockito::Server::new();

    let mock_components = json!([
        {
            "id": "10000",
            "name": "Backend",
            "description": "Backend components",
            "project": "TEST",
            "projectId": 10001,
            "self": format!("{}/rest/api/latest/component/10000", server.url())
        },
        {
            "id": "10001",
            "name": "Frontend",
            "description": "Frontend components",
            "project": "TEST",
            "projectId": 10001,
            "self": format!("{}/rest/api/latest/component/10001", server.url())
        }
    ]);

    server
        .mock("GET", "/rest/api/latest/project/TEST/components")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_components.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().list("TEST");

    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(components.len(), 2);

    assert_eq!(components[0].id, "10000");
    assert_eq!(components[0].name, "Backend");

    assert_eq!(components[1].id, "10001");
    assert_eq!(components[1].name, "Frontend");
}

#[test]
fn test_component_list_empty() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/project/EMPTY/components")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().list("EMPTY");

    assert!(result.is_ok());
    let components = result.unwrap();
    assert!(components.is_empty());
}

#[test]
fn test_component_list_project_not_found() {
    let mut server = mockito::Server::new();

    server
        .mock("GET", "/rest/api/latest/project/NOTFOUND/components")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorMessages": ["Project not found"]}).to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().list("NOTFOUND");

    assert!(result.is_err());
}

#[test]
fn test_component_list_by_project_id() {
    let mut server = mockito::Server::new();

    let mock_components = json!([
        {
            "id": "10000",
            "name": "Component by ID",
            "project": "BYID",
            "projectId": 12345,
            "self": format!("{}/rest/api/latest/component/10000", server.url())
        }
    ]);

    server
        .mock("GET", "/rest/api/latest/project/12345/components")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_components.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let result = jira.components().list("12345");

    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].name, "Component by ID");
}

#[test]
fn test_component_create_component_struct() {
    // Test the CreateComponent struct serialization
    let component = CreateComponent {
        name: "Test Component".to_string(),
        description: Some("Test description".to_string()),
        project: "TEST".to_string(),
    };

    let json = serde_json::to_string(&component).unwrap();
    assert!(json.contains("Test Component"));
    assert!(json.contains("Test description"));
    assert!(json.contains("TEST"));

    // Test deserialization
    let deserialized: CreateComponent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Test Component");
    assert_eq!(
        deserialized.description,
        Some("Test description".to_string())
    );
    assert_eq!(deserialized.project, "TEST");
}

#[test]
fn test_create_component_response_deserialization() {
    let json_data = json!({
        "id": "10000",
        "name": "Response Component",
        "description": "Response description",
        "project": "RESP",
        "self": "http://localhost/rest/api/latest/component/10000"
    });

    let response: CreateComponentResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.id, "10000");
    assert_eq!(response.name, "Response Component");
    assert_eq!(
        response.description,
        Some("Response description".to_string())
    );
    assert_eq!(response.project, "RESP");
    assert_eq!(
        response.url,
        "http://localhost/rest/api/latest/component/10000"
    );
}

#[test]
fn test_components_interface_multiple_operations() {
    let mut server = mockito::Server::new();

    // Mock get component
    let mock_component = json!({
        "id": "10000",
        "name": "Existing Component",
        "project": "MULTI",
        "self": format!("{}/rest/api/latest/component/10000", server.url())
    });

    server
        .mock("GET", "/rest/api/latest/component/10000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_component.to_string())
        .create();

    // Mock create component
    let mock_create_response = json!({
        "id": "10001",
        "name": "New Component",
        "project": "MULTI",
        "self": format!("{}/rest/api/latest/component/10001", server.url())
    });

    server
        .mock("POST", "/rest/api/latest/component")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(mock_create_response.to_string())
        .create();

    // Mock list components
    let mock_list = json!([mock_component, mock_create_response]);

    server
        .mock("GET", "/rest/api/latest/project/MULTI/components")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_list.to_string())
        .create();

    let jira = Jira::new(server.url(), Credentials::Anonymous).unwrap();
    let components_interface = jira.components();

    // Test multiple operations with the same interface
    let existing_component = components_interface.get("10000").unwrap();
    assert_eq!(existing_component.id, "10000");

    let create_data = CreateComponent {
        name: "New Component".to_string(),
        description: None,
        project: "MULTI".to_string(),
    };
    let new_component = components_interface.create(create_data).unwrap();
    assert_eq!(new_component.id, "10001");

    let all_components = components_interface.list("MULTI").unwrap();
    assert_eq!(all_components.len(), 2);
}
